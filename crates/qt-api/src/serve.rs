//! Process setup shared by `qt-server` and `qt-lite-server`: allocator
//! tuning, environment configuration and socket binding.

use std::net::SocketAddr;
use std::process::ExitCode;

use axum::Router;

use crate::app::DEFAULT_MAX_BODY_BYTES;

/// Ngưỡng mmap cố định của glibc malloc (xem [`configure_allocator`]).
#[cfg(all(target_os = "linux", target_env = "gnu"))]
const MMAP_THRESHOLD_BYTES: libc::c_int = 4 * 1024 * 1024;

/// Chỉnh glibc malloc cho kiểu tải trộn của server; gọi đầu `main`.
///
/// Cấp phát < 4 MB (request một chương) dùng arena chung giữa các thread:
/// nhanh, không phình theo số thread. Cấp phát ≥ 4 MB (buffer của file lớn)
/// đi thẳng mmap nên trả về OS ngay khi request xong. Mặc định glibc tự nâng
/// ngưỡng này tới 32 MB sau lần free lớn đầu tiên, nên buffer file lớn rơi vào
/// arena và bị giữ. Bench Linux, 2×100 chương + 4 file 1,5 triệu ký tự đồng
/// thời + 2×100 chương, RSS sau tải: qt-server 769 MB (glibc mặc định) /
/// 687 MB (mimalloc) → 509 MB; lite 472 / 424 → 143 MB. File lớn chậm ~5%.
///
/// Người tự host đặt `MALLOC_MMAP_THRESHOLD_` hoặc tunable
/// `glibc.malloc.mmap_threshold` thì giữ giá trị của họ. Ngoài Linux glibc
/// là no-op.
pub fn configure_allocator() {
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        let user_set = std::env::var_os("MALLOC_MMAP_THRESHOLD_").is_some()
            || std::env::var("GLIBC_TUNABLES").is_ok_and(|value| value.contains("mmap_threshold"));
        if !user_set {
            // SAFETY: mallopt chỉ đổi tham số của malloc; gọi lúc khởi động.
            unsafe { libc::mallopt(libc::M_MMAP_THRESHOLD, MMAP_THRESHOLD_BYTES) };
        }
    }
}

/// Dictionary directory from `QT_DATA_DIR`, default `data`.
pub fn data_dir_from_env() -> String {
    std::env::var("QT_DATA_DIR").unwrap_or_else(|_| "data".to_string())
}

/// JSON body limit from `QT_MAX_BODY_MB` (MiB, positive integer), default
/// [`DEFAULT_MAX_BODY_BYTES`]. An invalid value is an error rather than a
/// silent fallback.
pub fn max_body_bytes_from_env() -> Result<usize, String> {
    parse_max_body_mb(std::env::var("QT_MAX_BODY_MB").ok().as_deref())
}

fn parse_max_body_mb(value: Option<&str>) -> Result<usize, String> {
    let Some(raw) = value else {
        return Ok(DEFAULT_MAX_BODY_BYTES);
    };
    raw.trim()
        .parse::<usize>()
        .ok()
        .filter(|mib| *mib > 0)
        .and_then(|mib| mib.checked_mul(1024 * 1024))
        .ok_or_else(|| format!("QT_MAX_BODY_MB must be a positive integer (MiB), got {raw:?}"))
}

/// Bind `0.0.0.0:$QT_PORT` (default 3000) and serve `app` until it fails.
pub async fn serve(app: Router, name: &str, data_dir: &str) -> ExitCode {
    let port: u16 = std::env::var("QT_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: failed to bind {addr}: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("{name} listening on http://{addr} (data-dir: {data_dir})");
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("error: server failed: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_body_defaults_to_32_mib_and_parses_mib() {
        assert_eq!(parse_max_body_mb(None), Ok(32 * 1024 * 1024));
        assert_eq!(parse_max_body_mb(Some("5")), Ok(5 * 1024 * 1024));
        assert_eq!(parse_max_body_mb(Some(" 64 ")), Ok(64 * 1024 * 1024));
        for bad in ["0", "-1", "abc", "1.5", ""] {
            assert!(parse_max_body_mb(Some(bad)).is_err(), "{bad:?}");
        }
    }
}
