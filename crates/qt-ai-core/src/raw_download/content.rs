//! Nội dung chương tải về → file raw theo đúng dạng thư viện đang dùng: dòng đầu tiêu đề, một dòng trống, rồi
//! mỗi đoạn một dòng. Chỉ nội dung trống mới là lỗi; chương xin nghỉ một câu vẫn ghi, chỉ gắn nhãn "Ngắn".

use regex::Regex;
use std::sync::OnceLock;

pub const SHORT_CHARS: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prepared {
    pub text: String,
    /// Số ký tự của thân (không tính tiêu đề, không tính khoảng trắng).
    pub body_chars: usize,
    pub short: bool,
}

fn decode_entities(text: &str) -> String {
    static ENTITY: OnceLock<Regex> = OnceLock::new();
    let regex = ENTITY.get_or_init(|| Regex::new(r"&(#[0-9]+|#[xX][0-9a-fA-F]+|[a-zA-Z]+);").expect("regex entity"));
    regex
        .replace_all(text, |c: &regex::Captures| {
            let body = &c[1];
            let decoded = if let Some(hex) = body.strip_prefix("#x").or_else(|| body.strip_prefix("#X")) {
                u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
            } else if let Some(dec) = body.strip_prefix('#') {
                dec.parse::<u32>().ok().and_then(char::from_u32)
            } else {
                match body {
                    "amp" => Some('&'),
                    "lt" => Some('<'),
                    "gt" => Some('>'),
                    "quot" => Some('"'),
                    "apos" => Some('\''),
                    "nbsp" => Some(' '),
                    "ensp" | "emsp" | "thinsp" => Some(' '),
                    "hellip" => Some('…'),
                    "mdash" => Some('—'),
                    "ldquo" => Some('“'),
                    "rdquo" => Some('”'),
                    "lsquo" => Some('‘'),
                    "rsquo" => Some('’'),
                    _ => None,
                }
            };
            decoded.map(String::from).unwrap_or_else(|| c[0].to_string())
        })
        .into_owned()
}

/// `<p>`, `<br>`, `<div>` thành xuống dòng; bỏ mọi thẻ (kể cả `<script>`, `<style>` cùng nội dung); giải thực thể.
pub fn html_to_text(html: &str) -> String {
    static DROP: OnceLock<Regex> = OnceLock::new();
    static BREAK: OnceLock<Regex> = OnceLock::new();
    static TAG: OnceLock<Regex> = OnceLock::new();
    let drop = DROP.get_or_init(|| Regex::new(r"(?is)<(script|style)\b.*?</(script|style)\s*>|<!--.*?-->").expect("regex"));
    let breaks = BREAK.get_or_init(|| Regex::new(r"(?i)<\s*br\s*/?\s*>|<\s*/?\s*(p|div|h[1-6]|li|tr)\b[^>]*>").expect("regex"));
    let tag = TAG.get_or_init(|| Regex::new(r"(?s)<[^>]*>").expect("regex"));
    let text = drop.replace_all(html, "");
    let text = breaks.replace_all(&text, "\n");
    let text = tag.replace_all(&text, "");
    decode_entities(&text)
}

fn clean_line(line: &str) -> &str {
    line.trim_matches(|c: char| c.is_whitespace() || c == '\u{3000}' || c == '\u{feff}')
}

fn compact(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace() && *c != '\u{3000}').collect()
}

/// `None` khi thân trống (không còn chữ nào sau dòng tiêu đề).
pub fn prepare(content: &str, html: bool, title: &str) -> Option<Prepared> {
    let text = if html { html_to_text(content) } else { content.to_string() };
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<&str> = normalized.split('\n').map(clean_line).filter(|l| !l.is_empty()).collect();
    let title = clean_line(title);
    // Dòng đầu đã là tiêu đề (hoặc không có tiêu đề để chèn) thì dùng luôn dòng đầu.
    let first_is_heading = lines.first().is_some_and(|first| title.is_empty() || compact(first) == compact(title));
    let heading = if first_is_heading { lines.remove(0).to_string() } else { title.to_string() };
    if lines.is_empty() {
        return None;
    }
    let body_chars: usize = lines.iter().map(|l| compact(l).chars().count()).sum();
    if body_chars == 0 {
        return None;
    }
    Some(Prepared { text: format!("{heading}\n\n{}\n", lines.join("\n")), body_chars, short: body_chars < SHORT_CHARS })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_thanh_dong() {
        let html = "<p>天下大势&nbsp;，分久必合。</p><p>合久必分<br/>周末七国&amp;秦</p><script>alert(1)</script><!-- x --><div>&#20013;&#x6587;</div>";
        assert_eq!(prepare(html, true, "第1章 起").unwrap().text, "第1章 起\n\n天下大势 ，分久必合。\n合久必分\n周末七国&秦\n中文\n");
    }

    #[test]
    fn khong_chen_lap_tieu_de() {
        let text = "第1章  起\r\n\r\n　　石窟内，安静了一阵。\r\n　　他走了。";
        assert_eq!(prepare(text, false, "第1章 起").unwrap().text, "第1章  起\n\n石窟内，安静了一阵。\n他走了。\n");
        assert_eq!(prepare("石窟内。", false, "第2章").unwrap().text, "第2章\n\n石窟内。\n");
    }

    #[test]
    fn chuong_ngan_van_ghi_chi_trong_moi_la_loi() {
        let leave = prepare("请假条\n\n今晚事务繁忙，先请个假。", false, "请假条").unwrap();
        assert!(leave.short);
        assert_eq!(leave.body_chars, 12);
        assert_eq!(prepare("第3章\n\n", false, "第3章"), None);
        assert_eq!(prepare("   \n　　\n", false, "第3章"), None);
        assert_eq!(prepare("<p> </p><br>", true, "第3章"), None);
        assert!(!prepare(&"字".repeat(60), false, "t").unwrap().short);
    }
}
