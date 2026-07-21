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
    let threshold = if algo == 0 { phrase_len } else { phrase_len.max(3) };
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

#[cfg(test)]
mod tests {
    use super::*;

    fn dict(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
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
}
