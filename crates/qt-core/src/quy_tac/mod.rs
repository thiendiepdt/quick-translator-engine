//! Quy Tắc: đảo trật tự và chọn nghĩa trên token sau khi convert.
//! Spec: docs/superpowers/specs/2026-11-06-quy-tac-design.md. Mặc định tắt.
#![allow(dead_code)] // Task 8 nối vào engine rồi gỡ dòng này.

mod numbers;
mod program;
mod tags;

/// Lỗi khi đọc file Quy Tắc; `line` là số dòng (tính từ 1), 0 khi không gắn dòng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuyTacError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for QuyTacError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line > 0 {
            write!(formatter, "QuyTac dòng {}: {}", self.line, self.message)
        } else {
            formatter.write_str(&self.message)
        }
    }
}

impl std::error::Error for QuyTacError {}
