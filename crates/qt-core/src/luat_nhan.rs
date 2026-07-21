//! QT2025's pattern-based “Luật Nhân” translation rules.

use crate::dict::Dictionaries;
use crate::number::{
    chinese_digit, convert_chinese_number_to_i64, is_number_start, number_to_vietnamese_text,
    translate_range_number,
};
use fancy_regex::Regex as FancyRegex;
use regex::Regex;
use std::collections::HashMap;

struct CompiledRule {
    key: String,
    value: String,
    regex: FancyRegex,
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
}

impl LuatNhan {
    pub fn new(dicts: &Dictionaries) -> Self {
        let mut dictionary_n = dicts.pronouns.clone();
        for (key, value) in &dicts.only_name_one_meaning {
            dictionary_n
                .entry(key.clone())
                .or_insert_with(|| value.clone());
        }

        let mut ordered_rules = dicts.luat_nhan.clone();
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
                    .unwrap_or_else(|error| panic!("invalid LuatNhan rule {key:?}: {error}"));
                CompiledRule { key, value, regex }
            })
            .collect();

        let s_rules = ordered_rules
            .iter()
            .filter(|(key, _)| key.as_str() != "{s}" && key.contains("{s}") && !key.contains("{n}"))
            .cloned()
            .map(|(key, value)| {
                let pattern = compile_s_pattern(&key);
                let regex = FancyRegex::new(&pattern)
                    .unwrap_or_else(|error| panic!("invalid LuatNhan rule {key:?}: {error}"));
                CompiledRule { key, value, regex }
            })
            .collect();

        Self {
            n_rules,
            s_rules,
            dictionary_n,
            ho_nguoi: dicts.ho_nguoi.clone(),
            hau_tu: dicts.hau_tu.clone(),
        }
    }

    pub fn contains(
        &self,
        chinese: &[char],
        only_vietphrase: &HashMap<String, String>,
        vietphrase: &HashMap<String, String>,
    ) -> Option<RuleMatch> {
        let text: String = chinese.iter().collect();
        let mut best = self.match_n(&text);
        let mut best_index = best.as_ref().map_or(usize::MAX, |matched| matched.index);

        for index in 0..chinese.len().min(best_index) {
            if let Some(length) = self.find_ho_hau_phrase(chinese, index, vietphrase) {
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
            if let Some(matched) = self.match_s(&text, only_vietphrase) {
                if matched.index < best_index {
                    best = Some(matched);
                }
            }
        }
        best
    }

    pub fn translate(&self, chinese: &str, rule_key: &str, value_n: &str) -> Option<String> {
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
            let chars: Vec<char> = chinese.chars().collect();
            for split in 1..chars.len() {
                let ho: String = chars[..split].iter().collect();
                let hau: String = chars[split..].iter().collect();
                if let (Some(ho_value), Some(hau_value)) =
                    (self.ho_nguoi.get(&ho), self.hau_tu.get(&hau))
                {
                    return Some(format!("{} {}", ho_value.trim(), hau_value.trim()));
                }
            }
        }
        None
    }

    fn match_n(&self, chinese: &str) -> Option<RuleMatch> {
        for rule in &self.n_rules {
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
                        if let Some(value) = self.dictionary_n.get(&key) {
                            return Some(RuleMatch {
                                index: match_index,
                                length: match_length - (captured_chars.len() - length),
                                key: rule.key.clone(),
                                value_n: value.clone(),
                            });
                        }
                    }
                } else if rule.key.starts_with("{n}") {
                    for offset in 0..captured_chars.len() {
                        let key: String = captured_chars[offset..].iter().collect();
                        if let Some(value) = self.dictionary_n.get(&key) {
                            return Some(RuleMatch {
                                index: match_index + offset,
                                length: match_length - offset,
                                key: rule.key.clone(),
                                value_n: value.clone(),
                            });
                        }
                    }
                } else if let Some(value) = self.dictionary_n.get(captured) {
                    return Some(RuleMatch {
                        index: match_index,
                        length: match_length,
                        key: rule.key.clone(),
                        value_n: value.clone(),
                    });
                }
            }
        }
        None
    }

    fn match_s(
        &self,
        chinese: &str,
        only_vietphrase: &HashMap<String, String>,
    ) -> Option<RuleMatch> {
        for rule in &self.s_rules {
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
        vietphrase: &HashMap<String, String>,
    ) -> Option<usize> {
        for length in (2..=6).rev() {
            if start + length > chinese.len() {
                continue;
            }
            let phrase: String = chinese[start..start + length].iter().collect();
            if !self.is_ho_hau(&phrase) || vietphrase.contains_key(&phrase) {
                continue;
            }
            let covered_by_longer_phrase = (length + 1..=20).any(|longer| {
                start + longer <= chinese.len()
                    && vietphrase
                        .contains_key(&chinese[start..start + longer].iter().collect::<String>())
            });
            if !covered_by_longer_phrase {
                return Some(length);
            }
        }
        None
    }

    fn is_ho_hau(&self, phrase: &str) -> bool {
        let chars: Vec<char> = phrase.chars().collect();
        (1..chars.len()).any(|split| {
            self.ho_nguoi
                .contains_key(&chars[..split].iter().collect::<String>())
                && self
                    .hau_tu
                    .contains_key(&chars[split..].iter().collect::<String>())
        })
    }
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
            let matched = rules.contains(&chars, &empty, &empty).expect(input);
            assert_eq!(matched.index, 0, "{input}");
            assert_eq!(
                rules
                    .translate(input, &matched.key, &matched.value_n)
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
        let matched = rules.contains(&chars, &empty, &empty).unwrap();
        assert_eq!(
            rules.translate("在他身后", &matched.key, &matched.value_n),
            Some("sau lưng hắn".into())
        );
    }
}
