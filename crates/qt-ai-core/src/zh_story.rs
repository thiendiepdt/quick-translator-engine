//! Chuyển raw phồn thể của một truyện sang giản thể (`zh_convert::to_simplified`), để glossary, bảng nhân vật, sổ
//! tiêu đề — khoá đều là giản thể — khớp được tên trong raw.
//!
//! Chỉ đụng chương nhận ra là phồn thể (`is_traditional`) hoặc không phải UTF-8 (GBK / Big5 → UTF-8): chương giản
//! thể để yên, vì bộ chuyển vẫn đổi ngoặc 「」 và vài từ vùng miền trong raw giản thể. Bản gốc từng chương chép vào
//! `raw-phon-the/` (không đè bản cũ hơn). Khoá Hán có chữ phồn trong glossary, cụm đặc trưng, nhật ký tự thêm, bảng
//! nhân vật, sổ tiêu đề cũng đổi sang giản thể; khoá trùng sau khi đổi thì giữ mục vốn đã là giản thể. Tên file
//! chương (mã chương) giữ nguyên.
//!
//! Hoàn tác lần chuyển gần nhất: `work/zh-convert-undo.json` ghi băm bản đã ghi của từng chương và nguyên văn trước /
//! sau của file hồ sơ — chương hay file đã bị sửa tiếp sau đó thì giữ nguyên.

use crate::cast::{cast_to_json, load_cast};
use crate::error::{CoreError, Result};
use crate::story_fs::{
    list_raw_chapter_ids, load_story_config, now_ms, read_text, write_atomic, write_atomic_bytes, StoryPaths,
};
use crate::titles::{load_titles, titles_to_json};
use crate::vn_lexicon::fnv64;
use crate::zh_convert::{decode_text, is_traditional, to_simplified, traditional_score};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const TRADITIONAL_DIR: &str = "raw-phon-the";
const UNDO_FILE: &str = "zh-convert-undo.json";
/// Mở truyện chỉ đọc chừng này chương đầu để quyết định có nhắc chuyển không.
pub const STATUS_SAMPLE: usize = 20;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZhStoryStatus {
    /// Số chương đã đọc (mẫu đầu truyện, hoặc cả truyện).
    pub scanned: usize,
    /// Chương phồn thể — sẽ được chuyển.
    pub traditional: usize,
    /// Chương giản thể nhưng mã hoá khác UTF-8 — sẽ được ghi lại UTF-8.
    pub not_utf8: usize,
    /// Lần chuyển gần nhất còn hoàn tác được: số chương.
    pub undo_chapters: Option<usize>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZhStoryOutcome {
    pub chapters: usize,
    /// Khoá Hán đã đổi sang giản thể (glossary, bảng nhân vật, sổ tiêu đề…).
    pub keys: usize,
    /// Khoá đổi xong trùng một mục giản thể có sẵn — giữ mục có sẵn.
    pub merged: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZhUndoOutcome {
    pub restored: usize,
    /// Chương đã sửa / dịch lại raw sau lần chuyển — giữ nguyên.
    pub conflicts: Vec<String>,
    pub profile_restored: usize,
    pub profile_conflicts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UndoChapter {
    id: String,
    written_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UndoFile {
    name: String,
    original: String,
    written: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UndoRecord {
    created_at: u64,
    chapters: Vec<UndoChapter>,
    files: Vec<UndoFile>,
}

fn undo_path(paths: &StoryPaths) -> PathBuf {
    paths.work_dir.join(UNDO_FILE)
}

fn backup_dir(paths: &StoryPaths) -> PathBuf {
    paths.root.join(TRADITIONAL_DIR)
}

fn load_undo(paths: &StoryPaths) -> Option<UndoRecord> {
    std::fs::read_to_string(undo_path(paths)).ok().and_then(|text| serde_json::from_str(&text).ok())
}

/// Chương cần chuyển: (byte gốc, văn bản sẽ ghi). Phồn thể → giản thể; giản thể mã hoá khác → chỉ đổi sang UTF-8.
fn plan_chapter(path: &Path) -> Option<(Vec<u8>, String, bool)> {
    let bytes = std::fs::read(path).ok()?;
    let decoded = decode_text(&bytes)?;
    if is_traditional(&decoded.text) {
        let simplified = to_simplified(&decoded.text);
        return Some((bytes, simplified, true));
    }
    (!decoded.utf8).then_some((bytes, decoded.text, false))
}

/// `sample` = chỉ đọc chừng ấy chương đầu; None = cả truyện.
pub fn story_zh_status(paths: &StoryPaths, sample: Option<usize>) -> Result<ZhStoryStatus> {
    let ids = list_raw_chapter_ids(paths)?;
    let mut status = ZhStoryStatus {
        undo_chapters: load_undo(paths).map(|record| record.chapters.len()),
        ..ZhStoryStatus::default()
    };
    for id in ids.iter().take(sample.unwrap_or(usize::MAX)) {
        status.scanned += 1;
        match plan_chapter(&paths.raw_dir.join(format!("{id}.txt"))) {
            Some((_, _, true)) => status.traditional += 1,
            Some((_, _, false)) => status.not_utf8 += 1,
            None => {}
        }
    }
    Ok(status)
}

fn needs_key(key: &str) -> bool {
    traditional_score(key).1 > 0
}

/// Đổi khoá có chữ phồn sang giản thể, giữ thứ tự; trùng thì mục vốn là giản thể thắng. Trả số khoá đã đổi.
fn simplify_keys<V>(map: &mut IndexMap<String, V>, merged: &mut usize) -> usize {
    if !map.keys().any(|key| needs_key(key)) {
        return 0;
    }
    let mut changed = 0;
    for (key, value) in std::mem::take(map) {
        if !needs_key(&key) {
            if map.insert(key, value).is_some() {
                *merged += 1;
            }
            continue;
        }
        changed += 1;
        let simplified = to_simplified(&key);
        if map.contains_key(&simplified) {
            *merged += 1;
        } else {
            map.insert(simplified, value);
        }
    }
    changed
}

/// Đổi khoá trong hồ sơ (chưa ghi): các file đổi kèm bản trước / sau, số khoá đổi.
fn plan_profile(paths: &StoryPaths, outcome: &mut ZhStoryOutcome) -> Result<Vec<UndoFile>> {
    let mut files = Vec::new();
    let mut add = |name: &str, path: &Path, keys: usize, written: String, outcome: &mut ZhStoryOutcome| -> Result<()> {
        if keys > 0 {
            files.push(UndoFile { name: name.to_string(), original: read_text(path)?, written });
            outcome.keys += keys;
        }
        Ok(())
    };
    if let Ok(mut story) = load_story_config(paths) {
        let mut keys = 0;
        for group in story.glossary.values_mut() {
            keys += simplify_keys(group, &mut outcome.merged);
        }
        keys += simplify_keys(&mut story.style.signature_phrases, &mut outcome.merged);
        for entry in &mut story.auto_glossary_log {
            if needs_key(&entry.source) {
                entry.source = to_simplified(&entry.source);
                keys += 1;
            }
        }
        add("story.json", &paths.story_json, keys, story.to_json_pretty(), outcome)?;
    }
    let mut cast = load_cast(paths);
    let keys = simplify_keys(&mut cast.characters, &mut outcome.merged) + simplify_keys(&mut cast.addressing, &mut outcome.merged);
    add("cast.json", &paths.cast_json, keys, cast_to_json(&cast), outcome)?;
    let mut titles = load_titles(paths);
    let keys = simplify_keys(&mut titles.titles, &mut outcome.merged);
    add("titles.json", &paths.titles_json, keys, titles_to_json(&titles)?, outcome)?;
    Ok(files)
}

fn profile_path<'a>(paths: &'a StoryPaths, name: &str) -> Option<&'a Path> {
    match name {
        "story.json" => Some(paths.story_json.as_path()),
        "cast.json" => Some(paths.cast_json.as_path()),
        "titles.json" => Some(paths.titles_json.as_path()),
        _ => None,
    }
}

/// Chuyển các chương phồn thể / khác UTF-8 sang giản thể UTF-8 và đổi khoá hồ sơ theo. Không có gì để đổi thì không
/// ghi gì (giữ lần hoàn tác trước).
pub fn convert_story(paths: &StoryPaths) -> Result<ZhStoryOutcome> {
    let mut outcome = ZhStoryOutcome::default();
    let mut chapters = Vec::new();
    for id in list_raw_chapter_ids(paths)? {
        let path = paths.raw_dir.join(format!("{id}.txt"));
        if let Some((bytes, text, _)) = plan_chapter(&path) {
            chapters.push((id, path, bytes, text));
        }
    }
    let files = plan_profile(paths, &mut outcome)?;
    if chapters.is_empty() && files.is_empty() {
        return Ok(outcome);
    }
    outcome.chapters = chapters.len();
    // Ghi lần hoàn tác và bản gốc trước rồi mới sửa: dừng giữa chừng vẫn hoàn tác được phần đã ghi.
    let record = UndoRecord {
        created_at: now_ms(),
        chapters: chapters.iter().map(|(id, _, _, text)| UndoChapter { id: id.clone(), written_hash: fnv64(text) }).collect(),
        files: files.clone(),
    };
    std::fs::create_dir_all(&paths.work_dir).map_err(CoreError::io(&paths.work_dir))?;
    let json = serde_json::to_string(&record).map_err(|e| CoreError::Internal(e.to_string()))?;
    write_atomic(&undo_path(paths), &json)?;
    let backup = backup_dir(paths);
    std::fs::create_dir_all(&backup).map_err(CoreError::io(&backup))?;
    for (id, _, bytes, _) in &chapters {
        let saved = backup.join(format!("{id}.txt"));
        if !saved.exists() {
            write_atomic_bytes(&saved, bytes)?;
        }
    }
    for (_, path, _, text) in &chapters {
        write_atomic(path, text)?;
    }
    for file in &files {
        write_atomic(profile_path(paths, &file.name).expect("tên file hồ sơ cố định"), &file.written)?;
    }
    Ok(outcome)
}

/// Trả raw và hồ sơ về trước lần chuyển gần nhất, trừ chỗ đã bị sửa tiếp; bản gốc đã trả về thì xoá khỏi
/// `raw-phon-the/` (folder rỗng thì xoá luôn).
pub fn undo_story_convert(paths: &StoryPaths) -> Result<ZhUndoOutcome> {
    let Some(record) = load_undo(paths) else { return Ok(ZhUndoOutcome::default()) };
    let mut outcome = ZhUndoOutcome::default();
    let backup = backup_dir(paths);
    for chapter in record.chapters {
        let raw = paths.raw_dir.join(format!("{}.txt", chapter.id));
        let saved = backup.join(format!("{}.txt", chapter.id));
        let untouched = read_text(&raw).ok().is_some_and(|text| fnv64(&text) == chapter.written_hash);
        match std::fs::read(&saved) {
            Ok(bytes) if untouched => {
                write_atomic_bytes(&raw, &bytes)?;
                let _ = std::fs::remove_file(&saved);
                outcome.restored += 1;
            }
            _ => outcome.conflicts.push(chapter.id),
        }
    }
    for file in record.files {
        let Some(path) = profile_path(paths, &file.name) else { continue };
        if read_text(path).ok().as_deref() == Some(file.written.as_str()) {
            write_atomic(path, &file.original)?;
            outcome.profile_restored += 1;
        } else {
            outcome.profile_conflicts.push(file.name);
        }
    }
    let _ = std::fs::remove_file(undo_path(paths));
    if std::fs::read_dir(&backup).is_ok_and(|mut entries| entries.next().is_none()) {
        let _ = std::fs::remove_dir(&backup);
    }
    Ok(outcome)
}
