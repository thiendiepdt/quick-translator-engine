//! Chinese/Latin number recognition and conversion used by `TranslateAll`.
//! Mirrors `TransLuatNhan` plus `PreScanForNumbers`/`NumberModifier` from QT2025.

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NumberInfo {
    pub start: usize,
    pub length: usize,
    pub text: String,
}

fn chinese_digit(c: char) -> Option<i64> {
    match c {
        '零' | '〇' => Some(0),
        '一' => Some(1),
        '二' | '两' => Some(2),
        '三' => Some(3),
        '四' => Some(4),
        '五' => Some(5),
        '六' => Some(6),
        '七' => Some(7),
        '八' => Some(8),
        '九' => Some(9),
        _ => None,
    }
}

fn chinese_unit(c: char) -> Option<i64> {
    match c {
        '十' => Some(10),
        '百' => Some(100),
        '千' => Some(1_000),
        '万' => Some(10_000),
        '亿' => Some(100_000_000),
        _ => None,
    }
}

fn is_chinese_number_char(c: char) -> bool {
    chinese_digit(c).is_some() || chinese_unit(c).is_some()
}

pub(crate) fn is_number_start(c: char) -> bool {
    c.is_ascii_digit() || is_chinese_number_char(c)
}

fn latin_to_chinese(c: char) -> Option<char> {
    match c {
        '0' => Some('零'),
        '1' => Some('一'),
        '2' => Some('二'),
        '3' => Some('三'),
        '4' => Some('四'),
        '5' => Some('五'),
        '6' => Some('六'),
        '7' => Some('七'),
        '8' => Some('八'),
        '9' => Some('九'),
        _ => None,
    }
}

/// Manual equivalent of QT's compiled `NumberPatternRegex` at one start index.
fn find_longest_number(chars: &[char], start: usize) -> Option<NumberInfo> {
    if start >= chars.len() {
        return None;
    }

    let mut cursor = start;
    let mut matched = false;
    loop {
        if cursor < chars.len() && chars[cursor].is_ascii_digit() {
            matched = true;
            while cursor < chars.len() && chars[cursor].is_ascii_digit() {
                cursor += 1;
            }
            if cursor + 1 < chars.len()
                && matches!(chars[cursor], '.' | ',')
                && chars[cursor + 1].is_ascii_digit()
            {
                cursor += 1;
                while cursor < chars.len() && chars[cursor].is_ascii_digit() {
                    cursor += 1;
                }
            }
        } else if cursor < chars.len() && is_chinese_number_char(chars[cursor]) {
            matched = true;
            while cursor < chars.len() && is_chinese_number_char(chars[cursor]) {
                cursor += 1;
            }
        } else {
            break;
        }

        while cursor < chars.len() && matches!(chars[cursor], ' ' | '\t') {
            cursor += 1;
        }
    }

    if !matched {
        return None;
    }

    let mut trimmed_end = cursor;
    while trimmed_end > start && matches!(chars[trimmed_end - 1], ' ' | '\t') {
        trimmed_end -= 1;
    }
    let matched_chars = &chars[start..trimmed_end];
    if matched_chars.len() == 1 && matches!(matched_chars[0], '百' | '千' | '万' | '亿') {
        return None;
    }

    let has_latin = matched_chars.iter().any(|c| c.is_ascii_digit());
    let has_chinese = matched_chars.iter().any(|c| is_chinese_number_char(*c));
    let has_chinese_value = matched_chars.iter().any(|c| chinese_digit(*c).is_some());

    let text = if has_latin && has_chinese && has_chinese_value {
        matched_chars
            .iter()
            .filter_map(|c| {
                latin_to_chinese(*c).or_else(|| is_chinese_number_char(*c).then_some(*c))
            })
            .collect()
    } else {
        matched_chars.iter().collect()
    };

    Some(NumberInfo {
        start,
        length: trimmed_end - start,
        text,
    })
}

pub(crate) fn prescan_numbers(
    chars: &[char],
    only_vietphrase: &HashMap<String, String>,
) -> HashMap<usize, NumberInfo> {
    let mut numbers = HashMap::new();
    let mut cursor = 0;
    while cursor < chars.len() {
        if is_number_start(chars[cursor]) {
            if let Some(info) = find_longest_number(chars, cursor) {
                let length = info.length;
                if !only_vietphrase.contains_key(&info.text) {
                    numbers.insert(info.start, info);
                    cursor += length;
                    continue;
                }
            }
        }
        cursor += 1;
    }
    numbers
}

/// Move `余`/`多` after a following 百/千/万/亿, matching QT's `NumberModifier`.
pub(crate) fn number_modifier(chars: &[char]) -> Vec<char> {
    let mut output = Vec::with_capacity(chars.len());
    let mut cursor = 0;
    while cursor < chars.len() {
        let c = chars[cursor];
        if matches!(c, '余' | '多')
            && cursor + 1 < chars.len()
            && matches!(chars[cursor + 1], '百' | '千' | '万' | '亿')
        {
            output.push(chars[cursor + 1]);
            output.push(c);
            cursor += 2;
        } else {
            output.push(c);
            cursor += 1;
        }
    }
    output
}

fn convert_under_ten_thousand(chars: &[char]) -> Option<i64> {
    if chars.is_empty() {
        return Some(0);
    }
    if chars == ['十'] {
        return Some(10);
    }
    if chars[0] == '十' {
        return 10_i64.checked_add(convert_under_ten_thousand(&chars[1..])?);
    }

    let mut total = 0_i64;
    let mut digit = 0_i64;
    for c in chars {
        if let Some(value) = chinese_digit(*c) {
            digit = value;
        } else if let Some(unit) = chinese_unit(*c) {
            total = total.checked_add(if digit == 0 {
                unit
            } else {
                digit.checked_mul(unit)?
            })?;
            digit = 0;
        }
    }
    total.checked_add(digit)
}

fn parse_mixed_unit(chars: &[char]) -> Option<i64> {
    let mut cursor = 0;
    while cursor < chars.len() && chars[cursor].is_ascii_digit() {
        cursor += 1;
    }
    if cursor == 0 {
        return None;
    }
    let left: String = chars[..cursor].iter().collect();
    while cursor < chars.len() && chars[cursor].is_whitespace() {
        cursor += 1;
    }
    let unit = match chars.get(cursor) {
        Some('万') => 10_000_i64,
        Some('亿') => 100_000_000_i64,
        _ => return None,
    };
    cursor += 1;
    while cursor < chars.len() && chars[cursor].is_whitespace() {
        cursor += 1;
    }
    if cursor + 1 != chars.len() || !chars[cursor].is_ascii_digit() {
        return None;
    }
    let right = chars[cursor].to_digit(10)? as i64;
    left.parse::<i64>()
        .ok()?
        .checked_mul(unit)?
        .checked_add(right.checked_mul(unit / 10)?)
}

pub(crate) fn convert_chinese_number_to_i64(input: &str) -> Option<i64> {
    let text = input.trim();
    if text.is_empty() {
        return Some(0);
    }
    if let Ok(number) = text.parse::<i64>() {
        return Some(number);
    }

    let chars: Vec<char> = text.chars().collect();
    if let Some(number) = parse_mixed_unit(&chars) {
        return Some(number);
    }

    if let Some(index) = chars.iter().rposition(|c| *c == '亿') {
        let leading = if index == 0 {
            1
        } else {
            convert_chinese_number_to_i64(&chars[..index].iter().collect::<String>())?
        };
        let base = leading.checked_mul(100_000_000)?;
        let trailing = &chars[index + 1..];
        if trailing.len() == 1 {
            if let Some(value) = chinese_digit(trailing[0]) {
                return base.checked_add(value.checked_mul(10_000_000)?);
            }
        }
        return base.checked_add(convert_chinese_number_to_i64(
            &trailing.iter().collect::<String>(),
        )?);
    }

    if let Some(index) = chars.iter().rposition(|c| *c == '万') {
        let leading = if index == 0 {
            1
        } else {
            convert_chinese_number_to_i64(&chars[..index].iter().collect::<String>())?
        };
        let base = leading.checked_mul(10_000)?;
        let trailing = &chars[index + 1..];
        if trailing.len() == 1 {
            if let Some(value) = chinese_digit(trailing[0]) {
                return base.checked_add(value.checked_mul(1_000)?);
            }
        }
        return base.checked_add(convert_chinese_number_to_i64(
            &trailing.iter().collect::<String>(),
        )?);
    }

    if chars.len() >= 3
        && !chars.iter().any(|c| chinese_unit(*c).is_some())
        && chars.iter().all(|c| chinese_digit(*c).is_some())
    {
        let digits: String = chars
            .iter()
            .map(|c| {
                char::from_digit(chinese_digit(*c).expect("checked digit") as u32, 10).expect("0-9")
            })
            .collect();
        return digits.parse().ok();
    }

    convert_under_ten_thousand(&chars)
}

fn format_grouped(number: i64) -> String {
    let raw = number.to_string();
    let (sign, digits) = raw
        .strip_prefix('-')
        .map_or(("", raw.as_str()), |d| ("-", d));
    let mut output = String::with_capacity(raw.len() + raw.len() / 3);
    output.push_str(sign);
    for (index, c) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            output.push(',');
        }
        output.push(c);
    }
    output
}

pub(crate) fn number_to_vietnamese_text(mut number: i64) -> String {
    if number == 0 {
        return "0".to_string();
    }
    if (-9_999..=9_999).contains(&number) {
        return format_grouped(number);
    }

    let mut parts = Vec::new();
    let hundred_millions = number / 100_000_000;
    if hundred_millions > 0 {
        parts.push(format!(
            "{} ức",
            number_to_vietnamese_text(hundred_millions)
        ));
        number %= 100_000_000;
    }
    let ten_thousands = number / 10_000;
    if ten_thousands > 0 {
        parts.push(format!("{} vạn", format_grouped(ten_thousands)));
        number %= 10_000;
    }
    if number > 0 {
        parts.push(format_grouped(number));
    }
    parts.join(" ")
}

fn try_convert_postfixed_range(chars: &[char]) -> Option<String> {
    if chars.len() < 3 {
        return None;
    }
    let first = chinese_digit(chars[chars.len() - 2])?;
    let second = chinese_digit(chars[chars.len() - 1])?;
    if first == 0 {
        return None;
    }
    let prefix: String = chars[..chars.len() - 2].iter().collect();
    let base = convert_chinese_number_to_i64(&prefix)?;
    (base > 0 && base % 10 == 0).then(|| format!("{}-{}", base + first, base + second))
}

fn try_convert_range(chars: &[char]) -> Option<String> {
    if chars.len() >= 4
        && chars[..chars.len() - 3]
            .iter()
            .all(|c| matches!(c, '十' | '百' | '千'))
        && matches!(chars[chars.len() - 1], '万' | '亿')
    {
        let prefix: String = chars[..chars.len() - 3].iter().collect();
        let first = positive_chinese_digit(chars[chars.len() - 3], false)?;
        let second = positive_chinese_digit(chars[chars.len() - 2], false)?;
        if first >= second {
            return None;
        }
        let base = convert_chinese_number_to_i64(&prefix)?;
        let multiplier = match chars[chars.len() - 4] {
            '千' => 100,
            '百' => 10,
            _ => 1,
        };
        let unit = chinese_unit(chars[chars.len() - 1])?;
        return Some(format!(
            "{}-{}",
            number_to_vietnamese_text((base + first * multiplier) * unit),
            number_to_vietnamese_text((base + second * multiplier) * unit)
        ));
    }

    if chars.len() == 4 && matches!(chars[2], '十' | '百' | '千') && matches!(chars[3], '万' | '亿')
    {
        let first = positive_chinese_digit(chars[0], true)?;
        let second = positive_chinese_digit(chars[1], false)?;
        if first >= second {
            return None;
        }
        let small_unit = chinese_unit(chars[2])?;
        let label = if chars[3] == '万' { "vạn" } else { "ức" };
        return Some(format!(
            "{}-{} {label}",
            first * small_unit,
            second * small_unit
        ));
    }

    if chars.len() == 3 && matches!(chars[2], '十' | '百' | '千' | '万' | '亿') {
        let first = positive_chinese_digit(chars[0], true)?;
        let second = positive_chinese_digit(chars[1], false)?;
        if first >= second {
            return None;
        }
        return Some(match chars[2] {
            '亿' => format!("{first}-{second} ức"),
            '万' => format!("{first}-{second} vạn"),
            '千' => format!("{first}-{second} ngàn"),
            unit => {
                let unit = chinese_unit(unit)?;
                format!("{}-{}", first * unit, second * unit)
            }
        });
    }

    if chars.len() >= 3 && matches!(chars[chars.len() - 3], '万' | '亿') {
        let first = positive_chinese_digit(chars[chars.len() - 2], false)?;
        let second = positive_chinese_digit(chars[chars.len() - 1], false)?;
        if first >= second {
            return None;
        }
        let prefix: String = chars[..chars.len() - 2].iter().collect();
        let base = convert_chinese_number_to_i64(&prefix)?;
        let multiplier = if chars[chars.len() - 3] == '亿' {
            10_000_000
        } else {
            1_000
        };
        return Some(format!(
            "{}-{}",
            number_to_vietnamese_text(base + first * multiplier),
            number_to_vietnamese_text(base + second * multiplier)
        ));
    }

    None
}

fn positive_chinese_digit(c: char, allow_liang: bool) -> Option<i64> {
    if c == '两' && allow_liang {
        return Some(2);
    }
    matches!(
        c,
        '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九'
    )
    .then(|| chinese_digit(c).expect("matched digit"))
}

pub(crate) fn translate_number(text: &str) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    if text.contains('.') || text.contains(',') || (chars.len() > 1 && chars[0] == '0') {
        return Some(text.to_string());
    }
    if let Some(range) = try_convert_range(&chars) {
        return Some(range);
    }
    if let Some(range) = try_convert_postfixed_range(&chars) {
        return Some(range);
    }
    if chars.len() == 2 {
        if let (Some(first), Some(second)) = (chinese_digit(chars[0]), chinese_digit(chars[1])) {
            return Some(format!("{first}-{second}"));
        }
    }
    convert_chinese_number_to_i64(text).map(number_to_vietnamese_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_longest_number_and_skips_unit_alone() {
        let chars: Vec<char> = "三万五百人".chars().collect();
        let info = find_longest_number(&chars, 0).unwrap();
        assert_eq!(info.length, 4);
        assert_eq!(info.text, "三万五百");
        assert!(find_longest_number(&['万'], 0).is_none());
    }

    #[test]
    fn converts_chinese_and_mixed_numbers() {
        assert_eq!(convert_chinese_number_to_i64("十"), Some(10));
        assert_eq!(convert_chinese_number_to_i64("一百二十三"), Some(123));
        assert_eq!(convert_chinese_number_to_i64("一二三"), Some(123));
        assert_eq!(convert_chinese_number_to_i64("3万5"), Some(35_000));
        assert_eq!(
            convert_chinese_number_to_i64("一亿二千三百四十五万六千七百八十九"),
            Some(123_456_789)
        );
    }

    #[test]
    fn formats_qt_vietnamese_units_and_ranges() {
        assert_eq!(number_to_vietnamese_text(1_234), "1,234");
        assert_eq!(
            number_to_vietnamese_text(123_456_789),
            "1 ức 2,345 vạn 6,789"
        );
        assert_eq!(translate_number("三四万").as_deref(), Some("3-4 vạn"));
        assert_eq!(
            translate_number("三四千万").as_deref(),
            Some("3000-4000 vạn")
        );
        assert_eq!(
            translate_number("十一二万").as_deref(),
            Some("11 vạn-12 vạn")
        );
        assert_eq!(
            translate_number("三万四五").as_deref(),
            Some("3 vạn 4,000-3 vạn 5,000")
        );
        assert_eq!(translate_number("二十三四").as_deref(), Some("23-24"));
        assert_eq!(translate_number("一二").as_deref(), Some("1-2"));
        assert_eq!(translate_number("001").as_deref(), Some("001"));
        assert_eq!(translate_number("1.25").as_deref(), Some("1.25"));
    }

    #[test]
    fn modifier_swaps_more_before_large_unit() {
        let chars: Vec<char> = "三余万五多百".chars().collect();
        assert_eq!(
            number_modifier(&chars).iter().collect::<String>(),
            "三万余五百多"
        );
    }

    #[test]
    fn prescan_respects_raw_vietphrase_priority() {
        let chars: Vec<char> = "一二三".chars().collect();
        let mut dictionary = HashMap::new();
        dictionary.insert("一二三".to_string(), "một hai ba".to_string());
        let matches = prescan_numbers(&chars, &dictionary);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches.get(&1).map(|n| n.text.as_str()), Some("二三"));
        assert_eq!(prescan_numbers(&chars, &HashMap::new()).len(), 1);
    }
}
