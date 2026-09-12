use crate::error::{CoreError, Result};
use crate::story::natural_chapter_compare;
use crate::story_fs::{load_state, resolve_root, save_state, story_paths, work_file, WORK_KINDS};
use std::fs;
use std::path::Path;

/// Kết quả xoá chương.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DeleteOutcome {
    /// Chương đã gỡ khỏi state (raw/ + work/ đã xoá), theo thứ tự tự nhiên.
    pub removed: Vec<String>,
    /// Tập con của `removed` có `out/<id>.txt` — bản dịch giữ nguyên trên đĩa cho người dùng.
    pub kept_outputs: Vec<String>,
}

/// Xoá hẳn chương khỏi truyện: gỡ khỏi state.json, xoá raw/<id>.txt và work/ của nó. **Không đụng
/// out/<id>.txt** — bản dịch đã xong là công sức, người dùng tự dọn nếu muốn. Kiểm đủ id trước, thiếu
/// một id là không xoá gì. Caller (GUI/CLI) tự chắc không có phiên đang chạy trên truyện này.
pub fn run_delete(root: &Path, ids: &[String]) -> Result<DeleteOutcome> {
    let paths = story_paths(&resolve_root(root));
    let mut state = load_state(&paths)?;
    if ids.is_empty() {
        return Err(CoreError::InvalidState("delete cần ít nhất một mã chương.".to_string()));
    }
    for id in ids {
        if !state.chapters.contains_key(id) {
            return Err(CoreError::StoryNotFound(format!("Không có chương {id} trong state.json.")));
        }
    }
    let mut targets: Vec<String> = ids.to_vec();
    targets.sort_by(|a, b| natural_chapter_compare(a, b));
    targets.dedup();
    let mut outcome = DeleteOutcome::default();
    for id in targets {
        state.chapters.shift_remove(&id);
        let raw = paths.raw_dir.join(format!("{id}.txt"));
        if let Err(source) = fs::remove_file(&raw) {
            if source.kind() != std::io::ErrorKind::NotFound {
                return Err(CoreError::Io { path: raw, source });
            }
        }
        for kind in WORK_KINDS {
            let _ = fs::remove_file(work_file(&paths, &id, kind));
        }
        if paths.out_dir.join(format!("{id}.txt")).is_file() {
            outcome.kept_outputs.push(id.clone());
        }
        outcome.removed.push(id);
    }
    save_state(&paths, &state)?;
    Ok(outcome)
}
