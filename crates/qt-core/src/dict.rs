//! Dictionary loading, merging, and QT's priority rules.

use std::collections::HashMap;
use crate::han_viet::HanVietMap;

/// All lookup maps used by the engine, after loading + priority merge.
#[derive(Default)]
pub struct Dictionaries {
    pub han_viet: HanVietMap,
    pub only_name: HashMap<String, String>,
    pub vietphrase: HashMap<String, String>,
    pub vietphrase_one_meaning: HashMap<String, String>,
}

/// Parse `key=value` lines the way TranslatorEngine does:
/// - a leading BOM on the first line is stripped
/// - split on '=' into ALL parts; keep the line only if exactly 2 parts result
///   (so a value containing '=' drops the line, matching QT)
/// - lines starting with '#' are comments (skipped)
pub fn parse_dict(content: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (i, raw) in content.lines().enumerate() {
        let line = if i == 0 { raw.trim_start_matches('\u{feff}') } else { raw };
        if line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() == 2 {
            out.push((parts[0].to_string(), parts[1].to_string()));
        }
    }
    out
}

impl Dictionaries {
    /// Pure builder mirroring loadOnlyNameDictionary → loadVietPhraseDictionary →
    /// vPDictToVPOneMeaningDict. See docs/engine/dictionaries.md §4.
    pub fn build(han_viet_src: &str, names_src: &str, names2_src: &str, vietphrase_src: &str) -> Dictionaries {
        let mut han_viet: HanVietMap = HashMap::new();
        for (k, v) in parse_dict(han_viet_src) {
            let mut it = k.chars();
            if let (Some(c), None) = (it.next(), it.next()) {
                han_viet.entry(c).or_insert(v); // first-wins
            }
        }

        // Names: first-wins into only_name; Names2: OVERRIDE (indexer assignment).
        let mut only_name: HashMap<String, String> = HashMap::new();
        for (k, v) in parse_dict(names_src) {
            only_name.entry(k).or_insert(v);
        }
        for (k, v) in parse_dict(names2_src) {
            only_name.insert(k, v); // override
        }

        // only_vietphrase: first-wins
        let mut only_vietphrase: HashMap<String, String> = HashMap::new();
        for (k, v) in parse_dict(vietphrase_src) {
            only_vietphrase.entry(k).or_insert(v);
        }

        // vietphrase = only_name first (all), then only_vietphrase for absent keys.
        let mut vietphrase: HashMap<String, String> = only_name.clone();
        for (k, v) in &only_vietphrase {
            vietphrase.entry(k.clone()).or_insert_with(|| v.clone());
        }

        // one-meaning: first segment split on '/' or '|'.
        let mut vietphrase_one_meaning: HashMap<String, String> = HashMap::new();
        for (k, v) in &vietphrase {
            let first = match v.split(['/', '|']).next() {
                Some(s) => s.to_string(),
                None => v.clone(),
            };
            vietphrase_one_meaning.insert(k.clone(), first);
        }

        Dictionaries { han_viet, only_name, vietphrase, vietphrase_one_meaning }
    }

    /// Load from a data directory using the standard QT filenames.
    pub fn load(data_dir: &std::path::Path) -> std::io::Result<Dictionaries> {
        let read = |rel: &str| -> std::io::Result<String> {
            std::fs::read_to_string(data_dir.join(rel))
        };
        let han_viet = read("Resources/ChinesePhienAmWords.txt")?;
        let names = read("Names.txt")?;
        let names2 = read("Names2/123.txt").unwrap_or_default();
        let vietphrase = read("VietPhrase/VietPhrase.txt")?;
        Ok(Dictionaries::build(&han_viet, &names, &names2, &vietphrase))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bom_and_skips_bad_lines() {
        let content = "\u{feff}一=nhất\n# comment\n二=nhị\na=b=c\nnoequals\n";
        let got = parse_dict(content);
        assert_eq!(got, vec![
            ("一".to_string(), "nhất".to_string()),
            ("二".to_string(), "nhị".to_string()),
        ]);
    }

    #[test]
    fn merge_priority_names_over_vietphrase_and_names2_over_names() {
        // names has 张=Trương ; names2 overrides 张=Trương2 ; vietphrase has 张=(should lose)
        let hv = "";
        let names = "张=TruongName\n李=Ly";
        let names2 = "张=TruongName2";
        let vietphrase = "张=TruongVP\n很好=rất tốt/rất ổn";
        let d = Dictionaries::build(hv, names, names2, vietphrase);

        // Names2 overrode Names in only_name
        assert_eq!(d.only_name.get("张").map(String::as_str), Some("TruongName2"));
        // vietphrase merged: name key wins over vietphrase key
        assert_eq!(d.vietphrase.get("张").map(String::as_str), Some("TruongName2"));
        // pure vietphrase entry present with full multi-meaning value
        assert_eq!(d.vietphrase.get("很好").map(String::as_str), Some("rất tốt/rất ổn"));
        // one-meaning takes first split on '/' or '|'
        assert_eq!(d.vietphrase_one_meaning.get("很好").map(String::as_str), Some("rất tốt"));
    }
}
