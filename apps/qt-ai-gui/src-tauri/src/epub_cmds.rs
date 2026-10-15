//! Export EPUB: tải thông tin sách từ trang nguồn (frontend bóc dữ liệu, vẽ bìa qua canvas), lưu tác giả +
//! ảnh bìa vào thư mục truyện, đóng gói EPUB.

use crate::error::{blocking, CmdResult, CommandError};
use crate::story_cmds::ExportOutcome;
use qt_ai_core::book_meta::{load_book_meta, read_cover, remove_cover, save_book_meta, save_cover, BookMeta};
use qt_ai_core::commands::export::{run_export_epub, ExportOptions};
use qt_ai_core::source_http::{fetch_image, fetch_text, SourceImage};
use qt_ai_core::story_fs::{resolve_root, story_paths};
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Trang / API thông tin sách — chỉ host trong danh sách của core (webview không gọi thẳng được: CORS, WAF).
#[tauri::command]
pub async fn source_fetch_text(url: String) -> CmdResult<String> {
    blocking(move || fetch_text(&url).map_err(|message| CommandError::new("source", message))).await
}

/// Ảnh bìa (thử lại lỗi tạm thời ở core); trả byte gốc để frontend vẽ lại thành JPEG.
#[tauri::command]
pub async fn source_fetch_image(url: String) -> CmdResult<SourceImage> {
    blocking(move || fetch_image(&url).map_err(|message| CommandError::new("source", message))).await
}

/// Bút danh chữ Hán → Hán-Việt viết hoa từng âm (`愤怒的乌贼` → `Phẫn Nộ Đích Ô Tặc`). Tra bảng trong bộ nhớ.
#[tauri::command]
pub fn han_viet_name(text: String) -> String {
    qt_ai_core::han_viet::han_viet_name(&text)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BookInfoView {
    pub meta: BookMeta,
    /// `cover.jpg` đã lưu (JPEG), không có thì `None`.
    pub cover: Option<Vec<u8>>,
}

#[tauri::command]
pub async fn book_info_get(root: String) -> CmdResult<BookInfoView> {
    blocking(move || {
        let paths = story_paths(&resolve_root(Path::new(&root)));
        Ok(BookInfoView { meta: load_book_meta(&paths), cover: read_cover(&paths) })
    })
    .await
}

/// Lưu tác giả / nguồn; `cover` = JPEG mới, `clear_cover` = xoá bìa đã lưu (link không còn hỗ trợ / tải lỗi).
#[tauri::command]
pub async fn book_info_save(root: String, meta: BookMeta, cover: Option<Vec<u8>>, clear_cover: bool) -> CmdResult<()> {
    blocking(move || {
        let paths = story_paths(&resolve_root(Path::new(&root)));
        if let Some(jpeg) = cover {
            save_cover(&paths, &jpeg)?;
        } else if clear_cover {
            remove_cover(&paths)?;
        }
        save_book_meta(&paths, &meta)?;
        Ok(())
    })
    .await
}

/// `cover`: JPEG đưa vào sách (bìa tải về hoặc bìa mặc định frontend vẽ), `None` = sách không có bìa.
#[tauri::command]
pub async fn export_epub(
    root: String,
    from: Option<String>,
    to: Option<String>,
    out: Option<String>,
    author: String,
    cover: Option<Vec<u8>>,
) -> CmdResult<ExportOutcome> {
    blocking(move || {
        let options = ExportOptions { from, to, out: out.map(PathBuf::from) };
        let result = run_export_epub(Path::new(&root), &options, &author, cover)?;
        Ok(ExportOutcome { out_path: result.out_path.display().to_string(), ids: result.ids, gaps: result.gaps })
    })
    .await
}
