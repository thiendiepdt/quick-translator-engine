//! Tìm & thay trong bản dịch đã chốt (out/) của một truyện: tìm theo chữ (không regex), tuỳ chọn nguyên từ / phân
//! biệt hoa thường / chỉ đoạn mà raw cùng vị trí có một chuỗi / phạm vi chương. Thay mọi chỗ khớp trừ các chỗ
//! người dùng bỏ chọn; lưu bản gốc các chương bị sửa vào `work/replace-undo.json` để hoàn tác lần thay gần nhất.
//!
//! Tìm cả trong hồ sơ truyện — glossary và các ô chữ của story.json, mốc xưng hô và đại từ lời kể của cast.json,
//! sổ tiêu đề titles.json: đổi tên trong chương mà hồ sơ giữ tên cũ thì các chương dịch sau quay về tên cũ. Hồ sơ
//! không gắn với chương nên không theo phạm vi chương; lọc theo raw thì so với khoá Hán của mục.
//!
//! Nhanh trên truyện nghìn chương: lọc cả chương bằng tìm chuỗi trước khi so từng ký tự, chỉ đọc raw khi lọc theo
//! raw hoặc cần ngữ cảnh cho chỗ được hiện, và quét các chương song song.

use crate::backfill::translated_chapters;
use crate::cast::{cast_to_json, load_cast, Cast};
use crate::error::{CoreError, Result};
use crate::paragraphs::paragraphs_of;
use crate::story::{natural_chapter_compare, StoryConfig};
use crate::story_fs::{load_story_config, now_ms, read_raw_chapter, read_text, write_atomic, StoryPaths};
use crate::titles::{load_titles, titles_to_json, TitleBook};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Số ký tự ngữ cảnh mỗi bên chỗ khớp.
const CONTEXT_CHARS: usize = 50;
/// Raw của đoạn hiện tối đa chừng này ký tự.
const RAW_CHARS: usize = 120;
const UNDO_FILE: &str = "replace-undo.json";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FindQuery {
    pub find: String,
    pub replace: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    /// Chỉ đoạn mà raw cùng vị trí có chuỗi này (rỗng = mọi đoạn).
    pub raw_filter: String,
    /// Phạm vi chương theo id, gồm cả hai đầu; None = không giới hạn.
    pub from: Option<String>,
    pub to: Option<String>,
}

/// Một chỗ khớp: `occurrence` là thứ tự của nó trong đoạn — cùng `chapter` + `paragraph` thành mã để bỏ chọn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HitRef {
    pub chapter: String,
    pub paragraph: usize,
    pub occurrence: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindHit {
    pub chapter: String,
    pub paragraph: usize,
    pub occurrence: usize,
    pub before: String,
    pub matched: String,
    pub after: String,
    /// Raw của đúng đoạn đó (khi căn được đoạn).
    pub raw: Option<String>,
}

/// Một chỗ khớp trong hồ sơ truyện. `field` là ô chứa nó:
/// - story.json: `glossary.<nhóm>`, `signaturePhrases`, `protagonist`, `summary`, `voice`, `toneRules`, `avoid`,
///   `customPrompt`;
/// - cast.json: `addressing` (mốc đổi xưng hô), `narration` (đại từ lời kể);
/// - titles.json: `titles`.
///
/// `key` là khoá Hán của mục (rỗng ở ô chữ trơn), `index` là dòng thứ mấy (tone rules, cần tránh) hoặc mốc thứ mấy
/// của cặp xưng hô; cùng `occurrence` thành mã để bỏ chọn.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRef {
    pub field: String,
    pub key: String,
    pub index: usize,
    pub occurrence: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileHit {
    #[serde(flatten)]
    pub at: ProfileRef,
    pub before: String,
    pub matched: String,
    pub after: String,
    /// Mốc xưng hô: chương bắt đầu áp.
    pub chapter: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterCount {
    pub chapter: String,
    pub count: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FindResult {
    pub total: usize,
    /// Mọi chương có chỗ khớp (đủ, không cắt) — để đếm số chỗ đang chọn khi danh sách chỗ bị cắt.
    pub chapters: Vec<ChapterCount>,
    /// Tối đa `limit` chỗ đầu tiên.
    pub hits: Vec<FindHit>,
    /// Chương có chữ cần tìm nhưng bỏ qua vì có lọc theo raw mà số đoạn raw ≠ bản dịch.
    pub skipped: usize,
    /// Mọi chỗ khớp trong hồ sơ (không cắt — mỗi mục chỉ vài chữ).
    pub profile: Vec<ProfileHit>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceOutcome {
    pub chapters: usize,
    pub occurrences: usize,
    /// Số chỗ đã thay trong hồ sơ.
    pub profile: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UndoChapter {
    id: String,
    original: String,
    written: String,
}

/// Một file hồ sơ bị sửa: `name` là tên file trong thư mục truyện (`story.json`, `cast.json`, `titles.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UndoFile {
    name: String,
    original: String,
    written: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UndoRecord {
    find: String,
    replace: String,
    occurrences: usize,
    created_at: u64,
    chapters: Vec<UndoChapter>,
    /// Bản ghi của app cũ chưa có hai field này.
    #[serde(default)]
    profile: usize,
    #[serde(default)]
    files: Vec<UndoFile>,
}

/// Lần thay có thể hoàn tác (không kèm nội dung file).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoInfo {
    pub find: String,
    pub replace: String,
    pub chapters: usize,
    pub occurrences: usize,
    /// Số chỗ đã thay trong hồ sơ.
    pub profile: usize,
    pub created_at: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoOutcome {
    pub restored: usize,
    /// Chương đã bị sửa tiếp sau lần thay — giữ nguyên, không hoàn tác.
    pub conflicts: Vec<String>,
    /// File hồ sơ trả về bản trước lần thay.
    pub profile_restored: usize,
    /// File hồ sơ đã bị sửa tiếp sau lần thay (học glossary, lưu hồ sơ) — giữ nguyên.
    pub profile_conflicts: Vec<String>,
}

fn fold(c: char, case_sensitive: bool) -> char {
    if case_sensitive {
        c
    } else {
        c.to_lowercase().next().unwrap_or(c)
    }
}

/// Vị trí (theo ký tự) các chỗ khớp không chồng nhau. Nguyên từ: ký tự kề hai đầu không được là chữ/số — chỉ xét
/// đầu nào của chuỗi tìm là chữ/số (tìm "ân" không dính "nhân", tìm "“Ân" vẫn khớp sau dấu cách).
fn match_positions(line: &[char], needle: &[char], query: &FindQuery) -> Vec<usize> {
    let n = needle.len();
    let folded: Vec<char> = needle.iter().map(|c| fold(*c, query.case_sensitive)).collect();
    let word = |c: char| c.is_alphanumeric();
    let mut positions = Vec::new();
    let mut i = 0;
    while n > 0 && i + n <= line.len() {
        let same = (0..n).all(|j| fold(line[i + j], query.case_sensitive) == folded[j]);
        let bounded = !query.whole_word
            || ((!word(needle[0]) || i == 0 || !word(line[i - 1]))
                && (!word(needle[n - 1]) || i + n == line.len() || !word(line[i + n])));
        if same && bounded {
            positions.push(i);
            i += n;
        } else {
            i += 1;
        }
    }
    positions
}

fn in_range(id: &str, query: &FindQuery) -> bool {
    let after_from = query.from.as_deref().is_none_or(|from| natural_chapter_compare(id, from) != Ordering::Less);
    let before_to = query.to.as_deref().is_none_or(|to| natural_chapter_compare(id, to) != Ordering::Greater);
    after_from && before_to
}

/// Chương có chỗ khớp: các dòng (giữ xuống dòng), vị trí dòng không rỗng = đoạn, chỗ khớp theo đoạn.
struct Scanned {
    id: String,
    path: PathBuf,
    text: String,
    lines: Vec<String>,
    content: Vec<usize>,
    /// Raw từng đoạn — chỉ đọc sẵn khi có lọc theo raw; không thì đọc khi cần ngữ cảnh.
    raw: Option<Vec<String>>,
    /// (đoạn, vị trí theo ký tự các chỗ khớp trong đoạn).
    matches: Vec<(usize, Vec<usize>)>,
}

enum Scan {
    NoMatch,
    /// Có chữ cần tìm nhưng lọc theo raw mà số đoạn raw ≠ bản dịch.
    Skipped,
    Found(Scanned),
}

fn load_raw(paths: &StoryPaths, id: &str, paragraphs: usize) -> Option<Vec<String>> {
    read_raw_chapter(paths, id).ok().map(|raw| paragraphs_of(&raw)).filter(|raw| raw.len() == paragraphs)
}

/// Gập hoa/thường để lọc nhanh cả chương / cả dòng trước khi so từng ký tự.
fn folded(text: &str, query: &FindQuery) -> String {
    if query.case_sensitive {
        text.to_string()
    } else {
        text.to_lowercase()
    }
}

fn scan_chapter(paths: &StoryPaths, id: &str, path: &Path, query: &FindQuery, needle: &[char], quick: &str) -> Result<Scan> {
    let text = read_text(path)?;
    if !folded(&text, query).contains(quick) {
        return Ok(Scan::NoMatch);
    }
    let lines: Vec<String> = text.split_inclusive('\n').map(str::to_string).collect();
    let content: Vec<usize> = lines.iter().enumerate().filter(|(_, l)| !l.trim().is_empty()).map(|(i, _)| i).collect();
    let raw = if query.raw_filter.is_empty() {
        None
    } else {
        match load_raw(paths, id, content.len()) {
            Some(raw) => Some(raw),
            None => return Ok(Scan::Skipped),
        }
    };
    let mut matches = Vec::new();
    for (paragraph, &line) in content.iter().enumerate() {
        if raw.as_ref().is_some_and(|raw| !raw[paragraph].contains(&query.raw_filter)) {
            continue;
        }
        let body = lines[line].trim_end_matches(['\r', '\n']);
        if !folded(body, query).contains(quick) {
            continue;
        }
        let chars: Vec<char> = body.chars().collect();
        let positions = match_positions(&chars, needle, query);
        if !positions.is_empty() {
            matches.push((paragraph, positions));
        }
    }
    if matches.is_empty() {
        return Ok(Scan::NoMatch);
    }
    Ok(Scan::Found(Scanned { id: id.to_string(), path: path.to_path_buf(), text, lines, content, raw, matches }))
}

/// Quét mọi chương trong phạm vi, song song theo khối, trả về đúng thứ tự chương.
fn scan(paths: &StoryPaths, query: &FindQuery) -> Result<Vec<Scan>> {
    let needle: Vec<char> = query.find.chars().collect();
    let quick = folded(&query.find, query);
    let list: Vec<(String, PathBuf)> =
        translated_chapters(paths).into_iter().filter(|(id, _)| in_range(id, query)).collect();
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(1, 8);
    let size = list.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let workers: Vec<_> = list
            .chunks(size)
            .map(|part| {
                let (needle, quick) = (&needle, &quick);
                scope.spawn(move || {
                    part.iter()
                        .map(|(id, path)| scan_chapter(paths, id, path, query, needle, quick))
                        .collect::<Result<Vec<_>>>()
                })
            })
            .collect();
        let mut all = Vec::with_capacity(list.len());
        for worker in workers {
            all.extend(worker.join().map_err(|_| CoreError::Internal("luồng quét chương bị lỗi".into()))??);
        }
        Ok(all)
    })
}

fn snippet(chars: &[char]) -> String {
    chars.iter().collect()
}

/// Ngữ cảnh của chỗ khớp tại `at`: (trước, chữ khớp, sau), mỗi bên tối đa `CONTEXT_CHARS` ký tự, cắt thì có "…".
fn context(chars: &[char], at: usize, width: usize) -> (String, String, String) {
    let start = at.saturating_sub(CONTEXT_CHARS);
    let end = (at + width + CONTEXT_CHARS).min(chars.len());
    let before = format!("{}{}", if start > 0 { "…" } else { "" }, snippet(&chars[start..at]).trim_start());
    let after = format!("{}{}", snippet(&chars[at + width..end]).trim_end(), if end < chars.len() { "…" } else { "" });
    (before, snippet(&chars[at..at + width]), after)
}

/// Một ô chữ của hồ sơ — dùng chung cho tìm (chỉ đọc) và thay (sửa tại chỗ). Các trường như `ProfileRef`.
struct Slot<'a> {
    field: String,
    key: String,
    index: usize,
    chapter: Option<String>,
    text: &'a mut String,
}

fn slot<'a>(field: impl Into<String>, key: &str, index: usize, text: &'a mut String) -> Slot<'a> {
    Slot { field: field.into(), key: key.to_string(), index, chapter: None, text }
}

fn story_slots(config: &mut StoryConfig) -> Vec<Slot<'_>> {
    let mut slots = Vec::new();
    for (group, entries) in config.glossary.iter_mut() {
        for (key, text) in entries.iter_mut() {
            slots.push(slot(format!("glossary.{group}"), key, 0, text));
        }
    }
    let style = &mut config.style;
    for (key, text) in style.signature_phrases.iter_mut() {
        slots.push(slot("signaturePhrases", key, 0, text));
    }
    slots.push(slot("protagonist", "", 0, &mut config.protagonist));
    slots.push(slot("summary", "", 0, &mut config.summary));
    slots.push(slot("voice", "", 0, &mut style.voice));
    for (index, text) in style.tone_rules.iter_mut().enumerate() {
        slots.push(slot("toneRules", "", index, text));
    }
    for (index, text) in style.avoid.iter_mut().enumerate() {
        slots.push(slot("avoid", "", index, text));
    }
    slots.push(slot("customPrompt", "", 0, &mut config.custom_prompt));
    slots
}

/// Ghi chú của mốc xưng hô chỉ để người đọc, không vào prompt — không tìm.
fn cast_slots(cast: &mut Cast) -> Vec<Slot<'_>> {
    let mut slots = Vec::new();
    for (key, timeline) in cast.addressing.iter_mut() {
        for (index, change) in timeline.changes.iter_mut().enumerate() {
            let chapter = Some(change.from.clone());
            slots.push(Slot { chapter, ..slot("addressing", key, index, &mut change.target) });
        }
    }
    for (key, character) in cast.characters.iter_mut() {
        slots.push(slot("narration", key, 0, &mut character.narration));
    }
    slots
}

fn title_slots(book: &mut TitleBook) -> Vec<Slot<'_>> {
    book.titles.iter_mut().map(|(key, text)| slot("titles", key, 0, text)).collect()
}

/// Hồ sơ truyện nạp để tìm/thay. Thiếu story.json (hay hỏng) thì không tìm phần đó — vẫn tìm được trong chương;
/// cast.json / titles.json thiếu hay hỏng là bảng rỗng như lúc dịch.
struct Profile {
    story: Option<StoryConfig>,
    cast: Cast,
    titles: TitleBook,
}

impl Profile {
    fn load(paths: &StoryPaths) -> Profile {
        Profile { story: load_story_config(paths).ok(), cast: load_cast(paths), titles: load_titles(paths) }
    }

    fn slots(&mut self) -> Vec<Slot<'_>> {
        let mut slots = self.story.as_mut().map(story_slots).unwrap_or_default();
        slots.extend(cast_slots(&mut self.cast));
        slots.extend(title_slots(&mut self.titles));
        slots
    }
}

/// Chữ tìm đã tách sẵn để so nhanh nhiều ô.
struct Needle {
    chars: Vec<char>,
    quick: String,
}

impl Needle {
    fn new(query: &FindQuery) -> Needle {
        Needle { chars: query.find.chars().collect(), quick: folded(&query.find, query) }
    }

    /// Ký tự của ô và vị trí các chỗ khớp; lọc theo raw thì chỉ ô có khoá Hán chứa chuỗi lọc.
    fn find_in(&self, slot: &Slot, query: &FindQuery) -> Option<(Vec<char>, Vec<usize>)> {
        if !query.raw_filter.is_empty() && !slot.key.contains(&query.raw_filter) {
            return None;
        }
        if !folded(slot.text.as_str(), query).contains(&self.quick) {
            return None;
        }
        let chars: Vec<char> = slot.text.chars().collect();
        let positions = match_positions(&chars, &self.chars, query);
        (!positions.is_empty()).then_some((chars, positions))
    }
}

fn find_in_profile(paths: &StoryPaths, query: &FindQuery) -> Vec<ProfileHit> {
    let needle = Needle::new(query);
    let width = needle.chars.len();
    let mut profile = Profile::load(paths);
    let mut hits = Vec::new();
    for slot in profile.slots() {
        let Some((chars, positions)) = needle.find_in(&slot, query) else { continue };
        for (occurrence, &at) in positions.iter().enumerate() {
            let (before, matched, after) = context(&chars, at, width);
            let at = ProfileRef { field: slot.field.clone(), key: slot.key.clone(), index: slot.index, occurrence };
            hits.push(ProfileHit { at, before, matched, after, chapter: slot.chapter.clone() });
        }
    }
    hits
}

/// Thay trong các ô, trừ chỗ bỏ chọn; trả số chỗ đã thay.
fn rewrite_slots(slots: Vec<Slot>, query: &FindQuery, needle: &Needle, skip: &HashSet<&ProfileRef>) -> usize {
    let width = needle.chars.len();
    let mut count = 0;
    for slot in slots {
        let Some((chars, positions)) = needle.find_in(&slot, query) else { continue };
        let mut rebuilt = String::new();
        let mut cursor = 0;
        let mut touched = false;
        for (occurrence, &at) in positions.iter().enumerate() {
            let at_ref = ProfileRef { field: slot.field.clone(), key: slot.key.clone(), index: slot.index, occurrence };
            if skip.contains(&at_ref) {
                continue;
            }
            rebuilt.extend(&chars[cursor..at]);
            rebuilt.push_str(&query.replace);
            cursor = at + width;
            count += 1;
            touched = true;
        }
        if touched {
            rebuilt.extend(&chars[cursor..]);
            *slot.text = rebuilt;
        }
    }
    count
}

const STORY_FILE: &str = "story.json";
const CAST_FILE: &str = "cast.json";
const TITLES_FILE: &str = "titles.json";

/// Đường dẫn file hồ sơ theo tên ghi trong lần hoàn tác; tên lạ thì không đụng.
fn profile_path<'a>(paths: &'a StoryPaths, name: &str) -> Option<&'a Path> {
    match name {
        STORY_FILE => Some(paths.story_json.as_path()),
        CAST_FILE => Some(paths.cast_json.as_path()),
        TITLES_FILE => Some(paths.titles_json.as_path()),
        _ => None,
    }
}

/// Thay trong hồ sơ trừ chỗ bỏ chọn (chưa ghi): trả các file đổi kèm bản trước / bản sau, và số chỗ đã thay.
fn replace_in_profile(paths: &StoryPaths, query: &FindQuery, excluded: &[ProfileRef]) -> Result<(Vec<UndoFile>, usize)> {
    let needle = Needle::new(query);
    let skip: HashSet<&ProfileRef> = excluded.iter().collect();
    let mut profile = Profile::load(paths);
    let mut files = Vec::new();
    let mut total = 0;
    let mut changed = |name: &str, count: usize, written: String| -> Result<()> {
        if count > 0 {
            let original = read_text(profile_path(paths, name).expect("tên file hồ sơ cố định"))?;
            files.push(UndoFile { name: name.to_string(), original, written });
            total += count;
        }
        Ok(())
    };
    if let Some(story) = profile.story.as_mut() {
        let count = rewrite_slots(story_slots(story), query, &needle, &skip);
        changed(STORY_FILE, count, story.to_json_pretty())?;
    }
    let count = rewrite_slots(cast_slots(&mut profile.cast), query, &needle, &skip);
    changed(CAST_FILE, count, cast_to_json(&profile.cast))?;
    let count = rewrite_slots(title_slots(&mut profile.titles), query, &needle, &skip);
    changed(TITLES_FILE, count, titles_to_json(&profile.titles)?)?;
    Ok((files, total))
}

pub fn find_in_story(paths: &StoryPaths, query: &FindQuery, limit: usize) -> Result<FindResult> {
    let mut result = FindResult::default();
    let width = query.find.chars().count();
    if width == 0 {
        return Ok(result);
    }
    result.profile = find_in_profile(paths, query);
    for found in scan(paths, query)? {
        let mut chapter = match found {
            Scan::NoMatch => continue,
            Scan::Skipped => {
                result.skipped += 1;
                continue;
            }
            Scan::Found(chapter) => chapter,
        };
        let count: usize = chapter.matches.iter().map(|(_, positions)| positions.len()).sum();
        result.total += count;
        result.chapters.push(ChapterCount { chapter: chapter.id.clone(), count });
        if result.hits.len() >= limit {
            continue;
        }
        // Ngữ cảnh raw chỉ đọc cho các chương có chỗ được hiện.
        if chapter.raw.is_none() {
            chapter.raw = load_raw(paths, &chapter.id, chapter.content.len());
        }
        for (paragraph, positions) in &chapter.matches {
            let chars: Vec<char> =
                chapter.lines[chapter.content[*paragraph]].trim_end_matches(['\r', '\n']).chars().collect();
            for (occurrence, &at) in positions.iter().enumerate() {
                if result.hits.len() >= limit {
                    break;
                }
                let (before, matched, after) = context(&chars, at, width);
                let raw = chapter.raw.as_ref().map(|raw| raw[*paragraph].chars().take(RAW_CHARS).collect());
                result.hits.push(FindHit {
                    chapter: chapter.id.clone(),
                    paragraph: *paragraph,
                    occurrence,
                    before,
                    matched,
                    after,
                    raw,
                });
            }
        }
    }
    Ok(result)
}

fn undo_path(paths: &StoryPaths) -> PathBuf {
    paths.work_dir.join(UNDO_FILE)
}

/// Thay mọi chỗ khớp trong chương trừ chương / chỗ bỏ chọn — không đụng hồ sơ. Có thay thì lưu bản gốc để hoàn
/// tác (đè lần thay trước).
pub fn replace_in_story(
    paths: &StoryPaths,
    query: &FindQuery,
    excluded_chapters: &[String],
    excluded_hits: &[HitRef],
) -> Result<ReplaceOutcome> {
    replace_once(paths, query, excluded_chapters, excluded_hits, None, &mut None)
}

/// Như `replace_in_story`, thay cả trong hồ sơ (story.json, cast.json, titles.json) trừ các chỗ `excluded_profile`;
/// chương và hồ sơ chung một lần hoàn tác.
pub fn replace_in_story_and_profile(
    paths: &StoryPaths,
    query: &FindQuery,
    excluded_chapters: &[String],
    excluded_hits: &[HitRef],
    excluded_profile: &[ProfileRef],
) -> Result<ReplaceOutcome> {
    replace_once(paths, query, excluded_chapters, excluded_hits, Some(excluded_profile), &mut None)
}

/// Một cặp trong lượt thay nhiều cặp: như tham số của `replace_in_story`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReplaceJob {
    pub query: FindQuery,
    pub excluded_chapters: Vec<String>,
    pub excluded_hits: Vec<HitRef>,
}

/// Thay nhiều cặp lần lượt, gộp thành MỘT lần hoàn tác: mỗi chương giữ bản trước cả lượt và bản cuối cùng,
/// `find`/`replace` là các cặp có thay nối bằng "; ". Không thay được gì thì giữ nguyên lần hoàn tác trước.
///
/// Job được lấy từ `jobs` từng cái một, ngay trước khi chạy: `excluded_hits` đánh số chỗ khớp theo bản dịch lúc đó
/// (sau các job trước), nên truyền iterator lười dựng job bằng `find_in_story` — vd
/// `requests.iter().map(|r| dựng_job(r))` — thì số thứ tự chỗ khớp luôn đúng. Lỗi dựng job dừng lượt; các job đã
/// chạy vẫn hoàn tác được. `chapters` của kết quả là số chương khác nhau bị sửa.
pub fn replace_batch_in_story(
    paths: &StoryPaths,
    jobs: impl IntoIterator<Item = Result<ReplaceJob>>,
) -> Result<ReplaceOutcome> {
    let mut undo = None;
    let mut occurrences = 0;
    for job in jobs {
        let job = job?;
        let done = replace_once(paths, &job.query, &job.excluded_chapters, &job.excluded_hits, None, &mut undo)?;
        occurrences += done.occurrences;
    }
    let chapters = undo.map_or(0, |record: UndoRecord| record.chapters.len());
    Ok(ReplaceOutcome { chapters, occurrences, profile: 0 })
}

/// Gộp phần thay của một job vào lần hoàn tác đang dồn: chương / file hồ sơ đã có giữ `original` cũ, lấy
/// `written` mới.
fn merge_undo<'a>(
    undo: &'a mut Option<UndoRecord>,
    query: &FindQuery,
    outcome: &ReplaceOutcome,
    chapters: Vec<UndoChapter>,
    files: Vec<UndoFile>,
) -> &'a UndoRecord {
    let first = undo.is_none();
    let record = undo.get_or_insert_with(|| UndoRecord {
        find: String::new(),
        replace: String::new(),
        occurrences: 0,
        created_at: 0,
        chapters: Vec::new(),
        profile: 0,
        files: Vec::new(),
    });
    // Phân cách theo job, không theo chuỗi rỗng: cụm thay "" vẫn giữ find/replace thẳng hàng.
    for (joined, part) in [(&mut record.find, &query.find), (&mut record.replace, &query.replace)] {
        if !first {
            joined.push_str("; ");
        }
        joined.push_str(part);
    }
    record.occurrences += outcome.occurrences;
    record.profile += outcome.profile;
    record.created_at = now_ms();
    for chapter in chapters {
        match record.chapters.iter_mut().find(|known| known.id == chapter.id) {
            Some(known) => known.written = chapter.written,
            None => record.chapters.push(chapter),
        }
    }
    for file in files {
        match record.files.iter_mut().find(|known| known.name == file.name) {
            Some(known) => known.written = file.written,
            None => record.files.push(file),
        }
    }
    record
}

/// Thân của `replace_in_story`; `excluded_profile` None = không đụng hồ sơ; `undo` = lần hoàn tác đang dồn của
/// lượt thay nhiều cặp (None = lượt mới).
fn replace_once(
    paths: &StoryPaths,
    query: &FindQuery,
    excluded_chapters: &[String],
    excluded_hits: &[HitRef],
    excluded_profile: Option<&[ProfileRef]>,
    undo: &mut Option<UndoRecord>,
) -> Result<ReplaceOutcome> {
    let width = query.find.chars().count();
    let mut outcome = ReplaceOutcome::default();
    if width == 0 || query.find == query.replace {
        return Ok(outcome);
    }
    let skip_chapters: HashSet<&str> = excluded_chapters.iter().map(String::as_str).collect();
    let skip_hits: HashSet<(&str, usize, usize)> =
        excluded_hits.iter().map(|hit| (hit.chapter.as_str(), hit.paragraph, hit.occurrence)).collect();
    let mut changed: Vec<(PathBuf, UndoChapter)> = Vec::new();
    for found in scan(paths, query)? {
        let Scan::Found(chapter) = found else { continue };
        if skip_chapters.contains(chapter.id.as_str()) {
            continue;
        }
        let mut lines = chapter.lines.clone();
        let mut count = 0;
        for (paragraph, positions) in &chapter.matches {
            let line = chapter.content[*paragraph];
            let full = &chapter.lines[line];
            let body = full.trim_end_matches(['\r', '\n']);
            let ending = &full[body.len()..];
            let chars: Vec<char> = body.chars().collect();
            let mut rebuilt = String::new();
            let mut cursor = 0;
            let mut touched = false;
            for (occurrence, &at) in positions.iter().enumerate() {
                if skip_hits.contains(&(chapter.id.as_str(), *paragraph, occurrence)) {
                    continue;
                }
                rebuilt.extend(&chars[cursor..at]);
                rebuilt.push_str(&query.replace);
                cursor = at + width;
                count += 1;
                touched = true;
            }
            if touched {
                rebuilt.extend(&chars[cursor..]);
                rebuilt.push_str(ending);
                lines[line] = rebuilt;
            }
        }
        if count > 0 {
            outcome.chapters += 1;
            outcome.occurrences += count;
            let written = lines.concat();
            changed.push((chapter.path, UndoChapter { id: chapter.id, original: chapter.text, written }));
        }
    }
    let (files, profile) = match excluded_profile {
        Some(excluded) => replace_in_profile(paths, query, excluded)?,
        None => (Vec::new(), 0),
    };
    outcome.profile = profile;
    if changed.is_empty() && files.is_empty() {
        return Ok(outcome);
    }
    // Ghi bản gốc trước rồi mới sửa chương / hồ sơ: lỡ dừng giữa chừng vẫn hoàn tác được phần đã ghi.
    let chapters = changed.iter().map(|(_, undo)| undo.clone()).collect();
    let record = merge_undo(undo, query, &outcome, chapters, files.clone());
    std::fs::create_dir_all(&paths.work_dir).map_err(CoreError::io(&paths.work_dir))?;
    let json = serde_json::to_string(record).map_err(|e| CoreError::Internal(e.to_string()))?;
    write_atomic(&undo_path(paths), &json)?;
    for (path, undo) in &changed {
        write_atomic(path, &undo.written)?;
    }
    for file in &files {
        write_atomic(profile_path(paths, &file.name).expect("tên file hồ sơ cố định"), &file.written)?;
    }
    Ok(outcome)
}

fn load_undo(paths: &StoryPaths) -> Option<UndoRecord> {
    std::fs::read_to_string(undo_path(paths)).ok().and_then(|text| serde_json::from_str(&text).ok())
}

pub fn undo_info(paths: &StoryPaths) -> Option<UndoInfo> {
    load_undo(paths).map(|record| UndoInfo {
        find: record.find,
        replace: record.replace,
        chapters: record.chapters.len(),
        occurrences: record.occurrences,
        profile: record.profile,
        created_at: record.created_at,
    })
}

/// Trả các chương và file hồ sơ về bản trước lần thay gần nhất — trừ cái đã bị sửa tiếp sau đó (dịch lại, sửa
/// tay, học glossary).
pub fn undo_last_replace(paths: &StoryPaths) -> Result<UndoOutcome> {
    let Some(record) = load_undo(paths) else { return Ok(UndoOutcome::default()) };
    let mut outcome = UndoOutcome::default();
    for chapter in record.chapters {
        let path = paths.out_dir.join(format!("{}.txt", chapter.id));
        if read_text(&path).ok().as_deref() == Some(chapter.written.as_str()) {
            write_atomic(&path, &chapter.original)?;
            outcome.restored += 1;
        } else {
            outcome.conflicts.push(chapter.id);
        }
    }
    for file in record.files {
        let Some(path) = profile_path(paths, &file.name) else { continue };
        if read_text(path).ok().as_deref() == Some(file.written.as_str()) {
            write_atomic(path, &file.original)?;
            outcome.profile_restored += 1;
        } else {
            outcome.profile_conflicts.push(file.name);
        }
    }
    let _ = std::fs::remove_file(undo_path(paths));
    Ok(outcome)
}
