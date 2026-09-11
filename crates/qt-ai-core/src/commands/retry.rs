use crate::error::{CoreError, Result};
use crate::story::natural_chapter_compare;
use crate::story_fs::{
    load_state, resolve_root, save_state, story_paths, work_file, ChapterState, ChapterStatus, StoryPaths, StoryState,
    WORK_KINDS,
};
use std::cmp::Ordering;
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
    if chapter.status == ChapterStatus::Queued {
        return Err(CoreError::InvalidState(format!("Chương {id} đang queued sẵn rồi.")));
    }
    requeue_chapter(&paths, &mut state, id)?;
    save_state(&paths, &state)
}

/// Kết quả dịch lại nhiều chương.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RetryRangeOutcome {
    /// Chương đã đưa về hàng đợi, theo thứ tự tự nhiên.
    pub retried: Vec<String>,
    /// Chương done bị đổi out/<id>.txt thành .bak (tập con của `retried`).
    pub backed_up: Vec<String>,
    /// Chương vốn đã queued, bỏ qua.
    pub already_queued: Vec<String>,
}

/// Dịch lại mọi chương trong [from..to] (None = từ đầu / tới cuối), cùng luật với `run_retry` từng
/// chương: done → giữ .bak, queued bỏ qua. Caller (GUI/CLI) tự chắc không có phiên đang chạy.
pub fn run_retry_range(root: &Path, from: Option<&str>, to: Option<&str>) -> Result<RetryRangeOutcome> {
    let paths = story_paths(&resolve_root(root));
    let mut state = load_state(&paths)?;
    let mut all: Vec<String> = state.chapters.keys().cloned().collect();
    all.sort_by(|a, b| natural_chapter_compare(a, b));
    for bound in [from, to].into_iter().flatten() {
        if !state.chapters.contains_key(bound) {
            return Err(CoreError::StoryNotFound(format!("Không có chương {bound} trong state.json.")));
        }
    }
    if let (Some(from), Some(to)) = (from, to) {
        if natural_chapter_compare(from, to) == Ordering::Greater {
            return Err(CoreError::InvalidState(format!("Khoảng ngược: --from {from} sau --to {to}.")));
        }
    }
    let in_range = all.into_iter().filter(|id| {
        from.is_none_or(|from| natural_chapter_compare(id, from) != Ordering::Less)
            && to.is_none_or(|to| natural_chapter_compare(id, to) != Ordering::Greater)
    });
    let mut outcome = RetryRangeOutcome::default();
    for id in in_range {
        let status = state.chapters[&id].status;
        if status == ChapterStatus::Queued {
            outcome.already_queued.push(id);
            continue;
        }
        if status == ChapterStatus::Done {
            outcome.backed_up.push(id.clone());
        }
        requeue_chapter(&paths, &mut state, &id)?;
        outcome.retried.push(id);
    }
    if !outcome.retried.is_empty() {
        save_state(&paths, &state)?;
    }
    Ok(outcome)
}

/// Một chương về hàng đợi trong `state` (chưa ghi đĩa): done → out/<id>.txt thành .bak; dọn work/.
fn requeue_chapter(paths: &StoryPaths, state: &mut StoryState, id: &str) -> Result<()> {
    if state.chapters[id].status == ChapterStatus::Done {
        let out = paths.out_dir.join(format!("{id}.txt"));
        if out.is_file() {
            let backup = retry_backup_path(paths, id);
            fs::rename(&out, &backup).map_err(|source| CoreError::Io { path: out.clone(), source })?;
        }
    }
    state.chapters.insert(id.to_string(), ChapterState::fresh(ChapterStatus::Queued));
    for kind in WORK_KINDS {
        let _ = fs::remove_file(work_file(paths, id, kind));
    }
    Ok(())
}
