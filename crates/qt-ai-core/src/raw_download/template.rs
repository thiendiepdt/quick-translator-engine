//! Mẫu tên file raw (spec 10.3). App sắp chương theo tên file (`natural_chapter_compare`), nên mẫu bắt buộc có
//! số thứ tự `{chuong}`; truyện cũ thì đoán mẫu từ tên đang có để file mới nối tiếp đúng thứ tự.

use crate::story::natural_chapter_compare;
use regex::Regex;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::sync::OnceLock;

pub const DEFAULT_TEMPLATE: &str = "{chuong:4}_{chapter-id}";
pub const MAX_NAME_CHARS: usize = 120;
const FORBIDDEN: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

fn token_regex() -> &'static Regex {
    static TOKEN: OnceLock<Regex> = OnceLock::new();
    TOKEN.get_or_init(|| Regex::new(r"\{([^{}]*)\}").expect("regex token"))
}

enum Token {
    Index(usize),
    Id,
    Title,
    Volume,
}

fn parse_token(token: &str) -> Option<Token> {
    match token {
        "chuong" => Some(Token::Index(0)),
        "chapter-id" => Some(Token::Id),
        "chapterTitle" => Some(Token::Title),
        "volume" => Some(Token::Volume),
        _ => {
            let width = token.strip_prefix("chuong:")?;
            match width.parse::<usize>() {
                Ok(n @ 1..=9) if width.len() == 1 => Some(Token::Index(n)),
                _ => None,
            }
        }
    }
}

pub fn validate_template(template: &str) -> Result<(), String> {
    let template = template.trim();
    if template.is_empty() {
        return Err("mẫu tên file không được để trống".into());
    }
    let mut has_index = false;
    for capture in token_regex().captures_iter(template) {
        match parse_token(&capture[1]) {
            Some(Token::Index(_)) => has_index = true,
            Some(_) => {}
            None => return Err(format!("token {} không hợp lệ — có {{chuong}}, {{chuong:4}}, {{chapter-id}}, {{chapterTitle}}, {{volume}}", &capture[0])),
        }
    }
    if !has_index {
        return Err("mẫu phải có {chuong} hoặc {chuong:4} — app sắp chương theo tên file".into());
    }
    Ok(())
}

pub struct NameParts<'a> {
    /// Vị trí trong mục lục, từ 1.
    pub index: usize,
    pub id: &'a str,
    pub title: &'a str,
    pub volume: Option<&'a str>,
}

/// Bỏ ký tự cấm trên Windows và ký tự điều khiển, gộp khoảng trắng, bỏ khoảng trắng và dấu chấm cuối, cắt
/// `MAX_NAME_CHARS` ký tự.
pub fn sanitize(name: &str) -> String {
    let cleaned: String = name.chars().map(|c| if FORBIDDEN.contains(&c) || c.is_control() { ' ' } else { c }).collect();
    let squashed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let cut: String = squashed.chars().take(MAX_NAME_CHARS).collect();
    cut.trim_end_matches([' ', '.']).trim_start().to_string()
}

pub fn render(template: &str, parts: &NameParts) -> String {
    let rendered = token_regex().replace_all(template.trim(), |capture: &regex::Captures| match parse_token(&capture[1]) {
        Some(Token::Index(0)) => parts.index.to_string(),
        Some(Token::Index(width)) => format!("{:0width$}", parts.index),
        Some(Token::Id) => parts.id.to_string(),
        Some(Token::Title) => parts.title.to_string(),
        Some(Token::Volume) => parts.volume.unwrap_or("").to_string(),
        None => capture[0].to_string(),
    });
    let name = sanitize(&rendered);
    if name.is_empty() {
        format!("{:04}", parts.index)
    } else {
        name
    }
}

/// Trùng tên file đã có thì thêm `-2`, `-3`…
pub fn unique_stem(stem: String, taken: &HashSet<String>) -> String {
    if !taken.contains(&stem) {
        return stem;
    }
    (2..).map(|n| format!("{stem}-{n}")).find(|candidate| !taken.contains(candidate)).expect("vô hạn")
}

/// Đoán mẫu từ tên file raw đang có; không đoán được thì `None`.
pub fn guess_template(existing: &[String]) -> Option<String> {
    if existing.is_empty() {
        return None;
    }
    let width = |digits: &[&str]| digits.iter().map(|d| d.len()).max().unwrap_or(4);
    static CHUONG: OnceLock<Regex> = OnceLock::new();
    static PAIR: OnceLock<Regex> = OnceLock::new();
    static TRIPLE: OnceLock<Regex> = OnceLock::new();
    let chuong = CHUONG.get_or_init(|| Regex::new(r"^chuong-(\d+)$").expect("regex"));
    let pair = PAIR.get_or_init(|| Regex::new(r"^(\d+)_(\d+)$").expect("regex"));
    let triple = TRIPLE.get_or_init(|| Regex::new(r"^(\d+)_(\d+)_.+$").expect("regex"));

    let all = |regex: &Regex, group: usize| -> Option<Vec<&str>> { existing.iter().map(|id| regex.captures(id).map(|c| c.get(group).expect("group").as_str())).collect() };
    if let Some(digits) = all(chuong, 1) {
        return Some(format!("chuong-{{chuong:{}}}", width(&digits)));
    }
    if let Some(digits) = all(pair, 1) {
        return Some(format!("{{chuong:{}}}_{{chapter-id}}", width(&digits)));
    }
    if let (Some(prefixes), Some(digits)) = (all(triple, 1), all(triple, 2)) {
        if prefixes.iter().all(|p| *p == prefixes[0]) {
            return Some(format!("{}_{{chuong:{}}}_{{chapterTitle}}", prefixes[0], width(&digits)));
        }
    }
    // DTH (mặc định `{thu_tu} - {ten_chuong}`) và Novel Reader (`0001 <tiêu đề>`).
    static DASHED: OnceLock<Regex> = OnceLock::new();
    static SPACED: OnceLock<Regex> = OnceLock::new();
    let dashed = DASHED.get_or_init(|| Regex::new(r"^(\d+) - .+$").expect("regex"));
    let spaced = SPACED.get_or_init(|| Regex::new(r"^(\d+) .+$").expect("regex"));
    if let Some(digits) = all(dashed, 1) {
        return Some(format!("{{chuong:{}}} - {{chapterTitle}}", width(&digits)));
    }
    if let Some(digits) = all(spaced, 1) {
        return Some(format!("{{chuong:{}}} {{chapterTitle}}", width(&digits)));
    }
    None
}

/// File mới có được sắp sau file cuối đã có không.
pub fn sorts_after(candidate: &str, last: &str) -> bool {
    natural_chapter_compare(candidate, last) == Ordering::Greater
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(index: usize, id: &'static str, title: &'static str) -> NameParts<'static> {
        NameParts { index, id, title, volume: None }
    }

    #[test]
    fn render_token_va_ky_tu_cam() {
        assert_eq!(render(DEFAULT_TEMPLATE, &parts(5, "7445", "x")), "0005_7445");
        assert_eq!(render("{chuong}", &parts(12, "", "")), "12");
        assert_eq!(render("{chuong:2}", &parts(123, "", "")), "123");
        assert_eq!(render("{chuong:4} {chapterTitle}", &parts(1, "", "第1章 何以?:*  为家 ")), "0001 第1章 何以 为家");
        assert_eq!(render("{chuong:4}-{volume}", &NameParts { index: 3, id: "", title: "", volume: Some("卷一") }), "0003-卷一");
        assert_eq!(render("{chuong:4}_{chapterTitle}...", &parts(1, "", "a")), "0001_a");
        let long = render("{chuong:4}_{chapterTitle}", &parts(1, "", "长".repeat(300).leak()));
        assert_eq!(long.chars().count(), MAX_NAME_CHARS);
    }

    #[test]
    fn kiem_mau() {
        assert!(validate_template(DEFAULT_TEMPLATE).is_ok());
        assert!(validate_template("chuong-{chuong}").is_ok());
        assert!(validate_template("{chapterTitle}").unwrap_err().contains("{chuong}"));
        assert!(validate_template("{chuong}_{foo}").unwrap_err().contains("{foo}"));
        assert!(validate_template("{chuong:0}").is_err());
        assert!(validate_template("{chuong:10}").is_err());
        assert!(validate_template(" ").is_err());
    }

    #[test]
    fn ten_trung_them_so() {
        let taken: HashSet<String> = ["a".to_string(), "a-2".to_string()].into();
        assert_eq!(unique_stem("a".into(), &taken), "a-3");
        assert_eq!(unique_stem("b".into(), &taken), "b");
    }

    #[test]
    fn doan_mau_tu_ten_cu() {
        let chuong: Vec<String> = (1..=1619).map(|i| format!("chuong-{i:04}")).collect();
        assert_eq!(guess_template(&chuong).as_deref(), Some("chuong-{chuong:4}"));
        let pair = vec!["0001_7445623259162542654".to_string(), "0002_7445626961335960126".to_string()];
        assert_eq!(guess_template(&pair).as_deref(), Some("{chuong:4}_{chapter-id}"));
        let triple = vec!["142631_0001_第1章 蓝田公子".to_string(), "142631_0002_第2章 狼心如铁".to_string()];
        assert_eq!(guess_template(&triple).as_deref(), Some("142631_{chuong:4}_{chapterTitle}"));
        let dth = vec!["0001 - 第1章 起".to_string(), "0002 - 第2章 承 - 下".to_string()];
        assert_eq!(guess_template(&dth).as_deref(), Some("{chuong:4} - {chapterTitle}"));
        let reader = vec!["0001 第1章 起".to_string(), "0002 第2章 承".to_string()];
        assert_eq!(guess_template(&reader).as_deref(), Some("{chuong:4} {chapterTitle}"));
        let half = vec!["0001 - 第1章 起".to_string(), "0002 第2章 承".to_string()];
        assert_eq!(guess_template(&half).as_deref(), Some("{chuong:4} {chapterTitle}"), "lẫn hai kiểu thì theo kiểu chung hơn");
        let mixed = vec!["chuong-0001".to_string(), "0002_1".to_string()];
        assert_eq!(guess_template(&mixed), None);
        assert_eq!(guess_template(&[]), None);
    }

    #[test]
    fn kiem_thu_tu() {
        assert!(sorts_after("chuong-0975", "chuong-0974"));
        assert!(!sorts_after("0975_x", "chuong-0974"));
        assert!(sorts_after("1284_7701", "1283_7701005084057928217"));
    }
}
