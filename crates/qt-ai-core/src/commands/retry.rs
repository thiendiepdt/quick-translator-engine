use crate::error::{CoreError, Result};
use crate::story_fs::{
    load_state, resolve_root, save_state, story_paths, work_file, ChapterState, ChapterStatus, StoryPaths, WORK_KINDS,
};
use std::fs;
use std::path::{Path, PathBuf};

/// Bản sao lưu của `out/<id>.txt` khi dịch lại chương đã done (đè bak cũ, không tích luỹ).
pub fn retry_backup_path(paths: &StoryPaths, id: &str) -> PathBuf {
    paths.out_dir.join(format!("{id}.txt.bak"))
}

/// Đưa chương về hàng đợi dịch lại từ đầu (reviewRound 0, dọn work/).
/// - error/skipped: ca "cứu chương hỏng" thông thường.
/// - done: `out/<id>.txt` đổi tên thành `.txt.bak` để lỡ tay còn lấy lại; accept lần sau ghi bản mới.
/// - translating: chương kẹt sau khi phiên chết — caller phải tự chắc không có phiên nào đang chạy
///   trên truyện này (GUI chặn khi phiên đang chạy), vì runner có thể đang ghi work/ của chính chương đó.
/// - queued: từ chối, chương đã ở hàng đợi.
pub fn run_retry(root: &Path, id: &str) -> Result<()> {
    let paths = story_paths(&resolve_root(root));
    let mut state = load_state(&paths)?;
    let chapter = state
        .chapters
        .get(id)
        .ok_or_else(|| CoreError::StoryNotFound(format!("Không có chương {id} trong state.json.")))?;
    match chapter.status {
        ChapterStatus::Queued => {
            return Err(CoreError::InvalidState(format!("Chương {id} đang queued sẵn rồi.")))
        }
        ChapterStatus::Done => {
            let out = paths.out_dir.join(format!("{id}.txt"));
            if out.is_file() {
                let backup = retry_backup_path(&paths, id);
                fs::rename(&out, &backup).map_err(|source| CoreError::Io { path: out.clone(), source })?;
            }
        }
        ChapterStatus::Error | ChapterStatus::Skipped | ChapterStatus::Translating => {}
    }
    state.chapters.insert(id.to_string(), ChapterState::fresh(ChapterStatus::Queued));
    save_state(&paths, &state)?;
    for kind in WORK_KINDS {
        let _ = fs::remove_file(work_file(&paths, id, kind));
    }
    Ok(())
}
