//! Phiên âm Hán-Việt từng chữ (bảng `ChinesePhienAmWords.txt` của QuickTranslator) cho tên tác giả trong
//! export EPUB: `愤怒的乌贼` → `Phẫn Nộ Đích Ô Tặc`, như cách các trang truyện Việt ghi bút danh.

use std::collections::HashMap;
use std::sync::OnceLock;

const PHIEN_AM: &str = include_str!("../../../QT2025/Resources/ChinesePhienAmWords.txt");

fn table() -> &'static HashMap<char, &'static str> {
    static TABLE: OnceLock<HashMap<char, &'static str>> = OnceLock::new();
    TABLE.get_or_init(|| {
        PHIEN_AM
            .trim_start_matches('\u{feff}')
            .lines()
            .filter_map(|line| {
                let (han, reading) = line.trim().split_once('=')?;
                let mut chars = han.chars();
                let (Some(han), None) = (chars.next(), chars.next()) else { return None };
                let reading = reading.trim();
                (!reading.is_empty()).then_some((han, reading))
            })
            .collect()
    })
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Có chữ nào phiên âm được không — tên đã là Latin / Hán-Việt thì để nguyên.
pub fn has_han(text: &str) -> bool {
    text.chars().any(|c| table().contains_key(&c))
}

/// Mỗi chữ Hán thành một âm viết hoa đầu, cách nhau một dấu cách; chữ không có trong bảng (Latin, số, `·`)
/// giữ nguyên thành cụm riêng.
pub fn han_viet_name(text: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    let mut other = String::new();
    let flush = |other: &mut String, words: &mut Vec<String>| {
        if !other.is_empty() {
            words.push(std::mem::take(other));
        }
    };
    for c in text.chars() {
        if let Some(reading) = table().get(&c) {
            flush(&mut other, &mut words);
            words.push(capitalize(reading));
        } else if c.is_whitespace() {
            flush(&mut other, &mut words);
        } else {
            other.push(c);
        }
    }
    flush(&mut other, &mut words);
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phien_am_but_danh_viet_hoa_tung_am() {
        assert_eq!(han_viet_name("愤怒的乌贼"), "Phẫn Nộ Đích Ô Tặc");
        assert_eq!(han_viet_name("清炒五花肉"), "Thanh Sao Ngũ Hoa Nhục");
        assert_eq!(han_viet_name("江湖不见"), "Giang Hồ Bất Kiến");
    }

    #[test]
    fn giu_chu_latin_so_va_dau_tach_thanh_cum_rieng() {
        assert_eq!(han_viet_name("小A不是123"), "Tiểu A Bất Thị 123");
        assert_eq!(han_viet_name("约翰·史密斯"), "Ước Hàn · Sử Mật Tư");
        assert_eq!(han_viet_name("  Phẫn Nộ  "), "Phẫn Nộ");
        assert_eq!(han_viet_name(""), "");
    }

    #[test]
    fn nhan_ra_ten_con_chu_han() {
        assert!(has_han("愤怒的乌贼"));
        assert!(!has_han("Phẫn Nộ Đích Ô Tặc"));
        assert!(!has_han(""));
    }
}
