//! Dùng chung cho các file test đi qua chốt chương / ghép prompt.

use std::sync::OnceLock;

/// Trỏ `QT_AI_BASE_DIR` vào một thư mục rỗng (một lần cho cả tiến trình test): không đọc cài đặt thật trong
/// %APPDATA% của máy chạy test — bù chữ chương ngắn, prompt / rule / glossary nền, bảng thành ngữ đã tải.
pub fn isolate_base() {
    static BASE: OnceLock<tempfile::TempDir> = OnceLock::new();
    BASE.get_or_init(|| {
        let dir = tempfile::Builder::new().prefix("qt-ai-base-").tempdir().unwrap();
        std::env::set_var(qt_ai_core::base::BASE_DIR_ENV, dir.path());
        dir
    });
}
