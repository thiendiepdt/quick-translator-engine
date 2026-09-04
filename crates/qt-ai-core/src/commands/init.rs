use crate::error::Result;
use crate::story::StoryConfig;
use crate::story_fs::{
    ensure_story_dirs, list_raw_chapter_ids, load_state, resolve_root, save_state, save_story_config,
    story_paths, ChapterState, ChapterStatus, StoryState,
};
use crate::templates::copy_templates;
use std::path::Path;

/// Dựng khung folder truyện. Idempotent: giữ state/story sẵn có, chỉ thêm chương raw mới thành queued.
/// `qt_ai_command` là lệnh chạy CLI để render vào AGENTS.md (đường dẫn binary, có ngoặc kép nếu cần).
pub fn run_init(root: &Path, qt_ai_command: &str) -> Result<String> {
    let paths = story_paths(&resolve_root(root));
    ensure_story_dirs(&paths)?;
    if !paths.story_json.exists() {
        save_story_config(&paths, &StoryConfig::empty())?;
    }
    let mut state = if paths.state_json.exists() { load_state(&paths)? } else { StoryState::new() };
    let mut added = 0;
    for id in list_raw_chapter_ids(&paths)? {
        if state.chapters.contains_key(&id) {
            continue;
        }
        state.chapters.insert(id, ChapterState::fresh(ChapterStatus::Queued));
        added += 1;
    }
    save_state(&paths, &state)?;
    copy_templates(&paths.root, qt_ai_command)?;
    Ok(format!(
        "Đã init {}: {} chương ({} mới thêm vào hàng đợi).",
        paths.root.display(),
        state.chapters.len(),
        added
    ))
}
