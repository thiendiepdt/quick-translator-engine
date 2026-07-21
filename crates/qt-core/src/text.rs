//! String assembly: wrapping, sentence-start capitalization, spacing.
//! Mirrors appendTranslatedWord / WrapTranslation / nextCharIsChinese.

use crate::han_viet::{is_chinese, HanVietMap};

/// Range-preserving punctuation subset of QT2025's `StandardizeInput`.
/// Every replacement is one UTF-16 code unit so source offsets stay valid.
pub(crate) fn normalize_chinese_punctuation(c: char) -> char {
    match c {
        '，' | '、' => ',',
        '。' | '．' => '.',
        '：' => ':',
        '？' => '?',
        '！' => '!',
        '\u{3000}' => ' ',
        _ => c,
    }
}

/// QT's standardizer inserts a separator after sentence punctuation, except
/// inside a decimal or when whitespace/a quote already follows.
pub(crate) fn needs_space_after_sentence_punctuation(chars: &[char], index: usize) -> bool {
    let Some(&current) = chars.get(index) else {
        return false;
    };
    let Some(&next) = chars.get(index + 1) else {
        return false;
    };
    if !matches!(current, '.' | '?' | '!') || matches!(next, ' ' | '"' | '\'') {
        return false;
    }
    !(current == '.' && index > 0 && chars[index - 1].is_ascii_digit() && next.is_ascii_digit())
}

pub fn wrap_translation(t: &str, wrap_type: i32) -> String {
    if wrap_type == 0 {
        t.to_string()
    } else {
        format!("[{t}]")
    }
}

/// Capitalize first char; if it starts with '[' (wrapped), capitalize the char after '['.
pub fn to_upper_case(text: &str) -> String {
    if text.is_empty() {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    if chars[0] != '[' || chars.len() < 2 {
        let head: String = chars[0].to_uppercase().collect();
        let tail: String = chars[1..].iter().collect();
        format!("{head}{tail}")
    } else {
        let head: String = chars[1].to_uppercase().collect();
        let tail: String = chars[2..].iter().collect();
        format!("[{head}{tail}")
    }
}

const SENTENCE_ENDERS: [&str; 11] = [
    "\n", "\t", ". ", "\"", "'", "? ", "! ", ".\" ", "?\" ", "!\" ", ": ",
];

/// Append `translated` to `result`, tracking `last` (the previously appended chunk).
/// - after a sentence-ender → capitalize first letter
/// - after a space or '(' → join directly
/// - otherwise → insert one leading space
///
/// Then, if the new chunk starts with , . ? ! and result ends with a space, drop that space.
pub fn append_translated_word(result: &mut String, translated: &str, last: &mut String) -> isize {
    let before = utf16_len(result) as isize;
    let new_last = if SENTENCE_ENDERS.iter().any(|e| last.ends_with(e)) {
        to_upper_case(translated)
    } else if last.ends_with(' ') || last.ends_with('(') {
        translated.to_string()
    } else {
        format!(" {translated}")
    };
    *last = new_last;

    let starts_punct = translated.is_empty()
        || matches!(
            translated.chars().next(),
            Some(',') | Some('.') | Some('?') | Some('!')
        );
    if starts_punct && result.ends_with(' ') {
        result.pop();
    }
    result.push_str(last);
    utf16_len(result) as isize - before
}

pub(crate) fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

pub fn next_char_is_chinese(chars: &[char], end_idx: usize, han_viet: &HanVietMap) -> bool {
    if chars.len() > end_idx + 1 {
        is_chinese(chars[end_idx + 1], han_viet)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrap_and_upper() {
        assert_eq!(wrap_translation("x", 0), "x");
        assert_eq!(wrap_translation("x", 1), "[x]");
        assert_eq!(to_upper_case("hắn"), "Hắn");
        assert_eq!(to_upper_case("[hắn]"), "[Hắn]");
    }

    #[test]
    fn normalizes_qt_chinese_punctuation_without_changing_utf16_length() {
        let input = "，、。．：？！　";
        let normalized: String = input.chars().map(normalize_chinese_punctuation).collect();
        assert_eq!(normalized, ",,..:?! ");
        assert_eq!(
            input.encode_utf16().count(),
            normalized.encode_utf16().count()
        );
    }

    #[test]
    fn sentence_spacing_skips_decimal_points() {
        let sentence: Vec<char> = "他.他".chars().collect();
        assert!(needs_space_after_sentence_punctuation(&sentence, 1));
        let decimal: Vec<char> = "1.5".chars().collect();
        assert!(!needs_space_after_sentence_punctuation(&decimal, 1));
    }

    #[test]
    fn join_default_inserts_space() {
        let mut r = String::new();
        let mut last = String::new();
        append_translated_word(&mut r, "hắn", &mut last); // first word, last empty → " hắn"
        append_translated_word(&mut r, "rất", &mut last); // last="hắn" → " rất"
        assert_eq!(r, " hắn rất");
    }

    #[test]
    fn join_capitalizes_after_sentence_end() {
        let mut r = String::from("A.");
        let mut last = String::from(". "); // simulate previous ended a sentence
        append_translated_word(&mut r, "hắn", &mut last);
        assert!(r.ends_with("Hắn"));
    }

    #[test]
    fn join_drops_space_before_punct() {
        let mut r = String::from("hắn ");
        let mut last = String::from("hắn ");
        append_translated_word(&mut r, ",", &mut last);
        assert_eq!(r, "hắn,");
    }
}
