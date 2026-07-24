//! Dictionary loading, merging, and QT's priority rules.

use crate::han_viet::HanVietMap;
use std::collections::HashMap;
use std::collections::HashSet;

/// Raw QT2025 contents for every dictionary that can be replaced per request.
/// VietPhrase and ChinesePhienAmWords stay fixed and are intentionally not
/// included here.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DictionaryDefaults {
    pub names: String,
    pub names2: String,
    pub luat_nhan: String,
    pub pronouns: String,
    pub danh_tu: String,
    pub ho_nguoi: String,
    pub hau_tu: String,
    pub ignored_chinese_phrases: String,
}

impl DictionaryDefaults {
    /// Load optional QT2025 dictionaries. Missing optional files preserve the
    /// engine's historical behavior by becoming empty dictionaries.
    pub fn load(data_dir: &std::path::Path) -> Self {
        let read = |rel: &str| std::fs::read_to_string(data_dir.join(rel)).unwrap_or_default();
        Self {
            names: read("Names.txt"),
            names2: read("Names2/123.txt"),
            luat_nhan: read("LuatNhan.txt"),
            pronouns: read("Resources/Pronouns.txt"),
            danh_tu: read("Resources/DanhTu.txt"),
            ho_nguoi: read("Resources/HoNguoi.txt"),
            hau_tu: read("Resources/HauTu.txt"),
            ignored_chinese_phrases: read("IgnoredChinesePhrases.txt"),
        }
    }

    /// Build an engine dictionary set by combining these customizable defaults
    /// with the two fixed dictionaries.
    pub fn build_dictionaries(
        &self,
        chinese_phien_am_words: &str,
        vietphrase: &str,
    ) -> Dictionaries {
        Dictionaries::build_full(
            chinese_phien_am_words,
            &self.names,
            &self.names2,
            vietphrase,
            &self.pronouns,
            &self.danh_tu,
            &self.ho_nguoi,
            &self.hau_tu,
            &self.luat_nhan,
            &self.ignored_chinese_phrases,
        )
    }
}

/// Raw contents for the dictionaries that callers may replace for one
/// translation. `None` keeps the dictionary loaded with the engine; `Some("")`
/// deliberately replaces it with an empty dictionary.
#[derive(Default)]
pub struct DictionarySourceOverrides<'a> {
    pub names: Option<&'a str>,
    pub names2: Option<&'a str>,
    pub luat_nhan: Option<&'a str>,
    pub pronouns: Option<&'a str>,
    pub danh_tu: Option<&'a str>,
    pub ho_nguoi: Option<&'a str>,
    pub hau_tu: Option<&'a str>,
    pub ignored_chinese_phrases: Option<&'a str>,
}

/// Compact, request-scoped entries layered over the two fixed dictionaries.
///
/// Unlike [`DictionarySourceOverrides`], these maps do not replace the base
/// files. They only override matching keys for the current request.
#[derive(Default)]
pub struct DictionaryPatches {
    pub vietphrase: HashMap<String, String>,
    pub chinese_phien_am_words: HanVietMap,
}

/// Parsed, request-scoped replacements for every runtime text dictionary
/// plus compact patches layered over fixed VietPhrase and
/// ChinesePhienAmWords dictionaries.
#[derive(Default)]
pub struct DictionaryOverrides {
    pub(crate) names: Option<HashMap<String, String>>,
    pub(crate) names2: Option<HashMap<String, String>>,
    pub(crate) luat_nhan: Option<Vec<(String, String)>>,
    pub(crate) pronouns: Option<HashMap<String, String>>,
    pub(crate) danh_tu: Option<HashMap<String, String>>,
    pub(crate) ho_nguoi: Option<HashMap<String, String>>,
    pub(crate) hau_tu: Option<HashMap<String, String>>,
    pub(crate) ignored_chinese_phrases: Option<Vec<String>>,
    pub(crate) vietphrase_patches: HashMap<String, String>,
    pub(crate) chinese_phien_am_words_patches: HanVietMap,
}

impl DictionaryOverrides {
    pub fn from_sources(sources: DictionarySourceOverrides<'_>) -> Self {
        Self {
            names: sources.names.map(first_wins_map),
            names2: sources.names2.map(first_wins_map),
            luat_nhan: sources.luat_nhan.map(parse_luat_nhan),
            pronouns: sources.pronouns.map(first_wins_map),
            danh_tu: sources.danh_tu.map(first_wins_map),
            ho_nguoi: sources.ho_nguoi.map(first_wins_map),
            hau_tu: sources.hau_tu.map(first_wins_map),
            ignored_chinese_phrases: sources
                .ignored_chinese_phrases
                .map(parse_ignored_chinese_phrases),
            vietphrase_patches: HashMap::new(),
            chinese_phien_am_words_patches: HanVietMap::new(),
        }
    }

    pub fn with_patches(mut self, patches: DictionaryPatches) -> Self {
        self.vietphrase_patches = patches.vietphrase;
        self.chinese_phien_am_words_patches = patches.chinese_phien_am_words;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_none()
            && self.names2.is_none()
            && self.luat_nhan.is_none()
            && self.pronouns.is_none()
            && self.danh_tu.is_none()
            && self.ho_nguoi.is_none()
            && self.hau_tu.is_none()
            && self.ignored_chinese_phrases.is_none()
            && self.vietphrase_patches.is_empty()
            && self.chinese_phien_am_words_patches.is_empty()
    }
}

pub(crate) trait DictionaryLookup {
    fn get(&self, key: &str) -> Option<&str>;

    fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
}

impl DictionaryLookup for HashMap<String, String> {
    fn get(&self, key: &str) -> Option<&str> {
        HashMap::get(self, key).map(String::as_str)
    }
}

/// All lookup maps used by the engine, after loading + priority merge.
#[derive(Default)]
pub struct Dictionaries {
    pub han_viet: HanVietMap,
    /// Parsed Names.txt before Names2 is applied.
    pub primary_names: HashMap<String, String>,
    /// Parsed Names2 before it is merged over Names.txt.
    pub secondary_names: HashMap<String, String>,
    pub only_name: HashMap<String, String>,
    pub only_name_one_meaning: HashMap<String, String>,
    /// Raw VietPhrase entries before Names are merged in. Number pre-scanning
    /// consults this map so an explicit dictionary entry wins over conversion.
    pub only_vietphrase: HashMap<String, String>,
    pub vietphrase: HashMap<String, String>,
    pub vietphrase_one_meaning: HashMap<String, String>,
    pub pronouns: HashMap<String, String>,
    pub danh_tu: HashMap<String, String>,
    pub ho_nguoi: HashMap<String, String>,
    pub hau_tu: HashMap<String, String>,
    /// Luật Nhân entries in QT's priority order: key length descending,
    /// followed by lexical order ascending.
    pub luat_nhan: Vec<(String, String)>,
    pub ignored_chinese_phrases: Vec<String>,
}

/// Parse `key=value` lines the way TranslatorEngine does:
/// - a leading BOM on the first line is stripped
/// - split on '=' into ALL parts; keep the line only if exactly 2 parts result
///   (so a value containing '=' drops the line, matching QT)
///
/// Note: '#'-prefixed lines are NOT treated as comments here. In the original
/// engine, only `loadLuatNhanDictionary` skips '#' lines; HanViet/VietPhrase/
/// Names/Names2 loaders do not. This generic helper is used by the latter, so
/// it must not skip '#'. The LuatNhan loader applies its own '#'-skipping.
/// See docs/engine/dictionaries.md §2.
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
        let primary_names = first_wins_map(names_src);
        let secondary_names = first_wins_map(names2_src);
        let mut only_name = primary_names.clone();
        for (key, value) in &secondary_names {
            only_name.insert(key.clone(), value.clone());
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

        let only_name_one_meaning = only_name
            .iter()
            .map(|(key, value)| {
                (
                    key.clone(),
                    value.split(['/', '|']).next().unwrap_or(value).to_string(),
                )
            })
            .collect();

        Dictionaries {
            han_viet,
            primary_names,
            secondary_names,
            only_name,
            only_name_one_meaning,
            only_vietphrase,
            vietphrase,
            vietphrase_one_meaning,
            ..Default::default()
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn build_full(
        han_viet_src: &str,
        names_src: &str,
        names2_src: &str,
        vietphrase_src: &str,
        pronouns_src: &str,
        danh_tu_src: &str,
        ho_nguoi_src: &str,
        hau_tu_src: &str,
        luat_nhan_src: &str,
        ignored_src: &str,
    ) -> Dictionaries {
        let mut dictionaries =
            Dictionaries::build(han_viet_src, names_src, names2_src, vietphrase_src);
        dictionaries.pronouns = first_wins_map(pronouns_src);
        dictionaries.danh_tu = first_wins_map(danh_tu_src);
        dictionaries.ho_nguoi = first_wins_map(ho_nguoi_src);
        dictionaries.hau_tu = first_wins_map(hau_tu_src);
        dictionaries.luat_nhan = parse_luat_nhan(luat_nhan_src);
        dictionaries.ignored_chinese_phrases = parse_ignored_chinese_phrases(ignored_src);
        dictionaries
    }

    /// Load from a data directory using the standard QT filenames, retaining
    /// the raw customizable defaults for API clients that need to edit them.
    pub fn load_with_defaults(
        data_dir: &std::path::Path,
    ) -> std::io::Result<(Dictionaries, DictionaryDefaults)> {
        let read =
            |rel: &str| -> std::io::Result<String> { std::fs::read_to_string(data_dir.join(rel)) };
        let han_viet = read("Resources/ChinesePhienAmWords.txt")?;
        let vietphrase = read("VietPhrase/VietPhrase.txt")?;
        let defaults = DictionaryDefaults::load(data_dir);
        let dictionaries = defaults.build_dictionaries(&han_viet, &vietphrase);
        Ok((dictionaries, defaults))
    }

    /// Load parsed dictionaries only. CLI callers use this when they do not
    /// need to expose the raw defaults.
    pub fn load(data_dir: &std::path::Path) -> std::io::Result<Dictionaries> {
        Self::load_with_defaults(data_dir).map(|(dictionaries, _)| dictionaries)
    }
}

fn first_wins_map(source: &str) -> HashMap<String, String> {
    let mut dictionary = HashMap::new();
    for (key, value) in parse_dict(source) {
        dictionary.entry(key).or_insert(value);
    }
    dictionary
}

fn parse_ignored_chinese_phrases(source: &str) -> Vec<String> {
    source
        .lines()
        .enumerate()
        .map(|(index, line)| {
            if index == 0 {
                line.trim_start_matches('\u{feff}').to_string()
            } else {
                line.to_string()
            }
        })
        .filter(|line| !line.is_empty())
        .collect()
}

fn parse_luat_nhan(source: &str) -> Vec<(String, String)> {
    let mut seen = HashSet::new();
    let mut rules = Vec::new();
    for (index, raw) in source.lines().enumerate() {
        let line = if index == 0 {
            raw.trim_start_matches('\u{feff}')
        } else {
            raw
        };
        if line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() == 2 && seen.insert(parts[0].to_string()) {
            rules.push((parts[0].to_string(), parts[1].to_string()));
        }
    }
    rules.sort_by(|(left, _), (right, _)| {
        right
            .chars()
            .count()
            .cmp(&left.chars().count())
            .then_with(|| left.cmp(right))
    });
    rules
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

    #[test]
    fn loader_requires_only_fixed_dictionaries() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("qt-core-fixed-dicts-{unique}"));
        std::fs::create_dir_all(directory.join("Resources")).unwrap();
        std::fs::create_dir_all(directory.join("VietPhrase")).unwrap();
        std::fs::write(
            directory.join("Resources/ChinesePhienAmWords.txt"),
            "他=tha\n",
        )
        .unwrap();
        std::fs::write(
            directory.join("VietPhrase/VietPhrase.txt"),
            "很好=rất tốt\n",
        )
        .unwrap();

        let dictionaries = Dictionaries::load(&directory).unwrap();
        assert!(dictionaries.only_name.is_empty());
        assert_eq!(
            dictionaries.only_vietphrase.get("很好").map(String::as_str),
            Some("rất tốt")
        );

        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn loader_retains_raw_customizable_defaults() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!("qt-core-default-dicts-{unique}"));
        std::fs::create_dir_all(directory.join("Resources")).unwrap();
        std::fs::create_dir_all(directory.join("VietPhrase")).unwrap();
        std::fs::create_dir_all(directory.join("Names2")).unwrap();
        std::fs::write(
            directory.join("Resources/ChinesePhienAmWords.txt"),
            "他=tha\n",
        )
        .unwrap();
        std::fs::write(
            directory.join("VietPhrase/VietPhrase.txt"),
            "很好=rất tốt\n",
        )
        .unwrap();
        std::fs::write(directory.join("Names.txt"), "萧炎=Tiêu Viêm\n").unwrap();
        std::fs::write(directory.join("Names2/123.txt"), "药老=Dược Lão\n").unwrap();
        std::fs::write(directory.join("Resources/Pronouns.txt"), "她=nàng\n").unwrap();

        let (dictionaries, defaults) = Dictionaries::load_with_defaults(&directory).unwrap();
        assert_eq!(defaults.names, "萧炎=Tiêu Viêm\n");
        assert_eq!(defaults.names2, "药老=Dược Lão\n");
        assert_eq!(defaults.pronouns, "她=nàng\n");
        assert!(defaults.luat_nhan.is_empty());
        assert_eq!(
            dictionaries.only_name.get("药老").map(String::as_str),
            Some("Dược Lão")
        );

        let _ = std::fs::remove_dir_all(directory);
    }
}
