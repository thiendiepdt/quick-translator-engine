//! Dictionary loading, merging, and QT's priority rules.

use crate::han_viet::HanVietMap;
use std::collections::HashMap;
use std::collections::HashSet;

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
///
/// Note: '#'-prefixed lines are NOT treated as comments here. In the original
/// engine, only `loadLuatNhanDictionary` skips '#' lines; HanViet/VietPhrase/
/// Names/Names2 loaders do not. This generic helper is used by the latter, so
/// it must not skip '#'. '#'-skipping will be applied by the LuatNhan loader
/// specifically in a later plan. See docs/engine/dictionaries.md §2.
pub fn parse_dict(content: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (i, raw) in content.lines().enumerate() {
        let line = if i == 0 {
            raw.trim_start_matches('\u{feff}')
        } else {
            raw
        };
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
    pub fn build(
        han_viet_src: &str,
        names_src: &str,
        names2_src: &str,
        vietphrase_src: &str,
    ) -> Dictionaries {
        let mut han_viet: HanVietMap = HashMap::new();
        for (k, v) in parse_dict(han_viet_src) {
            let mut it = k.chars();
            if let (Some(c), None) = (it.next(), it.next()) {
                han_viet.entry(c).or_insert(v); // first-wins
            }
        }

        // Names: first-wins into only_name; Names2: OVERRIDE (indexer assignment),
        // but within Names2 itself the FIRST occurrence of a duplicate key wins
        // (engine guards the override with !onlyNamePhuDictionary.ContainsKey(key),
        // decompiled ~2103-2116).
        let mut only_name: HashMap<String, String> = HashMap::new();
        for (k, v) in parse_dict(names_src) {
            only_name.entry(k).or_insert(v);
        }
        let mut seen_names2: HashSet<String> = HashSet::new();
        for (k, v) in parse_dict(names2_src) {
            if seen_names2.insert(k.clone()) {
                only_name.insert(k, v); // override Names, but only on first Names2 occurrence
            }
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

        Dictionaries {
            han_viet,
            only_name,
            vietphrase,
            vietphrase_one_meaning,
        }
    }

    /// Load from a data directory using the standard QT filenames.
    pub fn load(data_dir: &std::path::Path) -> std::io::Result<Dictionaries> {
        let read =
            |rel: &str| -> std::io::Result<String> { std::fs::read_to_string(data_dir.join(rel)) };
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
    fn parses_bom_keeps_hash_and_skips_non_kv_lines() {
        // '#' is NOT special to parse_dict: only loadLuatNhanDictionary skips '#'
        // lines in the original engine; HanViet/VietPhrase/Names/Names2 loaders
        // (which use this generic helper) do not, so a '#'-prefixed line with an
        // '=' must be kept like any other key=value line.
        let content = "\u{feff}一=nhất\n# comment\n二=nhị\na=b=c\nnoequals\n#note=x\n";
        let got = parse_dict(content);
        assert_eq!(
            got,
            vec![
                ("一".to_string(), "nhất".to_string()),
                ("二".to_string(), "nhị".to_string()),
                ("#note".to_string(), "x".to_string()),
            ]
        );
        assert!(got.contains(&("#note".to_string(), "x".to_string())));
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
        assert_eq!(
            d.only_name.get("张").map(String::as_str),
            Some("TruongName2")
        );
        // vietphrase merged: name key wins over vietphrase key
        assert_eq!(
            d.vietphrase.get("张").map(String::as_str),
            Some("TruongName2")
        );
        // pure vietphrase entry present with full multi-meaning value
        assert_eq!(
            d.vietphrase.get("很好").map(String::as_str),
            Some("rất tốt/rất ổn")
        );
        // one-meaning takes first split on '/' or '|'
        assert_eq!(
            d.vietphrase_one_meaning.get("很好").map(String::as_str),
            Some("rất tốt")
        );
    }

    #[test]
    fn names2_first_wins_within_names2_and_overrides_names() {
        // names2 overrides names, but within names2 itself the FIRST duplicate wins.
        let hv = "";
        let names = "甲=One";
        let names2 = "甲=Two\n甲=Three";
        let vietphrase = "";
        let d = Dictionaries::build(hv, names, names2, vietphrase);

        assert_eq!(d.only_name.get("甲"), Some(&"Two".to_string()));
    }
}
