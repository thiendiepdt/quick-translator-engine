//! Port apps/qt-ai-cli/src/story-fs.ts: layout folder truyện + đọc/ghi state.json, story.json.

use crate::error::{CoreError, Result};
use crate::story::{natural_chapter_compare, StoryConfig};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct StoryPaths {
    pub root: PathBuf,
    pub story_json: PathBuf,
    pub state_json: PathBuf,
    pub raw_dir: PathBuf,
    pub out_dir: PathBuf,
    pub work_dir: PathBuf,
}

pub fn story_paths(root: &Path) -> StoryPaths {
    StoryPaths {
        root: root.to_path_buf(),
        story_json: root.join("story.json"),
        state_json: root.join("state.json"),
        raw_dir: root.join("raw"),
        out_dir: root.join("out"),
        work_dir: root.join("work"),
    }
}

/// Root tuyệt đối giữ nguyên; tương đối thì theo cwd của tiến trình.
pub fn resolve_root(root: &Path) -> PathBuf {
    if root.is_absolute() {
        root.to_path_buf()
    } else {
        std::env::current_dir().map(|cwd| cwd.join(root)).unwrap_or_else(|_| root.to_path_buf())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChapterStatus {
    Queued,
    Translating,
    Done,
    Error,
    Skipped,
}

impl ChapterStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ChapterStatus::Queued => "queued",
            ChapterStatus::Translating => "translating",
            ChapterStatus::Done => "done",
            ChapterStatus::Error => "error",
            ChapterStatus::Skipped => "skipped",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterState {
    pub status: ChapterStatus,
    pub review_round: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Chương done nhưng hết vòng review vẫn còn vi phạm rule — người dùng xem lại sau.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
    pub updated_at: u64,
}

impl ChapterState {
    pub fn fresh(status: ChapterStatus) -> Self {
        ChapterState { status, review_round: 0, reason: None, warnings: None, updated_at: now_ms() }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HarnessSettings {
    pub min_length_ratio: f64,
    pub max_review_rounds: u32,
    pub chapters_per_session: u32,
}

impl Default for HarnessSettings {
    fn default() -> Self {
        HarnessSettings { min_length_ratio: 0.75, max_review_rounds: 3, chapters_per_session: 10 }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoryState {
    pub version: u32,
    pub settings: HarnessSettings,
    pub chapters: IndexMap<String, ChapterState>,
}

impl StoryState {
    pub fn new() -> Self {
        StoryState { version: 1, settings: HarnessSettings::default(), chapters: IndexMap::new() }
    }
}

impl Default for StoryState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

pub fn read_text(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(CoreError::io(path))
}

pub fn write_text(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content).map_err(CoreError::io(path))
}

/// Ghi atomic: file tạm cùng thư mục rồi rename đè.
pub fn write_atomic(path: &Path, content: &str) -> Result<()> {
    let mut tmp = path.as_os_str().to_owned();
    tmp.push(".tmp");
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, content).map_err(CoreError::io(&tmp))?;
    fs::rename(&tmp, path).map_err(CoreError::io(path))
}

pub fn list_raw_chapter_ids(paths: &StoryPaths) -> Result<Vec<String>> {
    if !paths.raw_dir.exists() {
        return Ok(vec![]);
    }
    let mut ids: Vec<String> = fs::read_dir(&paths.raw_dir)
        .map_err(CoreError::io(&paths.raw_dir))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.strip_suffix(".txt").map(str::to_string)
        })
        .collect();
    ids.sort_by(|a, b| natural_chapter_compare(a, b));
    Ok(ids)
}

pub fn read_raw_chapter(paths: &StoryPaths, id: &str) -> Result<String> {
    read_text(&paths.raw_dir.join(format!("{id}.txt")))
}

pub fn load_story_config(paths: &StoryPaths) -> Result<StoryConfig> {
    if !paths.story_json.exists() {
        return Err(CoreError::StoryNotFound(format!(
            "Không thấy story.json trong {} — chạy: qt-ai init",
            paths.root.display()
        )));
    }
    let text = read_text(&paths.story_json)?;
    let invalid = || {
        CoreError::InvalidStory(format!(
            "story.json hỏng (không phải JSON object): {}",
            paths.story_json.display()
        ))
    };
    let parsed: Value = serde_json::from_str(&text).map_err(|_| invalid())?;
    if !parsed.is_object() {
        return Err(invalid());
    }
    Ok(StoryConfig::normalize(&parsed))
}

pub fn save_story_config(paths: &StoryPaths, config: &StoryConfig) -> Result<()> {
    if paths.story_json.exists() {
        let mut bak = paths.story_json.as_os_str().to_owned();
        bak.push(".bak");
        let bak = PathBuf::from(bak);
        fs::copy(&paths.story_json, &bak).map_err(CoreError::io(&bak))?;
    }
    write_atomic(&paths.story_json, &config.to_json_pretty())
}

fn chapter_state_from(value: &Value) -> Option<ChapterState> {
    let record = value.as_object()?;
    let status: ChapterStatus = serde_json::from_value(record.get("status")?.clone()).ok()?;
    let review_round = record.get("reviewRound")?.as_u64()? as u32;
    let updated_at = record.get("updatedAt")?.as_u64()?;
    let reason = record.get("reason").and_then(Value::as_str).map(String::from);
    let warnings = record
        .get("warnings")
        .and_then(Value::as_array)
        .map(|items| items.iter().filter_map(|w| w.as_str().map(String::from)).collect());
    Some(ChapterState { status, review_round, reason, warnings, updated_at })
}

/// Lenient như bản TS: version phải là 1; settings thiếu field nào lấy mặc định field đó;
/// chương sai schema bị bỏ qua thay vì làm hỏng cả file.
pub fn load_state(paths: &StoryPaths) -> Result<StoryState> {
    if !paths.state_json.exists() {
        return Err(CoreError::StoryNotFound(format!(
            "Không thấy state.json trong {} — chạy: qt-ai init",
            paths.root.display()
        )));
    }
    let text = read_text(&paths.state_json)?;
    let parsed: Value =
        serde_json::from_str(&text).map_err(|e| CoreError::InvalidState(format!("state.json hỏng: {e}")))?;
    let record = match parsed.as_object() {
        Some(object) if object.get("version").and_then(Value::as_u64) == Some(1) => object,
        _ => {
            return Err(CoreError::InvalidState(format!(
                "state.json sai schema (cần object version 1): {}",
                paths.state_json.display()
            )))
        }
    };
    let fallback = HarnessSettings::default();
    let settings_value = record.get("settings").and_then(Value::as_object);
    let setting = |key: &str| settings_value.and_then(|s| s.get(key));
    let settings = HarnessSettings {
        min_length_ratio: setting("minLengthRatio").and_then(Value::as_f64).unwrap_or(fallback.min_length_ratio),
        max_review_rounds: setting("maxReviewRounds")
            .and_then(Value::as_u64)
            .map(|n| n as u32)
            .unwrap_or(fallback.max_review_rounds),
        chapters_per_session: setting("chaptersPerSession")
            .and_then(Value::as_u64)
            .map(|n| n as u32)
            .unwrap_or(fallback.chapters_per_session),
    };
    let mut chapters = IndexMap::new();
    if let Some(map) = record.get("chapters").and_then(Value::as_object) {
        for (id, value) in map {
            if let Some(chapter) = chapter_state_from(value) {
                chapters.insert(id.clone(), chapter);
            }
        }
    }
    Ok(StoryState { version: 1, settings, chapters })
}

pub fn save_state(paths: &StoryPaths, state: &StoryState) -> Result<()> {
    let json = serde_json::to_string_pretty(state).map_err(|e| CoreError::Internal(e.to_string()))?;
    write_atomic(&paths.state_json, &format!("{json}\n"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkKind {
    Prompt,
    Draft,
    Glossary,
    Check,
    Review,
}

pub const WORK_KINDS: [WorkKind; 5] =
    [WorkKind::Prompt, WorkKind::Draft, WorkKind::Glossary, WorkKind::Check, WorkKind::Review];

impl WorkKind {
    fn suffix(self) -> &'static str {
        match self {
            WorkKind::Prompt => ".prompt.md",
            WorkKind::Draft => ".draft.md",
            WorkKind::Glossary => ".glossary.json",
            WorkKind::Check => ".check.json",
            WorkKind::Review => ".review.md",
        }
    }
}

pub fn work_file(paths: &StoryPaths, id: &str, kind: WorkKind) -> PathBuf {
    paths.work_dir.join(format!("{id}{}", kind.suffix()))
}

pub fn ensure_story_dirs(paths: &StoryPaths) -> Result<()> {
    for dir in [&paths.raw_dir, &paths.out_dir, &paths.work_dir] {
        fs::create_dir_all(dir).map_err(CoreError::io(dir))?;
    }
    Ok(())
}
