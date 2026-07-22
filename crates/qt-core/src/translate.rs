//! The TranslateAll longest-match loop, Luật Nhân, number branch, and ranges.

use std::collections::HashMap;

use crate::dict::DictionaryLookup;

fn substr(chars: &[char], start: usize, len: usize) -> String {
    chars[start..start + len].iter().collect()
}

/// Mirrors isLongestPhraseInSentence (docs/engine/translation-algorithm.md §4).
pub fn is_longest_phrase_in_sentence(
    chars: &[char],
    start: usize,
    phrase_len: usize,
    dict: &dyn DictionaryLookup,
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
    only_name: &dyn DictionaryLookup,
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
use crate::luat_nhan::LuatNhan;
use crate::number::{number_modifier, prescan_numbers, translate_number};
use crate::text::{
    append_translated_word, needs_space_after_sentence_punctuation, next_char_is_chinese,
    utf16_len, wrap_translation,
};
use crate::{CharRange, Options, TranslationResult};

#[derive(Default)]
struct TranslationOutput {
    result: String,
    last: String,
    source_ranges: Vec<CharRange>,
    target_ranges: Vec<CharRange>,
    target_length: usize,
}

impl TranslationOutput {
    fn append_word(&mut self, text: &str) {
        let delta = append_translated_word(&mut self.result, text, &mut self.last);
        self.target_length = self.target_length.saturating_add_signed(delta);
    }

    fn append_char(&mut self, c: char) {
        self.result.push(c);
        self.last.push(c);
        self.target_length += c.len_utf16();
    }

    fn append_space(&mut self) {
        self.result.push(' ');
        self.last.push(' ');
        self.target_length += 1;
    }

    fn finish(self) -> TranslationResult {
        TranslationResult {
            translated_text: self.result,
            source_ranges: self.source_ranges,
            target_ranges: self.target_ranges,
        }
    }
}

fn process_translation(
    chars: &[char],
    translation: &str,
    source_range: CharRange,
    source_end: usize,
    opts: &Options,
    output: &mut TranslationOutput,
    han_viet: &HanVietMap,
) {
    let text = wrap_translation(translation, opts.wrap_type);
    let text_length = utf16_len(&text);
    output.append_word(&text);
    output.source_ranges.push(source_range);
    output.target_ranges.push(CharRange {
        start: output.target_length.saturating_sub(text_length),
        length: text_length,
    });
    if next_char_is_chinese(chars, source_end, han_viet) {
        output.append_space();
    }
}

fn process_han_viet(
    chars: &[char],
    opts: &Options,
    num2: &mut usize,
    output: &mut TranslationOutput,
    han_viet: &HanVietMap,
    source_ranges: &[CharRange],
) {
    let c = chars[*num2];
    let target_start = output.target_length;
    let target_length;
    if is_chinese(c, han_viet) {
        let t = wrap_translation(&char_to_han_viet(c, han_viet), opts.wrap_type);
        target_length = utf16_len(&t);
        output.append_word(&t);
        if next_char_is_chinese(chars, *num2, han_viet) {
            output.append_space();
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
        output.append_space();
        output.append_char(c);
        target_length = c.len_utf16();
    } else {
        output.append_char(c);
        target_length = c.len_utf16();
        if needs_space_after_sentence_punctuation(chars, *num2) {
            output.append_space();
        }
    }
    output.source_ranges.push(source_ranges[*num2]);
    output.target_ranges.push(CharRange {
        start: target_start,
        length: target_length,
    });
    *num2 += 1;
}

#[cfg(test)]
fn source_ranges(chars: &[char]) -> Vec<CharRange> {
    let mut start = 0usize;
    chars
        .iter()
        .map(|ch| {
            let range = CharRange {
                start,
                length: ch.len_utf16(),
            };
            start += ch.len_utf16();
            range
        })
        .collect()
}

fn merged_source_range(source_ranges: &[CharRange], start: usize, length: usize) -> CharRange {
    let ranges = &source_ranges[start..start + length];
    let source_start = ranges.iter().map(|range| range.start).min().unwrap_or(0);
    let source_end = ranges
        .iter()
        .map(|range| range.start + range.length)
        .max()
        .unwrap_or(source_start);
    CharRange {
        start: source_start,
        length: source_end - source_start,
    }
}

/// Main loop, mirrors TranslateAll (docs/engine/translation-algorithm.md §3).
#[cfg(test)]
pub fn translate_all(
    chars: &[char],
    opts: &Options,
    dict: &HashMap<String, String>,
    only_name: &HashMap<String, String>,
    only_vietphrase: &HashMap<String, String>,
    han_viet: &HanVietMap,
) -> TranslationResult {
    let source_ranges = source_ranges(chars);
    translate_all_mapped(
        chars,
        &source_ranges,
        opts,
        dict,
        only_name,
        only_vietphrase,
        dict,
        han_viet,
        &LuatNhan::default(),
        None,
        None,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn translate_all_mapped(
    chars: &[char],
    source_ranges: &[CharRange],
    opts: &Options,
    dict: &dyn DictionaryLookup,
    only_name: &dyn DictionaryLookup,
    only_vietphrase: &HashMap<String, String>,
    full_vietphrase: &dyn DictionaryLookup,
    han_viet: &HanVietMap,
    luat_nhan: &LuatNhan,
    dictionary_n: Option<&dyn DictionaryLookup>,
    ho_nguoi: Option<&dyn DictionaryLookup>,
    hau_tu: Option<&dyn DictionaryLookup>,
) -> TranslationResult {
    debug_assert_eq!(chars.len(), source_ranges.len());
    let numbers = prescan_numbers(chars, only_vietphrase);
    let chars = number_modifier(chars);
    let chars = chars.as_slice();
    let mut output = TranslationOutput::default();
    let len = chars.len();
    if len == 0 {
        return output.finish();
    }
    let mut num2 = 0usize;
    let mut num3 = -1isize;
    let mut num4 = -1isize;
    let mut num5 = -1isize;
    while num2 < len {
        let mut flag = false;
        let mut flag2 = true;
        let mut num6 = opts.scan_range;
        let number = numbers.get(&num2);
        while num6 > 0 && number.is_none_or(|value| num6 >= value.length) {
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
                        process_translation(
                            chars,
                            value2,
                            merged_source_range(source_ranges, num2, num6),
                            num2 + num6 - 1,
                            opts,
                            &mut output,
                            han_viet,
                        );
                        flag = true;
                        num2 += num6;
                        break;
                    }
                } else if !text.contains('\n')
                    && !text.contains('\t')
                    && flag2
                    && num6 > 2
                    && num3 < (num2 + num6 - 1) as isize
                {
                    if num2 as isize >= num4 {
                        let matched = luat_nhan.contains(
                            &chars[num2..num2 + num6],
                            only_vietphrase,
                            full_vietphrase,
                            dictionary_n,
                            ho_nguoi,
                            hau_tu,
                        );
                        if let Some(matched) = matched {
                            num4 = (num2 + matched.index) as isize;
                            num5 = num4 + matched.length as isize;
                            if matched.index == 0
                                && (matched.key.contains("{n}")
                                    || !contains_name(chars, num2, matched.length, only_name))
                            {
                                let chinese = substr(chars, num2, matched.length);
                                if let Some(translation) = luat_nhan.translate(
                                    &chinese,
                                    &matched.key,
                                    &matched.value_n,
                                    ho_nguoi,
                                    hau_tu,
                                ) {
                                    process_translation(
                                        chars,
                                        translation.trim(),
                                        merged_source_range(source_ranges, num2, matched.length),
                                        num2 + matched.length - 1,
                                        opts,
                                        &mut output,
                                        han_viet,
                                    );
                                    flag = true;
                                    num2 += matched.length;
                                }
                            }
                        } else {
                            num4 = num2 as isize - 1;
                            num5 = num4 - 1;
                            num3 = (num2 + num6 - 1) as isize;
                            flag2 = false;
                            let mut longer = 100usize;
                            while num2 + longer < len
                                && is_chinese(chars[num2 + longer - 1], han_viet)
                            {
                                longer += 1;
                            }
                            if num2 + longer <= len
                                && luat_nhan
                                    .contains(
                                        &chars[num2..num2 + longer],
                                        only_vietphrase,
                                        full_vietphrase,
                                        dictionary_n,
                                        ho_nguoi,
                                        hau_tu,
                                    )
                                    .is_none()
                            {
                                num3 = (num2 + longer - 1) as isize;
                            }
                        }
                    } else if num4 < (num2 + num6) as isize && num6 as isize <= num5 - num4 {
                        // Faithful no-op: QT adjusts HandleNhanBy's by-value
                        // scan length here and immediately returns.
                    }
                    if flag {
                        break;
                    }
                }
            }
            num6 -= 1;
        }
        if flag {
            continue;
        }
        if let Some(number) = number {
            if let Some(translation) = translate_number(&number.text) {
                process_translation(
                    chars,
                    &translation,
                    merged_source_range(source_ranges, num2, number.length),
                    num2 + number.length - 1,
                    opts,
                    &mut output,
                    han_viet,
                );
                num2 += number.length;
                continue;
            }
        }
        process_han_viet(chars, opts, &mut num2, &mut output, han_viet, source_ranges);
    }
    output.finish()
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
        let got = translate_all(&chars, &opts, &d, &names, &HashMap::new(), &hanviet);
        assert_eq!(got.translated_text, " tha rất tốt");
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
        let got = translate_all(&chars, &opts, &d, &names, &HashMap::new(), &hanviet);
        assert_eq!(got.translated_text, " hồng trung nhân");
    }

    #[test]
    fn converts_numbers_but_raw_vietphrase_has_priority() {
        let chars: Vec<char> = "一百二十三人".chars().collect();
        let hanviet = hv(&[('人', "nhân")]);
        let result = translate_all(
            &chars,
            &Options::default(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &hanviet,
        );
        assert_eq!(result.translated_text, " 123 nhân");
        assert_eq!(
            result.source_ranges[0],
            CharRange {
                start: 0,
                length: 5
            }
        );
        assert_eq!(
            result.target_ranges[0],
            CharRange {
                start: 1,
                length: 3
            }
        );

        let raw = dict(&[("一百二十三", "một trăm hai ba")]);
        let result = translate_all(
            &chars,
            &Options::default(),
            &raw,
            &HashMap::new(),
            &raw,
            &hanviet,
        );
        assert_eq!(result.translated_text, " một trăm hai ba nhân");
    }

    #[test]
    fn source_ranges_are_utf16_offsets() {
        let chars: Vec<char> = "😀很好".chars().collect();
        let phrases = dict(&[("很好", "rất tốt")]);
        let result = translate_all(
            &chars,
            &Options::default(),
            &phrases,
            &HashMap::new(),
            &HashMap::new(),
            &hv(&[('很', "ngận"), ('好', "hảo")]),
        );
        assert_eq!(result.translated_text, "😀 rất tốt");
        assert_eq!(
            result.source_ranges,
            vec![
                CharRange {
                    start: 0,
                    length: 2
                },
                CharRange {
                    start: 2,
                    length: 2
                }
            ]
        );
        assert_eq!(
            result.target_ranges[1],
            CharRange {
                start: 3,
                length: 7
            }
        );
    }
}
