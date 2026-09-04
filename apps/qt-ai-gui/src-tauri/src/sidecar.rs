//! Đường dẫn binary `qt-ai` để render vào AGENTS.md cho agent gọi.
//! Bundle: Tauri đặt sidecar cạnh app exe với tên `qt-ai.exe`. Dev: dùng bản build trong target/ của workspace.

use std::path::PathBuf;

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

/// Lệnh chạy qt-ai, bọc ngoặc kép (đường dẫn Windows hay có khoảng trắng). Không tìm thấy → "qt-ai" trần.
pub fn qt_ai_command() -> String {
    candidates()
        .into_iter()
        .find(|path| path.is_file())
        .and_then(|path| path.canonicalize().ok())
        .map(|path| format!("\"{}\"", path.display().to_string().trim_start_matches(r"\\?\")))
        .unwrap_or_else(|| "\"qt-ai\"".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lenh_boc_ngoac_kep_va_ket_thuc_bang_qt_ai() {
        let command = qt_ai_command();
        assert!(command.starts_with('"') && command.ends_with('"'));
        assert!(command.to_lowercase().contains("qt-ai"));
    }
}
