//! QT2025-compatible input normalization with ranges mapped back to the
//! caller's original UTF-16 text.

use crate::han_viet::HanVietMap;
use crate::CharRange;
use std::collections::{HashMap, HashSet};
use unicode_general_category::{get_general_category, GeneralCategory};

#[derive(Clone)]
struct MappedChar {
    ch: char,
    source: CharRange,
}

pub(crate) struct StandardizedInput {
    pub chars: Vec<char>,
    pub source_ranges: Vec<CharRange>,
}

pub(crate) struct Standardizer {
    simplified: HashMap<char, char>,
    html_entities: HashMap<String, Vec<char>>,
    chinese: HashSet<char>,
    ignored: Vec<Vec<char>>,
}

impl Standardizer {
    pub fn new(han_viet: &HanVietMap, ignored_source: &[String]) -> Self {
        let mut standardizer = Self {
            simplified: load_strconv_t2s_map(),
            html_entities: load_webutility_entities(),
            chinese: han_viet.keys().copied().collect(),
            ignored: Vec::new(),
        };

        let mut ignored = Vec::new();
        for phrase in ignored_source {
            let normalized = standardizer.standardize_without_ignored(phrase);
            let text: String = normalized
                .into_iter()
                .map(|item| item.ch)
                .collect::<String>()
                .trim_matches(['\t', '\n'])
                .to_string();
            let chars: Vec<char> = text.chars().collect();
            if !chars.is_empty() && !ignored.contains(&chars) {
                ignored.push(chars);
            }
        }
        // QT sorts by descending length, then descending lexical order.
        ignored.sort_by(|left, right| right.len().cmp(&left.len()).then_with(|| right.cmp(left)));
        standardizer.ignored = ignored;
        standardizer
    }

    pub fn standardize(&self, original: &str) -> StandardizedInput {
        let mut mapped = self.standardize_without_ignored(original);
        for ignored in &self.ignored {
            mapped = replace_all(&mapped, ignored, &[]);
        }
        mapped = replace_all(&mapped, &['\t', '\n', '\n'], &[]);
        StandardizedInput {
            chars: mapped.iter().map(|item| item.ch).collect(),
            source_ranges: mapped.iter().map(|item| item.source).collect(),
        }
    }

    fn standardize_without_ignored(&self, original: &str) -> Vec<MappedChar> {
        if original.is_empty() {
            return Vec::new();
        }

        let mut source_start = 0usize;
        let initial: Vec<MappedChar> = original
            .chars()
            .map(|ch| {
                let source = CharRange {
                    start: source_start,
                    length: ch.len_utf16(),
                };
                source_start += ch.len_utf16();
                MappedChar { ch, source }
            })
            .collect();

        let mut mapped = self.to_simplified(&initial);
        mapped = decode_html_entities(&mapped, &self.html_entities);
        mapped = replace_punctuation(&mapped);
        mapped = replace_all(&mapped, &[' ', ' '], &[' ']);
        mapped = replace_all(&mapped, &[' ', '\r', '\n'], &['\n']);
        mapped = replace_all(&mapped, &[' ', '\n'], &['\n']);
        mapped = replace_all(&mapped, &[' ', ','], &[',']);

        for item in &mut mapped {
            if ('\u{FF01}'..='\u{FF5E}').contains(&item.ch) {
                item.ch = char::from_u32(item.ch as u32 - 0xFF01 + 0x21).unwrap_or(item.ch);
            }
        }

        self.insert_qt_spacing(&mapped)
    }

    fn to_simplified(&self, input: &[MappedChar]) -> Vec<MappedChar> {
        input
            .iter()
            .map(|item| MappedChar {
                ch: self.simplified.get(&item.ch).copied().unwrap_or(item.ch),
                source: item.source,
            })
            .collect()
    }

    fn insert_qt_spacing(&self, input: &[MappedChar]) -> Vec<MappedChar> {
        if input.len() <= 1 {
            return input.to_vec();
        }
        let mut output = Vec::with_capacity(input.len() + input.len() / 8);
        for index in 0..input.len() - 1 {
            let item = &input[index];
            let current = item.ch;
            let next = input[index + 1].ch;
            if current.is_control() && !matches!(current, '\t' | '\n' | '\r') {
                continue;
            }
            if is_decimal_digit(current) && next == '》' {
                output.push(item.clone());
                output.push(MappedChar {
                    ch: ' ',
                    source: item.source,
                });
                continue;
            }
            if current == '.'
                && index > 0
                && is_decimal_digit(input[index - 1].ch)
                && is_decimal_digit(next)
            {
                output.push(item.clone());
                continue;
            }
            if self.chinese.contains(&current) {
                output.push(item.clone());
                if !self.chinese.contains(&next)
                    && !matches!(
                        next,
                        ',' | '.' | ':' | ';' | '"' | '\'' | '?' | ' ' | '!' | ')' | '\n'
                    )
                {
                    output.push(MappedChar {
                        ch: ' ',
                        source: item.source,
                    });
                }
                continue;
            }

            match current {
                '\t' | '\n' | ' ' | '"' | '\'' | '(' => output.push(item.clone()),
                '!' | '.' | '?' => {
                    output.push(item.clone());
                    if !matches!(next, '"' | ' ' | '\'') {
                        output.push(MappedChar {
                            ch: ' ',
                            source: item.source,
                        });
                    }
                }
                _ => {
                    output.push(item.clone());
                    if self.chinese.contains(&next) {
                        output.push(MappedChar {
                            ch: ' ',
                            source: item.source,
                        });
                    }
                }
            }
        }
        // The original method appends the final character without applying
        // any of the loop's filtering or spacing rules.
        output.push(input[input.len() - 1].clone());

        let six_dots: Vec<char> = ". . . . . .".chars().collect();
        replace_all(&output, &six_dots, &['.', '.', '.'])
    }
}

/// Character mapping generated from
/// `Microsoft.VisualBasic.Strings.StrConv(..., SimplifiedChinese, 0)` for all
/// non-surrogate BMP code points. QT2025 uses that exact API; embedding the
/// table makes its character-wise behavior deterministic on Linux as well.
fn load_strconv_t2s_map() -> HashMap<char, char> {
    include_str!("strconv_t2s.tsv")
        .lines()
        .filter_map(|line| {
            let (source, target) = line.split_once('=')?;
            Some((
                char::from_u32(u32::from_str_radix(source, 16).ok()?)?,
                char::from_u32(u32::from_str_radix(target, 16).ok()?)?,
            ))
        })
        .collect()
}

fn is_decimal_digit(ch: char) -> bool {
    get_general_category(ch) == GeneralCategory::DecimalNumber
}

/// Named-entity table accepted by .NET `WebUtility.HtmlDecode`. The .NET API
/// intentionally recognizes the older entity set rather than every WHATWG
/// HTML5 name (for example, it leaves `&NotEqualTilde;` unchanged).
fn load_webutility_entities() -> HashMap<String, Vec<char>> {
    include_str!("webutility_entities.tsv")
        .lines()
        .filter_map(|line| {
            let (name, values) = line.split_once('=')?;
            let decoded: Option<Vec<char>> = values
                .split(',')
                .map(|value| char::from_u32(u32::from_str_radix(value, 16).ok()?))
                .collect();
            Some((name.to_string(), decoded?))
        })
        .collect()
}

fn replace_punctuation(input: &[MappedChar]) -> Vec<MappedChar> {
    let mut output = Vec::with_capacity(input.len());
    for item in input {
        let replacement: &[char] = match item.ch {
            '，' => &[',', ' '],
            '。' | '．' => &['.'],
            '：' => &[':', ' '],
            '“' | '「' => &['"'],
            '”' | '」' => &['"', ' '],
            '‘' => &['\''],
            '’' => &['\'', ' '],
            '？' => &['?'],
            '！' => &['!'],
            '、' => &[',', ' '],
            '\u{3000}' => &[' '],
            '…' => &['.', '.', '.'],
            '\0' => &[],
            _ => {
                output.push(item.clone());
                continue;
            }
        };
        output.extend(replacement.iter().map(|ch| MappedChar {
            ch: *ch,
            source: item.source,
        }));
    }
    output
}

fn decode_html_entities(
    input: &[MappedChar],
    named_entities: &HashMap<String, Vec<char>>,
) -> Vec<MappedChar> {
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0usize;
    while index < input.len() {
        if input[index].ch == '&' {
            let end = input[index + 1..]
                .iter()
                .position(|item| item.ch == ';')
                .map(|relative| index + 1 + relative);
            if let Some(end) = end {
                let encoded: String = input[index..=end].iter().map(|item| item.ch).collect();
                if let Some(decoded) = decode_html_entity(&encoded, named_entities) {
                    let source = merged_range(input, index, end - index + 1);
                    output.extend(decoded.into_iter().map(|ch| MappedChar { ch, source }));
                    index = end + 1;
                    continue;
                }
            }
        }
        output.push(input[index].clone());
        index += 1;
    }
    output
}

fn decode_html_entity(
    encoded: &str,
    named_entities: &HashMap<String, Vec<char>>,
) -> Option<Vec<char>> {
    let body = encoded.strip_prefix('&')?.strip_suffix(';')?;
    if let Some(number) = body.strip_prefix("#x").or_else(|| body.strip_prefix("#X")) {
        let value = u32::from_str_radix(number, 16).ok()?;
        return Some(vec![char::from_u32(value)?]);
    }
    if let Some(number) = body.strip_prefix('#') {
        let value = number.parse::<u32>().ok()?;
        return Some(vec![char::from_u32(value)?]);
    }
    named_entities.get(body).cloned()
}

fn replace_all(input: &[MappedChar], needle: &[char], replacement: &[char]) -> Vec<MappedChar> {
    if needle.is_empty() || input.len() < needle.len() {
        return input.to_vec();
    }
    let mut output = Vec::with_capacity(input.len());
    let mut index = 0usize;
    while index < input.len() {
        let matches = index + needle.len() <= input.len()
            && input[index..index + needle.len()]
                .iter()
                .map(|item| item.ch)
                .eq(needle.iter().copied());
        if matches {
            let source = merged_range(input, index, needle.len());
            output.extend(replacement.iter().map(|ch| MappedChar { ch: *ch, source }));
            index += needle.len();
        } else {
            output.push(input[index].clone());
            index += 1;
        }
    }
    output
}

fn merged_range(input: &[MappedChar], start: usize, length: usize) -> CharRange {
    let slice = &input[start..start + length];
    let source_start = slice
        .iter()
        .map(|item| item.source.start)
        .min()
        .unwrap_or(0);
    let source_end = slice
        .iter()
        .map(|item| item.source.start + item.source.length)
        .max()
        .unwrap_or(source_start);
    CharRange {
        start: source_start,
        length: source_end - source_start,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standardizer(ignored: &[&str]) -> Standardizer {
        let mut hv = HanVietMap::new();
        for ch in "这是一本书他她本章完等等".chars() {
            hv.insert(ch, String::new());
        }
        Standardizer::new(
            &hv,
            &ignored.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        )
    }

    #[test]
    fn matches_qt_standardization_steps() {
        let standardizer = standardizer(&[]);
        let value = standardizer.standardize("這是一本書。 他&amp;她 ＡＢＣ１２３ 等等……");
        assert_eq!(
            value.chars.iter().collect::<String>(),
            "这是一本书. 他 & 她 ABC123 等等..."
        );
    }

    #[test]
    fn traditional_conversion_matches_windows_strconv_samples() {
        let samples = [
            ("這是一本書", "这是一本书"),
            ("著作與看著", "着作与看着"),
            ("乾坤一擲", "乾坤一掷"),
            ("後臺發展", "後台发展"),
            ("滑鼠裡面", "滑鼠里面"),
        ];
        let mut hv = HanVietMap::new();
        for ch in samples.iter().flat_map(|(input, _)| input.chars()) {
            hv.insert(ch, String::new());
        }
        for ch in samples.iter().flat_map(|(_, expected)| expected.chars()) {
            hv.insert(ch, String::new());
        }
        let standardizer = Standardizer::new(&hv, &[]);
        for (input, expected) in samples {
            assert_eq!(
                standardizer
                    .standardize(input)
                    .chars
                    .iter()
                    .collect::<String>(),
                expected,
                "{input}"
            );
        }
    }

    #[test]
    fn removes_standardized_ignored_phrases() {
        let standardizer = standardizer(&["(本章完)"]);
        let value = standardizer.standardize("(本章完)");
        assert!(value.chars.is_empty());
    }

    #[test]
    fn maps_expansions_back_to_original_utf16_ranges() {
        let standardizer = standardizer(&[]);
        let value = standardizer.standardize("😀……");
        assert_eq!(value.chars.iter().collect::<String>(), "😀...");
        assert_eq!(
            value.source_ranges[0],
            CharRange {
                start: 0,
                length: 2
            }
        );
        assert_eq!(
            value.source_ranges[1],
            CharRange {
                start: 2,
                length: 2
            }
        );
        assert_eq!(
            value.source_ranges[3],
            CharRange {
                start: 2,
                length: 2
            }
        );

        let entity = standardizer.standardize("&amp;");
        assert_eq!(entity.chars, vec!['&']);
        assert_eq!(
            entity.source_ranges[0],
            CharRange {
                start: 0,
                length: 5
            }
        );

        let legacy = standardizer.standardize("&NotEqualTilde;");
        assert_eq!(legacy.chars.iter().collect::<String>(), "&NotEqualTilde;");
        let numeric = standardizer.standardize("&#x1F600;");
        assert_eq!(numeric.chars, vec!['😀']);
        assert_eq!(
            numeric.source_ranges[0],
            CharRange {
                start: 0,
                length: 9
            }
        );
    }
}
