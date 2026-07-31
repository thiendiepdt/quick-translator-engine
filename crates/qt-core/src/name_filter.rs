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
/// Occurrence floor that lets a surname-led exact-VietPhrase entry through
/// the hybrid hard reject (protagonist names already present in VietPhrase).
const HIGH_FREQUENCY_OCCURRENCES: usize = 5;
const BOOK_TITLE_SCORE: f32 = 0.90;
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
/// Hardcoded stop list carried over verbatim from QT2025 `LocNameOff.source`.
const QT_STOPWORDS: &[&str] = &[
    "她", "着", "的", "你", "我", "了", "他", "什么", "也", "什", "们", "在", "您", "那", "这",
    "这个", "不过", "尔", "啊", "吧", "一边", "没", "哪个", "就是", "有些", "很", "非常", "还是",
    "再有", "发现", "数", "十几",
];
const CHINESE_NUMBER_CHARS: &[char] = &[
    '零', '一', '二', '三', '四', '五', '六', '七', '八', '九', '十', '百', '千', '万', '亿', '两',
    '〇',
];

fn contains_stopword(text: &str) -> bool {
    QT_STOPWORDS.iter().any(|word| text.contains(word))
}

fn is_chinese_number_char(ch: char) -> bool {
    CHINESE_NUMBER_CHARS.contains(&ch)
}

/// QT2025 `IsChineseNumberSequence`: three or more characters, all numerals.
fn is_chinese_number_sequence(text: &str) -> bool {
    text.chars().count() >= 3 && text.chars().all(is_chinese_number_char)
}

/// Two or more consecutive numeral characters mark n-gram fragments cut out
/// of amounts and dates (`新历一百`), which are never names.
fn has_chinese_number_run(text: &str) -> bool {
    let mut run = 0usize;
    for ch in text.chars() {
        if is_chinese_number_char(ch) {
            run += 1;
            if run >= 2 {
                return true;
            }
        } else {
            run = 0;
        }
    }
    false
}

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
    BookTitle,
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
        match options.mode {
            NameFilterMode::QtCompatible => {
                self.filter_names_qt(document, options, memory, overrides)
            }
            NameFilterMode::Hybrid => {
                self.filter_names_hybrid(document, options, memory, overrides)
            }
        }
    }

    /// Faithful port of QT2025 `LocNameOff.LocNameQT`, with book memory kept
    /// as an engine-level extension on top (rejects suppress, known values win).
    fn filter_names_qt(
        &self,
        document: &NameFilterDocument,
        options: &NameFilterOptions,
        memory: &NameFilterMemory,
        overrides: Option<&DictionaryOverrides>,
    ) -> NameFilterResult {
        let text = document.text();
        let utf16_offsets = utf16_offsets(text);
        let jieba = jieba();
        let names = dictionary_pair(
            overrides.and_then(|value| value.names.as_ref()),
            overrides.and_then(|value| value.names2.as_ref()),
            &self.dicts.primary_names,
            &self.dicts.secondary_names,
        );
        let names2 = names.secondary;
        let ho_nguoi = overrides
            .and_then(|value| value.ho_nguoi.as_ref())
            .unwrap_or(&self.dicts.ho_nguoi);
        let danh_tu = overrides
            .and_then(|value| value.danh_tu.as_ref())
            .unwrap_or(&self.dicts.danh_tu);
        let hau_tu = overrides
            .and_then(|value| value.hau_tu.as_ref())
            .unwrap_or(&self.dicts.hau_tu);
        let in_vietphrase =
            |term: &str| names.contains(term) || self.dicts.only_vietphrase.contains_key(term);

        let tokens = jieba.cut(text, true);
        let segments: Vec<&str> = tokens.iter().map(|token| token.word).collect();
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for token in &tokens {
            let length = token.word.chars().count();
            if (QT_MIN_LENGTH..=QT_MAX_LENGTH).contains(&length)
                && token
                    .word
                    .chars()
                    .all(|ch| is_chinese(ch, &self.dicts.han_viet))
            {
                *counts.entry(token.word).or_default() += 1;
            }
        }
        let threshold = options.min_occurrences.max(1);
        let phrases: HashSet<String> = counts
            .iter()
            .filter(|(word, count)| **count >= threshold && !contains_stopword(word))
            .map(|(word, _)| (*word).to_string())
            .collect();
        let phrases = keep_shortest_per_prefix(phrases);
        let mut phrases = filter_names2_chain(phrases, text, names2);
        if !names2.is_empty() {
            phrases = filter_names2_chain(phrases, text, names2);
        }
        validate_and_merge_terms(
            &mut phrases,
            &segments,
            &self.dicts.only_vietphrase,
            &in_vietphrase,
            danh_tu,
            ho_nguoi,
        );

        let mut ordered: Vec<(String, bool)> =
            phrases.into_iter().map(|phrase| (phrase, false)).collect();
        ordered.sort();
        for title in book_titles(text) {
            if ordered.iter().all(|(existing, _)| *existing != title) {
                ordered.push((title, true));
            }
        }

        let mut candidates = Vec::new();
        for (candidate, is_book_title) in ordered {
            // Book-title keys are emitted the way the translation pipeline
            // standardizes them (`《 X 》` with spaces, as QT2025 does via
            // StandardizeInput) — otherwise the Names2 entry never matches.
            let display = if is_book_title {
                self.standardized_translation_key(&candidate)
            } else {
                candidate.clone()
            };
            let known_value = memory.known_names.get(&display);
            if memory.rejected_names.contains(&display) {
                continue;
            }
            let is_known = known_value.is_some();
            if is_known && !options.include_known {
                continue;
            }
            if !is_known
                && !is_book_title
                && (in_vietphrase(&candidate)
                    || is_part_of_names2(&candidate, names2, ho_nguoi)
                    || is_chinese_number_sequence(&candidate))
            {
                continue;
            }
            let byte_starts: Vec<usize> = text
                .match_indices(candidate.as_str())
                .map(|(index, _)| index)
                .collect();
            let facts = candidate_facts(
                document,
                &candidate,
                &byte_starts,
                &utf16_offsets,
                ho_nguoi,
                danh_tu,
                hau_tu,
            );
            let occurrences = facts.ranges.len();
            if occurrences == 0 {
                continue;
            }
            let suggested = known_value.cloned().unwrap_or_else(|| {
                qt_formatted_value(
                    &display,
                    danh_tu,
                    &self.dicts.vietphrase_one_meaning,
                    &self.dicts.han_viet,
                )
            });
            if !is_known && !is_book_title && !suggested.contains(' ') {
                continue;
            }
            let mut sources = vec![if is_book_title {
                NameCandidateSource::BookTitle
            } else {
                NameCandidateSource::QtJieba
            }];
            let mut reasons = vec![if is_book_title {
                "tựa sách trong 《》".to_string()
            } else {
                "lọc theo thuật toán QT2025".to_string()
            }];
            if is_known {
                sources.push(NameCandidateSource::BookMemory);
                reasons.push("đã được duyệt trong bộ nhớ truyện".to_string());
            }
            if occurrences > 1 {
                reasons.push(format!("xuất hiện {occurrences} lần"));
            }
            candidates.push(NameCandidate {
                text: display,
                suggested,
                entity_type: if is_book_title {
                    NameEntityType::Title
                } else {
                    facts.entity_type
                },
                score: if is_known {
                    1.0
                } else {
                    qt_score(occurrences, threshold.min(occurrences))
                },
                occurrences,
                ranges: facts.ranges,
                reasons,
                sources,
                known: is_known,
            });
        }

        candidates.sort_by_key(|candidate| {
            candidate
                .ranges
                .first()
                .map(|range| range.start)
                .unwrap_or(usize::MAX)
        });
        candidates.truncate(options.max_candidates.clamp(1, 1_000));
        NameFilterResult {
            candidates,
            scanned_characters: document.scanned_characters,
        }
    }

    fn filter_names_hybrid(
        &self,
        document: &NameFilterDocument,
        options: &NameFilterOptions,
        memory: &NameFilterMemory,
        overrides: Option<&DictionaryOverrides>,
    ) -> NameFilterResult {
        let text = document.text();
        let max_length = options
            .max_name_length
            .clamp(QT_MIN_LENGTH, HYBRID_MAX_LENGTH);
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
            let is_known = known_value.is_some();
            if is_known && !options.include_known {
                continue;
            }
            let exact_vietphrase = self.dicts.only_vietphrase.contains_key(&candidate);
            let jieba_common = jieba.has_word(&candidate);
            let proper_remainder = facts.has_surname
                && surname_proper_remainder(&candidate, ho_nguoi, &self.dicts.only_vietphrase);
            if !is_known {
                // QT2025-style hard rejects: function words, amounts/dates and
                // exact common VietPhrase entries are never names. An exact
                // VietPhrase entry survives only with a naming context or a
                // surname backed by OOV/high frequency (protagonists already
                // present in VietPhrase, e.g. 李顺).
                if contains_stopword(&candidate)
                    || is_chinese_number_sequence(&candidate)
                    || has_chinese_number_run(&candidate)
                {
                    continue;
                }
                if exact_vietphrase
                    && !facts.has_trigger
                    && !(facts.has_surname
                        && (!jieba_common || occurrences >= HIGH_FREQUENCY_OCCURRENCES))
                {
                    continue;
                }
                // A raw n-gram without naming trigger or entity suffix needs
                // real evidence: a compound surname, a surname followed by a
                // known proper noun (姜 + 太阿), or repeated occurrences.
                // Single-char surnames prove nothing at n-gram density given
                // how permissive HoNguoi is.
                if !facts.has_trigger && !facts.has_suffix && !seed.from_jieba {
                    let prefix2: String = candidate.chars().take(2).collect();
                    let compound_surname =
                        candidate.chars().count() > 2 && ho_nguoi.contains_key(&prefix2);
                    if !compound_surname && !proper_remainder && occurrences < 3 {
                        continue;
                    }
                }
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

            let score = hybrid_score(
                occurrences,
                seed.from_jieba,
                &facts,
                is_known,
                jieba_common,
                proper_remainder,
            );
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
                    danh_tu,
                    &self.dicts.vietphrase_one_meaning,
                )
            });
            if !is_known && !suggested.contains(' ') {
                continue;
            }
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

        candidates = prune_weaker_nested_candidates(candidates);
        for title in book_titles(text) {
            let display = self.standardized_translation_key(&title);
            if candidates.iter().any(|candidate| candidate.text == display)
                || memory.rejected_names.contains(&display)
            {
                continue;
            }
            let byte_starts: Vec<usize> = text
                .match_indices(title.as_str())
                .map(|(index, _)| index)
                .collect();
            let facts = candidate_facts(
                document,
                &title,
                &byte_starts,
                &utf16_offsets,
                ho_nguoi,
                danh_tu,
                hau_tu,
            );
            if facts.ranges.is_empty() {
                continue;
            }
            let known_value = memory.known_names.get(&display);
            let is_known = known_value.is_some();
            if is_known && !options.include_known {
                continue;
            }
            let occurrences = facts.ranges.len();
            candidates.push(NameCandidate {
                suggested: known_value.cloned().unwrap_or_else(|| {
                    qt_formatted_value(
                        &display,
                        danh_tu,
                        &self.dicts.vietphrase_one_meaning,
                        &self.dicts.han_viet,
                    )
                }),
                text: display,
                entity_type: NameEntityType::Title,
                score: if is_known { 1.0 } else { BOOK_TITLE_SCORE },
                occurrences,
                ranges: facts.ranges,
                reasons: vec!["tựa sách trong 《》".to_string()],
                sources: vec![NameCandidateSource::BookTitle],
                known: is_known,
            });
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

    /// Render `raw` exactly the way the translation pipeline standardizes
    /// input (QT2025 `StandardizeInput`), so a dictionary key built from it
    /// will match during translation (`《X》` becomes `《 X 》`).
    fn standardized_translation_key(&self, raw: &str) -> String {
        let standardized = self.standardizer.standardize(raw);
        standardized
            .chars
            .iter()
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Suggest a Vietnamese value for a model-provided entity span.
    pub fn suggest_name(&self, text: &str) -> String {
        suggested_name(
            text,
            &self.dicts.only_vietphrase,
            &self.dicts.han_viet,
            true,
            &self.dicts.danh_tu,
            &self.dicts.vietphrase_one_meaning,
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
    jieba_common: bool,
    proper_remainder: bool,
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
    if proper_remainder {
        score += 0.25;
    }
    // For real segmenter tokens, absence from the lexicon is a strong
    // proper-noun signal; presence marks common vocabulary. Raw n-grams are
    // almost always OOV, so absence proves nothing for them.
    if jieba_common {
        score -= 0.12;
    } else if from_jieba {
        score += 0.10;
    }
    score.clamp(0.0, 0.99)
}

fn suggested_name(
    text: &str,
    vietphrase: &HashMap<String, String>,
    han_viet: &HashMap<char, String>,
    prefer_han_viet: bool,
    danh_tu: &HashMap<String, String>,
    vietphrase_one_meaning: &HashMap<String, String>,
) -> String {
    if text.chars().count() > 2 && ends_with_any_key(text, danh_tu) {
        return title_case_with_danh_tu(text, danh_tu, vietphrase_one_meaning, han_viet);
    }
    if !prefer_han_viet {
        if let Some(value) = vietphrase.get(text) {
            if let Some(first) = value.split(['/', '|']).next() {
                return title_words(first);
            }
        }
    }
    title_case_han_viet(text, han_viet)
}

fn title_case_han_viet(text: &str, han_viet: &HashMap<char, String>) -> String {
    text.chars()
        .map(|ch| {
            han_viet
                .get(&ch)
                .map(|value| title_words(value))
                .unwrap_or_else(|| ch.to_string())
        })
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Port of QT2025 `LocNameOff.BuildFormattedHanViet` (checkVietPhrase=true).
fn qt_formatted_value(
    candidate: &str,
    danh_tu: &HashMap<String, String>,
    vietphrase_one_meaning: &HashMap<String, String>,
    han_viet: &HashMap<char, String>,
) -> String {
    if candidate.trim().is_empty() {
        return String::new();
    }
    if candidate.chars().count() > 2 && ends_with_any_key(candidate, danh_tu) {
        return title_case_with_danh_tu(candidate, danh_tu, vietphrase_one_meaning, han_viet);
    }
    if let Some(value) = vietphrase_one_meaning.get(candidate) {
        let value = value.trim();
        if !value.is_empty() {
            return value.to_string();
        }
    }
    title_case_han_viet(candidate, han_viet)
}

/// Port of QT2025 `LocNameOff.TitleCaseWithDictionary`: DanhTu entries are
/// `suffix={0} template` patterns applied to the translated prefix.
fn title_case_with_danh_tu(
    candidate: &str,
    danh_tu: &HashMap<String, String>,
    vietphrase_one_meaning: &HashMap<String, String>,
    han_viet: &HashMap<char, String>,
) -> String {
    if let Some(value) = danh_tu.get(candidate) {
        return value.replace("{0}", "").trim().to_string();
    }
    let mut keys: Vec<&String> = danh_tu.keys().collect();
    keys.sort_by(|left, right| {
        right
            .chars()
            .count()
            .cmp(&left.chars().count())
            .then_with(|| left.cmp(right))
    });
    for key in keys {
        if key.is_empty() || !candidate.ends_with(key.as_str()) {
            continue;
        }
        let template = &danh_tu[key.as_str()];
        let prefix = candidate[..candidate.len() - key.len()].trim();
        let translated = if prefix.chars().count() <= 2 {
            title_case_han_viet(prefix, han_viet)
        } else if let Some(value) = vietphrase_one_meaning.get(prefix) {
            if is_title_case(value) {
                value.clone()
            } else {
                title_words(value)
            }
        } else {
            title_case_han_viet(prefix, han_viet)
        };
        if template.contains("{0}") {
            return template.replace("{0}", &translated).trim().to_string();
        }
        return format!("{translated} {template}").trim().to_string();
    }
    title_words(candidate)
}

fn is_title_case(value: &str) -> bool {
    if value.trim().is_empty() {
        return false;
    }
    value.split_whitespace().all(|word| {
        word.chars()
            .find(|ch| ch.is_alphabetic())
            .is_none_or(|first| first.is_uppercase())
    })
}

/// Surname followed by a remainder whose first VietPhrase meaning is written
/// in Title Case — the pattern of a full name built on a known proper noun
/// (姜 + 太阿=Thái A).
fn surname_proper_remainder(
    candidate: &str,
    ho_nguoi: &HashMap<String, String>,
    vietphrase: &HashMap<String, String>,
) -> bool {
    prefixes(candidate).any(|prefix| {
        if !ho_nguoi.contains_key(prefix) {
            return false;
        }
        let remainder = &candidate[prefix.len()..];
        if remainder.is_empty() {
            return false;
        }
        vietphrase
            .get(remainder)
            .and_then(|value| value.split(['/', '|']).next())
            .is_some_and(is_title_case)
    })
}

/// `MatchAnyStartEnd(text, keys, isPrefix: false)` from QT2025.
fn ends_with_any_key(text: &str, keys: &HashMap<String, String>) -> bool {
    keys.keys()
        .any(|key| text != key && text.ends_with(key.as_str()))
}

/// `MatchAnyStartEnd(text, keys, isPrefix: true)` from QT2025.
fn starts_with_any_key(text: &str, keys: &HashMap<String, String>) -> bool {
    keys.keys()
        .any(|key| text != key && text.starts_with(key.as_str()))
}

/// QT2025 `FilterUnnecessaryPhrasesOptimized`: within each two-character
/// prefix group only the shortest phrase survives.
fn keep_shortest_per_prefix(phrases: HashSet<String>) -> HashSet<String> {
    let mut groups: HashMap<String, (usize, String)> = HashMap::new();
    for phrase in phrases {
        let length = phrase.chars().count();
        if length < 2 {
            continue;
        }
        let prefix: String = phrase.chars().take(2).collect();
        match groups.entry(prefix) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let (best_length, best) = entry.get();
                if length < *best_length || (length == *best_length && phrase < *best) {
                    entry.insert((length, phrase));
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert((length, phrase));
            }
        }
    }
    groups.into_values().map(|(_, phrase)| phrase).collect()
}

/// QT2025 `FilterUnnecessaryItems`: drop a phrase when a Names2 key chains
/// into it (key ends with the phrase's first character and the combined
/// string occurs in the chapter).
fn filter_names2_chain(
    phrases: HashSet<String>,
    text: &str,
    names2: &HashMap<String, String>,
) -> HashSet<String> {
    let mut removed: HashSet<String> = HashSet::new();
    for phrase in &phrases {
        let Some(first) = phrase.chars().next() else {
            continue;
        };
        for key in names2.keys() {
            if key == phrase || !phrase.starts_with(key.as_str()) {
                continue;
            }
            if key.ends_with(first) {
                let mut value = key.clone();
                value.push_str(&phrase[first.len_utf8()..]);
                if text.contains(&value) {
                    removed.insert(phrase.clone());
                    break;
                }
            }
        }
    }
    phrases
        .into_iter()
        .filter(|phrase| !removed.contains(phrase))
        .collect()
}

/// QT2025 `IsPartOfAnyPhraseInDictionary` against Names2.
fn is_part_of_names2(
    candidate: &str,
    names2: &HashMap<String, String>,
    ho_nguoi: &HashMap<String, String>,
) -> bool {
    if candidate.chars().count() <= 1 {
        return false;
    }
    for key in names2.keys() {
        if key.chars().count() <= 1 {
            continue;
        }
        if key.starts_with(candidate) || candidate.starts_with(key.as_str()) {
            return true;
        }
        if brackets_match(candidate, key) || brackets_match(key, candidate) {
            return true;
        }
        if candidate.contains(key.as_str()) && !starts_with_any_key(candidate, ho_nguoi) {
            return true;
        }
    }
    false
}

fn brackets_match(with_brackets: &str, to_check: &str) -> bool {
    let Some(open) = with_brackets.find('《') else {
        return false;
    };
    let Some(close) = with_brackets.find('》') else {
        return false;
    };
    if close <= open {
        return false;
    }
    let inner = with_brackets[open + '《'.len_utf8()..close].trim();
    to_check.contains(inner)
}

/// QT2025 `ValidateAndMergeTerms`: two-character terms must merge with a
/// following DanhTu segment or start with a surname; four-character terms
/// must be `<prefix><DanhTu>` pairs that are not two common VietPhrase words.
fn validate_and_merge_terms(
    terms: &mut HashSet<String>,
    segments: &[&str],
    only_vietphrase: &HashMap<String, String>,
    in_vietphrase: &dyn Fn(&str) -> bool,
    danh_tu: &HashMap<String, String>,
    ho_nguoi: &HashMap<String, String>,
) {
    let mut to_remove: HashSet<String> = HashSet::new();
    let mut to_add: HashSet<String> = HashSet::new();
    terms.retain(|term| !term.trim().is_empty());
    for term in terms.iter() {
        let length = term.chars().count();
        if length == 2 && !only_vietphrase.contains_key(term) {
            let has_surname = starts_with_any_key(term, ho_nguoi);
            let mut merged = false;
            for window in segments.windows(2) {
                if window[0] == term && danh_tu.contains_key(window[1]) {
                    to_add.insert(format!("{term}{}", window[1]));
                    merged = true;
                }
            }
            if !merged && !has_surname {
                to_remove.insert(term.clone());
            } else {
                to_add.insert(term.clone());
            }
        } else if length == 4 {
            let boundary = term
                .char_indices()
                .nth(2)
                .map(|(index, _)| index)
                .unwrap_or(term.len());
            let left = &term[..boundary];
            let right = &term[boundary..];
            if !danh_tu.contains_key(right) || (in_vietphrase(left) && in_vietphrase(right)) {
                to_remove.insert(term.clone());
            } else {
                to_add.insert(term.clone());
            }
        }
    }
    for term in to_remove {
        terms.remove(&term);
    }
    terms.extend(to_add);
}

/// QT2025 book-title extraction (`《.*?》`, no newline inside).
fn book_titles(text: &str) -> Vec<String> {
    let mut titles = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find('《') {
        let after_open = &rest[open + '《'.len_utf8()..];
        let Some(close) = after_open.find('》') else {
            break;
        };
        let inner = &after_open[..close];
        rest = &after_open[close + '》'.len_utf8()..];
        if inner.contains('\n') || inner.contains('\r') {
            continue;
        }
        let title = format!("《{inner}》");
        if !titles.contains(&title) {
            titles.push(title);
        }
    }
    titles
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

    fn qt2025_engine() -> Engine {
        Engine::from_dicts(Dictionaries::build_full(
            "冷=lãnh\n涯=nhai\n郡=quận\n姜=khương\n太=thái\n阿=a\n幽=u\n暗=ám\n山=sơn\n释=thích\n帝=đế\n书=thư\n洞=động\n里=lý\n转=chuyển\n身=thân\n他=tha\n看=khán\n着=trứ\n无=vô\n比=bỉ\n士=sĩ\n兵=binh\n穴=huyệt",
            "",
            "",
            "幽暗=u ám\n太阿=Thái A",
            "",
            "郡={0} quận",
            "冷=Lãnh\n姜=Khương\n幽=U",
            "",
            "",
            "",
        ))
    }

    #[test]
    fn qt_mode_rejects_exact_vietphrase_even_with_surname_prefix() {
        let options = NameFilterOptions {
            mode: NameFilterMode::QtCompatible,
            min_occurrences: 1,
            ..Default::default()
        };
        let result = qt2025_engine().filter_names(
            "幽暗洞穴，幽暗无比。",
            &options,
            &NameFilterMemory::default(),
            None,
        );
        assert!(result
            .candidates
            .iter()
            .all(|candidate| candidate.text != "幽暗"));
    }

    #[test]
    fn qt_mode_merges_two_char_surname_term_with_danh_tu_segment() {
        let options = NameFilterOptions {
            mode: NameFilterMode::QtCompatible,
            min_occurrences: 1,
            ..Default::default()
        };
        let result = qt2025_engine().filter_names(
            "冷涯郡士兵。",
            &options,
            &NameFilterMemory::default(),
            None,
        );
        let candidate = result
            .candidates
            .iter()
            .find(|candidate| candidate.text == "冷涯郡")
            .expect("merged DanhTu candidate");
        assert_eq!(candidate.suggested, "Lãnh Nhai quận");
    }

    #[test]
    fn qt_mode_extracts_book_titles_in_brackets() {
        let options = NameFilterOptions {
            mode: NameFilterMode::QtCompatible,
            min_occurrences: 1,
            ..Default::default()
        };
        let result = qt2025_engine().filter_names(
            "他看着《释帝书》。",
            &options,
            &NameFilterMemory::default(),
            None,
        );
        let candidate = result
            .candidates
            .iter()
            .find(|candidate| candidate.text == "《 释帝书 》")
            .expect("book title candidate in standardized translation form");
        assert_eq!(candidate.entity_type, NameEntityType::Title);
        assert!(candidate.sources.contains(&NameCandidateSource::BookTitle));
    }

    #[test]
    fn qt_mode_orders_candidates_by_first_occurrence() {
        let options = NameFilterOptions {
            mode: NameFilterMode::QtCompatible,
            min_occurrences: 1,
            ..Default::default()
        };
        let result = qt2025_engine().filter_names(
            "姜太阿走向冷涯郡。",
            &options,
            &NameFilterMemory::default(),
            None,
        );
        let positions: Vec<usize> = result
            .candidates
            .iter()
            .filter_map(|candidate| candidate.ranges.first().map(|range| range.start))
            .collect();
        let mut sorted = positions.clone();
        sorted.sort_unstable();
        assert_eq!(positions, sorted);
    }

    #[test]
    fn hybrid_rejects_common_vietphrase_word_despite_surname_prefix() {
        let result = qt2025_engine().filter_names(
            "幽暗洞穴里，幽暗无比。",
            &NameFilterOptions::default(),
            &NameFilterMemory::default(),
            None,
        );
        assert!(result
            .candidates
            .iter()
            .all(|candidate| candidate.text != "幽暗"));
    }

    #[test]
    fn hybrid_keeps_surname_followed_by_known_proper_noun() {
        let result = qt2025_engine().filter_names(
            "姜太阿转身。",
            &NameFilterOptions::default(),
            &NameFilterMemory::default(),
            None,
        );
        let candidate = result
            .candidates
            .iter()
            .find(|candidate| candidate.text == "姜太阿")
            .expect("surname + proper noun candidate");
        assert_eq!(candidate.suggested, "Khương Thái A");
    }

    #[test]
    fn hybrid_rejects_candidates_containing_stopwords() {
        let result = qt2025_engine().filter_names(
            "幽暗的洞穴。幽暗的洞穴。",
            &NameFilterOptions::default(),
            &NameFilterMemory::default(),
            None,
        );
        assert!(result
            .candidates
            .iter()
            .all(|candidate| !candidate.text.contains('的')));
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
