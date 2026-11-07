//! The TranslateAll longest-match loop, Luật Nhân, number branch, and ranges.

#[cfg(test)]
use std::collections::HashMap;

use crate::dict::DictionaryLookup;

fn substr(chars: &[char], start: usize, len: usize) -> String {
    chars[start..start + len].iter().collect()
}

/// Đổ `chars[start..start+cap]` vào buffer tái dùng, ghi lại offset byte sau
/// mỗi ký tự để cắt `&buffer[..offsets[n]]` cho từng độ dài mà không cấp phát.
pub(crate) fn fill_window(
    chars: &[char],
    start: usize,
    cap: usize,
    buffer: &mut String,
    offsets: &mut Vec<usize>,
) {
    buffer.clear();
    offsets.clear();
    offsets.push(0);
    for &c in &chars[start..start + cap] {
        buffer.push(c);
        offsets.push(buffer.len());
    }
}

/// Buffer tái dùng cho [`is_longest_phrase_in_sentence`] và [`contains_name`].
/// Cả hai chạy cho mọi cụm trúng từ điển (~0,4 lần mỗi ký tự); trước đây mỗi
/// lần cấp phát 2–3 String/Vec mới — bỏ được ~7% thời gian dịch.
#[derive(Default)]
pub(crate) struct PhraseScratch {
    key: String,
    buffer: String,
    offsets: Vec<usize>,
}

/// Mirrors isLongestPhraseInSentence (docs/engine/translation-algorithm.md §4).
/// Ngữ nghĩa quét n=20→threshold giữ nguyên; chỉ bỏ những n mà từ điển chắc
/// chắn không có key (theo chặn trên `max_key_len` — miss kiểu gì cũng miss).
pub(crate) fn is_longest_phrase_in_sentence(
    chars: &[char],
    start: usize,
    phrase_len: usize,
    dict: &dyn DictionaryLookup,
    algo: i32,
    scratch: &mut PhraseScratch,
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
    let PhraseScratch {
        buffer, offsets, ..
    } = scratch;
    for i in (start + 1)..=end {
        let cap = 20usize
            .min(chars.len().saturating_sub(i))
            .min(dict.max_key_len(chars[i]));
        if cap <= threshold {
            continue;
        }
        fill_window(chars, i, cap, buffer, offsets);
        let mut n = cap;
        while n > threshold {
            if dict.contains_key(&buffer[..offsets[n]]) {
                return false;
            }
            n -= 1;
        }
    }
    true
}

/// Mirrors containsName (docs/engine/translation-algorithm.md §5).
pub(crate) fn contains_name(
    chars: &[char],
    start: usize,
    phrase_len: usize,
    only_name: &dyn DictionaryLookup,
    scratch: &mut PhraseScratch,
) -> bool {
    if phrase_len < 2 {
        return false;
    }
    let PhraseScratch {
        key,
        buffer,
        offsets,
    } = scratch;
    key.clear();
    key.extend(&chars[start..start + phrase_len]);
    if only_name.contains_key(key) {
        return false;
    }
    let end = start + phrase_len - 1; // inclusive
    for i in (start + 1)..=end {
        let cap = 20usize
            .min(chars.len().saturating_sub(i))
            .min(only_name.max_key_len(chars[i]));
        if cap < 2 {
            continue;
        }
        fill_window(chars, i, cap, buffer, offsets);
        let mut n = cap;
        while n >= 2 {
            if only_name.contains_key(&buffer[..offsets[n]]) {
                return true;
            }
            n -= 1;
        }
    }
    false
}

use crate::han_viet::{char_to_han_viet, is_chinese, HanVietMap};
use crate::luat_nhan::{LuatNhan, LuatNhanScratch};
use crate::number::{number_modifier, prescan_numbers, translate_number};
use crate::text::{
    self, append_translated_word, needs_space_after_sentence_punctuation, next_char_is_chinese,
    utf16_len, wrap_translation,
};
use crate::{CharRange, Options, TranslationResult};

/// Một cụm đã ghi ra output: `start..start + len` là chỉ số ký tự nguồn (sau
/// chuẩn hoá và `number_modifier`), `out_start..out_end` là byte trong
/// `translated_text` (có thể dính dấu cách chèn trước). Quy Tắc đọc lại từ đây.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceToken {
    pub(crate) start: usize,
    pub(crate) len: usize,
    pub(crate) out_start: usize,
    pub(crate) out_end: usize,
    /// Chữ đầu gốc của nghĩa khi engine viết hoa nó vì đứng đầu câu
    /// (`text::capitalized_char`); `None` khi chữ hoa (nếu có) là của nghĩa.
    pub(crate) capital: Option<char>,
}

#[derive(Default)]
struct TranslationOutput<'t> {
    result: String,
    last: String,
    source_ranges: Vec<CharRange>,
    target_ranges: Vec<CharRange>,
    target_length: usize,
    tokens: Option<&'t mut Vec<SourceToken>>,
    /// `capitalized_char` của chunk vừa nối, chờ `record` ghi vào cụm; chỉ tính
    /// khi có ghi token (đường Quy Tắc).
    capital: Option<char>,
}

impl TranslationOutput<'_> {
    fn append_word(&mut self, text: &str) {
        if self.tokens.is_some() {
            self.capital = text::capitalized_char(&self.last, text);
        }
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

    /// Ghi cụm vừa nối (`out_start` tới cuối output). `append_translated_word`
    /// có thể đã bỏ dấu cách cuối output, và dấu đó có thể thuộc một cụm trước
    /// cả cụm liền trước: sau cụm nghĩa rỗng, range rỗng nằm ở cuối output.
    /// Range trước đó tăng dần nên các cụm chạy quá `out_start` là một đoạn
    /// cuối danh sách: cắt hết về `out_start`, chúng chỉ mất đúng dấu cách đó.
    fn record(&mut self, start: usize, len: usize, out_start: usize) {
        let out_end = self.result.len();
        debug_assert!(out_start <= out_end, "range cụm bị đảo: {out_start}..{out_end}");
        let capital = self.capital.take();
        if let Some(tokens) = self.tokens.as_deref_mut() {
            for token in tokens.iter_mut().rev() {
                if token.out_end <= out_start {
                    break;
                }
                token.out_end = out_start;
                token.out_start = token.out_start.min(out_start);
            }
            tokens.push(SourceToken {
                start,
                len,
                out_start,
                out_end,
                capital,
            });
        }
    }

    fn finish(self) -> TranslationResult {
        TranslationResult {
            translated_text: self.result,
            source_ranges: self.source_ranges,
            target_ranges: self.target_ranges,
        }
    }
}

/// Ghi cụm `chars[start..start + length]` đã dịch thành `translation`.
/// `source_ranges` là `None` khi caller không cần range (đường chỉ trả
/// text): khỏi tính và giữ hai mảng range cho cả văn bản.
#[allow(clippy::too_many_arguments)]
fn process_translation(
    chars: &[char],
    translation: &str,
    source_ranges: Option<&[CharRange]>,
    start: usize,
    length: usize,
    opts: &Options,
    output: &mut TranslationOutput,
    han_viet: &HanVietMap,
) {
    let text = wrap_translation(translation, opts.wrap_type);
    output.append_word(&text);
    let out_start = output.result.len() - output.last.len();
    output.record(start, length, out_start);
    if let Some(source_ranges) = source_ranges {
        let text_length = utf16_len(&text);
        output
            .source_ranges
            .push(merged_source_range(source_ranges, start, length));
        output.target_ranges.push(CharRange {
            start: output.target_length.saturating_sub(text_length),
            length: text_length,
        });
    }
    // Đường Quy Tắc (có ghi cụm): cụm nguồn và nghĩa cùng kết thúc bằng ngoặc
    // thẳng mà ngay sau là chữ Hán (`道, "` = `nói, "`) thì ngoặc đó là ngoặc
    // mở, như ngoặc đứng riêng ở `process_han_viet`: không chèn dấu cách sau
    // nó, chữ kế viết hoa. Đường hiện tại giữ nguyên (`nói, " đi`); ở đó Luật
    // Nhân thường nuốt cụm trước (`{s}道`), đường Quy Tắc thì không.
    let quote = chars[start + length - 1];
    let opens = output.tokens.is_some() && matches!(quote, '"' | '\'') && text.ends_with(quote);
    if next_char_is_chinese(chars, start + length - 1, han_viet) && !opens {
        output.append_space();
    }
}

fn process_han_viet(
    chars: &[char],
    opts: &Options,
    num2: &mut usize,
    output: &mut TranslationOutput,
    han_viet: &HanVietMap,
    source_ranges: Option<&[CharRange]>,
) {
    let c = chars[*num2];
    let target_start = output.target_length;
    let target_length;
    if is_chinese(c, han_viet) {
        let t = wrap_translation(&char_to_han_viet(c, han_viet), opts.wrap_type);
        target_length = utf16_len(&t);
        output.append_word(&t);
        let out_start = output.result.len() - output.last.len();
        output.record(*num2, 1, out_start);
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
        output.record(*num2, 1, output.result.len() - c.len_utf8());
        target_length = c.len_utf16();
    } else {
        output.append_char(c);
        output.record(*num2, 1, output.result.len() - c.len_utf8());
        target_length = c.len_utf16();
        if needs_space_after_sentence_punctuation(chars, *num2) {
            output.append_space();
        }
    }
    if let Some(source_ranges) = source_ranges {
        output.source_ranges.push(source_ranges[*num2]);
        output.target_ranges.push(CharRange {
            start: target_start,
            length: target_length,
        });
    }
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
        Some(&source_ranges),
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
        None,
    )
}

/// `source_ranges`: range nguồn của từng ký tự; `None` = không dựng range
/// (kết quả có `source_ranges`/`target_ranges` rỗng, text giữ nguyên).
#[allow(clippy::too_many_arguments)]
pub(crate) fn translate_all_mapped(
    chars: &[char],
    source_ranges: Option<&[CharRange]>,
    opts: &Options,
    dict: &dyn DictionaryLookup,
    only_name: &dyn DictionaryLookup,
    only_vietphrase: &dyn DictionaryLookup,
    full_vietphrase: &dyn DictionaryLookup,
    han_viet: &HanVietMap,
    luat_nhan: &LuatNhan,
    dictionary_n: Option<&dyn DictionaryLookup>,
    ho_nguoi: Option<&dyn DictionaryLookup>,
    hau_tu: Option<&dyn DictionaryLookup>,
    tokens: Option<&mut Vec<SourceToken>>,
) -> TranslationResult {
    debug_assert!(source_ranges.is_none_or(|ranges| ranges.len() == chars.len()));
    let numbers = prescan_numbers(chars, only_vietphrase);
    let chars = number_modifier(chars);
    let chars = chars.as_slice();
    let mut output = TranslationOutput {
        tokens,
        ..TranslationOutput::default()
    };
    let len = chars.len();
    if len == 0 {
        return output.finish();
    }
    let mut num2 = 0usize;
    let mut num3 = -1isize;
    let mut num4 = -1isize;
    let mut num5 = -1isize;
    // Buffer cửa sổ tái dùng giữa các vị trí: đổ một lần mỗi vị trí rồi cắt
    // theo offset byte cho từng độ dài, thay vì cấp phát String mỗi probe.
    let mut window = String::new();
    let mut window_offsets: Vec<usize> = Vec::new();
    let mut luat_nhan_scratch = LuatNhanScratch::default();
    let mut phrase_scratch = PhraseScratch::default();
    while num2 < len {
        let mut flag = false;
        let mut flag2 = true;
        let mut num6 = opts.scan_range;
        let number = numbers.get(&num2);
        // Chặn trên độ dài key theo ký tự đầu: num6 lớn hơn mức này thì
        // dict.get kiểu gì cũng miss nên khỏi tra; nhánh Luật Nhân bên dưới
        // vẫn chạy đủ mọi num6 như cũ.
        let window_cap = opts.scan_range.min(len - num2);
        let dict_max = dict.max_key_len(chars[num2]).min(window_cap);
        fill_window(chars, num2, dict_max, &mut window, &mut window_offsets);
        // Vị trí \n/\t đầu tiên trong cửa sổ — điều kiện vào nhánh Luật Nhân
        // (trước đây là text.contains, quét lại chuỗi ở mỗi độ dài).
        let first_break = chars[num2..num2 + window_cap]
            .iter()
            .position(|&c| c == '\n' || c == '\t')
            .unwrap_or(window_cap);
        while num6 > 0 && number.is_none_or(|value| num6 >= value.length) {
            if num2 + num6 <= len {
                let hit = if num6 <= dict_max {
                    dict.get(&window[..window_offsets[num6]])
                } else {
                    None
                };
                if let Some(value2) = hit {
                    let text = &window[..window_offsets[num6]];
                    // Hai điều kiện đều là hàm thuần nên tính lười: thuật toán
                    // 1 (mặc định) không cần is_longest — trước đây vẫn tính
                    // cho mọi cụm trúng, tốn ~8% thời gian dịch.
                    let name_ok = !opts.prioritized_name
                        || !contains_name(chars, num2, num6, only_name, &mut phrase_scratch);
                    let accepted = name_ok
                        && ((opts.translation_algorithm != 0 && opts.translation_algorithm != 2)
                            || is_longest_phrase_in_sentence(
                                chars,
                                num2,
                                num6,
                                dict,
                                opts.translation_algorithm,
                                &mut phrase_scratch,
                            )
                            || (opts.prioritized_name && only_name.contains_key(text)));
                    if accepted {
                        process_translation(
                            chars,
                            value2,
                            source_ranges,
                            num2,
                            num6,
                            opts,
                            &mut output,
                            han_viet,
                        );
                        flag = true;
                        num2 += num6;
                        break;
                    }
                } else if first_break >= num6
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
                            &mut luat_nhan_scratch,
                        );
                        if let Some(matched) = matched {
                            num4 = (num2 + matched.index) as isize;
                            num5 = num4 + matched.length as isize;
                            if matched.index == 0
                                && (matched.key.contains("{n}")
                                    || !contains_name(
                                        chars,
                                        num2,
                                        matched.length,
                                        only_name,
                                        &mut phrase_scratch,
                                    ))
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
                                        source_ranges,
                                        num2,
                                        matched.length,
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
                                        &mut luat_nhan_scratch,
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
                    source_ranges,
                    num2,
                    number.length,
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
        let scratch = &mut PhraseScratch::default();
        assert!(!is_longest_phrase_in_sentence(&chars, 0, 2, &d, 0, scratch));
        // algo 1: threshold = max(2,3)=3, so only len>3 overlaps reject; BCD is len 3 → not > 3 → longest
        assert!(is_longest_phrase_in_sentence(&chars, 0, 2, &d, 1, scratch));
    }

    #[test]
    fn contains_name_detects_inner_name() {
        let chars: Vec<char> = "ABCD".chars().collect();
        let names = dict(&[("BC", "Name")]);
        let scratch = &mut PhraseScratch::default();
        // phrase "ABCD" (len 4) is not itself a name, but "BC" name starts inside → true
        assert!(contains_name(&chars, 0, 4, &names, scratch));
        // if the phrase itself is a name, returns false
        let names2 = dict(&[("ABCD", "Name"), ("BC", "Name")]);
        assert!(!contains_name(&chars, 0, 4, &names2, scratch));
    }

    /// Scratch dùng chung qua nhiều lần gọi: nội dung cũ (cụm dài hơn, vị trí
    /// khác) không được ảnh hưởng kết quả lần sau.
    #[test]
    fn phrase_scratch_reuse_matches_fresh_scratch() {
        let chars: Vec<char> = "ABCDEFGH".chars().collect();
        let names = dict(&[("CD", "Name"), ("FG", "Name"), ("ABCDEFG", "Name")]);
        let shared = &mut PhraseScratch::default();
        for (start, len) in [(0, 7), (0, 8), (4, 2), (1, 4), (5, 3), (0, 2), (2, 2)] {
            let fresh = contains_name(&chars, start, len, &names, &mut PhraseScratch::default());
            assert_eq!(
                contains_name(&chars, start, len, &names, shared),
                fresh,
                "contains_name({start}, {len})"
            );
            for algo in [0, 1] {
                let fresh = is_longest_phrase_in_sentence(
                    &chars,
                    start,
                    len,
                    &names,
                    algo,
                    &mut PhraseScratch::default(),
                );
                assert_eq!(
                    is_longest_phrase_in_sentence(&chars, start, len, &names, algo, shared),
                    fresh,
                    "is_longest({start}, {len}, {algo})"
                );
            }
        }
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

    /// Vòng dịch có ghi token; kiểm bất biến của `SourceToken` rồi trả output
    /// cùng chữ (đã cắt khoảng trắng) của từng cụm.
    fn translate_tokens(
        text: &str,
        dict: &HashMap<String, String>,
        han_viet: &HanVietMap,
    ) -> (String, Vec<String>) {
        let chars: Vec<char> = text.chars().collect();
        let mut tokens = Vec::new();
        let result = translate_all_mapped(
            &chars,
            None,
            &Options::default(),
            dict,
            &HashMap::new(),
            dict,
            dict,
            han_viet,
            &LuatNhan::default(),
            None,
            None,
            None,
            Some(&mut tokens),
        );
        let output = result.translated_text;
        let (mut source, mut out) = (0, 0);
        for token in &tokens {
            assert_eq!(token.start, source, "cụm nối liền nhau trên nguồn: {tokens:?}");
            assert!(token.len > 0, "{tokens:?}");
            assert!(
                out <= token.out_start
                    && token.out_start <= token.out_end
                    && token.out_end <= output.len(),
                "range tăng dần, không đảo, nằm trong output {output:?}: {tokens:?}"
            );
            assert!(
                output.is_char_boundary(token.out_start) && output.is_char_boundary(token.out_end),
                "{output:?}: {tokens:?}"
            );
            // Giữa hai cụm chỉ có dấu cách chèn thêm, không mất chữ nào.
            assert!(output[out..token.out_start].bytes().all(|b| b == b' '), "{output:?}");
            source += token.len;
            out = token.out_end;
        }
        assert_eq!(source, chars.len(), "{tokens:?}");
        assert!(output[out..].bytes().all(|b| b == b' '), "{output:?}");
        let texts = tokens
            .iter()
            .map(|t| output[t.out_start..t.out_end].trim().to_string())
            .collect();
        (output, texts)
    }

    /// Vòng dịch ra `(output, cụm)`; `record` là `false` thì như đường hiện tại
    /// (không ghi cụm).
    fn translate_recorded(
        text: &str,
        dict: &HashMap<String, String>,
        han_viet: &HanVietMap,
        record: bool,
    ) -> (String, Vec<SourceToken>) {
        let chars: Vec<char> = text.chars().collect();
        let mut tokens = Vec::new();
        let result = translate_all_mapped(
            &chars,
            None,
            &Options::default(),
            dict,
            &HashMap::new(),
            dict,
            dict,
            han_viet,
            &LuatNhan::default(),
            None,
            None,
            None,
            record.then_some(&mut tokens),
        );
        (result.translated_text, tokens)
    }

    /// Cụm ghi lại chữ đầu gốc khi engine viết hoa nó vì đứng đầu câu; nghĩa
    /// vốn viết hoa (`Hảo Hán`) thì không ghi.
    #[test]
    fn record_notes_sentence_start_capitals() {
        let d = dict(&[("很好", "rất tốt"), ("好汉", "Hảo Hán")]);
        let han_viet = hv(&[('他', "tha")]);
        let (output, tokens) = translate_recorded("他. 很好. 好汉", &d, &han_viet, true);
        assert_eq!(output, " tha. Rất tốt. Hảo Hán");
        let capitals: Vec<Option<char>> = tokens.iter().map(|t| t.capital).collect();
        assert_eq!(capitals, [None, None, None, Some('r'), None, None, None]);
    }

    /// Đường Quy Tắc: nghĩa và cụm nguồn cùng kết thúc bằng ngoặc thẳng, ngay
    /// sau là chữ Hán, thì ngoặc là ngoặc mở: không chèn dấu cách, chữ kế viết
    /// hoa. Đường hiện tại (không ghi cụm) giữ nguyên như trước.
    #[test]
    fn dictionary_chunk_ending_in_an_opening_quote() {
        let d = dict(&[("道, \"", "nói, \""), ("去", "đi"), ("杀字", "chữ \"giết\"")]);
        let han_viet = hv(&[('去', "khứ"), ('就', "tựu")]);
        let (output, _) = translate_recorded("道, \"去", &d, &han_viet, false);
        assert_eq!(output, " nói, \" đi");
        let (output, _) = translate_recorded("道, \"去", &d, &han_viet, true);
        assert_eq!(output, " nói, \"Đi");
        // Ngoặc trong nghĩa mà cụm nguồn không kết thúc bằng ngoặc: không đổi.
        let (output, _) = translate_recorded("杀字就", &d, &han_viet, true);
        assert_eq!(output, " chữ \"giết\" tựu");
    }

    /// Cụm mở đầu bằng dấu câu bỏ dấu cách cuối output; dấu đó là cụm ký tự
    /// ' ' liền trước nên range của nó bị cắt về rỗng. 他 đi qua Hán Việt, 'Z'
    /// đi qua nguyên ký tự.
    #[test]
    fn record_trims_the_space_dropped_before_punctuation() {
        let d = dict(&[("好", ", hảo")]);
        let han_viet = hv(&[('他', "tha")]);
        let (output, texts) = translate_tokens("他 好Z", &d, &han_viet);
        assert_eq!(output, " tha, hảoZ");
        assert_eq!(texts, ["tha", "", ", hảo", "Z"]);
    }

    /// Nghĩa rỗng (的, 了 trong QT2025): dấu cách bị bỏ có thể thuộc cụm trước
    /// cả cụm liền trước. Trước đây range cụm giữa bị đảo.
    #[test]
    fn empty_meanings_keep_token_ranges_valid() {
        let d = dict(&[("的", ""), ("了", ""), ("X", ""), ("Y", ", ý")]);
        let han_viet = hv(&[('的', "đích"), ('了', "liễu")]);
        let (output, texts) = translate_tokens("的的XX", &d, &han_viet);
        assert_eq!(output, "");
        assert_eq!(texts, ["", "", "", ""]);
        let (output, texts) = translate_tokens("了的Y", &d, &han_viet);
        assert_eq!(output, " , ý");
        assert_eq!(texts, ["", "", ", ý"]);
    }

    #[test]
    fn token_ranges_hold_on_varied_input() {
        let d = dict(&[
            ("很好", "rất tốt"),
            ("说", "nói"),
            ("的", ""),
            ("了", ""),
            ("X", ""),
            ("Y", ", ý"),
            ("好", ", hảo"),
        ]);
        let han_viet = hv(&[
            ('他', "tha"),
            ('很', "ngận"),
            ('好', "hảo"),
            ('说', "thuyết"),
            ('的', "đích"),
            ('了', "liễu"),
            ('人', "nhân"),
        ]);
        let text = "他很好，“他”说：了的的X Y\n一百二十三人。他 好!'他'了的。\t的Y好\n\n";
        let (_, texts) = translate_tokens(text, &d, &han_viet);
        assert!(texts.iter().any(|t| t == "rất tốt"), "{texts:?}");
        assert!(texts.iter().any(|t| t == "123"), "{texts:?}");
        // Chuỗi ngẫu nhiên (cố định hạt giống) trên cùng bảng chữ.
        let alphabet: Vec<char> = "他很好，“”说：了的XY \n一二三百人。!'\"\t(".chars().collect();
        let mut seed: u64 = 7;
        for _ in 0..20_000 {
            let mut next = || {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                (seed >> 33) as usize
            };
            let len = next() % 10;
            let text: String = (0..len).map(|_| alphabet[next() % alphabet.len()]).collect();
            translate_tokens(&text, &d, &han_viet);
        }
    }
}
