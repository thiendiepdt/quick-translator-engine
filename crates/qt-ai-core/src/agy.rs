//! Phát hiện Antigravity CLI trên máy người dùng.

use crate::error::{CoreError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Đường dẫn cấu hình tay thắng; không có thì quét PATH rồi thư mục cài mặc định của installer Windows.
pub fn find_agy(configured: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = configured {
        return if path.is_file() { Ok(path.to_path_buf()) } else { Err(CoreError::AgyMissing) };
    }
    let candidates: &[&str] = if cfg!(windows) { &["agy.exe", "agy.cmd", "agy.bat", "agy"] } else { &["agy"] };
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            for name in candidates {
                let candidate = dir.join(name);
                if candidate.is_file() {
                    return Ok(candidate);
                }
            }
        }
    }
    if cfg!(windows) {
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            for name in candidates {
                let candidate = PathBuf::from(&local).join("agy").join("bin").join(name);
                if candidate.is_file() {
                    return Ok(candidate);
                }
            }
        }
    }
    Err(CoreError::AgyMissing)
}

fn run_capture(agy: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new(agy).args(args).output().map_err(|_| CoreError::AgyMissing)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let tail: String = stderr.chars().rev().take(500).collect::<String>().chars().rev().collect();
        return Err(CoreError::AgyFailed { code: output.status.code().unwrap_or(-1), stderr_tail: tail });
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn agy_version(agy: &Path) -> Result<String> {
    Ok(run_capture(agy, &["--version"])?.lines().next().unwrap_or("").trim().to_string())
}

/// `agy models` — mỗi dòng không rỗng một model (format output của agy có thể đổi; GUI chỉ hiển thị).
pub fn agy_models(agy: &Path) -> Result<Vec<String>> {
    Ok(run_capture(agy, &["models"])?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(String::from)
        .collect())
}
