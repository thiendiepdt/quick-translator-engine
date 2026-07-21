//! The TranslateAll longest-match loop and its predicates.
//! Number conversion and Luật Nhân are STUBBED in this plan (see docs/engine/).

use std::collections::HashMap;

fn substr(chars: &[char], start: usize, len: usize) -> String {
    chars[start..start + len].iter().collect()
}

/// Mirrors isLongestPhraseInSentence (docs/engine/translation-algorithm.md §4).
pub fn is_longest_phrase_in_sentence(
    chars: &[char],
    start: usize,
    phrase_len: usize,
    dict: &HashMap<String, String>,
    algo: i32,
) -> bool {
    if phrase_len < 2 {
        return true;
    }
    let threshold = if algo == 0 {
        phrase_len
    } else {
        phrase_len.max(3)
    };
    let end = start + phrase_len - 1; // inclusive
    for i in (start + 1)..=end {
        let mut n = 20usize;
        while n > threshold {
            if chars.len() >= i + n && dict.contains_key(&substr(chars, i, n)) {
                return false;
            }
            n -= 1;
        }
    }
    true
}

/// Mirrors containsName (docs/engine/translation-algorithm.md §5).
pub fn contains_name(
    chars: &[char],
    start: usize,
    phrase_len: usize,
    only_name: &HashMap<String, String>,
) -> bool {
    if phrase_len < 2 || only_name.contains_key(&substr(chars, start, phrase_len)) {
        return false;
    }
    let end = start + phrase_len - 1; // inclusive
    for i in (start + 1)..=end {
        let mut n = 20usize;
        while n >= 2 {
            if chars.len() >= i + n && only_name.contains_key(&substr(chars, i, n)) {
                return true;
            }
            n -= 1;
        }
    }
    false
}

use crate::han_viet::{char_to_han_viet, is_chinese, HanVietMap};
use crate::text::{append_translated_word, next_char_is_chinese, wrap_translation};
use crate::Options;

// ---- Stubs for the later plan (number conversion + Luật Nhân) ----
// Kept as real functions so the later plan only swaps bodies.

/// STUB: real version reorders 余/多 with a following 百/千/万/亿. Identity for MVP.
fn number_modifier(chars: &[char]) -> Vec<char> {
    chars.to_vec()
}

#[derive(Default)]
struct TranslationOutput {
    result: String,
    last: String,
}

fn process_translation(
    chars: &[char],
    translation: &str,
    start: usize,
    length: usize,
    opts: &Options,
    output: &mut TranslationOutput,
    han_viet: &HanVietMap,
) {
    let text = wrap_translation(translation, opts.wrap_type);
    append_translated_word(&mut output.result, &text, &mut output.last);
    if next_char_is_chinese(chars, start + length - 1, han_viet) {
        output.result.push(' ');
        output.last.push(' ');
    }
}

fn process_han_viet(
    chars: &[char],
    opts: &Options,
    num2: &mut usize,
    output: &mut TranslationOutput,
    han_viet: &HanVietMap,
) {
    let c = chars[*num2];
    if is_chinese(c, han_viet) {
        let t = wrap_translation(&char_to_han_viet(c, han_viet), opts.wrap_type);
        append_translated_word(&mut output.result, &t, &mut output.last);
        if next_char_is_chinese(chars, *num2, han_viet) {
            output.result.push(' ');
            output.last.push(' ');
        }
    } else if (c == '"' || c == '\'')
        && !output.last.ends_with(' ')
        && !output.last.ends_with('.')
        && !output.last.ends_with('?')
        && !output.last.ends_with('!')
        && !output.last.ends_with('\t')
        && *num2 < chars.len() - 1
        && chars[*num2 + 1] != ' '
        && chars[*num2 + 1] != ','
    {
        output.result.push(' ');
        output.result.push(c);
        output.last.push(' ');
        output.last.push(c);
    } else {
        output.result.push(c);
        output.last.push(c);
    }
    *num2 += 1;
}

/// Main loop, mirrors TranslateAll (docs/engine/translation-algorithm.md §3).
/// Number/Luật-Nhân branches are stubbed: `number_modifier` is identity and there is
/// no PreScanForNumbers / HandleNhanBy, so unmatched positions fall to HanViet.
pub fn translate_all(
    chars: &[char],
    opts: &Options,
    dict: &HashMap<String, String>,
    only_name: &HashMap<String, String>,
    han_viet: &HanVietMap,
) -> String {
    let chars = number_modifier(chars); // stub identity; keeps shape for later plan
    let chars = chars.as_slice();
    let mut output = TranslationOutput::default();
    let len = chars.len();
    if len == 0 {
        return output.result;
    }
    let mut num2 = 0usize;
    while num2 < len {
        let mut flag = false;
        let mut num6 = opts.scan_range;
        while num6 > 0 {
            if num2 + num6 <= len {
                let text = substr(chars, num2, num6);
                if let Some(value2) = dict.get(&text) {
                    let is_longest = is_longest_phrase_in_sentence(
                        chars,
                        num2,
                        num6,
                        dict,
                        opts.translation_algorithm,
                    );
                    let name_ok =
                        !opts.prioritized_name || !contains_name(chars, num2, num6, only_name);
                    let algo_ok = (opts.translation_algorithm != 0
                        && opts.translation_algorithm != 2)
                        || is_longest
                        || (opts.prioritized_name && only_name.contains_key(&text));
                    if name_ok && algo_ok {
                        process_translation(chars, value2, num2, num6, opts, &mut output, han_viet);
                        flag = true;
                        num2 += num6;
                        break;
                    }
                }
                // Luật Nhân branch (A2) — STUB: not implemented in this plan.
            }
            num6 -= 1;
        }
        if flag {
            continue;
        }
        // Number {s} branch (B) — STUB: no prescanned numbers in MVP.
        process_han_viet(chars, opts, &mut num2, &mut output, han_viet);
    }
    output.result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dict(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn hv(pairs: &[(char, &str)]) -> HanVietMap {
        pairs.iter().map(|(k, v)| (*k, v.to_string())).collect()
    }

    #[test]
    fn longest_phrase_algo0_rejects_overlapping_longer() {
        let chars: Vec<char> = "ABCD".chars().collect();
        // "AB" at 0 len 2; but "BCD" (len 3) exists overlapping at index 1 → not longest for algo 0
        let d = dict(&[("AB", "x"), ("BCD", "y")]);
        assert!(!is_longest_phrase_in_sentence(&chars, 0, 2, &d, 0));
        // algo 1: threshold = max(2,3)=3, so only len>3 overlaps reject; BCD is len 3 → not > 3 → longest
        assert!(is_longest_phrase_in_sentence(&chars, 0, 2, &d, 1));
    }

    #[test]
    fn contains_name_detects_inner_name() {
        let chars: Vec<char> = "ABCD".chars().collect();
        let names = dict(&[("BC", "Name")]);
        // phrase "ABCD" (len 4) is not itself a name, but "BC" name starts inside → true
        assert!(contains_name(&chars, 0, 4, &names));
        // if the phrase itself is a name, returns false
        let names2 = dict(&[("ABCD", "Name"), ("BC", "Name")]);
        assert!(!contains_name(&chars, 0, 4, &names2));
    }

    #[test]
    fn translates_longest_phrase_then_falls_back_to_hanviet() {
        // dict: 很好=rất tốt ; han-viet: 他=tha
        let chars: Vec<char> = "他很好".chars().collect();
        let d = dict(&[("很好", "rất tốt")]);
        let names = HashMap::new();
        let hanviet = hv(&[('他', "tha"), ('很', "ngận"), ('好', "hảo")]);
        let opts = Options::default();
        // 他 not in dict → HanViet 'tha'; then 很好 phrase → 'rất tốt'.
        // Faithful to the engine: last starts "" so the first word gets a LEADING
        // SPACE and stays lowercase (TranslateAll inits lastTranslatedWord = "").
        let got = translate_all(&chars, &opts, &d, &names, &hanviet);
        assert_eq!(got, " tha rất tốt");
    }

    #[test]
    fn name_priority_skips_phrase_covering_a_name() {
        // phrase 红中人 covers the 2-char name 中人 starting inside it.
        // With prioritized_name, containsName rejects the phrase (inner name has
        // length >= 2), so 红 falls to HanViet and 中人 is translated as the name.
        // (A single-char inner name would NOT trigger containsName, which only
        // scans lengths >= 2 — hence a 2-char inner name here.)
        let chars: Vec<char> = "红中人".chars().collect();
        let d = dict(&[("红中人", "cả cụm"), ("中人", "trung nhân")]);
        let names = dict(&[("中人", "trung nhân")]);
        let hanviet = hv(&[('红', "hồng"), ('中', "trung"), ('人', "nhân")]);
        let opts = Options {
            prioritized_name: true,
            ..Options::default()
        };
        let got = translate_all(&chars, &opts, &d, &names, &hanviet);
        assert_eq!(got, " hồng trung nhân");
    }
}
