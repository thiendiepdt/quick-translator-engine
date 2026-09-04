//! Port của qt-web/src/lib/ai-story.ts: schema story.json + normalize lenient + sắp xếp id chương.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Ordering;

pub const GLOSSARY_CATEGORIES: [&str; 7] =
    ["names", "places", "items", "creatures", "skills", "common", "signature_phrases"];

pub type StringMap = IndexMap<String, String>;
pub type Glossary = IndexMap<String, StringMap>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryStyle {
    pub voice: String,
    pub tone_rules: Vec<String>,
    pub signature_phrases: StringMap,
    pub avoid: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenreSetting {
    #[default]
    Ancient,
    Modern,
    /// Xuyên qua lại cổ đại ↔ hiện đại, đô thị tu tiên: prompt có cả hai bộ xưng hô, rule chỉ trung lập.
    Mixed,
}

impl GenreSetting {
    pub fn as_str(self) -> &'static str {
        match self {
            GenreSetting::Ancient => "ancient",
            GenreSetting::Modern => "modern",
            GenreSetting::Mixed => "mixed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenreNames {
    #[default]
    Han,
    Foreign,
    Mixed,
}

impl GenreNames {
    pub fn as_str(self) -> &'static str {
        match self {
            GenreNames::Han => "han",
            GenreNames::Foreign => "foreign",
            GenreNames::Mixed => "mixed",
        }
    }
}

/// Hai trục độc lập (port `StoryGenre` của qt-web). Thiếu/sai → ancient/han để truyện cũ không đổi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StoryGenre {
    pub setting: GenreSetting,
    pub names: GenreNames,
}

impl StoryGenre {
    /// Key tra `prompts.json` — cùng định dạng `genreKey` của web.
    pub fn key(&self) -> String {
        format!("{}/{}", self.setting.as_str(), self.names.as_str())
    }

    fn normalize(value: Option<&Value>) -> Self {
        let record = value.and_then(Value::as_object);
        let get = |key: &str| record.and_then(|r| r.get(key)).and_then(Value::as_str);
        StoryGenre {
            setting: match get("setting") {
                Some("modern") => GenreSetting::Modern,
                Some("mixed") => GenreSetting::Mixed,
                _ => GenreSetting::Ancient,
            },
            names: match get("names") {
                Some("foreign") => GenreNames::Foreign,
                Some("mixed") => GenreNames::Mixed,
                _ => GenreNames::Han,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckRule {
    pub pattern: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoGlossaryEntry {
    pub source: String,
    pub target: String,
    pub category: String,
    pub chapter: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutoGlossarySetting {
    Inherit,
    On,
    Off,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryConfig {
    pub name: String,
    pub source_url: String,
    pub protagonist: String,
    pub summary: String,
    pub genre: StoryGenre,
    pub glossary: Glossary,
    pub style: StoryStyle,
    pub custom_prompt: String,
    pub check_rules: Vec<CheckRule>,
    pub auto_glossary_log: Vec<AutoGlossaryEntry>,
    pub auto_glossary: AutoGlossarySetting,
}

pub fn empty_glossary() -> Glossary {
    GLOSSARY_CATEGORIES.iter().map(|key| (key.to_string(), StringMap::new())).collect()
}

fn string_value(value: Option<&Value>) -> String {
    value.and_then(Value::as_str).unwrap_or("").to_string()
}

fn string_list(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(|item| item.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

fn string_record(value: Option<&Value>) -> StringMap {
    value
        .and_then(Value::as_object)
        .map(|object| {
            object
                .iter()
                .filter_map(|(key, val)| {
                    let target = val.as_str()?;
                    (!key.trim().is_empty()).then(|| (key.clone(), target.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `a ?? b` của JS: chỉ rơi sang b khi a thiếu hoặc null.
fn coalesce<'a>(first: Option<&'a Value>, second: Option<&'a Value>) -> Option<&'a Value> {
    first.filter(|value| !value.is_null()).or(second)
}

impl StoryConfig {
    pub fn empty() -> Self {
        StoryConfig {
            name: String::new(),
            source_url: String::new(),
            protagonist: String::new(),
            summary: String::new(),
            genre: StoryGenre::default(),
            glossary: empty_glossary(),
            style: StoryStyle {
                voice: String::new(),
                tone_rules: vec![],
                signature_phrases: StringMap::new(),
                avoid: vec![],
            },
            custom_prompt: String::new(),
            check_rules: vec![],
            auto_glossary_log: vec![],
            auto_glossary: AutoGlossarySetting::Inherit,
        }
    }

    /// Port `normalizeAiStoryConfig`: lenient, bỏ field lạ, ép đúng schema.
    pub fn normalize(value: &Value) -> Self {
        let source = value.as_object();
        let get = |key: &str| source.and_then(|object| object.get(key));
        let glossary_value = get("glossary").and_then(Value::as_object);
        let glossary: Glossary = GLOSSARY_CATEGORIES
            .iter()
            .map(|key| (key.to_string(), string_record(glossary_value.and_then(|g| g.get(*key)))))
            .collect();
        let style_value = get("style").and_then(Value::as_object);
        let sget = |key: &str| style_value.and_then(|style| style.get(key));
        let check_rules = get("checkRules")
            .and_then(Value::as_array)
            .map(|rules| {
                rules
                    .iter()
                    .filter_map(|rule| {
                        let record = rule.as_object()?;
                        let pattern = string_value(record.get("pattern"));
                        let flags = string_value(record.get("flags"));
                        let message = string_value(record.get("message"));
                        if pattern.is_empty() || message.is_empty() {
                            return None;
                        }
                        Some(CheckRule { pattern, flags: (!flags.is_empty()).then_some(flags), message })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let auto_glossary_log = get("autoGlossaryLog")
            .and_then(Value::as_array)
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|entry| {
                        let record = entry.as_object()?;
                        let source = string_value(record.get("source")).trim().to_string();
                        let target = string_value(record.get("target")).trim().to_string();
                        let category = string_value(record.get("category"));
                        if source.is_empty()
                            || target.is_empty()
                            || !GLOSSARY_CATEGORIES.contains(&category.as_str())
                        {
                            return None;
                        }
                        Some(AutoGlossaryEntry {
                            source,
                            target,
                            category,
                            chapter: string_value(record.get("chapter")),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();
        let auto_glossary = match get("autoGlossary").and_then(Value::as_str) {
            Some("on") => AutoGlossarySetting::On,
            Some("off") => AutoGlossarySetting::Off,
            _ => AutoGlossarySetting::Inherit,
        };
        StoryConfig {
            name: string_value(get("name")),
            source_url: string_value(get("sourceUrl")),
            protagonist: string_value(get("protagonist")),
            summary: string_value(get("summary")),
            genre: StoryGenre::normalize(get("genre")),
            glossary,
            style: StoryStyle {
                voice: string_value(sget("voice")),
                tone_rules: string_list(coalesce(sget("toneRules"), sget("tone_rules"))),
                signature_phrases: string_record(coalesce(
                    sget("signaturePhrases"),
                    sget("signature_phrases"),
                )),
                avoid: string_list(sget("avoid")),
            },
            custom_prompt: string_value(get("customPrompt")),
            check_rules,
            auto_glossary_log,
            auto_glossary,
        }
    }

    /// `JSON.stringify(config, null, 2) + "\n"` — thứ tự key theo struct.
    pub fn to_json_pretty(&self) -> String {
        format!("{}\n", serde_json::to_string_pretty(self).expect("StoryConfig luôn serialize được"))
    }
}

#[derive(Debug, PartialEq)]
enum Token<'a> {
    Number(&'a str),
    Text(String),
}

fn tokens(input: &str) -> Vec<Token<'_>> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut in_digits: Option<bool> = None;
    for (index, ch) in input.char_indices() {
        let digit = ch.is_ascii_digit();
        if in_digits != Some(digit) {
            if let Some(kind) = in_digits {
                out.push(make_token(&input[start..index], kind));
            }
            start = index;
            in_digits = Some(digit);
        }
    }
    if let Some(kind) = in_digits {
        out.push(make_token(&input[start..], kind));
    }
    out
}

fn make_token(slice: &str, digits: bool) -> Token<'_> {
    if digits {
        Token::Number(slice)
    } else {
        Token::Text(slice.to_lowercase())
    }
}

fn compare_numbers(a: &str, b: &str) -> Ordering {
    let a = a.trim_start_matches('0');
    let b = b.trim_start_matches('0');
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

/// Xấp xỉ `localeCompare(_, "vi", {numeric: true, sensitivity: "base"})` cho id chương
/// (tên file): run chữ số so theo giá trị, run chữ so không phân biệt hoa thường, số đứng trước chữ.
pub fn natural_chapter_compare(left: &str, right: &str) -> Ordering {
    let a = tokens(left);
    let b = tokens(right);
    for (x, y) in a.iter().zip(b.iter()) {
        let ord = match (x, y) {
            (Token::Number(p), Token::Number(q)) => compare_numbers(p, q),
            (Token::Number(_), Token::Text(_)) => Ordering::Less,
            (Token::Text(_), Token::Number(_)) => Ordering::Greater,
            (Token::Text(p), Token::Text(q)) => p.cmp(q),
        };
        if ord != Ordering::Equal {
            return ord;
        }
    }
    a.len().cmp(&b.len()).then_with(|| left.cmp(right))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn normalize_bo_field_la_va_nhan_alias_snake_case() {
        let value = json!({
            "name": "Truyện A", "sourceUrl": "https://x", "protagonist": "Triệu Tĩnh Văn",
            "summary": "tóm tắt", "laField": 1,
            "glossary": { "names": { "赵静文": "Triệu Tĩnh Văn", " ": "bỏ", "k": 5 }, "nhomLa": {"a": "b"} },
            "style": { "voice": "lạnh", "tone_rules": ["ta/ngươi", 3], "signature_phrases": {"哼": "Hừ"}, "avoid": ["mình"] },
            "customPrompt": "  ", "checkRules": [ {"pattern": "x", "message": "m"}, {"pattern": "", "message": "m"}, {"pattern": "y", "flags": "i", "message": "n"}, "rác" ],
            "autoGlossaryLog": [ {"source": " 高塔 ", "target": "cao tháp", "category": "places", "chapter": "0001"}, {"source": "a", "target": "b", "category": "sai"} ],
            "autoGlossary": "on"
        });
        let story = StoryConfig::normalize(&value);
        assert_eq!(story.name, "Truyện A");
        assert_eq!(
            story.glossary.keys().cloned().collect::<Vec<_>>(),
            GLOSSARY_CATEGORIES.map(String::from).to_vec()
        );
        assert_eq!(story.glossary["names"].len(), 1);
        assert_eq!(story.style.tone_rules, vec!["ta/ngươi"]);
        assert_eq!(story.style.signature_phrases["哼"], "Hừ");
        assert_eq!(story.check_rules.len(), 2);
        assert_eq!(story.check_rules[1].flags.as_deref(), Some("i"));
        assert_eq!(story.auto_glossary_log.len(), 1);
        assert_eq!(story.auto_glossary_log[0].source, "高塔");
        assert_eq!(story.auto_glossary, AutoGlossarySetting::On);
        assert_eq!(story.custom_prompt, "  "); // normalize không trim customPrompt (web cũng vậy)
    }

    #[test]
    fn normalize_genre_thieu_hoac_sai_ve_ancient_han() {
        let none = StoryConfig::normalize(&json!({}));
        assert_eq!(none.genre, StoryGenre::default());
        assert_eq!(none.genre.key(), "ancient/han");
        let ok = StoryConfig::normalize(&json!({ "genre": { "setting": "modern", "names": "foreign" } }));
        assert_eq!(ok.genre, StoryGenre { setting: GenreSetting::Modern, names: GenreNames::Foreign });
        let bad = StoryConfig::normalize(&json!({ "genre": { "setting": "future", "names": 3 } }));
        assert_eq!(bad.genre, StoryGenre::default());
        let mixed = StoryConfig::normalize(&json!({ "genre": { "setting": "mixed" } }));
        assert_eq!(mixed.genre, StoryGenre { setting: GenreSetting::Mixed, names: GenreNames::Han });
        assert_eq!(mixed.genre.key(), "mixed/han");
        let json = ok.to_json_pretty();
        assert!(json.contains(
            "\"summary\": \"\",\n  \"genre\": {\n    \"setting\": \"modern\",\n    \"names\": \"foreign\"\n  },\n  \"glossary\""
        ));
    }

    #[test]
    fn empty_round_trip_giu_thu_tu_key_nhu_web() {
        let json = StoryConfig::empty().to_json_pretty();
        assert!(json.starts_with(
            "{\n  \"name\": \"\",\n  \"sourceUrl\": \"\",\n  \"protagonist\": \"\",\n  \"summary\": \"\",\n  \"genre\": {\n    \"setting\": \"ancient\",\n    \"names\": \"han\"\n  },\n  \"glossary\": {\n    \"names\": {},"
        ));
        assert!(json.ends_with("\"autoGlossary\": \"inherit\"\n}\n"));
        let back = StoryConfig::normalize(&serde_json::from_str(&json).unwrap());
        assert_eq!(back, StoryConfig::empty());
    }

    #[test]
    fn natural_compare_so_theo_gia_tri_so_va_khong_phan_biet_hoa_thuong() {
        let mut ids = vec!["chuong-0010", "chuong-0002", "10", "2", "Chuong-0001"];
        ids.sort_by(|a, b| natural_chapter_compare(a, b));
        assert_eq!(ids, vec!["2", "10", "Chuong-0001", "chuong-0002", "chuong-0010"]);
    }
}
