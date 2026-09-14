//! Port phần lắp prompt của qt-web/src/lib/ai-translation.ts. Base prompt + suffix đọc từ
//! prompts/prompts.json (do gen-golden.ts sinh) để không bao giờ lệch chữ với web.

use crate::story::{GenreTone, Glossary, StoryConfig, StoryGenre, StringMap};
use indexmap::IndexMap;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::LazyLock;

pub type TranslationGlossary = Glossary;

#[derive(Deserialize)]
struct Prompts {
    /// 9 base đã ghép sẵn ở web theo key `setting/names` — Rust không port logic ghép nên không thể drift.
    bases: IndexMap<String, String>,
    suffix: String,
    /// Mục giọng văn đứng riêng (key = tone), chèn vào base trước "## 1. Đại từ nhân xưng".
    #[serde(default)]
    tones: IndexMap<String, String>,
}

static PROMPTS: LazyLock<Prompts> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../prompts/prompts.json"))
        .expect("prompts/prompts.json hỏng — chạy `npm run golden` trong apps/qt-ai-cli")
});
static HAN_PERSON_NAME: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\p{Han}{3,4}$").unwrap());

pub fn base_prompt(genre: &StoryGenre) -> &'static str {
    PROMPTS
        .bases
        .get(&genre.key())
        .map(String::as_str)
        .unwrap_or_else(|| panic!("prompts.json thiếu base cho genre {}", genre.key()))
}

pub fn prompt_suffix() -> &'static str {
    &PROMPTS.suffix
}

/// Mục giọng văn cho `tone` (rỗng với neutral).
pub fn tone_prompt(tone: GenreTone) -> &'static str {
    PROMPTS.tones.get(tone.as_str()).map(String::as_str).unwrap_or("")
}

/// Dòng mở đầu mục đại từ trong mọi base (mixed có thêm " — chọn bảng theo cảnh").
const TONE_ANCHOR: &str = "\n## 1. Đại từ nhân xưng";

/// Chèn mục giọng văn vào base ngay trước "## 1. Đại từ nhân xưng" — cùng vị trí web ghép
/// (`...CORE_PHILOSOPHY, ...TONES[tone], ...setting.pronouns`), nên khớp từng byte với golden. Base người dùng
/// sửa tay mà mất dòng đó thì nối vào cuối; base đã có sẵn mục (dán tay) thì không chèn nữa.
pub fn insert_tone(base: &str, tone: GenreTone) -> String {
    let section = tone_prompt(tone);
    if section.is_empty() {
        return base.to_string();
    }
    let heading = section.lines().next().unwrap_or_default();
    if !heading.is_empty() && base.contains(heading) {
        return base.to_string();
    }
    match base.find(TONE_ANCHOR) {
        Some(at) => format!("{}\n{section}{}", &base[..at], &base[at..]),
        None => format!("{}\n\n{section}", base.trim_end()),
    }
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
    // Cặp xưng hô `甲→乙`: chương chạm tới khi có mặt ít nhất một bên.
    let sides = crate::story::addressing_sides(source);
    if sides.len() > 1 {
        return sides.iter().any(|side| glossary_entry_matches_source(side, text));
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
    let addressing_note = if glossary.get("addressing").is_some_and(|group| !group.is_empty()) {
        "\nNhóm `addressing`: `甲→乙: X–Y` nghĩa là trong thoại 甲 tự xưng X và gọi 乙 là Y (đã chốt ở chương trước, giữ y hệt; cặp ngược `乙→甲` có mục riêng).\n"
    } else {
        ""
    };
    let glossary_section = if glossary.is_empty() {
        String::new()
    } else {
        format!(
            "\n# Từ điển riêng của truyện\n\nCác mục này được ưu tiên và phải dùng nhất quán:\n\n{}\n{addressing_note}",
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
    let default_genre = StoryGenre::default();
    // Prompt riêng của truyện > base người dùng sửa ở app (file) > bản cứng trong binary.
    // Prompt riêng của truyện là toàn bộ ý người dùng → không chèn mục giọng.
    let base = match story.map(|s| s.custom_prompt.trim()).filter(|custom| !custom.is_empty()) {
        Some(custom) => custom.to_string(),
        None => {
            let genre = story.map(|s| &s.genre).unwrap_or(&default_genre);
            insert_tone(&crate::base::BaseStore::from_env().prompt(genre), genre.tone)
        }
    };
    format!("{base}{story_context}{glossary_section}{style_section}{}", prompt_suffix())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story::{GenreNames, GenreSetting};

    #[test]
    fn insert_tone_truoc_dai_tu_hoac_cuoi_base_khong_chen_hai_lan() {
        assert_eq!(insert_tone("A\n\n---\n\n## 1. Đại từ nhân xưng\n\nB", GenreTone::Neutral), "A\n\n---\n\n## 1. Đại từ nhân xưng\n\nB");
        let section = tone_prompt(GenreTone::Romance);
        assert!(section.starts_with("## Giọng văn: ngôn tình") && section.ends_with("---\n"));
        let with = insert_tone("A\n\n---\n\n## 1. Đại từ nhân xưng\n\nB", GenreTone::Romance);
        assert_eq!(with, format!("A\n\n---\n\n{section}\n## 1. Đại từ nhân xưng\n\nB"));
        // Base mixed: dòng đại từ có đuôi " — chọn bảng theo cảnh" vẫn là neo.
        assert!(insert_tone("X\n## 1. Đại từ nhân xưng — chọn bảng theo cảnh\nY", GenreTone::Romance).starts_with("X\n## Giọng văn"));
        // Base sửa tay mất dòng neo → nối cuối.
        assert_eq!(insert_tone("Prompt tự viết\n", GenreTone::Romance), format!("Prompt tự viết\n\n{section}"));
        // Đã có mục (dán tay) → giữ nguyên.
        assert_eq!(insert_tone(&with, GenreTone::Romance), with);
    }

    #[test]
    fn base_prompt_co_du_9_genre_va_khac_nhau() {
        let mut seen = std::collections::HashSet::new();
        for setting in [GenreSetting::Ancient, GenreSetting::Modern, GenreSetting::Mixed] {
            for names in [GenreNames::Han, GenreNames::Foreign, GenreNames::Mixed] {
                let base = base_prompt(&StoryGenre { setting, names, tone: GenreTone::Neutral });
                assert!(base.len() > 5000, "{setting:?}/{names:?}");
                assert!(seen.insert(base), "trùng base {setting:?}/{names:?}");
            }
        }
        assert_eq!(seen.len(), 9);
        assert!(base_prompt(&StoryGenre::default()).contains("| 我          | **ta**"));
        let modern = StoryGenre { setting: GenreSetting::Modern, names: GenreNames::Han, tone: GenreTone::Neutral };
        // Hiện đại: người kể ngôi một vẫn tự xưng `ta`, chỉ khác cổ đại ở phần thoại theo quan hệ.
        assert!(base_prompt(&modern).contains("| 我          | **ta** (lời kể ngôi một)"));
        assert!(!base_prompt(&modern).contains("| 我          | **tôi**"));
    }
}
