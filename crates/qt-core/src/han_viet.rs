//! Han-Việt phonetic transcription (single-char) and QT's `isChinese` definition.

use crate::text::{append_translated_word, needs_space_after_sentence_punctuation, utf16_len};
use crate::{CharRange, TranslationResult};
use rustc_hash::FxHashMap;

pub type HanVietMap = FxHashMap<char, String>;

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

/// Mode HanViet: transcribe each char; a single space separates two consecutive
/// Chinese chars. Uses append_translated_word so sentence-start capitalization applies.
#[cfg(test)]
pub fn chinese_to_han_viet(chars: &[char], han_viet: &HanVietMap) -> TranslationResult {
    let mut source_start = 0usize;
    let source_ranges: Vec<CharRange> = chars
        .iter()
        .map(|ch| {
            let range = CharRange {
                start: source_start,
                length: ch.len_utf16(),
            };
            source_start += ch.len_utf16();
            range
        })
        .collect();
    chinese_to_han_viet_mapped(chars, han_viet, &source_ranges)
}

pub(crate) fn chinese_to_han_viet_mapped(
    chars: &[char],
    han_viet: &HanVietMap,
    mapped_source_ranges: &[CharRange],
) -> TranslationResult {
    debug_assert_eq!(chars.len(), mapped_source_ranges.len());
    let mut result = String::new();
    let mut last = String::new();
    let mut source_ranges = Vec::with_capacity(chars.len());
    let mut target_ranges = Vec::with_capacity(chars.len());
    let mut target_start = 0usize;
    // Original inits LastTranslatedWord = "" — the first word gets a leading
    // space and no capitalization. Faithful to TranslatorEngine.ChineseToHanViet.
    let len = chars.len();
    if len == 0 {
        return TranslationResult {
            translated_text: result,
            source_ranges,
            target_ranges,
        };
    }
    for i in 0..len {
        let c = chars[i];
        let source_length = c.len_utf16();
        let range_target_start = target_start;
        let target_length;
        if is_chinese(c, han_viet) {
            let reading = char_to_han_viet(c, han_viet);
            target_length = utf16_len(&reading);
            let delta = append_translated_word(&mut result, &reading, &mut last);
            target_start = target_start.saturating_add_signed(delta);
            if i + 1 < len && is_chinese(chars[i + 1], han_viet) {
                result.push(' ');
                last.push(' ');
                target_start += 1;
            }
        } else {
            result.push(c);
            last.push(c);
            target_length = source_length;
            target_start += target_length;
            if needs_space_after_sentence_punctuation(chars, i) {
                result.push(' ');
                last.push(' ');
                target_start += 1;
            }
        }
        source_ranges.push(mapped_source_ranges[i]);
        target_ranges.push(CharRange {
            start: range_target_start,
            length: target_length,
        });
    }
    TranslationResult {
        translated_text: result,
        source_ranges,
        target_ranges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hv() -> HanVietMap {
        let mut m = HanVietMap::default();
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

    #[test]
    fn maps_utf16_ranges_for_han_viet_and_non_bmp_text() {
        let result = chinese_to_han_viet(&['😀', '一'], &hv());
        assert_eq!(result.translated_text, "😀 nhất");
        assert_eq!(
            result.source_ranges,
            vec![
                CharRange {
                    start: 0,
                    length: 2
                },
                CharRange {
                    start: 2,
                    length: 1
                }
            ]
        );
        assert_eq!(
            result.target_ranges[0],
            CharRange {
                start: 0,
                length: 2
            }
        );
        assert_eq!(
            result.target_ranges[1],
            CharRange {
                start: 2,
                length: 4
            }
        );
    }
}
