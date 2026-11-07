//! Đọc file Quy Tắc: lớp từ (`[lớp]`), nhóm `[số]` quét ký tự, và các nhóm quy
//! tắc chạy trên token. Cú pháp ở spec mục 5. Ngữ nghĩa theo bản thử
//! `quytac.py` (`Program`, `compile_char_rules`).

use std::cmp::Reverse;
use std::sync::LazyLock;

use regex::{Captures, Regex};
use rustc_hash::FxHashMap;

use super::QuyTacError;

/// Chữ số Hán trừ 〇, cho `zh_digit`.
const HAN_DIGITS: &str = "零一二三四五六七八九";
const NUM_NO_ZERO: &str = "零一二三四五六七八九十百千万亿两";

/// Một chữ số Hán trong khuôn `[số]`. Chuẩn hoá đầu vào của QT chèn dấu cách quanh 〇
/// (không có trong bảng Hán Việt): "二〇一九" thành "二 〇 一九", "二〇〇〇年" thành
/// "二 〇〇〇 年". Chỉ dấu cách sát 〇 được bỏ qua; dấu cách thật giữa hai chữ số khác thì không.
fn zh_digit(others: &str) -> String {
    format!(r"\s?〇\s?|[{others}]")
}

/// Một lớp từ: chữ Hán → nghĩa thay trong khuôn (có thể rỗng).
pub(crate) type Class = FxHashMap<String, String>;

/// Nhãn khoá có sẵn, ngoài các lớp từ khai báo trong `[lớp]`.
pub(crate) const BUILTIN_SLOTS: [&str; 21] = [
    "đại-từ",
    "tên",
    "người",
    "danh-từ",
    "cụm",
    "động-từ",
    "danh-động",
    "hệ-từ",
    "có-đích",
    "tính-từ",
    "thành-ngữ",
    "số",
    "số-lượng",
    "thời-gian",
    "nơi",
    "phó-từ",
    "dấu",
    "chỉ-định",
    "nơi-cụm",
    "nơi-từ",
    "động-từ-tại",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LeftKind {
    Ve,
    VeDong,
    MenhDe,
}

impl LeftKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            LeftKind::Ve => "vế",
            LeftKind::VeDong => "vế-động",
            LeftKind::MenhDe => "mệnh-đề",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ElKind {
    Start,
    End,
    Left(LeftKind),
    Gap,
    Many(String),
    Slot(String),
    Lit(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct El {
    pub(crate) kind: ElKind,
    pub(crate) opt: bool,
    pub(crate) neg: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RefKey {
    /// `{1}`, `{2}`… theo thứ tự khoá trong mẫu (tính từ 1).
    Index(usize),
    /// `{danh-từ}`: khoá đầu tiên mang tên đó.
    Name(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Part {
    Text(String),
    Ref { key: RefKey, filter: Option<String> },
}

#[derive(Debug)]
pub(crate) struct Rule {
    pub(crate) pre: Vec<El>,
    pub(crate) pat: Vec<El>,
    pub(crate) post: Vec<El>,
    pub(crate) tmpl: Vec<Part>,
    pub(crate) result: String,
    /// Chữ cố định bắt buộc của mẫu: đoạn không chứa thì khỏi thử.
    pub(crate) needs: Vec<Vec<String>>,
}

#[derive(Debug)]
pub(crate) struct Section {
    pub(crate) rounds: usize,
    pub(crate) result: String,
    pub(crate) rules: Vec<Rule>,
    /// Quy tắc mở đầu bằng chữ cố định, theo chữ đầu.
    pub(crate) first: FxHashMap<char, Vec<usize>>,
    /// Quy tắc mở đầu bằng `{vế}`…, theo chữ đầu của chữ cố định ngay sau.
    pub(crate) left: FxHashMap<char, Vec<usize>>,
    pub(crate) generic: Vec<usize>,
}

#[derive(Debug)]
pub(crate) struct CharRule {
    pub(crate) regex: Regex,
    pub(crate) slots: Vec<String>,
    pub(crate) tmpl: Vec<Part>,
    pub(crate) result: String,
    sort_key: usize,
}

#[derive(Debug)]
pub(crate) struct Program {
    pub(crate) classes: FxHashMap<String, Class>,
    pub(crate) char_rules: Vec<CharRule>,
    pub(crate) sections: Vec<Section>,
}

static CONTEXT_SPLIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s)\|(?:\s|$)").unwrap());
static TEMPLATE_REF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{([^{}|]+)(?:\|([^{}]+))?\}").unwrap());
static CHAR_SLOT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{([^{}]+)\}").unwrap());
/// Độ ưu tiên luật `[số]` như bản thử: bỏ khoá, tiêu đề, khuôn, ký tự tuỳ chọn
/// rồi đếm ký tự còn lại.
static SORT_STRIP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{[^{}]*\}|\[số\]| = .*|.\?").unwrap());

fn err(line: usize, message: impl Into<String>) -> QuyTacError {
    QuyTacError {
        line,
        message: message.into(),
    }
}

impl Program {
    pub(crate) fn parse(source: &str) -> Result<Program, QuyTacError> {
        #[derive(PartialEq)]
        enum Mode {
            Outside,
            Classes,
            Chars,
            Rules,
        }
        let mut classes: FxHashMap<String, Class> = FxHashMap::default();
        let mut char_lines: Vec<(usize, String)> = Vec::new();
        let mut sections: Vec<Section> = Vec::new();
        let mut mode = Mode::Outside;
        for (index, raw) in source.lines().enumerate() {
            let line_no = index + 1;
            let raw = if index == 0 {
                raw.trim_start_matches('\u{feff}')
            } else {
                raw
            };
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') {
                let header: Vec<&str> = line
                    .trim_matches(|c| c == '[' || c == ']')
                    .split_whitespace()
                    .collect();
                let Some(&name) = header.first() else {
                    return Err(err(line_no, "tiêu đề nhóm rỗng"));
                };
                mode = match name {
                    "lớp" => Mode::Classes,
                    "số" => Mode::Chars,
                    _ => {
                        let rounds = match header.get(1) {
                            Some(value) => value.parse::<usize>().map_err(|_| {
                                err(line_no, format!("số lượt không hợp lệ: {value}"))
                            })?,
                            None => 1,
                        };
                        let result = header.get(2).copied().unwrap_or("cụm").to_string();
                        sections.push(Section::new(rounds.max(1), result));
                        Mode::Rules
                    }
                };
                continue;
            }
            match mode {
                Mode::Outside => return Err(err(line_no, "dòng nằm ngoài mọi nhóm")),
                Mode::Classes => parse_class(line, line_no, &mut classes)?,
                Mode::Chars => char_lines.push((line_no, line.to_string())),
                Mode::Rules => {
                    let section = sections.last_mut().expect("Mode::Rules luôn có nhóm");
                    let rule = parse_rule(line, line_no, &section.result, &classes)?;
                    section.add(rule);
                }
            }
        }
        let mut char_rules = char_lines
            .iter()
            .map(|(line_no, line)| compile_char_rule(line, *line_no, &classes))
            .collect::<Result<Vec<_>, _>>()?;
        // Nhiều chữ cố định hơn đi trước; hoà thì giữ thứ tự file (sort ổn định).
        char_rules.sort_by_key(|rule| Reverse(rule.sort_key));
        for section in &mut sections {
            section.finish();
        }
        Ok(Program {
            classes,
            char_rules,
            sections,
        })
    }
}

fn parse_class(
    line: &str,
    line_no: usize,
    classes: &mut FxHashMap<String, Class>,
) -> Result<(), QuyTacError> {
    let (name, members) = line
        .split_once(':')
        .ok_or_else(|| err(line_no, "dòng lớp thiếu ':'"))?;
    let table = classes.entry(name.trim().to_string()).or_default();
    for member in members.split_whitespace() {
        let (zh, vi) = member.split_once('=').unwrap_or((member, ""));
        table.insert(zh.to_string(), vi.replace('_', " "));
    }
    Ok(())
}

fn slot_name(
    name: &str,
    line_no: usize,
    classes: &FxHashMap<String, Class>,
) -> Result<String, QuyTacError> {
    if classes.contains_key(name) || BUILTIN_SLOTS.contains(&name) {
        Ok(name.to_string())
    } else {
        Err(err(line_no, format!("khoá không có: {{{name}}}")))
    }
}

fn parse_elements(
    text: &str,
    line_no: usize,
    classes: &FxHashMap<String, Class>,
) -> Result<Vec<El>, QuyTacError> {
    let mut out = Vec::new();
    for part in text.split_whitespace() {
        let neg = part.starts_with('!');
        let mut part = if neg { &part[1..] } else { part };
        let opt = part.ends_with('?');
        if opt {
            part = &part[..part.len() - 1];
        }
        let kind = match part {
            "^" => ElKind::Start,
            "$" => ElKind::End,
            "{vế}" => ElKind::Left(LeftKind::Ve),
            "{vế-động}" => ElKind::Left(LeftKind::VeDong),
            "{mệnh-đề}" => ElKind::Left(LeftKind::MenhDe),
            "{đoạn}" => ElKind::Gap,
            _ if part.starts_with('{') && part.ends_with("}+") => {
                ElKind::Many(slot_name(&part[1..part.len() - 2], line_no, classes)?)
            }
            _ if part.starts_with('{') && part.ends_with('}') => {
                ElKind::Slot(slot_name(&part[1..part.len() - 1], line_no, classes)?)
            }
            _ => {
                let mut lits: Vec<String> = Vec::new();
                for lit in part.trim_matches(|c| c == '(' || c == ')').split('|') {
                    if lit.is_empty() {
                        return Err(err(line_no, format!("chữ cố định rỗng trong {part}")));
                    }
                    if !lits.iter().any(|known| known == lit) {
                        lits.push(lit.to_string());
                    }
                }
                ElKind::Lit(lits)
            }
        };
        out.push(El { kind, opt, neg });
    }
    Ok(out)
}

fn parse_template(
    source: &str,
    cap_names: &[String],
    line_no: usize,
) -> Result<Vec<Part>, QuyTacError> {
    let mut parts = Vec::new();
    let mut last = 0;
    for caps in TEMPLATE_REF.captures_iter(source) {
        let whole = caps.get(0).expect("nhóm 0 luôn có");
        if whole.start() > last {
            parts.push(Part::Text(source[last..whole.start()].to_string()));
        }
        let key = &caps[1];
        let key = match key.parse::<usize>() {
            Ok(index) => {
                if index == 0 || index > cap_names.len() {
                    return Err(err(
                        line_no,
                        format!(
                            "khuôn tham chiếu {{{index}}} nhưng mẫu chỉ có {} khoá",
                            cap_names.len()
                        ),
                    ));
                }
                RefKey::Index(index)
            }
            Err(_) => {
                if !cap_names.iter().any(|name| name == key) {
                    return Err(err(
                        line_no,
                        format!("khuôn tham chiếu {{{key}}} nhưng mẫu không có khoá đó"),
                    ));
                }
                RefKey::Name(key.to_string())
            }
        };
        parts.push(Part::Ref {
            key,
            filter: caps.get(2).map(|m| m.as_str().to_string()),
        });
        last = whole.end();
    }
    if last < source.len() {
        parts.push(Part::Text(source[last..].to_string()));
    }
    Ok(parts)
}

fn parse_rule(
    line: &str,
    line_no: usize,
    default_result: &str,
    classes: &FxHashMap<String, Class>,
) -> Result<Rule, QuyTacError> {
    let (lhs, rhs) = line
        .split_once(" = ")
        .ok_or_else(|| err(line_no, "thiếu ' = '"))?;
    let (tmpl_src, result) = match rhs.split_once(" -> ") {
        Some((tmpl, result)) => (tmpl, result.trim().to_string()),
        None => (rhs, default_result.to_string()),
    };
    let pieces: Vec<&str> = CONTEXT_SPLIT.split(lhs).collect();
    let (pre, pat, post) = match pieces.as_slice() {
        [pat] => ("", *pat, ""),
        [pat, post] => ("", *pat, *post),
        [pre, pat, post] => (*pre, *pat, *post),
        _ => return Err(err(line_no, "quá nhiều dấu '|' ngữ cảnh")),
    };
    let pre = parse_elements(pre, line_no, classes)?;
    let pat = parse_elements(pat, line_no, classes)?;
    let post = parse_elements(post, line_no, classes)?;
    if pat.is_empty() {
        return Err(err(line_no, "mẫu rỗng"));
    }
    let mut cap_names: Vec<String> = Vec::new();
    for (index, el) in pat.iter().enumerate() {
        let next_is_lit = matches!(pat.get(index + 1).map(|e| &e.kind), Some(ElKind::Lit(_)));
        match &el.kind {
            ElKind::Left(kind) => {
                if index != 0 || !next_is_lit {
                    return Err(err(
                        line_no,
                        format!(
                            "{{{}}} phải đứng đầu mẫu và theo sau là chữ cố định",
                            kind.name()
                        ),
                    ));
                }
                cap_names.push(kind.name().to_string());
            }
            ElKind::Gap => {
                if !next_is_lit {
                    return Err(err(line_no, "{đoạn} phải theo sau là chữ cố định"));
                }
                cap_names.push("đoạn".to_string());
            }
            ElKind::Slot(name) | ElKind::Many(name) => cap_names.push(name.clone()),
            _ => {}
        }
    }
    let tmpl = parse_template(tmpl_src.trim(), &cap_names, line_no)?;
    let needs = pat
        .iter()
        .filter(|el| !el.opt && !el.neg)
        .filter_map(|el| match &el.kind {
            ElKind::Lit(lits) => Some(lits.clone()),
            _ => None,
        })
        .collect();
    Ok(Rule {
        pre,
        pat,
        post,
        tmpl,
        result,
        needs,
    })
}

fn compile_char_rule(
    line: &str,
    line_no: usize,
    classes: &FxHashMap<String, Class>,
) -> Result<CharRule, QuyTacError> {
    let (lhs, rhs) = line
        .split_once(" = ")
        .ok_or_else(|| err(line_no, "thiếu ' = '"))?;
    let (tmpl_src, result) = match rhs.split_once(" -> ") {
        Some((tmpl, result)) => (tmpl, result.trim().to_string()),
        None => (rhs, "số-cụm".to_string()),
    };
    // Nhóm (a|b) của người viết không chiếm chỗ đánh số của khoá.
    let mut grouped = String::with_capacity(lhs.len() + 8);
    let mut chars = lhs.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '(' && chars.peek() != Some(&'?') {
            grouped.push_str("(?:");
        } else {
            grouped.push(c);
        }
    }
    let mut slots: Vec<String> = Vec::new();
    let mut failure: Option<String> = None;
    let pattern = CHAR_SLOT.replace_all(&grouped, |caps: &Captures| {
        let spec = &caps[1];
        let (name, bounds) = spec.split_once(':').unwrap_or((spec, ""));
        let (lo, hi) = bounds.split_once('-').unwrap_or((bounds, ""));
        slots.push(name.to_string());
        match (name, bounds.is_empty()) {
            ("số", false) => {
                format!(r"\s*((?:{}|\d){{{lo},{hi}}})\s*", zh_digit(NUM_NO_ZERO))
            }
            ("số", true) => {
                format!(r"\s*((?:{})+|\d+(?:\.\d+)?)\s*", zh_digit(NUM_NO_ZERO))
            }
            // Năm lịch: chữ số Hán đọc từng chữ, hoặc số Ả Rập trông như năm.
            // 500年, 5000年 là "500 năm", không phải năm lịch.
            ("năm", false) if lo == "3" => {
                format!(r"\s*((?:{}){{3,4}}|1[5-9]\d\d|20\d\d)\s*", zh_digit(HAN_DIGITS))
            }
            ("năm", false) => {
                format!(r"\s*((?:{}|\d){{{lo},{hi}}})\s*", zh_digit(HAN_DIGITS))
            }
            ("năm", true) => {
                format!(r"\s*((?:{}|\d)+)\s*", zh_digit(HAN_DIGITS))
            }
            _ => match classes.get(name) {
                Some(class) => {
                    let mut keys: Vec<&String> = class.keys().collect();
                    keys.sort_by(|a, b| {
                        b.chars()
                            .count()
                            .cmp(&a.chars().count())
                            .then_with(|| a.cmp(b))
                    });
                    let alternatives: Vec<String> =
                        keys.iter().map(|key| regex::escape(key)).collect();
                    format!("({})", alternatives.join("|"))
                }
                None => {
                    failure.get_or_insert_with(|| format!("khoá không có: {{{name}}}"));
                    String::new()
                }
            },
        }
    });
    if let Some(message) = failure {
        return Err(err(line_no, message));
    }
    let regex = Regex::new(&pattern)
        .map_err(|error| err(line_no, format!("mẫu số không hợp lệ: {error}")))?;
    let tmpl = parse_template(tmpl_src.trim(), &slots, line_no)?;
    let sort_key = SORT_STRIP
        .replace_all(&format!("[số] {line}"), "")
        .chars()
        .count();
    Ok(CharRule {
        regex,
        slots,
        tmpl,
        result,
        sort_key,
    })
}

fn first_chars(lits: &[String]) -> Vec<char> {
    let mut out = Vec::new();
    for c in lits.iter().filter_map(|lit| lit.chars().next()) {
        if !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

impl Section {
    fn new(rounds: usize, result: String) -> Section {
        Section {
            rounds,
            result,
            rules: Vec::new(),
            first: FxHashMap::default(),
            left: FxHashMap::default(),
            generic: Vec::new(),
        }
    }

    fn add(&mut self, rule: Rule) {
        let index = self.rules.len();
        let head = &rule.pat[0];
        match &head.kind {
            ElKind::Left(_) => {
                if let ElKind::Lit(lits) = &rule.pat[1].kind {
                    for first in first_chars(lits) {
                        self.left.entry(first).or_default().push(index);
                    }
                }
            }
            ElKind::Lit(lits) if !head.opt => {
                for first in first_chars(lits) {
                    self.first.entry(first).or_default().push(index);
                }
            }
            _ => self.generic.push(index),
        }
        self.rules.push(rule);
    }

    /// Trong mỗi danh sách: mẫu dài hơn trước, rồi nhiều chữ cố định hơn; hoà giữ thứ tự file.
    fn finish(&mut self) {
        let rules = &self.rules;
        let key = |index: &usize| {
            let rule = &rules[*index];
            let lits = rule
                .pat
                .iter()
                .filter(|el| matches!(el.kind, ElKind::Lit(_)))
                .count();
            (Reverse(rule.pat.len()), Reverse(lits))
        };
        for list in self.first.values_mut().chain(self.left.values_mut()) {
            list.sort_by_key(key);
        }
        self.generic.sort_by_key(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "[lớp]
vị-trí: 上=trên 中央=giữa
lượng-từ: 层=tầng 个=

[số]
{số}点半 = {1} giờ rưỡi -> thời-gian
第{số}(章|回) = Chương {1|số}: -> lit

[của 3]
{danh-từ} {vị-trí} | !{danh-từ} = {vị-trí} {danh-từ} -> nơi-cụm
这个 {danh-từ}+ | !的 = {1} này
{vế-động} 的时候 = khi {vế-động}

[nghĩa]
^ | 可是 | = Nhưng -> lit
";

    #[test]
    fn parses_classes_sections_and_indexes() {
        let program = Program::parse(SAMPLE).unwrap();
        assert_eq!(program.classes["vị-trí"]["中央"], "giữa");
        assert_eq!(program.classes["lượng-từ"]["个"], "");
        assert_eq!(program.sections.len(), 2);
        let cua = &program.sections[0];
        assert_eq!(cua.rounds, 3);
        assert_eq!(cua.generic, vec![0]);
        assert_eq!(cua.first[&'这'], vec![1]);
        assert_eq!(cua.left[&'的'], vec![2]);
        assert_eq!(cua.rules[0].result, "nơi-cụm");
        assert_eq!(cua.rules[1].result, "cụm");
        assert_eq!(cua.rules[0].post.len(), 1);
        assert!(cua.rules[0].post[0].neg);
        let nghia = &program.sections[1];
        assert_eq!(nghia.rules[0].pre[0].kind, ElKind::Start);
        assert_eq!(nghia.rules[0].result, "lit");
    }

    #[test]
    fn char_rules_sort_by_fixed_characters_and_match() {
        let program = Program::parse(SAMPLE).unwrap();
        assert_eq!(program.char_rules.len(), 2);
        // " 第(章|回)" dài hơn " 点半" nên đứng trước.
        let chapter = &program.char_rules[0];
        assert_eq!(chapter.result, "lit");
        let caps = chapter.regex.captures("第 12 章").unwrap();
        assert_eq!(&caps[1], "12");
        assert!(program.char_rules[1].regex.is_match("三点半"));
    }

    #[test]
    fn templates_are_split_into_parts() {
        let program = Program::parse(SAMPLE).unwrap();
        let rule = &program.sections[0].rules[1];
        assert_eq!(
            rule.tmpl,
            vec![
                Part::Ref { key: RefKey::Index(1), filter: None },
                Part::Text(" này".to_string()),
            ]
        );
        let chapter = &program.char_rules[0];
        assert!(chapter.tmpl.contains(&Part::Ref {
            key: RefKey::Index(1),
            filter: Some("số".to_string()),
        }));
    }

    #[test]
    fn rejects_bad_lines_with_line_numbers() {
        let error = Program::parse("[của]\n{danh-từ} = {2}\n").unwrap_err();
        assert_eq!(error.line, 2);
        assert!(error.message.contains("{2}"), "{}", error.message);
        let error = Program::parse("[của]\n{không-có} = x\n").unwrap_err();
        assert!(error.message.contains("khoá không có"), "{}", error.message);
        let error = Program::parse("[của]\nabc\n").unwrap_err();
        assert!(error.message.contains("thiếu ' = '"), "{}", error.message);
        let error = Program::parse("abc = x\n").unwrap_err();
        assert_eq!(error.line, 1);
        let error = Program::parse("[số]\n{lạ}点 = x\n").unwrap_err();
        assert!(error.message.contains("khoá không có"), "{}", error.message);
    }

    #[test]
    fn default_rule_files_parse() {
        let base = include_str!("../../data/QuyTac.txt");
        let program = Program::parse(base).unwrap();
        let rules: usize = program.sections.iter().map(|s| s.rules.len()).sum();
        assert_eq!(rules + program.char_rules.len(), 204);
        let extra = include_str!("../../data/QuyTac-giong-muot.txt");
        Program::parse(&format!("{base}\n{extra}")).unwrap();
    }
}
