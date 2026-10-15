use crate::error::Result;
use crate::story::natural_chapter_compare;
use crate::story_fs::{load_state, resolve_root, story_paths, ChapterStatus, StoryState};
use crate::vn_check::live_warnings;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Counts {
    pub total: usize,
    pub queued: usize,
    pub translating: usize,
    pub done: usize,
    pub error: usize,
    pub skipped: usize,
    pub with_warnings: usize,
}

/// "Kèm cảnh báo" theo cảnh báo đã lưu trong state (bỏ dòng `[VN]` cũ); muốn tính cả kiểm tra VN sống thì
/// dùng `count_chapters_live`.
pub fn count_chapters(state: &StoryState) -> Counts {
    count_chapters_live(state, &HashMap::new())
}

/// Như `count_chapters` nhưng "kèm cảnh báo" tính theo `vn_check::live_warnings` — `vn_open` từ
/// `vn_check::open_by_chapter`.
pub fn count_chapters_live(state: &StoryState, vn_open: &HashMap<String, usize>) -> Counts {
    let mut counts = Counts { total: state.chapters.len(), ..Counts::default() };
    for (id, chapter) in &state.chapters {
        match chapter.status {
            ChapterStatus::Queued => counts.queued += 1,
            ChapterStatus::Translating => counts.translating += 1,
            ChapterStatus::Done => counts.done += 1,
            ChapterStatus::Error => counts.error += 1,
            ChapterStatus::Skipped => counts.skipped += 1,
        }
        let stored = chapter.warnings.as_deref().unwrap_or_default();
        if chapter.status == ChapterStatus::Done && !live_warnings(stored, vn_open.get(id).copied().unwrap_or(0)).is_empty() {
            counts.with_warnings += 1;
        }
    }
    counts
}

pub fn run_status(root: &Path) -> Result<String> {
    let state = load_state(&story_paths(&resolve_root(root)))?;
    let counts = count_chapters(&state);
    let mut ids: Vec<&String> = state.chapters.keys().collect();
    ids.sort_by(|a, b| natural_chapter_compare(a, b));
    let mut flagged: Vec<String> = Vec::new();
    for id in ids {
        let chapter = &state.chapters[id];
        match chapter.status {
            ChapterStatus::Error | ChapterStatus::Skipped => flagged.push(
                format!("  {id} [{}] {}", chapter.status.as_str(), chapter.reason.as_deref().unwrap_or(""))
                    .trim_end()
                    .to_string(),
            ),
            ChapterStatus::Translating => {
                flagged.push(format!("  {id} [translating] — đang dở, check/accept/skip trước"))
            }
            ChapterStatus::Done => {
                let warnings = live_warnings(chapter.warnings.as_deref().unwrap_or_default(), 0);
                if !warnings.is_empty() {
                    flagged.push(format!("  {id} [done, {} cảnh báo] {}", warnings.len(), warnings[0]));
                }
            }
            ChapterStatus::Queued => {}
        }
    }
    let mut lines = vec![format!(
        "Tổng {} chương — done: {}, queued: {}, translating: {}, error: {}, skipped: {}{}",
        counts.total,
        counts.done,
        counts.queued,
        counts.translating,
        counts.error,
        counts.skipped,
        if counts.with_warnings > 0 {
            format!(", done kèm cảnh báo: {}", counts.with_warnings)
        } else {
            String::new()
        }
    )];
    if !flagged.is_empty() {
        lines.push("Cần chú ý:".to_string());
        lines.extend(flagged);
    }
    lines.push(format!(
        "Giới hạn phiên: dịch tối đa {} chương/phiên rồi nghỉ.",
        state.settings.chapters_per_session
    ));
    Ok(lines.join("\n"))
}
