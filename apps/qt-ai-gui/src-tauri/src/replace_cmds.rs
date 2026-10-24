//! Trang "Tìm & thay": tìm trong bản dịch đã chốt và hồ sơ của truyện, thay các chỗ đang chọn, hoàn tác lần thay
//! gần nhất.
//! Logic ở `qt_ai_core::replace`.

use crate::error::{blocking, CmdResult, CommandError};
use crate::AppState;
use qt_ai_core::replace::{
    find_in_story, replace_in_story_and_profile, undo_info, undo_last_replace, FindQuery, FindResult, HitRef,
    ProfileRef, ReplaceOutcome, UndoInfo, UndoOutcome,
};
use qt_ai_core::story_fs::story_paths;
use std::path::Path;
use tauri::{AppHandle, Manager, Runtime};

/// Số chỗ khớp tối đa gửi qua IPC — tổng và số theo chương vẫn đủ; nút Thay xử lý mọi chỗ ở phía Rust.
const SHOWN_HITS: usize = 500;

pub(crate) fn refuse_if_running<R: Runtime>(app: &AppHandle<R>, root: &str) -> CmdResult<()> {
    if app.state::<AppState>().sessions.lock().unwrap().is_running(root) {
        return Err(CommandError::new(
            "session_locked",
            "Truyện này đang có phiên dịch chạy — bấm Dừng trước khi sửa chương đã dịch.",
        ));
    }
    Ok(())
}

pub fn find_inner(root: &Path, query: &FindQuery) -> CmdResult<FindResult> {
    Ok(find_in_story(&story_paths(root), query, SHOWN_HITS)?)
}

pub fn apply_inner(
    root: &Path,
    query: &FindQuery,
    excluded_chapters: &[String],
    excluded_hits: &[HitRef],
    excluded_profile: &[ProfileRef],
) -> CmdResult<ReplaceOutcome> {
    Ok(replace_in_story_and_profile(&story_paths(root), query, excluded_chapters, excluded_hits, excluded_profile)?)
}

/// Đọc mọi chương đã dịch — chạy ngoài luồng IPC.
#[tauri::command]
pub async fn replace_find(root: String, query: FindQuery) -> CmdResult<FindResult> {
    blocking(move || find_inner(Path::new(&root), &query)).await
}

/// Thay mọi chỗ khớp trong chương và hồ sơ trừ chương / chỗ bỏ chọn; từ chối khi truyện đang dịch.
#[tauri::command]
pub async fn replace_apply<R: Runtime>(
    app: AppHandle<R>,
    root: String,
    query: FindQuery,
    excluded_chapters: Vec<String>,
    excluded_hits: Vec<HitRef>,
    excluded_profile: Vec<ProfileRef>,
) -> CmdResult<ReplaceOutcome> {
    blocking(move || {
        refuse_if_running(&app, &root)?;
        apply_inner(Path::new(&root), &query, &excluded_chapters, &excluded_hits, &excluded_profile)
    })
    .await
}

/// Lần thay gần nhất còn hoàn tác được (file lưu bản gốc có thể vài MB nên đọc ngoài luồng IPC).
#[tauri::command]
pub async fn replace_undo_info(root: String) -> CmdResult<Option<UndoInfo>> {
    blocking(move || Ok(undo_info(&story_paths(Path::new(&root))))).await
}

#[tauri::command]
pub async fn replace_undo<R: Runtime>(app: AppHandle<R>, root: String) -> CmdResult<UndoOutcome> {
    blocking(move || {
        refuse_if_running(&app, &root)?;
        Ok(undo_last_replace(&story_paths(Path::new(&root)))?)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn tim_roi_thay() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("raw")).unwrap();
        fs::create_dir_all(dir.path().join("out")).unwrap();
        fs::write(dir.path().join("raw/0001.txt"), "人族。").unwrap();
        fs::write(dir.path().join("out/0001.txt"), "Người tộc.\n").unwrap();
        let query = FindQuery { find: "Người tộc".into(), replace: "Nhân tộc".into(), whole_word: true, ..FindQuery::default() };
        assert_eq!(find_inner(dir.path(), &query).unwrap().total, 1);
        assert_eq!(apply_inner(dir.path(), &query, &[], &[], &[]).unwrap().occurrences, 1);
        assert_eq!(fs::read_to_string(dir.path().join("out/0001.txt")).unwrap(), "Nhân tộc.\n");
    }
}
