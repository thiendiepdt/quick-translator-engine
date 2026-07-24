//! Name candidate extraction for Chinese web-novel chapters.
//!
//! `QtCompatible` intentionally keeps the original Quick Translator shape:
//! Jieba tokens, a 2-5 character window, frequency thresholding and dictionary
//! rejection. `Hybrid` adds context, surname/suffix rules, overlapping n-grams
//! and book-scoped accepted/rejected memory. Statistical NER and AI decisions
//! are merged by the API layer so this module stays deterministic and cheap.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use jieba_rs::Jieba;

use crate::han_viet::is_chinese;
use crate::{CharRange, DictionaryOverrides, Engine};

const QT_MIN_LENGTH: usize = 2;
const QT_MAX_LENGTH: usize = 5;
const HYBRID_MAX_LENGTH: usize = 8;
const PERSON_SUFFIXES: &[&str] = &[
    "先生", "小姐", "姑娘", "公子", "少爷", "夫人", "长老", "师父", "师傅", "师兄", "师姐", "师弟",
    "师妹", "大人", "真人", "道长", "掌门", "宗主", "教主", "老祖", "陛下", "殿下", "王爷", "将军",
    "队长", "院长", "主任", "老师", "同学",
];
const LOCATION_SUFFIXES: &[&str] = &[
    "国", "州", "郡", "省", "市", "县", "镇", "乡", "村", "城", "山", "峰", "谷", "岛", "海", "湖",
    "河", "江", "域", "界", "宫", "殿", "楼", "阁", "宗", "门", "派", "寺", "观", "院", "府", "庄",
    "寨", "森林", "沙漠", "大陆", "广场", "学院",
];
const ORGANIZATION_SUFFIXES: &[&str] = &[
    "宗", "门", "派", "教", "帮", "盟", "会", "阁", "殿", "宫", "院", "学院", "公司", "集团",
    "协会", "军", "队", "局", "部", "家族", "一族", "王朝", "帝国",
];
const NAME_TRIGGERS: &[&str] = &[
    "名为",
    "名字叫",
    "叫做",
    "叫",
    "姓",
    "名叫",
    "自称",
    "号称",
    "称为",
    "乃是",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameFilterMode {
    QtCompatible,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NameEntityType {
    Person,
    Location,
    Organization,
    Title,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NameCandidateSource {
    QtJieba,
    Ngram,
    ContextRule,
    SurnameRule,
    SuffixRule,
    BookMemory,
    OnnxNer,
    AiFallback,
}

#[derive(Debug, Clone)]
pub struct NameFilterOptions {
    pub mode: NameFilterMode,
    pub min_occurrences: usize,
    pub min_score: f32,
    pub max_candidates: usize,
    pub max_name_length: usize,
    pub include_known: bool,
}

impl Default for NameFilterOptions {
    fn default() -> Self {
        Self {
            mode: NameFilterMode::Hybrid,
            min_occurrences: 2,
            min_score: 0.60,
            max_candidates: 200,
            max_name_length: HYBRID_MAX_LENGTH,
            include_known: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct NameFilterMemory {
    /// Accepted book-level names and their user-approved Vietnamese values.
    pub known_names: HashMap<String, String>,
    /// Rejections are scoped to a book and suppress later chapters.
    pub rejected_names: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct NameCandidate {
    pub text: String,
    pub suggested: String,
    pub entity_type: NameEntityType,
    pub score: f32,
    pub occurrences: usize,
    pub ranges: Vec<CharRange>,
    pub reasons: Vec<String>,
    pub sources: Vec<NameCandidateSource>,
    pub known: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NameFilterResult {
    pub candidates: Vec<NameCandidate>,
    pub scanned_characters: usize,
}

/// Ignored-filtered chapter text used by every name-filter provider, with
/// ranges mapped back to the caller's original UTF-16 input.
#[derive(Debug, Clone)]
pub struct NameFilterDocument {
    text: String,
    source_ranges: Vec<CharRange>,
    blocked: Vec<bool>,
    utf16_boundaries: Vec<usize>,
    scanned_characters: usize,
}

impl NameFilterDocument {
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Map a provider range in filtered scan text back to the original
    /// UTF-16 input. Spans crossing an ignored phrase separator are rejected.
    pub fn map_range(&self, range: CharRange) -> Option<CharRange> {
        if range.length == 0 {
            return None;
        }
        let end = range.start.checked_add(range.length)?;
        let start_index = self.utf16_boundaries.binary_search(&range.start).ok()?;
        let end_index = self.utf16_boundaries.binary_search(&end).ok()?;
        if start_index >= end_index
            || self.blocked[start_index..end_index]
                .iter()
                .any(|value| *value)
        {
            return None;
        }
        let source = &self.source_ranges[start_index..end_index];
        let source_start = source.iter().map(|range| range.start).min()?;
        let source_end = source
            .iter()
            .map(|range| range.start + range.length)
            .max()?;
        Some(CharRange {
            start: source_start,
            length: source_end - source_start,
        })
    }
}

#[derive(Default)]
struct CandidateSeed {
    from_jieba: bool,
    from_ngram: bool,
    byte_starts: Vec<usize>,
}

struct CandidateFacts {
    ranges: Vec<CharRange>,
    has_trigger: bool,
    has_surname: bool,
    entity_type: NameEntityType,
    has_suffix: bool,
}

impl Engine {
    /// Build the shared ignored-aware document consumed by rules, NER, and AI.
    pub fn prepare_name_filter_document(
        &self,
        text: &str,
        overrides: Option<&DictionaryOverrides>,
    ) -> NameFilterDocument {
        let scanned_characters = text.chars().count();
        let scan = match overrides.and_then(|value| value.ignored_chinese_phrases.as_ref()) {
            Some(ignored) => self
                .standardizer
                .standardize_for_name_scan_with_ignored_source(text, ignored),
            None => self.standardizer.standardize_for_name_scan(text),
        };
        let text: String = scan.chars.iter().collect();
        let mut utf16_boundaries = Vec::with_capacity(scan.chars.len() + 1);
        let mut utf16_offset = 0usize;
        utf16_boundaries.push(utf16_offset);
        for ch in &scan.chars {
            utf16_offset += ch.len_utf16();
            utf16_boundaries.push(utf16_offset);
        }
        NameFilterDocument {
            text,
            source_ranges: scan.source_ranges,
            blocked: scan.blocked,
            utf16_boundaries,
            scanned_characters,
        }
    }

    /// Extract likely names without any network or model dependency.
    pub fn filter_names(
        &self,
        text: &str,
        options: &NameFilterOptions,
        memory: &NameFilterMemory,
        overrides: Option<&DictionaryOverrides>,
    ) -> NameFilterResult {
        let document = self.prepare_name_filter_document(text, overrides);
        self.filter_names_in_document(&document, options, memory, overrides)
    }

    /// Extract names from a document prepared once for all filter providers.
    pub fn filter_names_in_document(
        &self,
        document: &NameFilterDocument,
        options: &NameFilterOptions,
        memory: &NameFilterMemory,
        overrides: Option<&DictionaryOverrides>,
    ) -> NameFilterResult {
        let text = document.text();
        let max_length = match options.mode {
            NameFilterMode::QtCompatible => QT_MAX_LENGTH,
            NameFilterMode::Hybrid => options
                .max_name_length
                .clamp(QT_MIN_LENGTH, HYBRID_MAX_LENGTH),
        };
        let mut seeds: HashMap<String, CandidateSeed> = HashMap::new();
        let utf16_offsets = utf16_offsets(text);
        let jieba = jieba();

        for token in jieba.cut(text, true) {
            let length = token.word.chars().count();
            if (QT_MIN_LENGTH..=max_length).contains(&length)
                && token
                    .word
                    .chars()
                    .all(|ch| is_chinese(ch, &self.dicts.han_viet))
            {
                let seed = seeds.entry(token.word.to_string()).or_default();
                seed.from_jieba = true;
                seed.byte_starts.push(token.byte_start);
            }
        }

        if options.mode == NameFilterMode::Hybrid {
            add_hybrid_ngrams(text, max_length, &self.dicts.han_viet, &mut seeds);
            for known in memory.known_names.keys() {
                for (byte_start, _) in text.match_indices(known) {
                    seeds
                        .entry(known.clone())
                        .or_default()
                        .byte_starts
                        .push(byte_start);
                }
            }
        }

        let names = dictionary_pair(
            overrides.and_then(|value| value.names.as_ref()),
            overrides.and_then(|value| value.names2.as_ref()),
            &self.dicts.primary_names,
            &self.dicts.secondary_names,
        );
        let ho_nguoi = overrides
            .and_then(|value| value.ho_nguoi.as_ref())
            .unwrap_or(&self.dicts.ho_nguoi);
        let danh_tu = overrides
            .and_then(|value| value.danh_tu.as_ref())
            .unwrap_or(&self.dicts.danh_tu);
        let hau_tu = overrides
            .and_then(|value| value.hau_tu.as_ref())
            .unwrap_or(&self.dicts.hau_tu);

        let mut candidates = Vec::new();
        for (candidate, mut seed) in seeds {
            let known_value = memory.known_names.get(&candidate);
            if memory.rejected_names.contains(&candidate)
                || (known_value.is_none() && names.contains(&candidate))
            {
                continue;
            }
            seed.byte_starts.sort_unstable();
            seed.byte_starts.dedup();
            let facts = candidate_facts(
                document,
                &candidate,
                &seed.byte_starts,
                &utf16_offsets,
                ho_nguoi,
                danh_tu,
                hau_tu,
            );
            let occurrences = facts.ranges.len();
            if occurrences == 0 {
                continue;
            }
            if options.mode == NameFilterMode::QtCompatible
                && self.dicts.only_vietphrase.contains_key(&candidate)
                && !facts.has_surname
            {
                continue;
            }
            let is_known = known_value.is_some();
            if is_known && !options.include_known {
                continue;
            }

            let mut sources = Vec::new();
            let mut reasons = Vec::new();
            if seed.from_jieba {
                sources.push(NameCandidateSource::QtJieba);
                reasons.push("Jieba nhận diện thành một từ".to_string());
            }
            if seed.from_ngram {
                sources.push(NameCandidateSource::Ngram);
            }
            if facts.has_trigger {
                sources.push(NameCandidateSource::ContextRule);
                reasons.push("xuất hiện sau ngữ cảnh giới thiệu tên".to_string());
            }
            if facts.has_surname {
                sources.push(NameCandidateSource::SurnameRule);
                reasons.push("bắt đầu bằng họ người".to_string());
            }
            if facts.has_suffix {
                sources.push(NameCandidateSource::SuffixRule);
                reasons.push("có hậu tố thực thể".to_string());
            }
            if is_known {
                sources.push(NameCandidateSource::BookMemory);
                reasons.push("đã được duyệt trong bộ nhớ truyện".to_string());
            }
            if occurrences > 1 {
                reasons.push(format!("xuất hiện {occurrences} lần"));
            }

            let score = match options.mode {
                NameFilterMode::QtCompatible => qt_score(occurrences, options.min_occurrences),
                NameFilterMode::Hybrid => hybrid_score(
                    occurrences,
                    seed.from_jieba,
                    &facts,
                    is_known,
                    self.dicts.only_vietphrase.contains_key(&candidate),
                ),
            };
            let passes = if is_known {
                true
            } else {
                score >= options.min_score
                    && (occurrences >= options.min_occurrences
                        || facts.has_trigger
                        || facts.has_surname
                        || facts.has_suffix)
            };
            if !passes {
                continue;
            }

            let suggested = known_value.cloned().unwrap_or_else(|| {
                suggested_name(
                    &candidate,
                    &self.dicts.only_vietphrase,
                    &self.dicts.han_viet,
                    facts.has_surname || facts.has_trigger,
                )
            });
            candidates.push(NameCandidate {
                text: candidate,
                suggested,
                entity_type: facts.entity_type,
                score,
                occurrences,
                ranges: facts.ranges,
                reasons,
                sources,
                known: is_known,
            });
        }

        if options.mode == NameFilterMode::QtCompatible {
            candidates = prune_qt_prefixes(candidates);
        } else {
            candidates = prune_weaker_nested_candidates(candidates);
        }
        candidates.sort_by(|left, right| {
            right
                .known
                .cmp(&left.known)
                .then_with(|| right.score.total_cmp(&left.score))
                .then_with(|| right.occurrences.cmp(&left.occurrences))
                .then_with(|| {
                    left.ranges
                        .first()
                        .map(|range| range.start)
                        .cmp(&right.ranges.first().map(|range| range.start))
                })
        });
        candidates.truncate(options.max_candidates.clamp(1, 1_000));

        NameFilterResult {
            candidates,
            scanned_characters: document.scanned_characters,
        }
    }

    /// Suggest a Vietnamese value for a model-provided entity span.
    pub fn suggest_name(&self, text: &str) -> String {
        suggested_name(
            text,
            &self.dicts.only_vietphrase,
            &self.dicts.han_viet,
            true,
        )
    }

    pub fn contains_name(&self, text: &str, overrides: Option<&DictionaryOverrides>) -> bool {
        dictionary_pair(
            overrides.and_then(|value| value.names.as_ref()),
            overrides.and_then(|value| value.names2.as_ref()),
            &self.dicts.primary_names,
            &self.dicts.secondary_names,
        )
        .contains(text)
    }
}

fn jieba() -> &'static Jieba {
    static JIEBA: OnceLock<Jieba> = OnceLock::new();
    JIEBA.get_or_init(Jieba::new)
}

fn dictionary_pair<'a>(
    custom_primary: Option<&'a HashMap<String, String>>,
    custom_secondary: Option<&'a HashMap<String, String>>,
    default_primary: &'a HashMap<String, String>,
    default_secondary: &'a HashMap<String, String>,
) -> DictionaryPair<'a> {
    DictionaryPair {
        primary: custom_primary.unwrap_or(default_primary),
        secondary: custom_secondary.unwrap_or(default_secondary),
    }
}

struct DictionaryPair<'a> {
    primary: &'a HashMap<String, String>,
    secondary: &'a HashMap<String, String>,
}

impl DictionaryPair<'_> {
    fn contains(&self, value: &str) -> bool {
        self.secondary.contains_key(value) || self.primary.contains_key(value)
    }
}

fn add_hybrid_ngrams(
    text: &str,
    max_length: usize,
    han_viet: &HashMap<char, String>,
    seeds: &mut HashMap<String, CandidateSeed>,
) {
    let chars: Vec<(usize, char)> = text.char_indices().collect();
    let mut run_start = 0;
    while run_start < chars.len() {
        if !is_chinese(chars[run_start].1, han_viet) {
            run_start += 1;
            continue;
        }
        let mut run_end = run_start + 1;
        while run_end < chars.len() && is_chinese(chars[run_end].1, han_viet) {
            run_end += 1;
        }
        for start in run_start..run_end {
            for length in QT_MIN_LENGTH..=max_length.min(run_end - start) {
                let end = start + length;
                let byte_start = chars[start].0;
                let byte_end = chars
                    .get(end)
                    .map(|(offset, _)| *offset)
                    .unwrap_or(text.len());
                seeds
                    .entry(text[byte_start..byte_end].to_string())
                    .or_default()
                    .add_ngram(byte_start);
            }
        }
        run_start = run_end;
    }
}

fn candidate_facts(
    document: &NameFilterDocument,
    candidate: &str,
    byte_starts: &[usize],
    utf16_offsets: &HashMap<usize, usize>,
    ho_nguoi: &HashMap<String, String>,
    danh_tu: &HashMap<String, String>,
    hau_tu: &HashMap<String, String>,
) -> CandidateFacts {
    let text = document.text();
    let candidate_utf16_length = candidate.encode_utf16().count();
    let ranges = byte_starts
        .iter()
        .filter_map(|byte_start| {
            utf16_offsets.get(byte_start).and_then(|start| {
                document.map_range(CharRange {
                    start: *start,
                    length: candidate_utf16_length,
                })
            })
        })
        .collect();
    let has_surname = prefixes(candidate).any(|prefix| ho_nguoi.contains_key(prefix));
    let has_trigger = byte_starts.iter().any(|byte_start| {
        let prefix = previous_chars(text, *byte_start, 5);
        NAME_TRIGGERS
            .iter()
            .any(|trigger| prefix.ends_with(trigger))
    });
    let person_suffix = PERSON_SUFFIXES
        .iter()
        .any(|suffix| candidate.ends_with(suffix));
    let location_suffix = LOCATION_SUFFIXES
        .iter()
        .any(|suffix| candidate.ends_with(suffix));
    let organization_suffix = ORGANIZATION_SUFFIXES
        .iter()
        .any(|suffix| candidate.ends_with(suffix));
    let dictionary_suffix = suffixes(candidate)
        .any(|suffix| danh_tu.contains_key(suffix) || hau_tu.contains_key(suffix));
    let entity_type = if person_suffix || has_surname {
        NameEntityType::Person
    } else if organization_suffix {
        NameEntityType::Organization
    } else if location_suffix {
        NameEntityType::Location
    } else if dictionary_suffix {
        NameEntityType::Title
    } else {
        NameEntityType::Unknown
    };
    CandidateFacts {
        ranges,
        has_trigger,
        has_surname,
        entity_type,
        has_suffix: person_suffix || location_suffix || organization_suffix || dictionary_suffix,
    }
}

fn prefixes(value: &str) -> impl Iterator<Item = &str> {
    let mut ends: Vec<usize> = value
        .char_indices()
        .skip(1)
        .take(2)
        .map(|(i, _)| i)
        .collect();
    ends.reverse();
    ends.into_iter().map(|end| &value[..end])
}

fn suffixes(value: &str) -> impl Iterator<Item = &str> {
    let mut starts: Vec<usize> = value
        .char_indices()
        .map(|(index, _)| index)
        .rev()
        .take(3)
        .collect();
    starts.reverse();
    starts.into_iter().map(|start| &value[start..])
}

fn previous_chars(text: &str, byte_start: usize, count: usize) -> String {
    text[..byte_start]
        .chars()
        .rev()
        .take(count)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

fn utf16_offsets(text: &str) -> HashMap<usize, usize> {
    let mut offsets = HashMap::new();
    let mut utf16_start = 0;
    for (byte_start, character) in text.char_indices() {
        offsets.insert(byte_start, utf16_start);
        utf16_start += character.len_utf16();
    }
    offsets.insert(text.len(), utf16_start);
    offsets
}

impl CandidateSeed {
    fn add_ngram(&mut self, byte_start: usize) {
        self.from_ngram = true;
        self.byte_starts.push(byte_start);
    }
}

fn qt_score(occurrences: usize, threshold: usize) -> f32 {
    if occurrences < threshold {
        return 0.0;
    }
    (0.58 + (occurrences.saturating_sub(threshold) as f32 * 0.04)).min(0.82)
}

fn hybrid_score(
    occurrences: usize,
    from_jieba: bool,
    facts: &CandidateFacts,
    known: bool,
    exact_vietphrase: bool,
) -> f32 {
    if known {
        return 1.0;
    }
    let mut score = 0.18 + (occurrences.min(4) as f32 * 0.07);
    if from_jieba {
        score += 0.12;
    }
    if facts.has_trigger {
        score += 0.30;
    }
    if facts.has_surname {
        score += 0.30;
    }
    if facts.has_suffix {
        score += 0.20;
    }
    if exact_vietphrase && !facts.has_surname && !facts.has_trigger {
        score -= 0.32;
    }
    score.clamp(0.0, 0.99)
}

fn suggested_name(
    text: &str,
    vietphrase: &HashMap<String, String>,
    han_viet: &HashMap<char, String>,
    prefer_han_viet: bool,
) -> String {
    if !prefer_han_viet {
        if let Some(value) = vietphrase.get(text) {
            if let Some(first) = value.split(['/', '|']).next() {
                return title_words(first);
            }
        }
    }
    text.chars()
        .map(|ch| {
            han_viet
                .get(&ch)
                .map(|value| title_words(value))
                .unwrap_or_else(|| ch.to_string())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_words(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_uppercase().collect::<String>() + chars.as_str()
            })
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn prune_qt_prefixes(mut candidates: Vec<NameCandidate>) -> Vec<NameCandidate> {
    candidates.sort_by_key(|candidate| candidate.text.chars().count());
    let mut kept_prefixes = HashSet::new();
    candidates
        .into_iter()
        .filter(|candidate| {
            let prefix: String = candidate.text.chars().take(2).collect();
            kept_prefixes.insert(prefix)
        })
        .collect()
}

fn prune_weaker_nested_candidates(candidates: Vec<NameCandidate>) -> Vec<NameCandidate> {
    let strengths: HashMap<String, (f32, usize)> = candidates
        .iter()
        .map(|candidate| {
            (
                candidate.text.clone(),
                (candidate.score, candidate.occurrences),
            )
        })
        .collect();
    candidates
        .into_iter()
        .filter(|candidate| {
            let boundaries: Vec<usize> = candidate
                .text
                .char_indices()
                .map(|(index, _)| index)
                .chain(std::iter::once(candidate.text.len()))
                .collect();
            !(2..boundaries.len().saturating_sub(1)).any(|length| {
                let prefix = &candidate.text[..boundaries[length]];
                let suffix = &candidate.text[boundaries[boundaries.len() - 1 - length]..];
                [prefix, suffix].into_iter().any(|nested| {
                    strengths.get(nested).is_some_and(|(score, occurrences)| {
                        *occurrences >= candidate.occurrences && *score >= candidate.score + 0.15
                    })
                })
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Dictionaries;

    fn engine() -> Engine {
        engine_with_ignored("")
    }

    fn engine_with_ignored(ignored: &str) -> Engine {
        Engine::from_dicts(Dictionaries::build_full(
            "萧=tiêu\n炎=viêm\n林=lâm\n动=động\n天=thiên\n城=thành\n名=danh\n为=vi\n叫=khiếu\n张=trương\n三=tam\n走=tẩu\n来=lai\n本=bản\n章=chương\n完=hoàn",
            "",
            "",
            "走来=đi tới",
            "",
            "之城={0} chi thành",
            "萧=Tiêu\n张=Trương",
            "",
            "",
            ignored,
        ))
    }

    #[test]
    fn hybrid_keeps_a_rare_name_after_an_introduction_trigger() {
        let result = engine().filter_names(
            "来人名为萧炎。",
            &NameFilterOptions::default(),
            &NameFilterMemory::default(),
            None,
        );
        let candidate = result
            .candidates
            .iter()
            .find(|candidate| candidate.text == "萧炎")
            .expect("triggered rare name");
        assert_eq!(candidate.suggested, "Tiêu Viêm");
        assert_eq!(candidate.entity_type, NameEntityType::Person);
        assert_eq!(candidate.occurrences, 1);
    }

    #[test]
    fn book_memory_overrides_frequency_and_translation() {
        let memory = NameFilterMemory {
            known_names: HashMap::from([("林动".to_string(), "Lâm Động".to_string())]),
            ..Default::default()
        };
        let result =
            engine().filter_names("林动走来。", &NameFilterOptions::default(), &memory, None);
        let candidate = result
            .candidates
            .iter()
            .find(|candidate| candidate.text == "林动")
            .expect("known name");
        assert!(candidate.known);
        assert_eq!(candidate.score, 1.0);
        assert_eq!(candidate.suggested, "Lâm Động");
    }

    #[test]
    fn book_memory_is_returned_even_after_it_is_added_to_names2() {
        let memory = NameFilterMemory {
            known_names: HashMap::from([("林动".to_string(), "Lâm Động".to_string())]),
            ..Default::default()
        };
        let overrides = DictionaryOverrides::from_sources(crate::DictionarySourceOverrides {
            names2: Some("林动=Lâm Động"),
            ..Default::default()
        });
        let result = engine().filter_names(
            "林动走来。",
            &NameFilterOptions::default(),
            &memory,
            Some(&overrides),
        );
        assert!(result
            .candidates
            .iter()
            .any(|candidate| candidate.text == "林动" && candidate.known));
    }

    #[test]
    fn rejected_book_name_is_suppressed() {
        let memory = NameFilterMemory {
            rejected_names: HashSet::from(["萧炎".to_string()]),
            ..Default::default()
        };
        let result = engine().filter_names(
            "萧炎见到了萧炎。",
            &NameFilterOptions::default(),
            &memory,
            None,
        );
        assert!(result.candidates.iter().all(|value| value.text != "萧炎"));
    }

    #[test]
    fn qt_mode_requires_frequency_threshold() {
        let options = NameFilterOptions {
            mode: NameFilterMode::QtCompatible,
            min_occurrences: 2,
            min_score: 0.5,
            ..Default::default()
        };
        let result = engine().filter_names(
            "萧炎走来。萧炎走来。",
            &options,
            &NameFilterMemory::default(),
            None,
        );
        assert!(result.candidates.iter().any(|value| value.text == "萧炎"));
    }

    #[test]
    fn qt_mode_rejects_an_exact_common_vietphrase() {
        let options = NameFilterOptions {
            mode: NameFilterMode::QtCompatible,
            min_occurrences: 2,
            min_score: 0.5,
            ..Default::default()
        };
        let result =
            engine().filter_names("走来走来", &options, &NameFilterMemory::default(), None);
        assert!(result
            .candidates
            .iter()
            .all(|candidate| candidate.text != "走来"));
    }

    #[test]
    fn ranges_are_utf16_offsets() {
        let memory = NameFilterMemory {
            known_names: HashMap::from([("萧炎".to_string(), "Tiêu Viêm".to_string())]),
            ..Default::default()
        };
        let result = engine().filter_names("😀萧炎", &NameFilterOptions::default(), &memory, None);
        let candidate = result
            .candidates
            .iter()
            .find(|candidate| candidate.text == "萧炎")
            .unwrap();
        assert_eq!(
            candidate.ranges,
            vec![CharRange {
                start: 2,
                length: 2
            }]
        );
    }

    #[test]
    fn ignored_phrases_remove_candidates_and_preserve_original_ranges() {
        let memory = NameFilterMemory {
            known_names: HashMap::from([
                ("萧炎".to_string(), "Tiêu Viêm".to_string()),
                ("林动".to_string(), "Lâm Động".to_string()),
            ]),
            ..Default::default()
        };
        let result = engine_with_ignored("本章萧炎完").filter_names(
            "本章萧炎完。😀林动",
            &NameFilterOptions::default(),
            &memory,
            None,
        );

        assert!(result
            .candidates
            .iter()
            .all(|candidate| candidate.text != "萧炎"));
        let candidate = result
            .candidates
            .iter()
            .find(|candidate| candidate.text == "林动")
            .expect("name outside ignored text");
        assert_eq!(
            candidate.ranges,
            vec![CharRange {
                start: 8,
                length: 2,
            }]
        );
        assert_eq!(result.scanned_characters, 9);
    }

    #[test]
    fn ignored_phrase_overrides_replace_the_engine_default() {
        let memory = NameFilterMemory {
            known_names: HashMap::from([("萧炎".to_string(), "Tiêu Viêm".to_string())]),
            ..Default::default()
        };
        let overrides = DictionaryOverrides::from_sources(crate::DictionarySourceOverrides {
            ignored_chinese_phrases: Some(""),
            ..Default::default()
        });
        let result = engine_with_ignored("本章萧炎完").filter_names(
            "本章萧炎完",
            &NameFilterOptions::default(),
            &memory,
            Some(&overrides),
        );

        let candidate = result
            .candidates
            .iter()
            .find(|candidate| candidate.text == "萧炎")
            .expect("empty request override restores raw text");
        assert_eq!(
            candidate.ranges,
            vec![CharRange {
                start: 2,
                length: 2,
            }]
        );
    }

    #[test]
    fn ignored_phrases_do_not_join_names_across_the_removed_text() {
        let memory = NameFilterMemory {
            known_names: HashMap::from([("萧炎".to_string(), "Tiêu Viêm".to_string())]),
            ..Default::default()
        };
        let result = engine_with_ignored("本章完").filter_names(
            "萧本章完炎",
            &NameFilterOptions::default(),
            &memory,
            None,
        );

        assert!(result
            .candidates
            .iter()
            .all(|candidate| candidate.text != "萧炎"));
    }
}
