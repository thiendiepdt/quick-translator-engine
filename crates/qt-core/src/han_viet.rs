//! Han-Việt phonetic transcription (single-char) and QT's `isChinese` definition.

use std::collections::HashMap;

pub type HanVietMap = HashMap<char, String>;

/// QT's definition: a char is "Chinese" iff it has a Han-Việt reading.
pub fn is_chinese(c: char, han_viet: &HanVietMap) -> bool {
    han_viet.contains_key(&c)
}

/// Full-width `！`..`～` (U+FF01..U+FF5E) → ASCII `!`..`~`; others unchanged.
pub fn to_narrow(s: &str) -> String {
    s.chars()
        .map(|c| {
            if ('\u{FF01}'..='\u{FF5E}').contains(&c) {
                char::from_u32(c as u32 - 0xFF01 + 0x21).unwrap_or(c)
            } else {
                c
            }
        })
        .collect()
}

/// Transcribe one char: space → ""; known char → reading; else to_narrow(char).
pub fn char_to_han_viet(c: char, han_viet: &HanVietMap) -> String {
    if c == ' ' {
        return String::new();
    }
    match han_viet.get(&c) {
        Some(v) => v.clone(),
        None => to_narrow(&c.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hv() -> HanVietMap {
        let mut m = HanVietMap::new();
        m.insert('一', "nhất".to_string());
        m
    }

    #[test]
    fn is_chinese_uses_dict_not_unicode() {
        let m = hv();
        assert!(is_chinese('一', &m));
        assert!(!is_chinese('二', &m)); // valid hanzi, but not in dict → not "Chinese"
        assert!(!is_chinese('A', &m));
    }

    #[test]
    fn char_translation_and_to_narrow() {
        let m = hv();
        assert_eq!(char_to_han_viet('一', &m), "nhất");
        assert_eq!(char_to_han_viet(' ', &m), "");
        // full-width '３' U+FF13 → '3'; unknown non-fullwidth passes through
        assert_eq!(char_to_han_viet('３', &m), "3");
        assert_eq!(to_narrow("ＡＢ!"), "AB!");
    }
}
