use crate::error::{CoreError, Result};
use crate::story_fs::{
    load_state, now_ms, resolve_root, save_state, story_paths, work_file, ChapterState, ChapterStatus, WORK_KINDS,
};
use std::fs;
use std::path::Path;

pub fn run_skip(root: &Path, id: &str, reason: &str) -> Result<()> {
    if reason.trim().is_empty() {
        return Err(CoreError::InvalidState("skip cần --reason <lý do> không rỗng.".to_string()));
    }
    let paths = story_paths(&resolve_root(root));
    let mut state = load_state(&paths)?;
    let chapter = state
        .chapters
        .get(id)
        .cloned()
        .ok_or_else(|| CoreError::StoryNotFound(format!("Không có chương {id} trong state.json.")))?;
    if chapter.status == ChapterStatus::Done {
        return Err(CoreError::InvalidState(format!("Chương {id} đã done, không skip được.")));
    }
    state.chapters.insert(
        id.to_string(),
        ChapterState {
            status: ChapterStatus::Skipped,
            reason: Some(reason.trim().to_string()),
            updated_at: now_ms(),
            ..chapter
        },
    );
    save_state(&paths, &state)?;
    for kind in WORK_KINDS {
        let _ = fs::remove_file(work_file(&paths, id, kind));
    }
    Ok(())
}
