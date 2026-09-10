use crate::error::Result;
use crate::story::StoryConfig;
use crate::story_fs::{
    ensure_story_dirs, list_raw_chapter_ids, load_state, resolve_root, save_state, save_story_config,
    story_paths, work_file, ChapterState, ChapterStatus, StoryState, WORK_KINDS,
};
use crate::templates::copy_templates;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Dựng khung folder truyện. Idempotent: giữ state/story sẵn có, thêm chương raw mới thành queued và
/// gỡ chương có file raw đã mất (đổi tên/xoá) — trừ chương done vì bản dịch trong out/ vẫn dùng được.
/// `qt_ai_command` là lệnh chạy CLI để render vào AGENTS.md (đường dẫn binary, có ngoặc kép nếu cần).
pub fn run_init(root: &Path, qt_ai_command: &str) -> Result<String> {
    let paths = story_paths(&resolve_root(root));
    ensure_story_dirs(&paths)?;
    if !paths.story_json.exists() {
        save_story_config(&paths, &StoryConfig::empty())?;
    }
    let mut state = if paths.state_json.exists() { load_state(&paths)? } else { StoryState::new() };
    let raw_ids: BTreeSet<String> = list_raw_chapter_ids(&paths)?.into_iter().collect();
    let gone: Vec<String> = state
        .chapters
        .iter()
        .filter(|(id, chapter)| !raw_ids.contains(*id) && chapter.status != ChapterStatus::Done)
        .map(|(id, _)| id.clone())
        .collect();
    for id in &gone {
        state.chapters.shift_remove(id);
        for kind in WORK_KINDS {
            let _ = fs::remove_file(work_file(&paths, id, kind));
        }
    }
    let mut added = 0;
    for id in raw_ids {
        if state.chapters.contains_key(&id) {
            continue;
        }
        state.chapters.insert(id, ChapterState::fresh(ChapterStatus::Queued));
        added += 1;
    }
    save_state(&paths, &state)?;
    copy_templates(&paths.root, qt_ai_command)?;
    let removed = if gone.is_empty() { String::new() } else { format!(", {} gỡ vì raw đã mất", gone.len()) };
    Ok(format!(
        "Đã init {}: {} chương ({} mới thêm vào hàng đợi{}).",
        paths.root.display(),
        state.chapters.len(),
        added,
        removed
    ))
}
