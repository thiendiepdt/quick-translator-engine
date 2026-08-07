use rustc_hash::FxHashMap;
use std::collections::HashSet;

const MAX_LOOKUP_CHARACTERS: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LacVietMeaning {
    pub source: String,
    pub definition: String,
}

/// Look up the longest prefixes first, then fill in definitions for each
/// character in the selected phrase. The latter makes a multi-character name
/// useful in one dialog instead of forcing the reader to reopen it per glyph.
pub(crate) fn lookup_lac_viet(
    text: &str,
    dictionary: &FxHashMap<String, String>,
) -> Vec<LacVietMeaning> {
    let characters = text
        .trim()
        .chars()
        .take(MAX_LOOKUP_CHARACTERS)
        .collect::<Vec<_>>();
    let mut candidates = Vec::new();

    for end in (1..=characters.len()).rev() {
        candidates.push(characters[..end].iter().collect::<String>());
    }
    candidates.extend(characters.iter().map(char::to_string));

    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|candidate| seen.insert(candidate.clone()))
        .filter_map(|source| {
            dictionary.get(&source).map(|definition| LacVietMeaning {
                source,
                definition: definition.replace("\\n", "\n").replace("\\t", "\t"),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_phrase_prefixes_then_each_character_without_duplicates() {
        let dictionary: FxHashMap<String, String> = [
            ("金美".to_string(), "cụm".to_string()),
            ("金".to_string(), "vàng".to_string()),
            ("美".to_string(), "đẹp\\n\tnghĩa phụ".to_string()),
            ("婷".to_string(), "xinh đẹp".to_string()),
        ]
        .into_iter()
        .collect();

        assert_eq!(
            lookup_lac_viet("金美婷", &dictionary),
            vec![
                LacVietMeaning {
                    source: "金美".to_string(),
                    definition: "cụm".to_string(),
                },
                LacVietMeaning {
                    source: "金".to_string(),
                    definition: "vàng".to_string(),
                },
                LacVietMeaning {
                    source: "美".to_string(),
                    definition: "đẹp\n\tnghĩa phụ".to_string(),
                },
                LacVietMeaning {
                    source: "婷".to_string(),
                    definition: "xinh đẹp".to_string(),
                },
            ]
        );
    }
}
