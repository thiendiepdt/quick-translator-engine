//! Port qt-web/src/lib/ai-glossary.ts: vòng phản hồi glossary sau mỗi chương.

use crate::prompt::TranslationGlossary;
use crate::story::{addressing_sides, AutoGlossaryEntry, AutoGlossarySetting, Glossary, StoryConfig, ADDRESSING_ARROW, GLOSSARY_CATEGORIES};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::sync::LazyLock;

static HAN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\p{Han}").unwrap());
static ARROW: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"→|->|=>").unwrap());
static DASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*[–—\-/]\s*").unwrap());

/// Key glossary có mặt trong chương: tên thường thì nguyên văn; cặp `甲→乙` thì cả hai bên đều có.
pub fn glossary_key_touches_source(key: &str, text: &str) -> bool {
    let sides = addressing_sides(key);
    if sides.len() > 1 {
        sides.iter().all(|side| text.contains(side))
    } else {
        text.contains(key)
    }
}

/// Cặp xưng hô `甲→乙`: hai tên Hán đều có trong raw, target `X–Y` (nhận `-`, `—`, `/`, chuẩn về `–`).
/// Không đòi target nằm nguyên văn trong bản dịch — `anh–em` là hai từ rời.
fn sanitize_addressing(source: &str, target: &str, raw: &str) -> Option<(String, String)> {
    let sides: Vec<&str> = ARROW.split(source).map(str::trim).collect();
    if sides.len() != 2 || sides.iter().any(|side| side.is_empty() || !HAN.is_match(side) || !raw.contains(side)) {
        return None;
    }
    let parts: Vec<&str> = DASH.split(target).map(str::trim).filter(|part| !part.is_empty()).collect();
    if parts.len() != 2 {
        return None;
    }
    Some((sides.join(ADDRESSING_ARROW), parts.join("–")))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedPair {
    pub source: String,
    pub target: String,
    pub category: String,
}

/// Mọi key đã có trong prompt dịch (workspace + truyện).
pub fn collect_glossary_keys(workspace: &TranslationGlossary, story: &Glossary) -> HashSet<String> {
    workspace
        .values()
        .chain(story.values())
        .flat_map(|group| group.keys().cloned())
        .collect()
}

/// Model chỉ đề xuất; quyền quyết ở đây: ≥2 chữ Hán, có trong raw, target nguyên văn trong bản dịch,
/// key chưa tồn tại, không trùng; category lạ rơi về "names".
pub fn sanitize_extracted(
    parsed: &Value,
    raw: &str,
    translation: &str,
    existing: &HashSet<String>,
) -> Vec<ExtractedPair> {
    let Some(items) = parsed.as_array() else { return vec![] };
    let mut seen: HashSet<String> = HashSet::new();
    let mut pairs = Vec::new();
    for item in items {
        let Some(record) = item.as_object() else { continue };
        let source = record.get("source").and_then(Value::as_str).unwrap_or("").trim().to_string();
        let target = record.get("target").and_then(Value::as_str).unwrap_or("").trim().to_string();
        if source.is_empty() || target.is_empty() {
            continue;
        }
        if record.get("category").and_then(Value::as_str) == Some("addressing") {
            if let Some((key, value)) = sanitize_addressing(&source, &target, raw) {
                if !seen.contains(&key) && !existing.contains(&key) {
                    seen.insert(key.clone());
                    pairs.push(ExtractedPair { source: key, target: value, category: "addressing".to_string() });
                }
            }
            continue;
        }
        if HAN.find_iter(&source).count() < 2 {
            continue;
        }
        if seen.contains(&source) || existing.contains(&source) {
            continue;
        }
        if !raw.contains(&source) || !translation.contains(&target) {
            continue;
        }
        let category = record
            .get("category")
            .and_then(Value::as_str)
            .filter(|category| GLOSSARY_CATEGORIES.contains(category))
            .unwrap_or("names")
            .to_string();
        seen.insert(source.clone());
        pairs.push(ExtractedPair { source, target, category });
    }
    pairs
}

/// Chỉ thêm key mới (entry sẵn có luôn thắng), nối cuối nhóm, ghi nhật ký nguồn gốc.
pub fn append_auto_glossary(story: &StoryConfig, pairs: &[ExtractedPair], chapter: &str) -> StoryConfig {
    let mut updated = story.clone();
    let mut existing = collect_glossary_keys(&TranslationGlossary::new(), &updated.glossary);
    for pair in pairs {
        if existing.contains(&pair.source) {
            continue;
        }
        existing.insert(pair.source.clone());
        updated
            .glossary
            .entry(pair.category.clone())
            .or_default()
            .insert(pair.source.clone(), pair.target.clone());
        updated.auto_glossary_log.push(AutoGlossaryEntry {
            source: pair.source.clone(),
            target: pair.target.clone(),
            category: pair.category.clone(),
            chapter: chapter.to_string(),
        });
    }
    updated
}

/// Cài đặt theo truyện thắng; "inherit" mới rơi về toggle chung.
pub fn resolve_auto_glossary_enabled(setting: AutoGlossarySetting, settings_enabled: bool) -> bool {
    match setting {
        AutoGlossarySetting::On => true,
        AutoGlossarySetting::Off => false,
        AutoGlossarySetting::Inherit => settings_enabled,
    }
}
