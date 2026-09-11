//! Binary `qt-ai` đi kèm app. AGENTS.md chỉ ghi tên lệnh trần (`qt-ai`), còn folder chứa binary được
//! chèn vào PATH của agy lúc chạy phiên — nhờ vậy folder truyện init ở máy này mang sang máy khác
//! (hoặc app dời chỗ) vẫn gọi được CLI, không còn đường dẫn tuyệt đối chết trong file.
//! Bundle: Tauri đặt sidecar cạnh app exe với tên `qt-ai.exe` (dev: `qt-ai-<triple>.exe` hoặc target/).

use std::path::{Path, PathBuf};

fn candidates() -> Vec<PathBuf> {
    let exe_name = if cfg!(windows) { "qt-ai.exe" } else { "qt-ai" };
    let mut list = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            list.push(dir.join(exe_name));
            list.push(dir.join(format!("qt-ai-{}{}", target_triple(), if cfg!(windows) { ".exe" } else { "" })));
        }
    }
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    list.push(workspace.join("target").join("release").join(exe_name));
    list.push(workspace.join("target").join("debug").join(exe_name));
    list
}

fn target_triple() -> &'static str {
    if cfg!(all(windows, target_arch = "x86_64")) {
        "x86_64-pc-windows-msvc"
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "aarch64-apple-darwin"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "x86_64-apple-darwin"
    } else {
        "x86_64-unknown-linux-gnu"
    }
}

fn found() -> Option<PathBuf> {
    candidates().into_iter().find(|path| path.is_file())
}

/// Folder chứa binary qt-ai (để chèn PATH). None nếu chưa build sidecar.
pub fn qt_ai_dir() -> Option<PathBuf> {
    found().and_then(|path| path.parent().map(Path::to_path_buf))
}

/// Tên lệnh render vào AGENTS.md: tên file binary không kèm folder (thường `qt-ai`; dev có thể là
/// `qt-ai-<triple>`), chạy được nhờ `qt_ai_dir` nằm trong PATH của agy.
pub fn qt_ai_command() -> String {
    found()
        .and_then(|path| path.file_stem().map(|stem| stem.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "qt-ai".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lenh_la_ten_tran_khong_co_folder_hay_ngoac_kep() {
        let command = qt_ai_command();
        assert!(command.starts_with("qt-ai"), "{command}");
        assert!(!command.contains('"') && !command.contains('\\') && !command.contains('/'), "{command}");
    }

    #[test]
    fn folder_neu_co_thi_chua_binary_ten_khop_lenh() {
        if let Some(dir) = qt_ai_dir() {
            let stem = qt_ai_command();
            assert!(dir.join(&stem).is_file() || dir.join(format!("{stem}.exe")).is_file());
        }
    }
}
