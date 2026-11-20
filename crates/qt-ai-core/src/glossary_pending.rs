//! Cổng "nghi trùng" của glossary tự học: tác giả gõ nhầm một chữ đồng âm (萧诧/萧姹), đảo chữ (尹凡戈/尹戈凡), tên
//! ngắn lệch tên đầy đủ có `·` (曼斯 "Mans" / 曼斯·龙德施泰特 "Mance"), hay khoá ngắn là đầu / đuôi khoá dài mà đọc khác
//! âm (天罡步 Thiên Cương / 天罡星斗… Thiên Cang). Glossary chỉ so khoá y hệt nên mỗi biến thể thành một tên riêng.
//!
//! Mục nghi trùng vẫn vào glossary như trước (bản dịch không đổi hành vi), kèm một câu hỏi trong
//! `glossary-pending.json` cạnh story.json; tab Glossary của app hỏi người dùng: cùng một (dùng tên cũ / dùng tên
//! mới, rồi Rà chương cũ) hay hai thứ khác nhau. Không bao giờ tự gộp: 恒山派/衡山派, 李玉/李瑜 là tên khác thật.
//! Pinyin ở `data/pinyin.tsv` (sinh từ Lạc Việt bằng `apps/qt-ai-cli/scripts/gen-pinyin.mjs`).

use crate::error::Result;
use crate::story::Glossary;
use crate::story_fs::{load_story_config, save_story_config, write_atomic, StoryPaths};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

static PINYIN: LazyLock<HashMap<char, Vec<&'static str>>> = LazyLock::new(|| {
    include_str!("../data/pinyin.tsv")
        .lines()
        .filter_map(|line| {
            let (ch, readings) = line.split_once('\t')?;
            Some((ch.chars().next()?, readings.split(',').collect()))
        })
        .collect()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SimilarKind {
    /// Cùng độ dài, khác đúng một chữ, hai chữ đọc cùng pinyin.
    Homophone,
    /// Cùng bộ chữ, khác thứ tự (≥3 chữ).
    Anagram,
    /// Một đoạn của tên có `·` cũng là khoá riêng mà cách dịch lệch.
    DotSegment,
    /// Khoá ngắn là đầu / đuôi khoá dài, cả hai phiên âm từng chữ mà có tiếng lệch âm.
    Prefix,
    /// Mục mới trái cách dịch ở chương cũ: `source_count` đoạn cũ có khoá, `similar_count` đoạn trong đó thiếu cách
    /// dịch mới. Không có mục cũ để gộp; người dùng Rà chương cũ hoặc bỏ qua.
    OldUsage,
}

impl SimilarKind {
    /// Gộp được bằng cách chép nguyên cách dịch của bên kia (hai khoá là một tên).
    pub fn mergeable(self) -> bool {
        matches!(self, SimilarKind::Homophone | SimilarKind::Anagram)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingEntry {
    pub source: String,
    pub target: String,
    pub category: String,
    pub chapter: String,
    pub similar_source: String,
    pub similar_target: String,
    pub similar_category: String,
    pub kind: SimilarKind,
    /// Số lần mỗi khoá có trong raw/ của truyện lúc phát hiện.
    pub source_count: usize,
    pub similar_count: usize,
}

pub fn pending_path(paths: &StoryPaths) -> PathBuf {
    paths.root.join("glossary-pending.json")
}

/// File thiếu hoặc hỏng → rỗng (không chặn chốt chương).
pub fn load_pending(paths: &StoryPaths) -> Vec<PendingEntry> {
    std::fs::read_to_string(pending_path(paths)).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
}

pub fn save_pending(paths: &StoryPaths, pending: &[PendingEntry]) -> Result<()> {
    let path = pending_path(paths);
    if pending.is_empty() {
        if path.exists() {
            std::fs::remove_file(&path).map_err(|source| crate::error::CoreError::Io { path: path.clone(), source })?;
        }
        return Ok(());
    }
    write_atomic(&path, &format!("{}\n", serde_json::to_string_pretty(pending).expect("PendingEntry luôn serialize được")))
}

fn homophone(a: char, b: char) -> bool {
    match (PINYIN.get(&a), PINYIN.get(&b)) {
        (Some(x), Some(y)) => x.iter().any(|reading| y.contains(reading)),
        _ => false,
    }
}

fn named(category: &str) -> bool {
    category == "names" || category == "places"
}

/// Mỗi chữ Hán ứng đúng một tiếng: trả các tiếng viết thường.
fn per_char(key: &str, target: &str) -> Option<Vec<String>> {
    let syllables: Vec<String> = target.split_whitespace().map(str::to_lowercase).collect();
    (syllables.len() == key.chars().count()).then_some(syllables)
}

fn kind_between(source: &str, target: &str, category: &str, key: &str, value: &str, group: &str) -> Option<SimilarKind> {
    let src: Vec<char> = source.chars().collect();
    let other: Vec<char> = key.chars().collect();
    if named(category) && named(group) && src.len() == other.len() && src.len() >= 2 {
        let diffs: Vec<usize> = (0..src.len()).filter(|&i| src[i] != other[i]).collect();
        if let [at] = diffs.as_slice() {
            if homophone(src[*at], other[*at]) {
                return Some(SimilarKind::Homophone);
            }
        }
        if src.len() >= 3 && diffs.len() >= 2 {
            let (mut a, mut b) = (src.clone(), other.clone());
            a.sort_unstable();
            b.sort_unstable();
            if a == b {
                return Some(SimilarKind::Anagram);
            }
        }
    }
    if source.contains('·') != key.contains('·') {
        let (long, long_target, short, short_target) =
            if source.contains('·') { (source, target, key, value) } else { (key, value, source, target) };
        if short.chars().count() >= 2
            && long.split('·').any(|segment| segment == short)
            && !long_target.to_lowercase().contains(&short_target.to_lowercase())
        {
            return Some(SimilarKind::DotSegment);
        }
    }
    if src.len() != other.len() && src.len().min(other.len()) >= 2 {
        let (long, long_target, short, short_target) =
            if src.len() > other.len() { (source, target, key, value) } else { (key, value, source, target) };
        let offset = if long.starts_with(short) {
            Some(0)
        } else if long.ends_with(short) {
            Some(long.chars().count() - short.chars().count())
        } else {
            None
        };
        // Tên dài chứa nguyên tên ngắn (益阳城 "thành Ích Dương") là bình thường — tiếng Việt hay đảo "thành / núi /
        // tàu" lên trước nên không so theo vị trí. Chỉ báo khi tên ngắn vắng hẳn, mà đúng vị trí vẫn có tiếng khớp
        // (天罡 "Thiên Cương" / 天罡步 "Thiên Cang Bộ": khớp "thiên", lệch "cương").
        if let (Some(offset), Some(long_syllables), Some(short_syllables)) =
            (offset, per_char(long, long_target), per_char(short, short_target))
        {
            // Lệch âm thật hầu như cùng phụ âm đầu (Thán / Thản, Vũ / Võ, Tư / Ty, Dao / Diêu); lệch do đảo trật tự
            // ("bộ lạc Quả" / "bộ lạc Tang Quả") thì khác. Đo trên thư viện: bỏ được phần lớn báo nhầm kiểu đảo.
            let head = |syllable: &str| syllable.chars().next().map(crate::check::fold_vietnamese);
            let aligned = |i: usize| &long_syllables[offset + i];
            let some_same = short_syllables.iter().enumerate().any(|(i, syllable)| aligned(i) == syllable);
            let diffs: Vec<(&String, &String)> =
                short_syllables.iter().enumerate().map(|(i, syllable)| (aligned(i), syllable)).filter(|(a, b)| a != b).collect();
            let same_heads = !diffs.is_empty() && diffs.iter().all(|(a, b)| head(a) == head(b));
            if some_same && same_heads && !long_target.to_lowercase().contains(&short_target.to_lowercase()) {
                return Some(SimilarKind::Prefix);
            }
        }
    }
    None
}

/// Mục glossary cũ mà mục mới `source = target` nghi trùng (khác cách dịch): (khoá, cách dịch, nhóm, kiểu).
pub fn similar_existing(source: &str, target: &str, category: &str, glossary: &Glossary) -> Option<(String, String, String, SimilarKind)> {
    if category == "addressing" {
        return None;
    }
    for (group, entries) in glossary {
        if group == "addressing" {
            continue;
        }
        for (key, value) in entries {
            if key == source || value == target {
                continue;
            }
            if let Some(kind) = kind_between(source, target, category, key, value, group) {
                return Some((key.clone(), value.clone(), group.clone(), kind));
            }
        }
    }
    None
}

/// Có đáng hỏi không, sau khi đã đếm số lần trong raw. Tên 2 chữ đồng âm hay là hai người khác nhau (刘峰 / 陆峰,
/// 白起 / 白七) nên chỉ hỏi khi một bên hiếm hẳn (≥5 lần ít hơn) — dấu hiệu tác giả gõ nhầm (萧姹 7 / 萧诧 955). Đo trên
/// thư viện: bỏ 120 / 448 câu hỏi đồng âm, phần lớn là tên khác thật.
pub fn worth_asking(entry: &PendingEntry) -> bool {
    if entry.kind != SimilarKind::Homophone || entry.source.chars().count() > 2 {
        return true;
    }
    let (a, b) = (entry.source_count.max(1), entry.similar_count.max(1));
    a.max(b) >= 5 * a.min(b)
}

/// Lựa chọn của người dùng cho một mục nghi trùng.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PendingChoice {
    /// Cùng một — mục mới dùng cách dịch của mục cũ.
    KeepOld,
    /// Cùng một — mục cũ đổi sang cách dịch của mục mới.
    UseNew,
    /// Hai thứ khác nhau (hoặc đã tự sửa tay) — chỉ bỏ câu hỏi.
    Dismiss,
}

/// Gợi ý Rà chương cũ sau khi gộp: trong các đoạn raw có `key`, thay `from` → `to`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackfillHint {
    pub key: String,
    pub from: String,
    pub to: String,
}

/// Áp lựa chọn cho câu hỏi của khoá `source`, ghi story.json và hàng chờ. Gộp chỉ cho kiểu đồng âm / đảo chữ.
pub fn resolve_pending(paths: &StoryPaths, source: &str, choice: PendingChoice) -> Result<Option<BackfillHint>> {
    let mut pending = load_pending(paths);
    let Some(index) = pending.iter().position(|entry| entry.source == source) else {
        return Err(crate::error::CoreError::InvalidState(format!("Không còn câu hỏi nghi trùng cho {source}.")));
    };
    let entry = pending.remove(index);
    let mut hint = None;
    if choice != PendingChoice::Dismiss && entry.kind.mergeable() {
        let mut story = load_story_config(paths)?;
        let (group, key, from, to) = match choice {
            PendingChoice::KeepOld => (&entry.category, &entry.source, &entry.target, &entry.similar_target),
            _ => (&entry.similar_category, &entry.similar_source, &entry.similar_target, &entry.target),
        };
        story.glossary.entry(group.clone()).or_default().insert(key.clone(), to.clone());
        save_story_config(paths, &story)?;
        hint = Some(BackfillHint { key: key.clone(), from: from.clone(), to: to.clone() });
    }
    save_pending(paths, &pending)?;
    Ok(hint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story::StringMap;

    fn glossary(entries: &[(&str, &str, &str)]) -> Glossary {
        let mut glossary = Glossary::new();
        for (group, key, value) in entries {
            glossary.entry(group.to_string()).or_insert_with(StringMap::new).insert(key.to_string(), value.to_string());
        }
        glossary
    }

    #[test]
    fn bat_bon_kieu_nghi_trung() {
        let g = glossary(&[
            ("names", "萧诧", "Tiêu Sá"),
            ("names", "尹凡戈", "Doãn Phàm Qua"),
            ("names", "曼斯·龙德施泰特", "Mance von Rundstedt"),
            ("skills", "天罡步", "Thiên Cương Bộ"),
        ]);
        let kind = |s: &str, t: &str, c: &str| similar_existing(s, t, c, &g).map(|(_, _, _, kind)| kind);
        assert_eq!(kind("萧姹", "Tiêu Xá", "names"), Some(SimilarKind::Homophone));
        assert_eq!(kind("尹戈凡", "Doãn Qua Phàm", "names"), Some(SimilarKind::Anagram));
        assert_eq!(kind("曼斯", "Mans", "names"), Some(SimilarKind::DotSegment));
        assert_eq!(kind("天罡", "Thiên Cang", "skills"), Some(SimilarKind::Prefix));
    }

    #[test]
    fn ten_hai_chu_dong_am_chi_hoi_khi_mot_ben_hiem_han() {
        let entry = |source: &str, a: usize, b: usize, kind: SimilarKind| PendingEntry {
            source: source.into(),
            target: String::new(),
            category: "names".into(),
            chapter: String::new(),
            similar_source: String::new(),
            similar_target: String::new(),
            similar_category: "names".into(),
            kind,
            source_count: a,
            similar_count: b,
        };
        assert!(worth_asking(&entry("萧姹", 7, 955, SimilarKind::Homophone)));
        assert!(!worth_asking(&entry("刘峰", 40, 60, SimilarKind::Homophone)));
        assert!(worth_asking(&entry("韩文胜", 25, 30, SimilarKind::Homophone)), "tên ≥3 chữ luôn hỏi");
        assert!(worth_asking(&entry("夜照门", 12, 15, SimilarKind::Anagram)), "đảo chữ luôn hỏi");
    }

    #[test]
    fn khong_bat_ten_khac_han_hoac_cung_cach_dich() {
        let g = glossary(&[("names", "萧诧", "Tiêu Sá"), ("names", "李玉", "Lý Ngọc"), ("items", "灵石", "linh thạch")]);
        // Cùng cách dịch: không có gì để hỏi.
        assert!(similar_existing("萧姹", "Tiêu Sá", "names", &g).is_none());
        // Khác chữ không đồng âm.
        assert!(similar_existing("萧红", "Tiêu Hồng", "names", &g).is_none());
        // Đồng âm nhưng ngoài names / places thì không xét luật gõ nhầm.
        assert!(similar_existing("灵时", "linh thì", "items", &g).is_none());
        // Cặp xưng hô không qua cổng.
        assert!(similar_existing("萧姹→李玉", "ta–ngươi", "addressing", &g).is_none());
    }
}
