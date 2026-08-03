//! QT2025's pattern-based “Luật Nhân” translation rules.

use crate::dict::{Dictionaries, DictionaryLookup, IndexedLookup, KeyLenIndex};
use crate::number::{
    chinese_digit, convert_chinese_number_to_i64, is_number_start, number_to_vietnamese_text,
    translate_range_number,
};
use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use std::collections::{HashMap, HashSet};

struct CompiledRule {
    key: String,
    value: String,
    regex: FancyRegex,
    /// Ký tự literal mở đầu key (nếu có): mọi match đều phải chứa nó, nên
    /// cửa sổ không có ký tự này thì khỏi chạy regex.
    trigger: Option<char>,
}

/// Literal BẮT BUỘC đầu tiên trong key: mọi match của rule đều phải chứa ký
/// tự này, nên cửa sổ không có nó thì khỏi chạy regex. Bỏ qua placeholder
/// `{n}`/`{s}`, nhóm `(a|b)` (không đơn nhất), phần tùy chọn `[x]`/`x?` và
/// whitespace (được compile thành `\s*`). Không tìm được thì trả None —
/// rule luôn được chạy, an toàn tuyệt đối.
fn rule_trigger(key: &str) -> Option<char> {
    let chars: Vec<char> = key.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        match chars[index] {
            '{' => {
                while index < chars.len() && chars[index] != '}' {
                    index += 1;
                }
                index += 1;
            }
            '[' => {
                while index < chars.len() && chars[index] != ']' {
                    index += 1;
                }
                index += 1;
                // dấu '?' theo sau lớp tùy chọn
                if chars.get(index) == Some(&'?') {
                    index += 1;
                }
            }
            '(' => {
                while index < chars.len() && chars[index] != ')' {
                    index += 1;
                }
                index += 1;
            }
            c if c.is_whitespace() => index += 1,
            '?' | '|' | ')' | ']' | '}' => index += 1,
            c => {
                // literal theo sau bởi '?' là tùy chọn — không bắt buộc
                if chars.get(index + 1) == Some(&'?') {
                    index += 2;
                    continue;
                }
                return Some(c);
            }
        }
    }
    None
}

#[derive(Debug)]
pub(crate) struct RuleMatch {
    pub index: usize,
    pub length: usize,
    pub key: String,
    pub value_n: String,
}

#[derive(Default)]
pub(crate) struct LuatNhan {
    n_rules: Vec<CompiledRule>,
    s_rules: Vec<CompiledRule>,
    dictionary_n: HashMap<String, String>,
    ho_nguoi: HashMap<String, String>,
    hau_tu: HashMap<String, String>,
    ho_nguoi_index: KeyLenIndex,
    hau_tu_index: KeyLenIndex,
}

impl LuatNhan {
    pub fn new(dicts: &Dictionaries) -> Self {
        let mut rules = Self::try_from_rules(&dicts.luat_nhan)
            .unwrap_or_else(|error| panic!("invalid base LuatNhan dictionary: {error}"));
        let mut dictionary_n = dicts.pronouns.clone();
        for (key, value) in &dicts.only_name_one_meaning {
            dictionary_n
                .entry(key.clone())
                .or_insert_with(|| value.clone());
        }

        rules.dictionary_n = dictionary_n;
        rules.ho_nguoi = dicts.ho_nguoi.clone();
        rules.hau_tu = dicts.hau_tu.clone();
        rules.ho_nguoi_index = KeyLenIndex::from_keys(rules.ho_nguoi.keys());
        rules.hau_tu_index = KeyLenIndex::from_keys(rules.hau_tu.keys());
        rules
    }

    pub fn try_from_rules(rules: &[(String, String)]) -> Result<Self, String> {
        let mut ordered_rules = rules.to_vec();
        ordered_rules.sort_by(|(left, _), (right, _)| {
            right
                .chars()
                .count()
                .cmp(&left.chars().count())
                .then_with(|| left.cmp(right))
        });

        let mut n_sources: Vec<_> = ordered_rules
            .iter()
            .filter(|(key, _)| key.contains("{n}") && !key.contains("{s}"))
            .cloned()
            .collect();
        // LINQ OrderByDescending is stable, so ties retain the already sorted
        // LuatNhan dictionary order.
        n_sources.sort_by(|(left, _), (right, _)| {
            normalized_length(right).cmp(&normalized_length(left))
        });

        let n_rules = n_sources
            .into_iter()
            .map(|(key, value)| {
                let pattern = compile_n_pattern(&key);
                let regex = FancyRegex::new(&pattern)
                    .map_err(|error| format!("invalid LuatNhan rule {key:?}: {error}"))?;
                let trigger = rule_trigger(&key);
                Ok(CompiledRule {
                    key,
                    value,
                    regex,
                    trigger,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        let s_rules = ordered_rules
            .iter()
            .filter(|(key, _)| key.as_str() != "{s}" && key.contains("{s}") && !key.contains("{n}"))
            .cloned()
            .map(|(key, value)| {
                let pattern = compile_s_pattern(&key);
                let regex = FancyRegex::new(&pattern)
                    .map_err(|error| format!("invalid LuatNhan rule {key:?}: {error}"))?;
                let trigger = rule_trigger(&key);
                Ok(CompiledRule {
                    key,
                    value,
                    regex,
                    trigger,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(Self {
            n_rules,
            s_rules,
            ..Default::default()
        })
    }

    pub fn contains(
        &self,
        chinese: &[char],
        only_vietphrase: &dyn DictionaryLookup,
        vietphrase: &dyn DictionaryLookup,
        dictionary_n: Option<&dyn DictionaryLookup>,
        ho_nguoi: Option<&dyn DictionaryLookup>,
        hau_tu: Option<&dyn DictionaryLookup>,
    ) -> Option<RuleMatch> {
        let text: String = chinese.iter().collect();
        let dictionary_n = dictionary_n.unwrap_or(&self.dictionary_n);
        // Map nội bộ đi kèm index chặn-trên của chính nó; map do caller đưa
        // vào tự mang index qua trait (hoặc mặc định "không biết").
        let internal_ho_nguoi = IndexedLookup {
            inner: &self.ho_nguoi,
            index: &self.ho_nguoi_index,
        };
        let internal_hau_tu = IndexedLookup {
            inner: &self.hau_tu,
            index: &self.hau_tu_index,
        };
        let ho_nguoi = ho_nguoi.unwrap_or(&internal_ho_nguoi);
        let hau_tu = hau_tu.unwrap_or(&internal_hau_tu);
        // Ký tự có mặt trong cửa sổ — mồi lọc để khỏi chạy regex vô ích.
        let present: HashSet<char> = chinese.iter().copied().collect();
        let mut best = self.match_n(&text, &present, dictionary_n);
        let mut best_index = best.as_ref().map_or(usize::MAX, |matched| matched.index);

        for index in 0..chinese.len().min(best_index) {
            if let Some(length) =
                self.find_ho_hau_phrase(chinese, index, vietphrase, ho_nguoi, hau_tu)
            {
                best = Some(RuleMatch {
                    index,
                    length,
                    key: "{h}{t}".to_string(),
                    value_n: String::new(),
                });
                best_index = index;
                break;
            }
        }

        if best_index > 0 && chinese.iter().any(|ch| is_number_start(*ch)) {
            if let Some(matched) = self.match_s(&text, &present, only_vietphrase) {
                if matched.index < best_index {
                    best = Some(matched);
                }
            }
        }
        best
    }

    pub fn translate(
        &self,
        chinese: &str,
        rule_key: &str,
        value_n: &str,
        ho_nguoi: Option<&dyn DictionaryLookup>,
        hau_tu: Option<&dyn DictionaryLookup>,
    ) -> Option<String> {
        if rule_key.contains("{n}") {
            if value_n.trim().is_empty() {
                return None;
            }
            let template = self
                .n_rules
                .iter()
                .find(|rule| rule.key == rule_key)?
                .value
                .as_str();
            return Some(template.replace("{n}", value_n.trim()));
        }

        if rule_key == "{s}两" {
            let prefix = chinese.strip_suffix('两')?;
            let number = convert_chinese_number_to_i64(prefix)?;
            return Some(format!("{} lượng", number_to_vietnamese_text(number)));
        }

        let template = self
            .s_rules
            .iter()
            .find(|rule| rule.key == rule_key)
            .map(|rule| rule.value.as_str());

        if rule_key == "百分[之]?{s}" {
            let value_n = (!value_n.trim().is_empty()).then_some(value_n.trim())?;
            return Some(template?.replace("{s}", &convert_decimal(value_n)?));
        }

        if rule_key.contains("{s}") {
            let template = template?;
            if rule_key.contains('余') || rule_key.contains('多') {
                if let Some(index) = chinese.find(['余', '多']) {
                    let mut number_text = chinese.to_string();
                    number_text.remove(index);
                    if let Some(number) = convert_chinese_number_to_i64(number_text.trim()) {
                        if number != 0 || number_text.trim() == "0" {
                            return Some(
                                template.replace("{s}", &number_to_vietnamese_text(number)),
                            );
                        }
                    }
                }
            }
            return translate_s_rule(chinese, rule_key, template);
        }

        if rule_key == "{h}{t}" {
            let ho_nguoi = ho_nguoi.unwrap_or(&self.ho_nguoi);
            let hau_tu = hau_tu.unwrap_or(&self.hau_tu);
            let chars: Vec<char> = chinese.chars().collect();
            for split in 1..chars.len() {
                let ho: String = chars[..split].iter().collect();
                let hau: String = chars[split..].iter().collect();
                if let (Some(ho_value), Some(hau_value)) = (ho_nguoi.get(&ho), hau_tu.get(&hau)) {
                    return Some(format!("{} {}", ho_value.trim(), hau_value.trim()));
                }
            }
        }
        None
    }

    fn match_n(
        &self,
        chinese: &str,
        present: &HashSet<char>,
        dictionary_n: &dyn DictionaryLookup,
    ) -> Option<RuleMatch> {
        for rule in &self.n_rules {
            if rule
                .trigger
                .is_some_and(|trigger| !present.contains(&trigger))
            {
                continue;
            }
            let captures = rule.regex.captures_iter(chinese);
            for captures in captures.flatten() {
                let Some(whole) = captures.get(0) else {
                    continue;
                };
                let Some(group) = captures.get(1) else {
                    continue;
                };
                let captured = group.as_str();
                let match_index = byte_to_char_index(chinese, whole.start());
                let match_length = whole.as_str().chars().count();
                let captured_chars: Vec<char> = captured.chars().collect();

                if rule.key.ends_with("{n}") {
                    for length in (1..=captured_chars.len()).rev() {
                        let key: String = captured_chars[..length].iter().collect();
                        if let Some(value) = dictionary_n.get(&key) {
                            return Some(RuleMatch {
                                index: match_index,
                                length: match_length - (captured_chars.len() - length),
                                key: rule.key.clone(),
                                value_n: value.to_string(),
                            });
                        }
                    }
                } else if rule.key.starts_with("{n}") {
                    for offset in 0..captured_chars.len() {
                        let key: String = captured_chars[offset..].iter().collect();
                        if let Some(value) = dictionary_n.get(&key) {
                            return Some(RuleMatch {
                                index: match_index + offset,
                                length: match_length - offset,
                                key: rule.key.clone(),
                                value_n: value.to_string(),
                            });
                        }
                    }
                } else if let Some(value) = dictionary_n.get(captured) {
                    return Some(RuleMatch {
                        index: match_index,
                        length: match_length,
                        key: rule.key.clone(),
                        value_n: value.to_string(),
                    });
                }
            }
        }
        None
    }

    fn match_s(
        &self,
        chinese: &str,
        present: &HashSet<char>,
        only_vietphrase: &dyn DictionaryLookup,
    ) -> Option<RuleMatch> {
        for rule in &self.s_rules {
            if rule
                .trigger
                .is_some_and(|trigger| !present.contains(&trigger))
            {
                continue;
            }
            let Some(captures) = rule.regex.captures(chinese).ok().flatten() else {
                continue;
            };
            let (Some(whole), Some(number)) = (captures.get(0), captures.get(1)) else {
                continue;
            };
            if only_vietphrase.contains_key(whole.as_str()) || number.as_str().trim().is_empty() {
                continue;
            }
            return Some(RuleMatch {
                index: byte_to_char_index(chinese, whole.start()),
                length: whole.as_str().chars().count(),
                key: rule.key.clone(),
                value_n: number.as_str().to_string(),
            });
        }
        None
    }

    fn find_ho_hau_phrase(
        &self,
        chinese: &[char],
        start: usize,
        vietphrase: &dyn DictionaryLookup,
        ho_nguoi: &dyn DictionaryLookup,
        hau_tu: &dyn DictionaryLookup,
    ) -> Option<usize> {
        // Không có họ nào bắt đầu bằng ký tự này thì mọi cách tách {h}{t}
        // đều fail — thoát ngay; đây là trường hợp của đa số vị trí.
        let ho_max = ho_nguoi.max_key_len(chinese[start]);
        if ho_max == 0 {
            return None;
        }
        let window_len = 20usize.min(chinese.len() - start);
        if window_len < 2 {
            return None;
        }
        // Chỉ đổ đúng phần buffer sẽ dùng: phrase tối đa 6 ký tự, còn vòng
        // "covered by longer" bị chặn bởi key vietphrase dài nhất có thể.
        let longer_cap = window_len.min(vietphrase.max_key_len(chinese[start]));
        let fill_len = window_len.min(6usize.max(longer_cap));
        let mut buffer = String::new();
        let mut offsets: Vec<usize> = Vec::new();
        crate::translate::fill_window(chinese, start, fill_len, &mut buffer, &mut offsets);
        for length in (2..=6).rev() {
            if length > window_len {
                continue;
            }
            let phrase = &buffer[..offsets[length]];
            if !is_ho_hau_window(chinese, start, &buffer, &offsets, length, ho_max, ho_nguoi, hau_tu)
                || vietphrase.contains_key(phrase)
            {
                continue;
            }
            let covered_by_longer_phrase = (length + 1..=longer_cap)
                .any(|longer| vietphrase.contains_key(&buffer[..offsets[longer]]));
            if !covered_by_longer_phrase {
                return Some(length);
            }
        }
        None
    }
}

/// Tách `{h}{t}` trên cửa sổ đã đổ sẵn: họ ở [..split], hậu tố ở [split..len].
/// Cả hai vế đều bị chặn sớm bằng index độ dài trước khi phải hash chuỗi.
#[allow(clippy::too_many_arguments)]
fn is_ho_hau_window(
    chinese: &[char],
    start: usize,
    buffer: &str,
    offsets: &[usize],
    length: usize,
    ho_max: usize,
    ho_nguoi: &dyn DictionaryLookup,
    hau_tu: &dyn DictionaryLookup,
) -> bool {
    (1..length).any(|split| {
        split <= ho_max
            && length - split <= hau_tu.max_key_len(chinese[start + split])
            && ho_nguoi.contains_key(&buffer[..offsets[split]])
            && hau_tu.contains_key(&buffer[offsets[split]..offsets[length]])
    })
}

fn compile_s_pattern(key: &str) -> String {
    let flexible = flexible_whitespace(&key.replace('(', "(?:").replace("{s}", " {s} "));
    if key == "百分[之]?{s}" {
        return flexible.replace("{s}", "([零一二三四五六七八九十百千万亿两〇点\\d]+)");
    }
    let mut pattern = flexible.replace(
        "{s}",
        "((?:\\d+\\s*[万亿])|(?:\\d+)|(?:[零一二三四五六七八九十百千万亿两〇]+))",
    );
    if key == "{s}两" {
        pattern.push_str("(?!(?:[零一二三四五六七八九十百千万亿两〇\\d]+){1,2})");
    }
    pattern
}

fn compile_n_pattern(key: &str) -> String {
    let pattern = key.replace('(', "(?:");
    let Some(index) = pattern.find("{n}") else {
        return pattern;
    };
    if index + 3 < pattern.len() {
        let prefix = &pattern[..index];
        let suffix = &pattern[index + 3..];
        format!(
            "{prefix}((?:(?!{})[^,\\. ?]){{1,10}}?){suffix}",
            regex::escape(suffix)
        )
    } else {
        pattern.replace("{n}", "([^,\\. ?]{1,10})")
    }
}

fn flexible_whitespace(pattern: &str) -> String {
    let trimmed = pattern.trim();
    let mut output = String::with_capacity(trimmed.len());
    let mut in_whitespace = false;
    for ch in trimmed.chars() {
        if ch.is_whitespace() {
            if !in_whitespace {
                output.push_str("\\s*");
                in_whitespace = true;
            }
        } else {
            output.push(ch);
            in_whitespace = false;
        }
    }
    output
}

fn normalized_length(key: &str) -> usize {
    let alternatives = Regex::new(r"\(([^()]*)\)").expect("static regex");
    let brackets = Regex::new(r"\[[^]]*\]").expect("static regex");
    let mut normalized = key.to_string();
    while let Some(captures) = alternatives.captures(&normalized) {
        let whole = captures.get(0).expect("whole match");
        let choices = captures.get(1).expect("alternatives").as_str();
        let length = choices
            .split('|')
            .map(|choice| choice.chars().count())
            .min()
            .unwrap_or(0);
        normalized.replace_range(whole.range(), &"c".repeat(length));
    }
    normalized = brackets.replace_all(&normalized, "c").into_owned();
    normalized.chars().filter(|ch| *ch != '?').count()
}

fn translate_s_rule(chinese: &str, rule_key: &str, template: &str) -> Option<String> {
    let count = rule_key.matches("{s}").count();
    if count == 0 {
        return None;
    }
    let rule = rule_key.replace('(', "(?:");
    let number_group = "((?:(?:\\d+(?:[.,]\\d+)?|[零一二三四五六七八九十百千万亿两〇]+)\\s*)+)";
    let pattern = if chinese.chars().any(|ch| ch.is_ascii_digit()) {
        flexible_whitespace(&rule.replace("{s}", " {s} ")).replace("{s}", number_group)
    } else {
        rule.replace("{s}", number_group)
    };
    let regex = FancyRegex::new(&format!("^{pattern}$")).ok()?;
    let captures = regex.captures(chinese.trim()).ok().flatten()?;
    if captures.len().saturating_sub(1) < count {
        return None;
    }

    let mut translated = template.to_string();
    if count == 1 {
        let number = captures.get(1)?.as_str().trim();
        if number.is_empty() {
            return None;
        }
        let translated_number = translate_single_number(number, template, "{s}")?;
        translated = translated.replace("{s}", &translated_number);
    } else {
        for index in 1..=count {
            let number = captures.get(index)?.as_str().trim();
            if number.is_empty() {
                return None;
            }
            let placeholder = format!("{{{index}}}");
            let converted =
                translate_plain_number(number, template_has_context(template, &placeholder))?;
            translated = translated.replace(&placeholder, &converted);
        }
    }

    let lower = translated.to_lowercase();
    if let Some(rest) = lower.strip_prefix("ngày ") {
        let digits: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
        if digits
            .parse::<i32>()
            .is_ok_and(|day| (1..10).contains(&day))
        {
            translated.replace_range(.."ngày".len(), "mùng");
        }
    }
    Some(translated)
}

fn translate_single_number(number: &str, template: &str, placeholder: &str) -> Option<String> {
    if number.contains(['.', ',']) || (number.chars().count() > 1 && number.starts_with('0')) {
        return Some(number.to_string());
    }
    if let Some(range) = translate_range_number(number) {
        return Some(range);
    }
    let chars: Vec<char> = number.chars().collect();
    if chars.len() == 2 {
        if let (Some(first), Some(second)) = (chinese_digit(chars[0]), chinese_digit(chars[1])) {
            return Some(format!("{first}-{second}"));
        }
    }
    translate_plain_number(number, template_has_context(template, placeholder))
}

fn translate_plain_number(number: &str, keep_numeric: bool) -> Option<String> {
    let value = convert_chinese_number_to_i64(number)?;
    Some(if keep_numeric {
        value.to_string()
    } else {
        number_to_vietnamese_text(value)
    })
}

fn template_has_context(template: &str, placeholder: &str) -> bool {
    let pattern = format!(r"(?i)(năm|chương)\s*{}", regex::escape(placeholder));
    Regex::new(&pattern).is_ok_and(|regex| regex.is_match(template))
}

fn convert_decimal(number: &str) -> Option<String> {
    let text = number.trim();
    if text.is_empty() {
        return None;
    }
    if let Some((integer, fractional)) = text.split_once('点') {
        let integer = convert_chinese_number_to_i64(integer)?;
        let fractional: String = fractional
            .chars()
            .map(|ch| {
                chinese_digit(ch)
                    .and_then(|digit| char::from_digit(digit as u32, 10))
                    .unwrap_or(ch)
            })
            .collect();
        return Some(format!("{integer}.{fractional}"));
    }
    let chars: Vec<char> = text.chars().collect();
    if chars.len() >= 2 && chars.iter().all(|ch| chinese_digit(*ch).is_some()) {
        return Some(
            chars
                .iter()
                .filter_map(|ch| char::from_digit(chinese_digit(*ch)? as u32, 10))
                .collect(),
        );
    }
    if chars.len() == 3 && chars[2] == '十' {
        if let (Some(first), Some(second)) = (chinese_digit(chars[0]), chinese_digit(chars[1])) {
            return Some(format!("{}-{}", first * 10, second * 10));
        }
    }
    convert_chinese_number_to_i64(text).map(|value| value.to_string())
}

fn byte_to_char_index(text: &str, byte_index: usize) -> usize {
    text[..byte_index].chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_chapter_percentage_fraction_and_date_rules() {
        let dicts = Dictionaries {
            luat_nhan: vec![
                (
                    "{s}年{s}月{s}号".into(),
                    "ngày {3} tháng {2} năm {1}".into(),
                ),
                ("{s}分之{s}".into(), "{2}/{1}".into()),
                ("百分[之]?{s}".into(), "{s}%".into()),
                ("第{s}(更|章)".into(), "chương {s}:".into()),
            ],
            ..Default::default()
        };
        let rules = LuatNhan::new(&dicts);
        let empty = HashMap::new();

        for (input, expected) in [
            ("第十二章", "chương 12:"),
            ("百分之三点五", "3.5%"),
            ("三分之四", "4/3"),
            ("2025年7月21号", "ngày 21 tháng 7 năm 2025"),
        ] {
            let chars: Vec<char> = input.chars().collect();
            let matched = rules
                .contains(&chars, &empty, &empty, None, None, None)
                .expect(input);
            assert_eq!(matched.index, 0, "{input}");
            assert_eq!(
                rules
                    .translate(input, &matched.key, &matched.value_n, None, None)
                    .as_deref(),
                Some(expected),
                "{input}"
            );
        }
    }

    #[test]
    fn resolves_n_from_pronouns() {
        let mut dicts = Dictionaries::default();
        dicts.pronouns.insert("他".into(), "hắn".into());
        dicts
            .luat_nhan
            .push(("在{n}身后".into(), "sau lưng {n}".into()));
        let rules = LuatNhan::new(&dicts);
        let empty = HashMap::new();
        let chars: Vec<char> = "在他身后".chars().collect();
        let matched = rules
            .contains(&chars, &empty, &empty, None, None, None)
            .unwrap();
        assert_eq!(
            rules.translate("在他身后", &matched.key, &matched.value_n, None, None),
            Some("sau lưng hắn".into())
        );
    }
}
