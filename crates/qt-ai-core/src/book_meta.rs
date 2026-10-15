//! Thông tin sách cho export EPUB: tác giả và ảnh bìa lấy từ trang nguồn, lưu ở `book.json` + `cover.jpg`
//! trong thư mục truyện — file riêng như cast.json, để bản app cũ (normalize story.json bỏ field lạ) không
//! làm mất, và lần export sau không phải tải lại.

use crate::error::{CoreError, Result};
use crate::source_http::sniff_image_mime;
use crate::story_fs::{write_atomic, write_atomic_bytes, StoryPaths};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const BOOK_JSON: &str = "book.json";
pub const COVER_FILE: &str = "cover.jpg";

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BookMeta {
    /// Tác giả hiển thị (Hán-Việt hoặc người dùng sửa).
    pub author: String,
    /// Bút danh gốc chữ Hán lấy từ nguồn — trang tiêu đề ghi kèm trong ngoặc để tra được.
    pub author_original: String,
    /// Nguồn đã lấy (`qq`, `fanqie`…); rỗng = chưa lấy hoặc link không hỗ trợ.
    pub source: String,
    /// Link ảnh bìa đã tải thành công (để biết `cover.jpg` từ đâu ra).
    pub cover_url: String,
    pub fetched_at: u64,
}

pub fn book_json_path(paths: &StoryPaths) -> PathBuf {
    paths.root.join(BOOK_JSON)
}

pub fn cover_path(paths: &StoryPaths) -> PathBuf {
    paths.root.join(COVER_FILE)
}

/// Không có file hoặc file hỏng → mặc định rỗng.
pub fn load_book_meta(paths: &StoryPaths) -> BookMeta {
    fs::read_to_string(book_json_path(paths)).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
}

pub fn save_book_meta(paths: &StoryPaths, meta: &BookMeta) -> Result<()> {
    let json = serde_json::to_string_pretty(meta).expect("BookMeta luôn serialize được");
    write_atomic(&book_json_path(paths), &format!("{json}\n"))
}

pub fn read_cover(paths: &StoryPaths) -> Option<Vec<u8>> {
    fs::read(cover_path(paths)).ok().filter(|bytes| sniff_image_mime(bytes) == Some("image/jpeg"))
}

/// Chỉ nhận JPEG (frontend đã vẽ lại qua canvas): app đọc và Kindle đời cũ không đọc WebP.
pub fn save_cover(paths: &StoryPaths, jpeg: &[u8]) -> Result<()> {
    if sniff_image_mime(jpeg) != Some("image/jpeg") {
        return Err(CoreError::InvalidState("Ảnh bìa phải là JPEG.".to_string()));
    }
    write_atomic_bytes(&cover_path(paths), jpeg)
}

pub fn remove_cover(paths: &StoryPaths) -> Result<()> {
    let path = cover_path(paths);
    match fs::remove_file(&path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(CoreError::io(&path)(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::story_fs::story_paths;

    #[test]
    fn luu_doc_book_json_va_cover() {
        let dir = tempfile::tempdir().unwrap();
        let paths = story_paths(dir.path());
        assert_eq!(load_book_meta(&paths), BookMeta::default());
        let meta = BookMeta {
            author: "Phẫn Nộ Đích Ô Tặc".into(),
            author_original: "愤怒的乌贼".into(),
            source: "qq".into(),
            cover_url: "https://x/y.jpg".into(),
            fetched_at: 1,
        };
        save_book_meta(&paths, &meta).unwrap();
        assert_eq!(load_book_meta(&paths), meta);
        assert!(read_cover(&paths).is_none());
        assert!(save_cover(&paths, b"RIFF\x00\x00\x00\x00WEBPVP8 ").is_err());
        save_cover(&paths, &[0xFF, 0xD8, 0xFF, 0xE0, 1, 2]).unwrap();
        assert_eq!(read_cover(&paths).unwrap().len(), 6);
        remove_cover(&paths).unwrap();
        remove_cover(&paths).unwrap();
        assert!(read_cover(&paths).is_none());
    }

    #[test]
    fn book_json_hong_thi_ve_mac_dinh() {
        let dir = tempfile::tempdir().unwrap();
        let paths = story_paths(dir.path());
        std::fs::write(book_json_path(&paths), "{hỏng").unwrap();
        assert_eq!(load_book_meta(&paths), BookMeta::default());
    }
}
