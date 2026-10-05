//! Tiêu đề chương lặp lại (truyện chia phần: `清宫炮灰小宫女1` … `清宫炮灰小宫女完`). Model dịch từng chương độc
//! lập nên cùng một tên gốc ra nhiều bản dịch, đuôi số mỗi chương một kiểu ("(1)", "(ba)", "(kết)"). Module này
//! tách dòng tiêu đề thành ba phần — đánh số chương, TÊN, đuôi — rồi ép lúc chốt chương: tên gốc nào đã có bản
//! dịch (`titles.json`) thì dùng lại y nguyên, đuôi viết theo một luật cố định. Phần đánh số chương giữ đúng chữ
//! model viết. Tên tác giả viết lệch nhau (清宫/清朝) là hai tên khác nhau, không gộp.
//!
//! Mọi bước đều "không chắc thì để yên". Dòng đầu chỉ được coi là tiêu đề khi có đánh số chương (`第N章`, `12.`)
//! hoặc có đuôi thứ tự (`陆续与袁子雅（2）`): chương không có tiêu đề mở đầu bằng một câu ngắn là thân bài, không
//! được đụng tới. Bản dịch không tách được phần đánh số hay đuôi → dòng giữ nguyên như model viết.

use crate::error::{CoreError, Result};
use crate::paragraphs::paragraphs_of;
use crate::story_fs::{list_raw_chapter_ids, read_text, write_atomic, ChapterStatus, StoryPaths, StoryState};
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// Dòng đầu dài hơn chừng này ký tự thì coi là thân bài, không phải tiêu đề.
pub const MAX_TITLE_CHARS: usize = 40;

const NUM: &str = "0-9０-９零〇一二两三四五六七八九十百千";

/// `第N章 ` và các kiểu gõ lệch: chữ đơn vị khác (回/节/张/日…) hoặc thiếu hẳn (`第1056 念青禾传讯`) — thiếu
/// thì phải có dấu cách/dấu câu sau số, để `第一次见面` không bị hiểu là chương 1.
static PREFIX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"^第\s*([{NUM}]+)\s*(?:[章回节節话話张張卷部集篇日]\s*|[\s:：、.．\-—]+)")).unwrap()
});
/// Số trần đầu dòng: `12. Tên`, `12、Tên`, `12 Tên`. Yếu hơn `第N章` nên dòng kết thúc bằng dấu câu
/// (`3 天后，他回来了。` là thân bài) không tính.
static BARE_PREFIX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^([0-9０-９]+)\s*(?:[.、．:：]\s*|\s+)").unwrap());
static TAIL_BRACKET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"[\s\-—_·:：]*[（(【\[]\s*([{NUM}]+|[上中下]|完结|大结局|完|终|番外)\s*[)）】\]]\s*$")).unwrap()
});
static TAIL_NUMBER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\s\-—_·:：]*([0-9０-９]+)\s*[)）]?\s*$").unwrap());
static TAIL_WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(完结|大结局|完|终|番外)\s*$").unwrap());

/// Đuôi đánh thứ tự trong một phần truyện.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tail {
    None,
    /// `bracket`: gốc viết trong ngoặc (`（一）`, `(2)`) → dịch "(1)"; số trần → "1".
    Number { n: u32, bracket: bool },
    /// 上 / 中 / 下 (0, 1, 2).
    Part(u8),
    /// 完 / 终 / 完结 / 大结局.
    End,
    /// 番外.
    Extra,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTitle {
    /// Số chương trong phần đánh số đầu dòng, nếu nhận ra.
    pub number: Option<u32>,
    /// Tên — khoá của `titles.json`.
    pub name: String,
    pub tail: Tail,
    /// Tên + đuôi nguyên văn (sau phần đánh số).
    full: String,
    /// Đuôi là chữ trần không ngoặc (`…完`, `…番外`): chỉ tin khi `name` là tên đã gặp, vì chữ đó có thể
    /// thuộc về tên (`游戏玩完`).
    bare_word: bool,
}

impl SourceTitle {
    /// Có dấu hiệu của một dòng tiêu đề: đánh số chương hoặc đuôi thứ tự. Dòng trần không có cả hai (`楔子`,
    /// hay câu mở đầu của chương không tiêu đề) không phân biệt được với thân bài.
    fn is_title(&self) -> bool {
        self.number.is_some() || self.tail != Tail::None
    }

    /// Coi cả `full` là tên, không có đuôi.
    fn whole(mut self) -> Self {
        self.name = std::mem::take(&mut self.full);
        self.full = self.name.clone();
        self.tail = Tail::None;
        self.bare_word = false;
        self
    }
}

/// Số Ả Rập (kể cả full-width) hoặc số chữ Hán tới hàng nghìn.
fn parse_number(text: &str) -> Option<u32> {
    let digits: Option<String> = text
        .chars()
        .map(|c| match c {
            '0'..='9' => Some(c),
            '０'..='９' => char::from_u32(c as u32 - '０' as u32 + '0' as u32),
            _ => None,
        })
        .collect();
    if let Some(digits) = digits {
        return digits.parse().ok();
    }
    let (mut total, mut current) = (0u32, 0u32);
    for c in text.chars() {
        let unit = match c {
            '十' => 10,
            '百' => 100,
            '千' => 1000,
            _ => {
                current = "零一二三四五六七八九".chars().position(|d| d == c).or(match c {
                    '〇' => Some(0),
                    '两' => Some(2),
                    _ => None,
                })? as u32;
                continue;
            }
        };
        total += current.max(1) * unit;
        current = 0;
    }
    Some(total + current)
}

fn tail_of(text: &str, bracket: bool) -> Option<Tail> {
    Some(match text {
        "上" => Tail::Part(0),
        "中" => Tail::Part(1),
        "下" => Tail::Part(2),
        "完" | "终" | "完结" | "大结局" => Tail::End,
        "番外" => Tail::Extra,
        number => Tail::Number { n: parse_number(number)?, bracket },
    })
}

/// (tên, đuôi, đuôi là chữ trần). Không nhận ra đuôi, hoặc bỏ đuôi thì hết tên → cả chuỗi là tên.
fn split_tail(rest: &str) -> (String, Tail, bool) {
    let attempts: [(&Regex, bool, bool); 3] = [(&TAIL_BRACKET, true, false), (&TAIL_NUMBER, false, false), (&TAIL_WORD, false, true)];
    for (pattern, bracket, bare_word) in attempts {
        let Some(caps) = pattern.captures(rest) else { continue };
        let name = rest[..caps.get(0).unwrap().start()].trim_end();
        if name.is_empty() {
            continue;
        }
        if let Some(tail) = tail_of(&caps[1], bracket) {
            return (name.to_string(), tail, bare_word);
        }
    }
    (rest.to_string(), Tail::None, false)
}

/// Tách dòng đầu của chương raw. None = không phải tiêu đề: rỗng, quá dài, chỉ có phần đánh số, hoặc không có
/// cả đánh số lẫn đuôi (xem `SourceTitle::is_title`).
pub fn parse_source_title(line: &str) -> Option<SourceTitle> {
    let line = line.trim_matches(|c: char| c == '\u{feff}' || c.is_whitespace());
    if line.is_empty() || line.chars().count() > MAX_TITLE_CHARS {
        return None;
    }
    const SENTENCE_END: [char; 10] = ['。', '！', '？', '!', '?', '.', '…', '”', '」', '』'];
    let numbered =
        PREFIX.captures(line).or_else(|| BARE_PREFIX.captures(line).filter(|_| !line.ends_with(SENTENCE_END)));
    let (number, rest) = match numbered.and_then(|caps| Some((parse_number(&caps[1])?, caps.get(0)?.end()))) {
        Some((number, end)) => (Some(number), &line[end..]),
        None => (None, line),
    };
    let rest = rest.trim();
    if rest.is_empty() {
        return None;
    }
    let (name, tail, bare_word) = split_tail(rest);
    Some(SourceTitle { number, name, tail, full: rest.to_string(), bare_word }).filter(SourceTitle::is_title)
}

/// Bỏ một dấu chấm kết câu model hay thêm sau tiêu đề; dấu lửng "..." giữ nguyên.
fn strip_final_period(text: &str) -> &str {
    let text = text.trim_end();
    match text.strip_suffix('.') {
        Some(body) if !body.ends_with('.') => body.trim_end(),
        _ => text,
    }
}

/// Dòng đầu (đoạn [[1]]) của một chương — raw hoặc bản dịch.
pub fn first_line(text: &str) -> Option<String> {
    paragraphs_of(text).into_iter().next()
}

/// Phần đánh số ở đầu dòng tiêu đề đã dịch ("Chương 146: "), đúng chữ model viết, và phần còn lại. Gốc có số
/// chương mà bản dịch không mở đầu bằng [chữ] + đúng số đó → None.
fn split_translated_prefix(line: &str, number: Option<u32>) -> Option<(String, &str)> {
    let line = line.trim();
    let Some(number) = number else { return Some((String::new(), line)) };
    let word_end = line.char_indices().find(|(_, c)| !c.is_alphabetic()).map(|(i, _)| i).unwrap_or(line.len());
    let after_word = line[word_end..].trim_start();
    let digits = after_word.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 || after_word[..digits].parse::<u32>().ok() != Some(number) {
        return None;
    }
    let rest = after_word[digits..].trim_start_matches(|c: char| c.is_whitespace() || ":：.-–—".contains(c));
    if rest.is_empty() {
        return None;
    }
    let prefix = line[..line.len() - rest.len()].trim_end();
    Some((format!("{prefix} "), rest))
}

const ROMAN: [&str; 20] = [
    "i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix", "x", "xi", "xii", "xiii", "xiv", "xv", "xvi", "xvii", "xviii",
    "xix", "xx",
];
const VIET_NUMBERS: [&str; 10] = ["một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín", "mười"];

/// Các cách model hay viết một đuôi trong bản dịch (chữ thường).
fn translated_tail_forms(tail: Tail) -> Vec<String> {
    let fixed = |forms: &[&str]| forms.iter().map(|form| form.to_string()).collect();
    match tail {
        Tail::None => vec![],
        Tail::Number { n, .. } => {
            let mut forms = vec![n.to_string()];
            let index = n.wrapping_sub(1) as usize;
            forms.extend(VIET_NUMBERS.get(index).map(|word| word.to_string()));
            forms.extend(ROMAN.get(index).map(|word| word.to_string()));
            forms
        }
        Tail::Part(0) => fixed(&["thượng", "trên", "1"]),
        Tail::Part(1) => fixed(&["trung", "giữa", "2"]),
        Tail::Part(_) => fixed(&["hạ", "dưới", "3"]),
        Tail::End => fixed(&["hết", "kết", "kết thúc", "hoàn", "hoàn kết", "xong", "cuối", "đại kết cục", "end"]),
        Tail::Extra => fixed(&["ngoại truyện", "phiên ngoại"]),
    }
}

/// `text` kết thúc bằng `word` (không phân biệt hoa thường) → vị trí byte nơi `word` bắt đầu.
fn ends_with_ignore_case(text: &str, word: &str) -> Option<usize> {
    let mut tail = text.char_indices().rev();
    let mut start = text.len();
    for expected in word.chars().rev() {
        let (index, found) = tail.next()?;
        if !found.to_lowercase().eq(expected.to_lowercase()) {
            return None;
        }
        start = index;
    }
    Some(start)
}

const OPENERS: [char; 4] = ['(', '（', '[', '【'];
const CLOSERS: [char; 4] = [')', '）', ']', '】'];
const SEPARATORS: [char; 6] = [' ', '-', '–', '—', ':', ','];

/// Tên trong phần còn lại của dòng tiêu đề đã dịch, sau khi bỏ đuôi tương ứng với đuôi gốc. Gốc có đuôi mà bản
/// dịch không kết thúc bằng một dạng nhận ra được → None.
fn strip_translated_tail(rest: &str, tail: Tail) -> Option<String> {
    let rest = strip_final_period(rest.trim());
    let name = if tail == Tail::None {
        rest
    } else {
        let (body, closed) = match rest.strip_suffix(CLOSERS) {
            Some(body) => (body.trim_end(), true),
            None => (rest, false),
        };
        // "… 11" với đuôi 1, "… Hà Nội" với đuôi "i": chữ/số dính liền phía trước thì không phải đuôi.
        let cut = translated_tail_forms(tail)
            .iter()
            .filter_map(|form| ends_with_ignore_case(body, form))
            .filter(|cut| !body[..*cut].chars().last().is_some_and(char::is_alphanumeric))
            .min()?;
        let mut before = body[..cut].trim_end();
        for lead in ["phần", "kỳ", "tập", "hồi"] {
            if let Some(start) = ends_with_ignore_case(before, lead) {
                if !before[..start].chars().last().is_some_and(char::is_alphanumeric) {
                    before = before[..start].trim_end();
                    break;
                }
            }
        }
        let opened = before.ends_with(OPENERS);
        if opened != closed {
            return None;
        }
        before.trim_end_matches(OPENERS).trim_end_matches(SEPARATORS)
    };
    (!name.is_empty()).then(|| name.to_string())
}

/// Đuôi viết theo luật cố định: số trần "1", số trong ngoặc "(1)", còn lại luôn trong ngoặc.
fn tail_suffix(tail: Tail) -> String {
    match tail {
        Tail::None => String::new(),
        Tail::Number { n, bracket: false } => format!(" {n}"),
        Tail::Number { n, bracket: true } => format!(" ({n})"),
        Tail::Part(0) => " (thượng)".to_string(),
        Tail::Part(1) => " (trung)".to_string(),
        Tail::Part(_) => " (hạ)".to_string(),
        Tail::End => " (hết)".to_string(),
        Tail::Extra => " (ngoại truyện)".to_string(),
    }
}

/// `titles.json`: tên tiêu đề gốc → bản dịch đã chốt. Mọi tên từng gặp đều được ghi (kể cả mới gặp một lần),
/// vì không biết trước tên nào sẽ lặp lại.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TitleBook {
    pub version: u32,
    pub titles: IndexMap<String, String>,
}

impl Default for TitleBook {
    fn default() -> Self {
        TitleBook { version: 1, titles: IndexMap::new() }
    }
}

impl TitleBook {
    fn target(&self, name: &str) -> Option<&str> {
        self.titles.get(name).map(|target| target.trim()).filter(|target| !target.is_empty())
    }

    fn set(&mut self, name: &str, target: &str) {
        self.titles.insert(name.to_string(), target.trim().to_string());
    }
}

/// File thiếu hoặc hỏng → sổ rỗng: tiêu đề chỉ là phần trang trí, không được chặn việc chốt chương.
pub fn load_titles(paths: &StoryPaths) -> TitleBook {
    std::fs::read_to_string(&paths.titles_json)
        .ok()
        .and_then(|text| serde_json::from_str::<TitleBook>(&text).ok())
        .filter(|book| book.version == 1)
        .unwrap_or_default()
}

pub fn save_titles(paths: &StoryPaths, book: &TitleBook) -> Result<()> {
    let json = serde_json::to_string_pretty(book).map_err(|e| CoreError::Internal(e.to_string()))?;
    write_atomic(&paths.titles_json, &format!("{json}\n"))
}

/// Dòng tiêu đề đúng chuẩn cho bản dịch `translated` của tiêu đề gốc `source`: phần đánh số của model + tên
/// (`target` nếu đã chốt, không thì tên model vừa dịch) + đuôi theo luật. Trả (dòng mới, tên dùng).
fn compose(source: &SourceTitle, translated: &str, target: Option<&str>) -> Option<(String, String)> {
    let (prefix, rest) = split_translated_prefix(translated, source.number)?;
    let name = match target {
        Some(target) => target.to_string(),
        None => strip_translated_tail(rest, source.tail)?,
    };
    Some((format!("{prefix}{name}{}", tail_suffix(source.tail)), name))
}

/// Lúc chốt chương: ép dòng tiêu đề theo sổ, ghi tên mới vào sổ. Trả dòng tiêu đề mới khi khác dòng model viết.
pub fn settle_title(book: &mut TitleBook, raw_first: &str, translated_first: &str) -> Option<String> {
    let mut source = parse_source_title(raw_first)?;
    if source.bare_word && !book.titles.contains_key(&source.name) {
        source = source.whole();
        if !source.is_title() {
            return None;
        }
    }
    let (line, name) = compose(&source, translated_first, book.target(&source.name))?;
    if book.target(&source.name).is_none() {
        book.set(&source.name, &name);
    }
    (line != translated_first.trim()).then_some(line)
}

/// Thay dòng đầu không rỗng của bản dịch bằng `line`.
pub fn replace_first_line(text: &str, line: &str) -> String {
    let start = text.len() - text.trim_start().len();
    let end = text[start..].find('\n').map(|offset| start + offset).unwrap_or(text.len());
    format!("{}{line}{}", &text[..start], &text[end..])
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleVariant {
    pub text: String,
    pub count: usize,
}

/// Một tên tiêu đề lặp lại ở từ hai chương raw trở lên.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleGroup {
    pub source: String,
    /// Bản dịch sẽ dùng: đã chốt trong sổ, không thì kiểu dịch chiếm đa số ở các chương đã dịch; rỗng = chưa có.
    pub target: String,
    /// Bản dịch đã nằm trong sổ (false = `target` mới là gợi ý theo đa số).
    pub settled: bool,
    pub chapters: usize,
    pub translated: usize,
    /// Các kiểu dịch tên đang có trong out/ — nhiều hơn một là đang lệch nhau.
    pub variants: Vec<TitleVariant>,
    /// Số chương đã dịch có dòng tiêu đề sẽ bị sửa nếu áp dụng `target`.
    pub pending: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleSurvey {
    pub groups: Vec<TitleGroup>,
    /// Chương đã dịch thuộc nhóm lặp mà dòng tiêu đề không tách được phần đánh số — để yên.
    pub unparsed: usize,
}

/// Một dòng tiêu đề trong out/ cần ghi lại.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleFix {
    pub chapter_id: String,
    pub line: String,
}

struct Seen {
    id: String,
    source: SourceTitle,
    /// Dòng tiêu đề hiện có trong out/ (chương đã dịch).
    translated: Option<String>,
}

/// Quét cả truyện: nhóm tên lặp, các kiểu dịch đang có, và những dòng tiêu đề trong out/ cần sửa để mọi chương
/// cùng tên dùng một bản dịch. Không ghi gì.
pub fn survey_titles(paths: &StoryPaths, state: &StoryState, book: &TitleBook) -> Result<(TitleSurvey, Vec<TitleFix>)> {
    let mut parsed: Vec<(String, SourceTitle)> = Vec::new();
    for id in list_raw_chapter_ids(paths)? {
        let raw = read_text(&paths.raw_dir.join(format!("{id}.txt")))?;
        if let Some(source) = first_line(&raw).as_deref().and_then(parse_source_title) {
            parsed.push((id, source));
        }
    }
    // Đuôi chữ trần (…完) chỉ tin khi tên đứng trước nó là tên có thật ở chương khác hoặc trong sổ.
    let known: HashSet<String> = parsed
        .iter()
        .filter(|(_, source)| !source.bare_word)
        .map(|(_, source)| source.name.clone())
        .chain(book.titles.keys().cloned())
        .collect();
    let mut groups: IndexMap<String, Vec<Seen>> = IndexMap::new();
    for (id, source) in parsed {
        let source = if source.bare_word && !known.contains(&source.name) { source.whole() } else { source };
        if !source.is_title() {
            continue;
        }
        let done = state.chapters.get(&id).is_some_and(|chapter| chapter.status == ChapterStatus::Done);
        let translated = done
            .then(|| std::fs::read_to_string(paths.out_dir.join(format!("{id}.txt"))).ok())
            .flatten()
            .and_then(|text| first_line(&text));
        groups.entry(source.name.clone()).or_default().push(Seen { id, source, translated });
    }

    let mut survey = TitleSurvey::default();
    let mut fixes = Vec::new();
    for (name, members) in groups.into_iter().filter(|(_, members)| members.len() >= 2) {
        let mut variants: IndexMap<String, usize> = IndexMap::new();
        for member in &members {
            let Some(line) = &member.translated else { continue };
            if let Some((_, text)) = compose(&member.source, line, None) {
                *variants.entry(text).or_default() += 1;
            }
        }
        let settled = book.target(&name);
        // Đa số; hoà thì kiểu xuất hiện sớm hơn.
        let majority = variants.iter().rev().max_by_key(|(_, count)| **count).map(|(text, _)| text.as_str());
        let target = settled.or(majority).unwrap_or_default().to_string();
        let mut pending = 0;
        for member in &members {
            let Some(line) = &member.translated else { continue };
            if target.is_empty() {
                continue;
            }
            match compose(&member.source, line, Some(&target)) {
                Some((fixed, _)) if fixed != *line => {
                    pending += 1;
                    fixes.push(TitleFix { chapter_id: member.id.clone(), line: fixed });
                }
                Some(_) => {}
                None => survey.unparsed += 1,
            }
        }
        survey.groups.push(TitleGroup {
            source: name,
            settled: settled.is_some(),
            target,
            chapters: members.len(),
            translated: members.iter().filter(|member| member.translated.is_some()).count(),
            variants: variants.into_iter().map(|(text, count)| TitleVariant { text, count }).collect(),
            pending,
        });
    }
    Ok((survey, fixes))
}

/// Chốt `targets` (tên gốc → bản dịch; rỗng thì bỏ qua) vào sổ, rồi ghi lại dòng tiêu đề của mọi chương đã dịch
/// thuộc nhóm lặp cho khớp sổ. Nhóm chưa có trong `targets` lẫn trong sổ thì chốt theo đa số. Trả số chương đã sửa.
pub fn apply_titles(paths: &StoryPaths, state: &StoryState, targets: &HashMap<String, String>) -> Result<usize> {
    let mut book = load_titles(paths);
    for (name, target) in targets {
        if !target.trim().is_empty() {
            book.set(name, target);
        }
    }
    let (survey, fixes) = survey_titles(paths, state, &book)?;
    for group in survey.groups.iter().filter(|group| !group.target.is_empty()) {
        book.set(&group.source, &group.target);
    }
    for fix in &fixes {
        let path = paths.out_dir.join(format!("{}.txt", fix.chapter_id));
        write_atomic(&path, &replace_first_line(&read_text(&path)?, &fix.line))?;
    }
    save_titles(paths, &book)?;
    Ok(fixes.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(line: &str) -> (Option<u32>, String, Tail) {
        let title = parse_source_title(line).unwrap();
        (title.number, title.name, title.tail)
    }

    #[test]
    fn tach_tieu_de_goc_moi_kieu_danh_so_va_duoi() {
        let number = |n, bracket| Tail::Number { n, bracket };
        assert_eq!(parsed("第1章 清宫炮灰小宫女1"), (Some(1), "清宫炮灰小宫女".into(), number(1, false)));
        assert_eq!(parsed("第22章 七零年代炮灰继女"), (Some(22), "七零年代炮灰继女".into(), Tail::None));
        assert_eq!(parsed("第216章 长风原之战（一）"), (Some(216), "长风原之战".into(), number(1, true)));
        assert_eq!(parsed("第972章 清衍伐北海(二)"), (Some(972), "清衍伐北海".into(), number(2, true)));
        assert_eq!(parsed("第220章 长风原之战（完）"), (Some(220), "长风原之战".into(), Tail::End));
        assert_eq!(parsed("第21章 清宫炮灰小宫女完"), (Some(21), "清宫炮灰小宫女".into(), Tail::End));
        assert_eq!(parsed("第9章 决战（下）"), (Some(9), "决战".into(), Tail::Part(2)));
        assert_eq!(parsed("第3章 北宋炮灰公主番外"), (Some(3), "北宋炮灰公主".into(), Tail::Extra));
        // Gõ lệch: BOM đầu file, sai/thiếu chữ 章, số chữ Hán, số trần, không đánh số.
        assert_eq!(parsed("\u{feff}第1章 新婚之夜"), (Some(1), "新婚之夜".into(), Tail::None));
        assert_eq!(parsed("第八十七张 楚潇潇的怀疑人生"), (Some(87), "楚潇潇的怀疑人生".into(), Tail::None));
        assert_eq!(parsed("第1056 念青禾传讯"), (Some(1056), "念青禾传讯".into(), Tail::None));
        assert_eq!(parsed("第28日 独家买卖"), (Some(28), "独家买卖".into(), Tail::None));
        assert_eq!(parsed("12. 夜雨"), (Some(12), "夜雨".into(), Tail::None));
        assert_eq!(parsed("陆续与袁子雅（2）"), (None, "陆续与袁子雅".into(), number(2, true)));
        // Tiêu đề chỉ là một con số thì con số là tên.
        assert_eq!(parsed("第5章 2077"), (Some(5), "2077".into(), Tail::None));
        // Không phải tiêu đề: rỗng, chỉ có đánh số, dài như thân bài, hoặc dòng trần không đánh số không đuôi
        // ("第一次…" không phải đánh số chương) — không phân biệt được với câu mở đầu của chương không tiêu đề.
        assert!(parse_source_title("  ").is_none() && parse_source_title("第5章").is_none());
        for bare in ["终 章 致新世界", "第一次见面", "楔子", "赵静文抬头。", "今天请个假", "3 天后，他回来了。"] {
            assert!(parse_source_title(bare).is_none(), "{bare}");
        }
        assert!(parse_source_title(&"字".repeat(MAX_TITLE_CHARS + 1)).is_none());
        assert_eq!(parse_number("一千零五十六"), Some(1056));
        assert_eq!(parse_number("十"), Some(10));
        assert_eq!(parse_number("二十三"), Some(23));
        assert_eq!(parse_number("１２"), Some(12));
    }

    #[test]
    fn chot_ten_lan_dau_roi_ep_cac_chuong_sau_va_viet_duoi_theo_luat() {
        let mut book = TitleBook::default();
        // Lần đầu: tên model dịch được chốt, đuôi "(Một)" viết lại thành "(1)".
        let first = settle_title(&mut book, "第216章 长风原之战（一）", "Chương 216: Trận chiến Trường Phong Nguyên (Một)");
        assert_eq!(first.as_deref(), Some("Chương 216: Trận chiến Trường Phong Nguyên (1)"));
        assert_eq!(book.titles["长风原之战"], "Trận chiến Trường Phong Nguyên");
        // Chương sau model đổi tên và đuôi → ép về tên đã chốt.
        let later = settle_title(&mut book, "第218章 长风原之战（三）", "Chương 218: Đại chiến đồng Trường Phong (ba)");
        assert_eq!(later.as_deref(), Some("Chương 218: Trận chiến Trường Phong Nguyên (3)"));
        let end = settle_title(&mut book, "第220章 长风原之战（完）", "Chương 220: Trận Trường Phong Nguyên (kết)");
        assert_eq!(end.as_deref(), Some("Chương 220: Trận chiến Trường Phong Nguyên (hết)"));

        // Số trần giữ số trần; chương mở đầu phần không đánh số; đuôi 完 trần chỉ tin vì tên đã gặp.
        assert_eq!(settle_title(&mut book, "第22章 七零年代炮灰继女", "Chương 22: Con gái riêng pháo hôi thập niên 70"), None);
        assert_eq!(
            settle_title(&mut book, "第23章 七零年代炮灰继女2", "Chương 23: Kế nữ pháo hôi những năm bảy mươi 2").as_deref(),
            Some("Chương 23: Con gái riêng pháo hôi thập niên 70 2")
        );
        assert_eq!(
            settle_title(&mut book, "第44章 七零年代炮灰继女完", "Chương 44: Kế nữ pháo hôi thập niên 70 (hoàn)").as_deref(),
            Some("Chương 44: Con gái riêng pháo hôi thập niên 70 (hết)")
        );
        // Đã đúng chuẩn thì không đổi gì.
        assert_eq!(settle_title(&mut book, "第23章 七零年代炮灰继女2", "Chương 23: Con gái riêng pháo hôi thập niên 70 2"), None);
        assert_eq!(book.titles.len(), 2);
    }

    #[test]
    fn khong_chac_thi_de_yen_dong_tieu_de() {
        let mut book = TitleBook::default();
        // Tên chưa gặp kết thúc bằng 完: chữ đó thuộc về tên → không tách, không thêm "(hết)".
        assert_eq!(settle_title(&mut book, "第5章 游戏玩完", "Chương 5: Trò chơi kết thúc"), None);
        assert_eq!(book.titles["游戏玩完"], "Trò chơi kết thúc");
        // Bản dịch không mở đầu bằng đúng số chương, hoặc gốc có đuôi mà bản dịch không có → không học, không sửa.
        assert_eq!(settle_title(&mut book, "第7章 夜雨1", "Mưa đêm 1"), None);
        assert_eq!(settle_title(&mut book, "第7章 夜雨1", "Chương 7: Mưa đêm"), None);
        assert_eq!(settle_title(&mut book, "第7章 夜雨1", "Chương 17: Mưa đêm 1"), None);
        // Số dính liền phía trước không phải đuôi: "… 11" với đuôi 1.
        assert_eq!(settle_title(&mut book, "第8章 夜雨1", "Chương 8: Mưa đêm 11"), None);
        assert!(!book.titles.contains_key("夜雨"));
        // Dòng đầu là thân bài: câu dài, hoặc câu ngắn của chương không có tiêu đề (kể cả kết thúc bằng 完).
        let body = "她抬起头看着窗外的雨，想起很多年前的那个夜晚，那时候她还只是一个不懂事的小姑娘，什么都不怕。";
        let before = book.clone();
        assert_eq!(settle_title(&mut book, body, "Nàng ngẩng đầu nhìn mưa ngoài cửa sổ."), None);
        assert_eq!(settle_title(&mut book, "赵静文抬头。", "Triệu Tĩnh Văn ngẩng đầu."), None);
        assert_eq!(settle_title(&mut book, "她把饭吃完", "Nàng ăn hết cơm."), None);
        assert_eq!(book, before, "thân bài không được ghi vào sổ");
        // Dấu chấm model thêm sau tiêu đề bị bỏ, dấu lửng thì giữ.
        assert_eq!(settle_title(&mut book, "第10章 原来如此", "Chương 10: Thì ra là thế...").as_deref(), None);
        assert_eq!(book.titles["原来如此"], "Thì ra là thế...");
        assert_eq!(settle_title(&mut book, "第11章 剑临天", "Chương 11: Kiếm Lâm Thiên.").as_deref(), Some("Chương 11: Kiếm Lâm Thiên"));
        // Tiêu đề không đánh số chương vẫn chốt được theo tên.
        assert_eq!(settle_title(&mut book, "陆续与袁子雅（1）", "Lục Tục và Viên Tử Nhã (một)").as_deref(), Some("Lục Tục và Viên Tử Nhã (1)"));
        assert_eq!(settle_title(&mut book, "陆续与袁子雅（2）", "Lục Tục cùng Viên Tử Nhã (2).").as_deref(), Some("Lục Tục và Viên Tử Nhã (2)"));
    }

    #[test]
    fn thay_dong_dau_giu_nguyen_phan_con_lai() {
        assert_eq!(replace_first_line("Chương 1: A\n\nĐoạn hai.\n", "Chương 1: B"), "Chương 1: B\n\nĐoạn hai.\n");
        assert_eq!(replace_first_line("\n\nChương 1: A\r\nĐoạn hai.", "X"), "\n\nX\nĐoạn hai.");
        assert_eq!(replace_first_line("Chỉ một dòng", "X"), "X");
    }
}
