//! Port phần lắp prompt của qt-web/src/lib/ai-translation.ts. Base prompt + suffix đọc từ
//! prompts/prompts.json (do gen-golden.ts sinh) để không bao giờ lệch chữ với web.

use crate::story::{Glossary, StoryConfig, StringMap};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::LazyLock;

pub type TranslationGlossary = Glossary;

#[derive(Deserialize)]
struct Prompts {
    base: String,
    suffix: String,
}

static PROMPTS: LazyLock<Prompts> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../prompts/prompts.json"))
        .expect("prompts/prompts.json hỏng — chạy `npm run golden` trong apps/qt-ai-cli")
});
static HAN_PERSON_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\p{Han}{3,4}$").unwrap());

pub fn base_prompt() -> &'static str {
    &PROMPTS.base
}

pub fn prompt_suffix() -> &'static str {
    &PROMPTS.suffix
}

/// `JSON.stringify(value, null, 2)` — cần feature `preserve_order` của serde_json để giữ thứ tự key.
pub fn json_pretty<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).expect("giá trị luôn serialize được")
}

fn non_empty_record(value: &StringMap) -> Option<StringMap> {
    let entries: StringMap = value
        .iter()
        .filter(|(source, target)| !source.trim().is_empty() && !target.trim().is_empty())
        .map(|(source, target)| (source.clone(), target.clone()))
        .collect();
    (!entries.is_empty()).then_some(entries)
}

fn merge_story_glossary(workspace: &TranslationGlossary, story: Option<&StoryConfig>) -> TranslationGlossary {
    let Some(story) = story else { return workspace.clone() };
    let mut merged = workspace.clone();
    for (key, entries) in &story.glossary {
        if let Some(normalized) = non_empty_record(entries) {
            let group = merged.entry(key.clone()).or_default();
            for (source, target) in normalized {
                group.insert(source, target); // key có sẵn giữ vị trí, key mới nối cuối — như spread của JS
            }
        }
    }
    merged
}

/// Entry "có mặt trong chương": nguyên văn, hoặc tên người 3–4 chữ Hán ở dạng bỏ họ.
pub fn glossary_entry_matches_source(source: &str, text: &str) -> bool {
    if text.contains(source) {
        return true;
    }
    if !HAN_PERSON_NAME.is_match(source) {
        return false;
    }
    let chars: Vec<char> = source.chars().collect();
    let without_surname: String = chars[1..].iter().collect();
    if text.contains(&without_surname) {
        return true;
    }
    chars.len() == 4 && text.contains(&chars[2..].iter().collect::<String>())
}

/// Chỉ giữ entry chương chạm tới; `signature_phrases` giữ nguyên; nhóm rỗng bị bỏ.
pub fn filter_glossary_for_source(glossary: &TranslationGlossary, source_text: &str) -> TranslationGlossary {
    glossary
        .iter()
        .filter_map(|(group, entries)| {
            if group == "signature_phrases" {
                return Some((group.clone(), entries.clone()));
            }
            let kept: StringMap = entries
                .iter()
                .filter(|(source, _)| glossary_entry_matches_source(source, source_text))
                .map(|(source, target)| (source.clone(), target.clone()))
                .collect();
            (!kept.is_empty()).then(|| (group.clone(), kept))
        })
        .collect()
}

pub fn build_system_prompt(
    workspace: &TranslationGlossary,
    story: Option<&StoryConfig>,
    source_text: Option<&str>,
) -> String {
    let merged = merge_story_glossary(workspace, story);
    let glossary = match source_text {
        Some(source) => filter_glossary_for_source(&merged, source),
        None => merged,
    };
    let glossary_section = if glossary.is_empty() {
        String::new()
    } else {
        format!(
            "\n# Từ điển riêng của truyện\n\nCác mục này được ưu tiên và phải dùng nhất quán:\n\n{}\n",
            json_pretty(&glossary)
        )
    };
    let story_context = match story {
        Some(s) if !s.name.is_empty() || !s.protagonist.is_empty() || !s.summary.is_empty() => {
            let mut context = serde_json::Map::new();
            for (key, value) in [("name", &s.name), ("protagonist", &s.protagonist), ("summary", &s.summary)] {
                if !value.is_empty() {
                    context.insert(key.to_string(), Value::String(value.clone()));
                }
            }
            format!("\n# Thông tin truyện\n\n{}\n", json_pretty(&context))
        }
        _ => String::new(),
    };
    let style_section = match story {
        Some(s)
            if !s.style.voice.is_empty()
                || !s.style.tone_rules.is_empty()
                || !s.style.signature_phrases.is_empty()
                || !s.style.avoid.is_empty() =>
        {
            let style = serde_json::json!({
                "voice": s.style.voice,
                "tone_rules": s.style.tone_rules,
                "signature_phrases": s.style.signature_phrases,
                "avoid": s.style.avoid,
            });
            format!(
                "\n# Style đặc thù của truyện\n\nStyle chỉ điều chỉnh từ vựng, xưng hô và register trong giới hạn trung thành; không được thêm hoặc bớt nội dung.\n\n{}\n",
                json_pretty(&style)
            )
        }
        _ => String::new(),
    };
    let base = story
        .map(|s| s.custom_prompt.trim())
        .filter(|custom| !custom.is_empty())
        .unwrap_or_else(|| base_prompt());
    format!("{base}{story_context}{glossary_section}{style_section}{}", prompt_suffix())
}
