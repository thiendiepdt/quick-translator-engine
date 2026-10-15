use crate::error::{blocking, CmdResult, CommandError};
use crate::sidecar::qt_ai_command;
use crate::summary_cache::SummaryCache;
use crate::AppState;
use qt_ai_core::commands::accept::run_accept;
use qt_ai_core::commands::delete::run_delete;
use qt_ai_core::commands::export::{run_export, ExportOptions};
use qt_ai_core::commands::init::run_init;
use qt_ai_core::commands::retry::{run_retry, run_retry_ids, run_retry_range};
use qt_ai_core::commands::skip::run_skip;
use qt_ai_core::commands::sync_progress::{run_sync_progress, ProgressDrift};
use qt_ai_core::base::{BaseSource, BaseStore};
use qt_ai_core::commands::status::count_chapters_live;
use qt_ai_core::vn_check::{live_warnings, open_by_chapter};
use qt_ai_core::prompt::prompt_suffix;
use qt_ai_core::story::{natural_chapter_compare, CheckRule, StoryConfig, StoryGenre};
use qt_ai_core::story_fs::{
    load_state, load_story_config, read_raw_chapter, read_text, save_state, save_story_config, story_paths, write_atomic,
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
    /// Độ lệch state.json ↔ out/ (Drive sync lệch). Chỉ tính lúc mở / quét lại truyện (`snapshot_with_drift`) vì
    /// phải đọc mọi out/; snapshot thường (polling trong phiên) để None.
    pub progress_drift: Option<DriftCounts>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DriftCounts {
    pub untracked: usize,
    pub missing: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterView {
    pub id: String,
    pub status: String,
    pub raw: String,
    /// raw/<id>.txt đã mất (xoá khi app đang mở) — `raw` rỗng, UI nhắc Quét lại để gỡ chương.
    pub raw_missing: bool,
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
    /// Chương còn phải dịch (queued + translating) — > 0 là truyện chưa dịch xong, phiên có việc để làm.
    pub pending: Option<usize>,
    /// Chỗ nhạy cảm VN chưa xét + cần xử lý; None = truyện chưa từng quét.
    pub vn_flags: Option<usize>,
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
    // Cảnh báo kiểm tra VN tính sống từ vn-check.json (xét / thay xong là hết), không lấy từ state.json.
    let vn_open = open_by_chapter(&qt_ai_core::vn_check::load_file(&paths));
    let counts = count_chapters_live(&state, &vn_open);
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
                warnings: live_warnings(
                    chapter.warnings.as_deref().unwrap_or_default(),
                    vn_open.get(id.as_str()).copied().unwrap_or(0),
                ),
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
        progress_drift: None,
    })
}

/// Snapshot kèm độ lệch tiến độ — cho mở truyện / quét lại / sau đồng bộ, không cho polling.
pub fn snapshot_with_drift(root: &Path, session_running: bool) -> CmdResult<StorySnapshot> {
    let mut snap = snapshot(root, session_running)?;
    let drift = qt_ai_core::commands::sync_progress::progress_drift(root)?;
    snap.progress_drift = Some(DriftCounts { untracked: drift.untracked.len(), missing: drift.missing.len() });
    Ok(snap)
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
    let raw_path = paths.raw_dir.join(format!("{id}.txt"));
    let raw_missing = !raw_path.is_file();
    Ok(ChapterView {
        id: id.to_string(),
        status: chapter.status.as_str().to_string(),
        raw: if raw_missing { String::new() } else { read_raw_chapter(&paths, id)? },
        raw_missing,
        output: optional_text(&paths.out_dir.join(format!("{id}.txt"))),
        draft: optional_text(&work_file(&paths, id, WorkKind::Draft)),
        review: optional_text(&work_file(&paths, id, WorkKind::Review)),
        warnings: live_warnings(
            chapter.warnings.as_deref().unwrap_or_default(),
            open_by_chapter(&qt_ai_core::vn_check::load_file(&paths)).get(id).copied().unwrap_or(0),
        ),
        reason: chapter.reason.clone(),
    })
}

/// Người dùng sửa tay bản dịch: ghi đè out/<id>.txt. Chỉ khi file đã có (chương chưa chốt thì không có
/// gì để sửa — nháp là của runner). Không đụng state.json.
pub fn save_chapter_output_inner(root: &Path, id: &str, text: &str) -> CmdResult<()> {
    let paths = story_paths(root);
    let out = paths.out_dir.join(format!("{id}.txt"));
    if !out.is_file() {
        return Err(CommandError::new("invalid_state", format!("Chương {id} chưa có bản dịch trong out/ để sửa.")));
    }
    Ok(write_atomic(&out, text)?)
}

pub fn save_story_inner(root: &Path, story: Value) -> CmdResult<StoryConfig> {
    let paths = story_paths(root);
    let config = StoryConfig::normalize(&story);
    // Cặp xưng hô người dùng sửa/thêm tay được ghim trong cast.json — harness không tự đổi theo mốc chương nữa.
    let no_pairs = qt_ai_core::story::StringMap::new();
    let old_pairs = load_story_config(&paths).ok().and_then(|old| old.glossary.get("addressing").cloned());
    if let Some(old_pairs) = old_pairs {
        let mut cast = qt_ai_core::cast::load_cast(&paths);
        let new_pairs = config.glossary.get("addressing").unwrap_or(&no_pairs);
        if qt_ai_core::cast::pin_edited_pairs(&old_pairs, new_pairs, &mut cast) {
            qt_ai_core::cast::save_cast(&paths, &cast)?;
        }
    }
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

/// Mở truyện đã init: quét raw/ lấy chương mới vào hàng đợi trước (run_init idempotent) và quét kiểm tra VN,
/// rồi snapshot.
/// Folder chưa có state.json vẫn trả story_not_found để UI hỏi "Khởi tạo?".
#[tauri::command]
pub async fn open_story<R: Runtime>(app: AppHandle<R>, root: String) -> CmdResult<StorySnapshot> {
    blocking(move || open_story_sync(&app.state::<AppState>(), &root)).await
}

/// Truyện đã init: quét raw/ lấy chương mới (run_init idempotent) rồi quét kiểm tra VN. Folder lạ thì bỏ qua.
pub fn rescan_if_initialized(path: &Path) -> CmdResult<()> {
    if story_paths(path).state_json.is_file() {
        run_init(path, &qt_ai_command())?;
        crate::vn_cmds::refresh_quietly(path, &crate::vn_cmds::lexicon());
    }
    Ok(())
}

fn open_story_sync(state: &State<'_, AppState>, root: &str) -> CmdResult<StorySnapshot> {
    let path = Path::new(root);
    rescan_if_initialized(path)?;
    let snap = snapshot_with_drift(path, session_running(state, root))?;
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
pub fn save_chapter_output(root: String, id: String, text: String) -> CmdResult<()> {
    save_chapter_output_inner(Path::new(&root), &id, &text)
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryRangeOutcomeView {
    pub retried: Vec<String>,
    pub backed_up: Vec<String>,
    pub already_queued: Vec<String>,
}

/// Dịch lại nhiều chương ([from..to], None = đầu/cuối). Chặn khi phiên đang chạy như `chapter_retry`.
#[tauri::command]
pub fn chapters_retry(
    state: State<'_, AppState>,
    root: String,
    from: Option<String>,
    to: Option<String>,
) -> CmdResult<RetryRangeOutcomeView> {
    if session_running(&state, &root) {
        return Err(CommandError::new(
            "session_locked",
            "Truyện này đang có phiên dịch chạy — bấm Dừng trước khi dịch lại chương.",
        ));
    }
    let outcome = run_retry_range(Path::new(&root), from.as_deref(), to.as_deref())?;
    Ok(RetryRangeOutcomeView {
        retried: outcome.retried,
        backed_up: outcome.backed_up,
        already_queued: outcome.already_queued,
    })
}

/// Dịch lại đúng các chương `ids` (nút "Dịch lại cả N" ở dòng hổng chương). Chặn khi phiên đang chạy.
#[tauri::command]
pub fn chapters_retry_ids(state: State<'_, AppState>, root: String, ids: Vec<String>) -> CmdResult<RetryRangeOutcomeView> {
    if session_running(&state, &root) {
        return Err(CommandError::new(
            "session_locked",
            "Truyện này đang có phiên dịch chạy — bấm Dừng trước khi dịch lại chương.",
        ));
    }
    let outcome = run_retry_ids(Path::new(&root), &ids)?;
    Ok(RetryRangeOutcomeView {
        retried: outcome.retried,
        backed_up: outcome.backed_up,
        already_queued: outcome.already_queued,
    })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteOutcomeView {
    pub removed: Vec<String>,
    pub kept_outputs: Vec<String>,
}

/// Xoá hẳn chương (gỡ state, xoá raw/ + work/; out/ giữ nguyên). Chặn khi phiên đang chạy như `chapter_retry`.
#[tauri::command]
pub fn chapters_delete(state: State<'_, AppState>, root: String, ids: Vec<String>) -> CmdResult<DeleteOutcomeView> {
    if session_running(&state, &root) {
        return Err(CommandError::new(
            "session_locked",
            "Truyện này đang có phiên dịch chạy — bấm Dừng trước khi xoá chương.",
        ));
    }
    let outcome = run_delete(Path::new(&root), &ids)?;
    Ok(DeleteOutcomeView { removed: outcome.removed, kept_outputs: outcome.kept_outputs })
}

/// Reset truyện về như lúc vừa tạo (chỉ giữ tên + link; raw/, out/, export/ và cài đặt harness còn nguyên) để
/// dịch lại từ đầu sau khi app đổi lớn. Chặn khi phiên đang chạy: runner đang ghi state/work của truyện.
#[tauri::command]
pub fn story_reset(state: State<'_, AppState>, root: String) -> CmdResult<StorySnapshot> {
    if session_running(&state, &root) {
        return Err(CommandError::new(
            "session_locked",
            "Truyện này đang có phiên dịch chạy — bấm Dừng trước khi reset truyện.",
        ));
    }
    qt_ai_core::commands::reset::run_reset(Path::new(&root), &qt_ai_command())?;
    snapshot(Path::new(&root), false)
}

/// Chương nào lệch giữa state.json và out/ — dialog Đồng bộ tiến độ xem trước. Đọc mọi out/ nên chạy ngoài luồng IPC.
#[tauri::command]
pub async fn progress_drift(root: String) -> CmdResult<ProgressDrift> {
    blocking(move || Ok(qt_ai_core::commands::sync_progress::progress_drift(Path::new(&root))?)).await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncProgressView {
    pub marked_done: Vec<String>,
    pub missing: Vec<String>,
    /// Snapshot sau đồng bộ, kèm độ lệch mới (banner ở trang Dịch tự ẩn).
    pub snapshot: StorySnapshot,
}

pub fn sync_progress_inner(root: &Path, session_running: bool) -> CmdResult<SyncProgressView> {
    if session_running {
        return Err(CommandError::new(
            "session_locked",
            "Truyện này đang có phiên dịch chạy — bấm Dừng trước khi đồng bộ tiến độ.",
        ));
    }
    let outcome = run_sync_progress(root)?;
    Ok(SyncProgressView {
        marked_done: outcome.marked_done,
        missing: outcome.missing,
        snapshot: snapshot_with_drift(root, false)?,
    })
}

/// Đánh dấu đã dịch các chương có bản dịch trong out/ mà state.json (bản cũ sau sync Drive) ghi chưa dịch.
/// Chặn khi phiên đang chạy: runner đang ghi state.json của truyện.
#[tauri::command]
pub async fn sync_progress<R: Runtime>(app: AppHandle<R>, root: String) -> CmdResult<SyncProgressView> {
    blocking(move || {
        let running = session_running(&app.state::<AppState>(), &root);
        sync_progress_inner(Path::new(&root), running)
    })
    .await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequeueUntrackedView {
    pub requeued: Vec<String>,
    pub snapshot: StorySnapshot,
}

pub fn requeue_untracked_inner(root: &Path, session_running: bool) -> CmdResult<RequeueUntrackedView> {
    if session_running {
        return Err(CommandError::new(
            "session_locked",
            "Truyện này đang có phiên dịch chạy — bấm Dừng trước khi dịch lại chương.",
        ));
    }
    let requeued = qt_ai_core::commands::sync_progress::requeue_untracked(root)?;
    Ok(RequeueUntrackedView { requeued, snapshot: snapshot_with_drift(root, false)? })
}

/// Lối thoát khi báo lệch nhầm: đưa các chương lệch về hàng đợi dịch lại, bản dịch cũ cất .bak.
/// Chặn khi phiên đang chạy như `chapter_retry`.
#[tauri::command]
pub async fn requeue_untracked<R: Runtime>(app: AppHandle<R>, root: String) -> CmdResult<RequeueUntrackedView> {
    blocking(move || {
        let running = session_running(&app.state::<AppState>(), &root);
        requeue_untracked_inner(Path::new(&root), running)
    })
    .await
}

#[tauri::command]
pub fn chapter_skip(root: String, id: String, reason: String) -> CmdResult<()> {
    Ok(run_skip(Path::new(&root), &id, &reason)?)
}

/// Chốt bằng --force; trả danh sách cảnh báo (state.json + dòng kiểm tra VN không lưu vào state).
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

/// Ghi nguyên `content` (BOM nếu có đã nằm trong chuỗi) ra `path`, tạo folder cha nếu thiếu.
/// Dùng cho export glossary → Names.txt; file nhỏ nên ghi thẳng, không qua spawn_blocking.
#[tauri::command]
pub fn write_text_file(path: String, content: String) -> CmdResult<()> {
    let target = Path::new(&path);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CommandError::new("io", format!("Không tạo được folder {}: {e}", parent.display())))?;
    }
    std::fs::write(target, content.as_bytes())
        .map_err(|e| CommandError::new("io", format!("Không ghi được {path}: {e}")))
}

#[cfg(test)]
pub(crate) mod tests {
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
    fn save_story_ghim_cap_xung_ho_nguoi_dung_sua() {
        let dir = story();
        let paths = story_paths(dir.path());
        let mut config = load_story_config(&paths).unwrap();
        config.glossary.entry("addressing".to_string()).or_default().insert("a→b".into(), "tôi–cô".into());
        save_story_inner(dir.path(), serde_json::to_value(&config).unwrap()).unwrap();
        let cast = qt_ai_core::cast::load_cast(&paths);
        assert!(cast.addressing["a→b"].pinned, "cặp người dùng thêm tay được ghim");
        // Lưu lại không đổi gì → không ghi thêm.
        let before = fs::read_to_string(&paths.cast_json).unwrap();
        save_story_inner(dir.path(), serde_json::to_value(&config).unwrap()).unwrap();
        assert_eq!(fs::read_to_string(&paths.cast_json).unwrap(), before);
    }

    #[test]
    fn chapter_view_raw_mat_thi_bao_raw_missing_thay_vi_loi() {
        let dir = story();
        let view = chapter_view(dir.path(), "0001").unwrap();
        assert!(!view.raw_missing && view.raw.contains("赵静文"));
        fs::remove_file(dir.path().join("raw").join("0001.txt")).unwrap();
        let view = chapter_view(dir.path(), "0001").unwrap();
        assert!(view.raw_missing && view.raw.is_empty());
        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["rawMissing"], true);
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
        let m = defaults(&StoryGenre { setting: GenreSetting::Modern, names: GenreNames::Foreign, ..StoryGenre::default() }, &store);
        assert!(m.base_prompt.contains("Emily"));
        assert!(!m.check_rules.iter().any(|r| r.message.contains("thê tử/phu quân")));
        let x = defaults(&StoryGenre { setting: GenreSetting::Mixed, names: GenreNames::Han, ..StoryGenre::default() }, &store);
        assert!(x.base_prompt.contains("theo cảnh"));
        assert!(!x.check_rules.iter().any(|r| r.message.contains("thê tử/phu quân") || r.message.contains("Xưng hô cổ trang")));
        let json = serde_json::to_value(&d).unwrap();
        assert!(json["checkRules"][0]["pattern"].is_string());
        assert!(json["basePrompt"].is_string());
    }

    #[test]
    fn open_story_quet_chuong_moi_khi_da_init_nhung_khong_init_folder_la() {
        // Không có State<AppState> trong unit test — kiểm phần quét open_story dùng.
        let dir = story();
        fs::write(dir.path().join("raw").join("0003.txt"), "第三章").unwrap();
        rescan_if_initialized(dir.path()).unwrap();
        assert_eq!(snapshot(dir.path(), false).unwrap().counts.total, 3);
        assert!(dir.path().join(qt_ai_core::vn_check::VN_FILE).is_file(), "mở truyện thì quét kiểm tra VN");
        let fresh = tempfile::tempdir().unwrap();
        rescan_if_initialized(fresh.path()).unwrap();
        assert!(!story_paths(fresh.path()).state_json.is_file());
        assert!(!fresh.path().join(qt_ai_core::vn_check::VN_FILE).exists());
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
    fn snapshot_canh_bao_vn_tinh_song_va_bo_dong_vn_cu() {
        use qt_ai_core::story_fs::ChapterStatus;
        use qt_ai_core::vn_check::{load_file, refresh, set_status, Status, VN_WARNING_PREFIX};
        let dir = story();
        let paths = story_paths(dir.path());
        fs::write(dir.path().join("raw").join("0001.txt"), "第1章\n\n坐船转到越南。").unwrap();
        fs::create_dir_all(&paths.out_dir).unwrap();
        fs::write(paths.out_dir.join("0001.txt"), "Chương 1\n\nNgồi thuyền qua Việt Nam.").unwrap();
        let mut state = load_state(&paths).unwrap();
        let chapter = state.chapters.get_mut("0001").unwrap();
        chapter.status = ChapterStatus::Done;
        // Dữ liệu cũ: accept trước đây ghi dòng [VN] vào state.json.
        chapter.warnings = Some(vec!["[VN] 7 chỗ cũ".into()]);
        save_state(&paths, &state).unwrap();
        refresh(&paths, &qt_ai_core::vn_lexicon::Lexicon::builtin()).unwrap();

        let snap = snapshot(dir.path(), false).unwrap();
        let row = &snap.chapters[0];
        assert_eq!(row.warnings.len(), 1, "{:?}", row.warnings);
        assert!(row.warnings[0].starts_with(VN_WARNING_PREFIX) && row.warnings[0].contains("1 chỗ"), "{:?}", row.warnings);
        assert_eq!(snap.counts.with_warnings, 1);
        assert_eq!(chapter_view(dir.path(), "0001").unwrap().warnings, row.warnings);

        let ids: Vec<String> = load_file(&paths).hits.iter().map(|h| h.id.clone()).collect();
        set_status(&paths, &ids, Status::Ok).unwrap();
        let snap = snapshot(dir.path(), false).unwrap();
        assert!(snap.chapters[0].warnings.is_empty(), "{:?}", snap.chapters[0].warnings);
        assert_eq!(snap.counts.with_warnings, 0);
        assert!(chapter_view(dir.path(), "0001").unwrap().warnings.is_empty());
    }

    /// Bản dịch "máy kia" ghi sau state.json: mtime đặt hẳn 1 giờ sau, khỏi phụ thuộc độ mịn đồng hồ hệ tệp.
    pub(crate) fn write_later(path: &Path, text: &str) {
        fs::write(path, text).unwrap();
        let later = std::time::SystemTime::now() + std::time::Duration::from_secs(3600);
        fs::File::options().write(true).open(path).unwrap().set_modified(later).unwrap();
    }

    #[test]
    fn snapshot_thuong_khong_tinh_lech_tien_do_chi_ban_mo_hoac_quet_lai_moi_tinh() {
        let dir = story();
        let paths = story_paths(dir.path());
        // Drive sync về out/ nhưng state.json là bản cũ: 0001 có bản dịch mà vẫn queued.
        write_later(&paths.out_dir.join("0001.txt"), "Chương một.");
        let snap = snapshot(dir.path(), false).unwrap();
        assert!(snap.progress_drift.is_none(), "snapshot hay gọi (polling phiên) không đọc out/");
        assert!(serde_json::to_value(&snap).unwrap()["progressDrift"].is_null());
        let snap = snapshot_with_drift(dir.path(), false).unwrap();
        assert_eq!(snap.progress_drift, Some(DriftCounts { untracked: 1, missing: 0 }));
        assert_eq!(serde_json::to_value(&snap).unwrap()["progressDrift"]["untracked"], 1);
    }

    #[test]
    fn requeue_untracked_chan_khi_phien_chay_xong_thi_cat_bak_va_het_lech() {
        let dir = story();
        let paths = story_paths(dir.path());
        write_later(&paths.out_dir.join("0002.txt"), "Chương hai.");
        assert_eq!(requeue_untracked_inner(dir.path(), true).unwrap_err().kind, "session_locked");
        assert!(paths.out_dir.join("0002.txt").exists(), "bị chặn thì không đụng gì");

        let result = requeue_untracked_inner(dir.path(), false).unwrap();
        assert_eq!(result.requeued, vec!["0002"]);
        assert!(paths.out_dir.join("0002.txt.bak").exists() && !paths.out_dir.join("0002.txt").exists());
        assert_eq!(result.snapshot.progress_drift, Some(DriftCounts { untracked: 0, missing: 0 }));
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["requeued"][0], "0002");
    }

    #[test]
    fn sync_progress_chan_khi_phien_chay_xong_thi_tra_snapshot_het_lech() {
        let dir = story();
        let paths = story_paths(dir.path());
        write_later(&paths.out_dir.join("0002.txt"), "Chương hai.");
        let error = sync_progress_inner(dir.path(), true).unwrap_err();
        assert_eq!(error.kind, "session_locked");
        assert_eq!(load_state(&paths).unwrap().chapters["0002"].status.as_str(), "queued", "bị chặn thì không ghi");

        let result = sync_progress_inner(dir.path(), false).unwrap();
        assert_eq!(result.marked_done, vec!["0002"]);
        assert!(result.missing.is_empty());
        assert_eq!(result.snapshot.counts.done, 1);
        assert_eq!(result.snapshot.progress_drift, Some(DriftCounts { untracked: 0, missing: 0 }));
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["markedDone"][0], "0002");
        assert_eq!(json["snapshot"]["progressDrift"]["untracked"], 0);
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
    fn save_chapter_output_ghi_de_out_chi_khi_da_co_ban_dich() {
        let dir = story();
        let paths = story_paths(dir.path());
        let error = save_chapter_output_inner(dir.path(), "0001", "sửa").unwrap_err();
        assert_eq!(error.kind, "invalid_state");
        fs::create_dir_all(&paths.out_dir).unwrap();
        fs::write(paths.out_dir.join("0001.txt"), "bản máy").unwrap();
        save_chapter_output_inner(dir.path(), "0001", "bản người sửa\n").unwrap();
        assert_eq!(chapter_view(dir.path(), "0001").unwrap().output.as_deref(), Some("bản người sửa\n"));
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

    #[test]
    fn write_text_file_ghi_nguyen_byte_va_tao_folder_cha() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("sub").join("Names.txt");
        write_text_file(out.display().to_string(), "\u{feff}甲=A\r\n".to_string()).unwrap();
        assert_eq!(fs::read(&out).unwrap(), "\u{feff}甲=A\r\n".as_bytes());
    }

    #[test]
    fn write_text_file_bao_loi_io_khi_path_la_folder() {
        let dir = tempfile::tempdir().unwrap();
        let err = write_text_file(dir.path().display().to_string(), "x".to_string()).unwrap_err();
        assert_eq!(err.kind, "io");
    }
}
