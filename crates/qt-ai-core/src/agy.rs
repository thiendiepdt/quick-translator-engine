//! Phát hiện Antigravity CLI trên máy người dùng.

use crate::error::{CoreError, Result};
use std::path::{Path, PathBuf};
use crate::process::quiet_command;

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
    let output = quiet_command(agy).args(args).output().map_err(|_| CoreError::AgyMissing)?;
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

/// Một model agy liệt kê: `id` đưa vào `--model`, `label` chỉ để hiển thị.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct AgyModel {
    pub id: String,
    pub label: String,
}

/// `agy models` — agy 1.3 in dòng "Fetching available models..." rồi mỗi model một dòng `id<TAB>nhãn`
/// (agy cũ in mỗi dòng một id trần). Chỉ lấy id không có khoảng trắng: dòng thông báo bị loại,
/// phần sau tab không bao giờ lọt vào `--model` (từng gây "model ... is not recognized").
pub fn agy_models(agy: &Path) -> Result<Vec<AgyModel>> {
    Ok(parse_models(&run_capture(agy, &["models"])?))
}

pub fn parse_models(output: &str) -> Vec<AgyModel> {
    output
        .lines()
        .filter_map(|line| {
            let (id, label) = match line.split_once('\t') {
                Some((id, label)) => (id.trim(), label.trim()),
                None => (line.trim(), ""),
            };
            let valid = !id.is_empty() && !id.contains(char::is_whitespace) && !id.ends_with("...");
            valid.then(|| AgyModel { id: id.to_string(), label: if label.is_empty() { id.to_string() } else { label.to_string() } })
        })
        .collect()
}
