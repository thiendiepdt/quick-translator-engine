//! Chuyển phồn thể → giản thể: raw của truyện (nút ở trang Dịch) và công cụ chuyển riêng không gắn truyện (cụm chữ,
//! một file, cả folder). Logic ở `qt_ai_core::zh_convert` / `qt_ai_core::zh_story`.

use crate::error::{blocking, CmdResult, CommandError};
use crate::replace_cmds::refuse_if_running;
use qt_ai_core::story_fs::story_paths;
use qt_ai_core::zh_convert::{convert_files, to_simplified, ConvertJob, ConvertReport, OutputMode};
use qt_ai_core::zh_story::{
    convert_story, story_zh_status, undo_story_convert, ZhStoryOutcome, ZhStoryStatus, ZhUndoOutcome, STATUS_SAMPLE,
};
use std::path::Path;
use tauri::{AppHandle, Runtime};

/// `full = false`: chỉ đọc mấy chương đầu (thanh nhắc lúc mở truyện); true: cả truyện (hộp chuyển).
#[tauri::command]
pub async fn zh_story_status(root: String, full: bool) -> CmdResult<ZhStoryStatus> {
    blocking(move || Ok(story_zh_status(&story_paths(Path::new(&root)), (!full).then_some(STATUS_SAMPLE))?)).await
}

/// Raw đổi thì cảnh báo nhạy cảm VN (quét cả raw) phải quét lại.
fn rescan_vn(root: &str) {
    crate::vn_cmds::refresh_quietly(Path::new(root), &crate::vn_cmds::lexicon());
}

#[tauri::command]
pub async fn zh_story_convert<R: Runtime>(app: AppHandle<R>, root: String) -> CmdResult<ZhStoryOutcome> {
    blocking(move || {
        refuse_if_running(&app, &root)?;
        let outcome = convert_story(&story_paths(Path::new(&root)))?;
        rescan_vn(&root);
        Ok(outcome)
    })
    .await
}

#[tauri::command]
pub async fn zh_story_undo<R: Runtime>(app: AppHandle<R>, root: String) -> CmdResult<ZhUndoOutcome> {
    blocking(move || {
        refuse_if_running(&app, &root)?;
        let outcome = undo_story_convert(&story_paths(Path::new(&root)))?;
        rescan_vn(&root);
        Ok(outcome)
    })
    .await
}

/// Ô chuyển nhanh: người dùng có thể dán cả chương nên vẫn chạy ngoài luồng IPC.
#[tauri::command]
pub async fn zh_convert_text(text: String) -> CmdResult<String> {
    blocking(move || Ok(to_simplified(&text))).await
}

pub fn convert_files_inner(job: &ConvertJob) -> CmdResult<ConvertReport> {
    if job.inputs.is_empty() {
        return Err(CommandError::new("invalid_input", "Chưa chọn file hoặc folder để chuyển."));
    }
    match &job.mode {
        OutputMode::Folder { dir } if dir.as_os_str().is_empty() => {
            Err(CommandError::new("invalid_input", "Chưa chọn folder để ghi bản giản thể."))
        }
        OutputMode::Suffix { suffix } if suffix.trim().is_empty() || suffix.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) => {
            Err(CommandError::new("invalid_input", "Hậu tố tên file trống hoặc có ký tự không dùng được trong tên file."))
        }
        _ => Ok(convert_files(job)),
    }
}

#[tauri::command]
pub async fn zh_convert_files(job: ConvertJob) -> CmdResult<ConvertReport> {
    blocking(move || convert_files_inner(&job)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn tu_choi_job_thieu_dau_vao_hoac_hau_to_sai() {
        let job = |inputs: Vec<PathBuf>, mode| ConvertJob { inputs, recursive: false, extensions: vec![], mode };
        assert!(convert_files_inner(&job(vec![], OutputMode::Replace)).is_err());
        assert!(convert_files_inner(&job(vec!["a.txt".into()], OutputMode::Suffix { suffix: " ".into() })).is_err());
        assert!(convert_files_inner(&job(vec!["a.txt".into()], OutputMode::Suffix { suffix: "a/b".into() })).is_err());
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.txt");
        fs::write(&file, "這是書").unwrap();
        let report = convert_files_inner(&job(vec![file.clone()], OutputMode::Suffix { suffix: "_gian".into() })).unwrap();
        assert_eq!(report.converted, 1);
        assert_eq!(fs::read_to_string(dir.path().join("a_gian.txt")).unwrap(), "这是书");
    }
}
