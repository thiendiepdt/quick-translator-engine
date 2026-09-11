use crate::error::{blocking, CmdResult, CommandError};
use crate::sidecar::qt_ai_command;
use crate::summary_cache::SummaryCache;
use crate::AppState;
use qt_ai_core::commands::accept::run_accept;
use qt_ai_core::commands::export::{run_export, ExportOptions};
use qt_ai_core::commands::init::run_init;
use qt_ai_core::commands::retry::run_retry;
use qt_ai_core::commands::skip::run_skip;
use qt_ai_core::base::{BaseSource, BaseStore};
use qt_ai_core::commands::status::count_chapters;
use qt_ai_core::prompt::prompt_suffix;
use qt_ai_core::story::{natural_chapter_compare, CheckRule, StoryConfig, StoryGenre};
use qt_ai_core::story_fs::{
    load_state, load_story_config, read_raw_chapter, read_text, save_state, save_story_config, story_paths,
    work_file, HarnessSettings, WorkKind,
};
use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, Runtime, State};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterRow {
    pub id: String,
    pub status: String,
    pub review_round: u32,
    pub reason: Option<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    pub total: usize,
    pub queued: usize,
    pub translating: usize,
    pub done: usize,
    pub error: usize,
    pub skipped: usize,
    pub with_warnings: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorySnapshot {
    pub root: String,
    pub chapters: Vec<ChapterRow>,
    pub counts: Counts,
    pub settings: HarnessSettings,
    pub story: StoryConfig,
    pub session_running: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterView {
    pub id: String,
    pub status: String,
    pub raw: String,
    pub output: Option<String>,
    pub draft: Option<String>,
    pub review: Option<String>,
    pub warnings: Vec<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportOutcome {
    pub out_path: String,
    pub ids: Vec<String>,
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentSummary {
    pub root: String,
    pub name: Option<String>,
    pub done: Option<usize>,
    pub total: Option<usize>,
}

/// Tóm tắt từng folder gần đây; folder hỏng/mất → chỉ có root (UI hiện mờ).
pub fn summarize_recent(roots: &[String], cache: &SummaryCache) -> Vec<RecentSummary> {
    roots.iter().map(|root| cache.summarize(Path::new(root))).collect()
}

/// Prompt gốc + bộ rule mặc định của hệ để UI hiện cho người dùng sửa thành bản riêng.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoryDefaults {
    pub base_prompt: String,
    /// "builtin" | "file" — file = người dùng đã sửa base ở Cài đặt.
    pub prompt_source: String,
    /// Phần đuôi luôn nối sau prompt (mặc định hay tuỳ chỉnh) — chỉ đọc.
    pub prompt_suffix: String,
    pub check_rules: Vec<CheckRule>,
    pub rules_source: String,
}

pub fn defaults(genre: &StoryGenre, store: &BaseStore) -> StoryDefaults {
    let label = |source: BaseSource| match source {
        BaseSource::Builtin => "builtin".to_string(),
        BaseSource::File(_) => "file".to_string(),
    };
    StoryDefaults {
        base_prompt: store.prompt(genre),
        prompt_source: label(store.prompt_source(genre)),
        prompt_suffix: prompt_suffix().to_string(),
        check_rules: store.rules(genre.setting),
        rules_source: label(store.rules_source(genre.setting)),
    }
}

#[tauri::command]
pub fn story_defaults(state: State<'_, AppState>, genre: StoryGenre) -> CmdResult<StoryDefaults> {
    Ok(defaults(&genre, &state.base_store()))
}

#[tauri::command]
pub async fn recent_summaries<R: Runtime>(app: AppHandle<R>) -> CmdResult<Vec<RecentSummary>> {
    blocking(move || {
        let state = app.state::<AppState>();
        let roots = state.config.lock().unwrap().recent.clone();
        Ok(summarize_recent(&roots, &state.summaries))
    })
    .await
}

pub fn snapshot(root: &Path, session_running: bool) -> CmdResult<StorySnapshot> {
    let paths = story_paths(root);
    let state = load_state(&paths)?;
    let story = load_story_config(&paths)?;
    let counts = count_chapters(&state);
    let mut ids: Vec<&String> = state.chapters.keys().collect();
    ids.sort_by(|a, b| natural_chapter_compare(a, b));
    let chapters = ids
        .into_iter()
        .map(|id| {
            let chapter = &state.chapters[id];
            ChapterRow {
                id: id.clone(),
                status: chapter.status.as_str().to_string(),
                review_round: chapter.review_round,
                reason: chapter.reason.clone(),
                warnings: chapter.warnings.clone().unwrap_or_default(),
            }
        })
        .collect();
    Ok(StorySnapshot {
        root: root.display().to_string(),
        chapters,
        counts: Counts {
            total: counts.total,
            queued: counts.queued,
            translating: counts.translating,
            done: counts.done,
            error: counts.error,
            skipped: counts.skipped,
            with_warnings: counts.with_warnings,
        },
        settings: state.settings,
        story,
        session_running,
    })
}

fn optional_text(path: &Path) -> Option<String> {
    path.is_file().then(|| read_text(path).ok()).flatten()
}

pub fn chapter_view(root: &Path, id: &str) -> CmdResult<ChapterView> {
    let paths = story_paths(root);
    let state = load_state(&paths)?;
    let chapter = state
        .chapters
        .get(id)
        .ok_or_else(|| CommandError::new("story_not_found", format!("Không có chương {id} trong state.json.")))?;
    Ok(ChapterView {
        id: id.to_string(),
        status: chapter.status.as_str().to_string(),
        raw: read_raw_chapter(&paths, id)?,
        output: optional_text(&paths.out_dir.join(format!("{id}.txt"))),
        draft: optional_text(&work_file(&paths, id, WorkKind::Draft)),
        review: optional_text(&work_file(&paths, id, WorkKind::Review)),
        warnings: chapter.warnings.clone().unwrap_or_default(),
        reason: chapter.reason.clone(),
    })
}

pub fn save_story_inner(root: &Path, story: Value) -> CmdResult<StoryConfig> {
    let paths = story_paths(root);
    let config = StoryConfig::normalize(&story);
    save_story_config(&paths, &config)?;
    Ok(config)
}

pub fn save_settings_inner(root: &Path, settings: HarnessSettings) -> CmdResult<HarnessSettings> {
    let paths = story_paths(root);
    let mut state = load_state(&paths)?;
    state.settings = settings.clone();
    save_state(&paths, &state)?;
    Ok(settings)
}

fn session_running(state: &State<'_, AppState>, root: &str) -> bool {
    state.sessions.lock().unwrap().is_running(root)
}

/// Mở truyện đã init: quét raw/ lấy chương mới vào hàng đợi trước (run_init idempotent), rồi snapshot.
/// Folder chưa có state.json vẫn trả story_not_found để UI hỏi "Khởi tạo?".
#[tauri::command]
pub async fn open_story<R: Runtime>(app: AppHandle<R>, root: String) -> CmdResult<StorySnapshot> {
    blocking(move || open_story_sync(&app.state::<AppState>(), &root)).await
}

fn open_story_sync(state: &State<'_, AppState>, root: &str) -> CmdResult<StorySnapshot> {
    let path = Path::new(root);
    if story_paths(path).state_json.is_file() {
        run_init(path, &qt_ai_command())?;
    }
    let snap = snapshot(path, session_running(state, root))?;
    let mut config = state.config.lock().unwrap();
    config.touch_recent(root);
    config.save(&state.config_path)?;
    Ok(snap)
}

#[tauri::command]
pub async fn init_story<R: Runtime>(app: AppHandle<R>, root: String) -> CmdResult<StorySnapshot> {
    blocking(move || {
        run_init(Path::new(&root), &qt_ai_command())?;
        open_story_sync(&app.state::<AppState>(), &root)
    })
    .await
}

#[tauri::command]
pub fn story_snapshot(state: State<'_, AppState>, root: String) -> CmdResult<StorySnapshot> {
    snapshot(Path::new(&root), session_running(&state, &root))
}

#[tauri::command]
pub fn read_chapter(root: String, id: String) -> CmdResult<ChapterView> {
    chapter_view(Path::new(&root), &id)
}

#[tauri::command]
pub fn save_story(root: String, story: Value) -> CmdResult<StoryConfig> {
    save_story_inner(Path::new(&root), story)
}

#[tauri::command]
pub fn save_settings(root: String, settings: HarnessSettings) -> CmdResult<HarnessSettings> {
    save_settings_inner(Path::new(&root), settings)
}

/// Chặn khi phiên đang chạy: runner có thể đang ghi work/ hoặc out/ của chính chương này.
#[tauri::command]
pub fn chapter_retry(state: State<'_, AppState>, root: String, id: String) -> CmdResult<()> {
    if session_running(&state, &root) {
        return Err(CommandError::new(
            "session_locked",
            "Truyện này đang có phiên dịch chạy — bấm Dừng trước khi dịch lại chương.",
        ));
    }
    Ok(run_retry(Path::new(&root), &id)?)
}

#[tauri::command]
pub fn chapter_skip(root: String, id: String, reason: String) -> CmdResult<()> {
    Ok(run_skip(Path::new(&root), &id, &reason)?)
}

/// Chốt bằng --force; trả danh sách cảnh báo đã ghi vào state.
#[tauri::command]
pub async fn chapter_force_accept(root: String, id: String) -> CmdResult<Vec<String>> {
    blocking(move || Ok(run_accept(Path::new(&root), &id, true)?.warnings)).await
}

#[tauri::command]
pub async fn export_chapters(
    root: String,
    from: Option<String>,
    to: Option<String>,
    out: Option<String>,
) -> CmdResult<ExportOutcome> {
    blocking(move || {
        let result = run_export(Path::new(&root), &ExportOptions { from, to, out: out.map(PathBuf::from) })?;
        Ok(ExportOutcome { out_path: result.out_path.display().to_string(), ids: result.ids, gaps: result.gaps })
    })
    .await
}

/// Mở folder/file trong trình quản lý file của hệ.
#[tauri::command]
pub async fn reveal_folder(path: String) -> CmdResult<()> {
    blocking(move || {
        let program = if cfg!(windows) {
            "explorer"
        } else if cfg!(target_os = "macos") {
            "open"
        } else {
            "xdg-open"
        };
        std::process::Command::new(program)
            .arg(&path)
            .spawn()
            .map(|_| ())
            .map_err(|e| CommandError::new("io", format!("Không mở được {path}: {e}")))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use qt_ai_core::commands::init::run_init;
    use qt_ai_core::commands::next::run_next;
    use qt_ai_core::story_fs::{story_paths, work_file, WorkKind};
    use std::fs;

    fn story() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("raw")).unwrap();
        fs::write(dir.path().join("raw").join("0001.txt"), "赵静文抬头。\n\n她沉默。").unwrap();
        fs::write(dir.path().join("raw").join("0002.txt"), "第二章").unwrap();
        run_init(dir.path(), "qt-ai").unwrap();
        dir
    }

    #[test]
    fn defaults_theo_genre() {
        use qt_ai_core::story::{GenreNames, GenreSetting};
        let store = BaseStore::none();
        let d = defaults(&StoryGenre::default(), &store);
        assert_eq!(d.prompt_source, "builtin");
        assert_eq!(d.rules_source, "builtin");
        assert!(d.base_prompt.len() > 200 && !d.base_prompt.contains("Dịch raw text tiếng Trung"));
        assert!(d.prompt_suffix.contains("Dịch raw text tiếng Trung"));
        assert!(d.check_rules.iter().any(|r| r.message.contains("vợ/chồng")));
        let m = defaults(&StoryGenre { setting: GenreSetting::Modern, names: GenreNames::Foreign }, &store);
        assert!(m.base_prompt.contains("Emily"));
        assert!(!m.check_rules.iter().any(|r| r.message.contains("thê tử/phu quân")));
        let x = defaults(&StoryGenre { setting: GenreSetting::Mixed, names: GenreNames::Han }, &store);
        assert!(x.base_prompt.contains("theo cảnh"));
        assert!(!x.check_rules.iter().any(|r| r.message.contains("thê tử/phu quân") || r.message.contains("Xưng hô cổ trang")));
        let json = serde_json::to_value(&d).unwrap();
        assert!(json["checkRules"][0]["pattern"].is_string());
        assert!(json["basePrompt"].is_string());
    }

    #[test]
    fn open_story_quet_chuong_moi_khi_da_init_nhung_khong_init_folder_la() {
        // Không có State<AppState> trong unit test — kiểm phần logic quét bằng cùng điều kiện open_story dùng.
        let dir = story();
        fs::write(dir.path().join("raw").join("0003.txt"), "第三章").unwrap();
        let paths = story_paths(dir.path());
        assert!(paths.state_json.is_file());
        run_init(dir.path(), "qt-ai").unwrap();
        assert_eq!(snapshot(dir.path(), false).unwrap().counts.total, 3);
        let fresh = tempfile::tempdir().unwrap();
        assert!(!story_paths(fresh.path()).state_json.is_file());
        assert_eq!(snapshot(fresh.path(), false).unwrap_err().kind, "story_not_found");
    }

    #[test]
    fn snapshot_dem_va_liet_ke_chuong_theo_thu_tu() {
        let dir = story();
        let snap = snapshot(dir.path(), false).unwrap();
        assert_eq!(snap.counts.total, 2);
        assert_eq!(snap.counts.queued, 2);
        assert_eq!(snap.chapters.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), vec!["0001", "0002"]);
        assert_eq!(snap.chapters[0].status, "queued");
        assert_eq!(snap.settings.max_review_rounds, 3);
        assert!(!snap.session_running);
        assert!(snapshot(&dir.path().join("khong-co"), false).is_err());
    }

    #[test]
    fn chapter_view_gom_raw_draft_review_output() {
        let dir = story();
        let paths = story_paths(dir.path());
        run_next(dir.path()).unwrap();
        fs::write(work_file(&paths, "0001", WorkKind::Draft), "[[1]] nháp").unwrap();
        fs::write(work_file(&paths, "0001", WorkKind::Review), "# review").unwrap();
        let view = chapter_view(dir.path(), "0001").unwrap();
        assert_eq!(view.status, "translating");
        assert!(view.raw.contains("赵静文"));
        assert_eq!(view.draft.as_deref(), Some("[[1]] nháp"));
        assert_eq!(view.review.as_deref(), Some("# review"));
        assert!(view.output.is_none());
        assert!(chapter_view(dir.path(), "9999").is_err());
    }

    #[test]
    fn save_story_normalize_va_save_settings_ghi_state() {
        let dir = story();
        let story = save_story_inner(
            dir.path(),
            serde_json::json!({"name": "A", "laField": 1, "glossary": {"names": {"赵静文": "Triệu Tĩnh Văn"}}}),
        )
        .unwrap();
        assert_eq!(story.name, "A");
        assert_eq!(story.glossary["names"]["赵静文"], "Triệu Tĩnh Văn");
        assert!(!fs::read_to_string(dir.path().join("story.json")).unwrap().contains("laField"));
        let settings = qt_ai_core::story_fs::HarnessSettings {
            min_length_ratio: 0.8,
            max_review_rounds: 2,
            chapters_per_session: 5,
        };
        save_settings_inner(dir.path(), settings.clone()).unwrap();
        assert_eq!(snapshot(dir.path(), false).unwrap().settings, settings);
    }

    #[test]
    fn summarize_recent_doc_ten_va_tien_do_folder_hong_thi_none() {
        let dir = story();
        save_story_inner(dir.path(), serde_json::json!({"name": "Nam Nữ Đế"})).unwrap();
        let roots = vec![dir.path().display().to_string(), "D:\\khong\\co".to_string()];
        let list = summarize_recent(&roots, &SummaryCache::new());
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name.as_deref(), Some("Nam Nữ Đế"));
        assert_eq!(list[0].total, Some(2));
        assert_eq!(list[0].done, Some(0));
        assert_eq!(list[1].root, "D:\\khong\\co");
        assert!(list[1].name.is_none() && list[1].total.is_none());
    }
}
