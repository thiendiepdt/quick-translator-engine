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
/// Port `TITLE_NAME`: danh xưng gắn họ (`老张`, `李老师`, `苏夫人`…) không phải tên riêng.
static TITLE_NAME: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:老|小|阿)?(?:欧阳|上官|司马|诸葛|慕容|东方|南宫|西门|公孙|令狐|皇甫|尉迟|长孙|宇文|独孤|端木|夏侯|轩辕|拓跋|百里|呼延|澹台|司徒|司空|万俟|闻人|太史|申屠|东郭|\p{Han})[一二三四五六七八九十大]?(?:姑娘|夫人|管家|管事|阿姨|姨|老师|大夫|捕头|长老|女士|先生|小姐|奶奶|爷爷|老头|师姐|师兄|师妹|师弟|哥|姐|少|导|总|董|经理)$|^老\p{Han}$").unwrap()
});
/// Port `BOOK_TITLE`: `《X》` lưu thành `X`.
static BOOK_TITLE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^《(.+)》$").unwrap());

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

/// Cửa công khai của `sanitize_addressing` cho lớp cast (đổi cặp đã có) — cùng luật hình thức với cặp mới.
pub fn parse_addressing_entry(source: &str, target: &str, raw: &str) -> Option<(String, String)> {
    sanitize_addressing(source, target, raw)
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
        let source = BOOK_TITLE.replace(&source, "$1").into_owned();
        let target = BOOK_TITLE.replace(&target, "$1").into_owned();
        if HAN.find_iter(&source).count() < 2 {
            continue;
        }
        if seen.contains(&source) || existing.contains(&source) {
            continue;
        }
        if !raw.contains(&source) || !translation.contains(&target) {
            continue;
        }
        // Nhóm ghi sai (không thuộc nhóm nào) thì bỏ — rơi vào names sẽ biến từ chung thành tên riêng.
        let category = match record.get("category").and_then(Value::as_str) {
            Some(category) if GLOSSARY_CATEGORIES.contains(&category) => category.to_string(),
            Some(_) => continue,
            None => "names".to_string(),
        };
        if category == "names" && TITLE_NAME.is_match(&source) {
            continue;
        }
        seen.insert(source.clone());
        pairs.push(ExtractedPair { source, target, category });
    }
    pairs
}

/// Tên `names` mà mỗi chữ Hán ứng đúng một tiếng (段誉 → Đoàn Dự): cặp (chữ, tiếng viết thường).
fn aligned_readings<'a>(source: &'a str, target: &'a str) -> Option<Vec<(char, String)>> {
    let chars: Vec<char> = source.chars().collect();
    let syllables: Vec<&str> = target.split_whitespace().collect();
    if chars.len() < 2 || chars.len() != syllables.len() || !chars.iter().all(|c| HAN.is_match(&c.to_string())) {
        return None;
    }
    Some(chars.into_iter().zip(syllables.iter().map(|s| s.to_lowercase())).collect())
}

/// Bỏ tên `names` mới có chữ Hán phiên khác mọi âm glossary đã chốt cho chữ đó (段誉 Đoàn Dự có sẵn thì
/// không học 段延庆 Đoạn Duyên Khánh) — auto-glossary học theo bản dịch, học cả lỗi thì chương sau lỗi theo.
pub fn drop_conflicting_readings(pairs: Vec<ExtractedPair>, glossary: &Glossary) -> Vec<ExtractedPair> {
    let mut readings: std::collections::HashMap<char, HashSet<String>> = std::collections::HashMap::new();
    for (source, target) in glossary.get("names").into_iter().flatten() {
        for (ch, reading) in aligned_readings(source, target).into_iter().flatten() {
            readings.entry(ch).or_default().insert(reading);
        }
    }
    pairs
        .into_iter()
        .filter(|pair| {
            pair.category != "names"
                || aligned_readings(&pair.source, &pair.target).into_iter().flatten().all(|(ch, reading)| {
                    readings.get(&ch).is_none_or(|known| known.contains(&reading))
                })
        })
        .collect()
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
