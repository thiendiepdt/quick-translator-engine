use crate::error::{CoreError, Result};
use crate::story_fs::{
    load_state, resolve_root, save_state, story_paths, work_file, ChapterState, ChapterStatus, WORK_KINDS,
};
use std::fs;
use std::path::Path;

/// Đưa chương error/skipped về hàng đợi dịch lại từ đầu (reviewRound 0, dọn work/).
pub fn run_retry(root: &Path, id: &str) -> Result<()> {
    let paths = story_paths(&resolve_root(root));
    let mut state = load_state(&paths)?;
    let chapter = state
        .chapters
        .get(id)
        .ok_or_else(|| CoreError::StoryNotFound(format!("Không có chương {id} trong state.json.")))?;
    match chapter.status {
        ChapterStatus::Done => {
            return Err(CoreError::InvalidState(format!(
                "Chương {id} đã done — muốn dịch lại thì xoá out/{id}.txt trước rồi tính."
            )))
        }
        ChapterStatus::Queued => {
            return Err(CoreError::InvalidState(format!("Chương {id} đang queued sẵn rồi.")))
        }
        ChapterStatus::Translating => {
            return Err(CoreError::InvalidState(format!(
                "Chương {id} đang translating — qt-ai next sẽ tự phát lại nó, không cần retry."
            )))
        }
        ChapterStatus::Error | ChapterStatus::Skipped => {}
    }
    state.chapters.insert(id.to_string(), ChapterState::fresh(ChapterStatus::Queued));
    save_state(&paths, &state)?;
    for kind in WORK_KINDS {
        let _ = fs::remove_file(work_file(&paths, id, kind));
    }
    Ok(())
}
