//! Đồng bộ tiến độ theo bản dịch: folder truyện đi qua Google Drive có lúc về đủ `out/` nhưng `state.json` là
//! bản cũ hơn → bấm Dịch sẽ dịch lại hàng trăm chương đã có bản dịch. Lệnh này đánh dấu done theo `out/`.

use crate::commands::retry::retry_backup_path;
use crate::error::{CoreError, Result};
use crate::story::natural_chapter_compare;
use crate::story_fs::{
    done_from_out, has_translation, load_state, resolve_root, save_state, story_paths, work_file, ChapterState,
    ChapterStatus, StoryPaths, StoryState, WORK_KINDS,
};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

/// Độ lệch giữa state.json và out/, theo thứ tự chương tự nhiên.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressDrift {
    /// Chương chưa done trong state nhưng out/<id>.txt đã có chữ thật, ghi SAU lần cuối chương đổi trạng thái.
    pub untracked: Vec<String>,
    /// Chương ghi done nhưng không thấy out/<id>.txt có chữ (Drive có thể chưa sync xong).
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncProgressOutcome {
    pub marked_done: Vec<String>,
    pub missing: Vec<String>,
}

/// mtime (ms) của file; đọc lỗi thì None.
fn modified_ms(path: &Path) -> Option<u64> {
    let modified = fs::metadata(path).and_then(|meta| meta.modified()).ok()?;
    modified.duration_since(UNIX_EPOCH).ok().map(|d| d.as_millis() as u64)
}

/// Chương chưa done mà out/<id>.txt có chữ, ghi sau lần cuối chương đổi trạng thái — không đọc out/ của chương done.
fn untracked_of(paths: &StoryPaths, state: &StoryState) -> Vec<String> {
    // Chỉ xét chương có trong state — chương raw mới/mất do init/quét lại lo.
    let mut untracked: Vec<String> = state
        .chapters
        .iter()
        .filter(|(_, chapter)| chapter.status != ChapterStatus::Done)
        .filter(|(id, chapter)| {
            let out = paths.out_dir.join(format!("{id}.txt"));
            // Bản dịch cũ hơn lần chương về hàng đợi là cố ý dịch lại (reset giữ nguyên out/) — không phải lệch.
            // Lệch thật: máy kia dịch SAU khi state.json bản cũ này được ghi, nên out/ mới hơn updatedAt.
            has_translation(&out) && modified_ms(&out).is_none_or(|ms| ms > chapter.updated_at)
        })
        .map(|(id, _)| id.clone())
        .collect();
    untracked.sort_by(|a, b| natural_chapter_compare(a, b));
    untracked
}

fn drift_of(paths: &StoryPaths, state: &StoryState) -> ProgressDrift {
    let mut missing: Vec<String> = state
        .chapters
        .iter()
        .filter(|(id, chapter)| {
            chapter.status == ChapterStatus::Done && !has_translation(&paths.out_dir.join(format!("{id}.txt")))
        })
        .map(|(id, _)| id.clone())
        .collect();
    missing.sort_by(|a, b| natural_chapter_compare(a, b));
    ProgressDrift { untracked: untracked_of(paths, state), missing }
}

/// Lối thoát khi báo lệch nhầm: đưa các chương `untracked` về hàng đợi dịch lại như `retry` — out/<id>.txt cất
/// sang `.txt.bak` (đè bak cũ), state mới (queued, reviewRound 0, updatedAt bây giờ), dọn work/. Trả id đã đưa về.
pub fn requeue_untracked(root: &Path) -> Result<Vec<String>> {
    let paths = story_paths(&resolve_root(root));
    let mut state = load_state(&paths)?;
    let untracked = untracked_of(&paths, &state);
    for id in &untracked {
        let out = paths.out_dir.join(format!("{id}.txt"));
        let backup = retry_backup_path(&paths, id);
        fs::rename(&out, &backup).map_err(CoreError::io(&out))?;
        state.chapters.insert(id.clone(), ChapterState::fresh(ChapterStatus::Queued));
        for kind in WORK_KINDS {
            let _ = fs::remove_file(work_file(&paths, id, kind));
        }
    }
    if !untracked.is_empty() {
        save_state(&paths, &state)?;
    }
    Ok(untracked)
}

/// Chỉ đọc: chương nào lệch giữa state.json và out/.
pub fn progress_drift(root: &Path) -> Result<ProgressDrift> {
    let paths = story_paths(&resolve_root(root));
    let state = load_state(&paths)?;
    Ok(drift_of(&paths, &state))
}

/// Đánh dấu done mọi chương `untracked` (updatedAt = mtime bản dịch, reviewRound 0, bỏ lý do error/skip);
/// chương `missing` giữ nguyên vì Drive có thể còn đang sync. Không có gì để đánh dấu thì không ghi state.
pub fn run_sync_progress(root: &Path) -> Result<SyncProgressOutcome> {
    let paths = story_paths(&resolve_root(root));
    let mut state = load_state(&paths)?;
    let drift = drift_of(&paths, &state);
    for id in &drift.untracked {
        state.chapters.insert(id.clone(), done_from_out(&paths.out_dir.join(format!("{id}.txt"))));
    }
    if !drift.untracked.is_empty() {
        save_state(&paths, &state)?;
    }
    Ok(SyncProgressOutcome { marked_done: drift.untracked, missing: drift.missing })
}
