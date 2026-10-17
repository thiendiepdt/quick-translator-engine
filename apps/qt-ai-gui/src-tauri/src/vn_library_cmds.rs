//! Màn chọn truyện → "Quét nhạy cảm VN": quét kiểm tra VN mọi truyện trong thư viện, chạy nền, từng truyện một với
//! ít luồng để máy không giật khi đang dịch (logic ở `qt_ai_core::vn_check::scan_library`). Quét tăng dần như tab
//! Kiểm tra VN (chỉ chương mới/đổi), nên lần sau gần như tức thì. Đo trên thư viện 62 truyện, 40.749 chương
//! (749 MB): lần đầu 33 giây (30 giây đọc đĩa), đĩa đã cache 7,7 giây với một luồng.

use crate::error::{blocking, CmdResult, CommandError};
use crate::story_cmds::RecentSummary;
use crate::vn_cmds::lexicon;
use crate::AppState;
use qt_ai_core::vn_check::{library_story_roots, library_threads, scan_library};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager, Runtime};

pub const VN_LIBRARY_EVENT: &str = "vn-library-progress";

/// Một lượt quét thư viện tại một thời điểm; nút Dừng bật cờ huỷ, lượt quét dừng sau truyện đang quét.
static RUNNING: AtomicBool = AtomicBool::new(false);
static CANCEL: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VnLibraryProgress {
    /// Số truyện đã quét xong (kể cả lỗi).
    pub done: usize,
    pub total: usize,
    /// Truyện vừa quét xong.
    pub root: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VnLibraryScan {
    pub scanned: usize,
    pub total: usize,
    pub failed: Vec<String>,
    pub cancelled: bool,
    /// Danh sách thư viện sau khi quét (huy hiệu ⚠ VN mới).
    pub stories: Vec<RecentSummary>,
}

#[tauri::command]
pub async fn vn_scan_library<R: Runtime>(app: AppHandle<R>) -> CmdResult<VnLibraryScan> {
    blocking(move || {
        if RUNNING.swap(true, Ordering::SeqCst) {
            return Err(CommandError::new("invalid_state", "Đang quét nhạy cảm VN cả thư viện."));
        }
        CANCEL.store(false, Ordering::SeqCst);
        let result = (|| {
            let state = app.state::<AppState>();
            let library = state
                .config
                .lock()
                .unwrap()
                .library_root
                .clone()
                .ok_or_else(|| CommandError::new("invalid_state", "Chưa chọn thư viện (folder cha chứa truyện)."))?;
            let library = PathBuf::from(library);
            let roots = library_story_roots(&library);
            let total = roots.len();
            let scan = scan_library(&roots, &lexicon(), library_threads(), &CANCEL, &mut |done, root| {
                let _ = app.emit(VN_LIBRARY_EVENT, &VnLibraryProgress { done, total, root: root.display().to_string() });
            });
            let stories = crate::library_cmds::list_library(Some(&library), &state.summaries);
            Ok(VnLibraryScan { scanned: scan.scanned, total, failed: scan.failed, cancelled: scan.cancelled, stories })
        })();
        RUNNING.store(false, Ordering::SeqCst);
        result
    })
    .await
}

/// Dừng lượt quét thư viện sau truyện đang quét (không cắt giữa chừng một truyện).
#[tauri::command]
pub fn vn_scan_library_cancel() {
    CANCEL.store(true, Ordering::SeqCst);
}
