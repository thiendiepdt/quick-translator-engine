//! Rà ngược bản dịch đã chốt theo một mục glossary (tab Glossary → "Rà chương cũ"): liệt kê đoạn raw có khoá mà
//! bản dịch thiếu cách dịch hiện tại, rồi thay cụm sai trong đúng các đoạn đó. Logic ở `qt_ai_core::backfill`.

use crate::error::{blocking, CmdResult, CommandError};
use crate::AppState;
use qt_ai_core::backfill::{backfill_glossary, scan_glossary_usage};
use qt_ai_core::story_fs::story_paths;
use serde::Serialize;
use std::path::Path;
use tauri::{AppHandle, Manager, Runtime};

/// Số đoạn mẫu trả về cho hộp thoại — đủ để nhận ra cụm sai, không đổ cả nghìn dòng qua IPC.
const SHOWN_HITS: usize = 50;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageHitView {
    pub chapter: String,
    pub paragraph: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageScanView {
    pub total: usize,
    pub chapters: usize,
    pub skipped: usize,
    pub hits: Vec<UsageHitView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackfillView {
    pub chapters: usize,
    pub paragraphs: usize,
    pub occurrences: usize,
    pub skipped: usize,
}

pub fn scan_inner(root: &Path, key: &str, target: &str) -> CmdResult<UsageScanView> {
    let scan = scan_glossary_usage(&story_paths(root), key, target)?;
    let mut chapters: Vec<&str> = scan.hits.iter().map(|hit| hit.chapter.as_str()).collect();
    chapters.dedup();
    Ok(UsageScanView {
        total: scan.hits.len(),
        chapters: chapters.len(),
        skipped: scan.skipped,
        hits: scan
            .hits
            .iter()
            .take(SHOWN_HITS)
            .map(|hit| UsageHitView { chapter: hit.chapter.clone(), paragraph: hit.paragraph, text: hit.text.clone() })
            .collect(),
    })
}

pub fn backfill_inner(root: &Path, key: &str, from: &str, to: &str, apply: bool) -> CmdResult<BackfillView> {
    let done = backfill_glossary(&story_paths(root), key, from, to, apply)?;
    Ok(BackfillView { chapters: done.chapters, paragraphs: done.paragraphs, occurrences: done.occurrences, skipped: done.skipped })
}

/// Đoạn đã dịch có `key` trong raw mà bản dịch không chứa `target` — đọc mọi chương nên chạy ngoài luồng IPC.
#[tauri::command]
pub async fn glossary_scan(root: String, key: String, target: String) -> CmdResult<UsageScanView> {
    blocking(move || scan_inner(Path::new(&root), &key, &target)).await
}

/// Thay `from` → `to` trong các đoạn có `key` ở raw; `apply = false` chỉ đếm. Phiên dịch đang chạy thì từ chối
/// ghi: phiên đang ghi out/ của chính truyện này.
#[tauri::command]
pub async fn glossary_backfill<R: Runtime>(
    app: AppHandle<R>,
    root: String,
    key: String,
    from: String,
    to: String,
    apply: bool,
) -> CmdResult<BackfillView> {
    blocking(move || {
        if apply && app.state::<AppState>().sessions.lock().unwrap().is_running(&root) {
            return Err(CommandError::new(
                "session_locked",
                "Truyện này đang có phiên dịch chạy — bấm Dừng trước khi sửa chương cũ.",
            ));
        }
        backfill_inner(Path::new(&root), &key, &from, &to, apply)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn quet_roi_thay_trong_chuong_cu() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("raw")).unwrap();
        fs::create_dir_all(dir.path().join("out")).unwrap();
        fs::write(dir.path().join("raw/0001.txt"), "陆云停来了。\n\n他笑了。").unwrap();
        fs::write(dir.path().join("out/0001.txt"), "Lục Đình Vân tới.\n\nHắn cười.\n").unwrap();
        let scan = scan_inner(dir.path(), "陆云停", "Lục Vân Đình").unwrap();
        assert_eq!((scan.total, scan.chapters, scan.hits[0].text.as_str()), (1, 1, "Lục Đình Vân tới."));
        let preview = backfill_inner(dir.path(), "陆云停", "Lục Đình Vân", "Lục Vân Đình", false).unwrap();
        assert_eq!((preview.chapters, preview.occurrences), (1, 1));
        backfill_inner(dir.path(), "陆云停", "Lục Đình Vân", "Lục Vân Đình", true).unwrap();
        assert_eq!(scan_inner(dir.path(), "陆云停", "Lục Vân Đình").unwrap().total, 0);
    }
}
