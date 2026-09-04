use crate::error::Result;
use crate::story::natural_chapter_compare;
use crate::story_fs::{load_state, resolve_root, story_paths, ChapterStatus, StoryState};
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

pub fn count_chapters(state: &StoryState) -> Counts {
    let mut counts = Counts { total: state.chapters.len(), ..Counts::default() };
    for chapter in state.chapters.values() {
        match chapter.status {
            ChapterStatus::Queued => counts.queued += 1,
            ChapterStatus::Translating => counts.translating += 1,
            ChapterStatus::Done => counts.done += 1,
            ChapterStatus::Error => counts.error += 1,
            ChapterStatus::Skipped => counts.skipped += 1,
        }
        if chapter.status == ChapterStatus::Done && chapter.warnings.as_ref().is_some_and(|w| !w.is_empty()) {
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
                if let Some(warnings) = chapter.warnings.as_ref().filter(|w| !w.is_empty()) {
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
