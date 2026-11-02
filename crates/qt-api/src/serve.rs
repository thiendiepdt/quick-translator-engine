//! Socket binding shared by `qt-server` and `qt-lite-server`.

use std::net::SocketAddr;
use std::process::ExitCode;

use axum::Router;

/// Dictionary directory from `QT_DATA_DIR`, default `data`.
pub fn data_dir_from_env() -> String {
    std::env::var("QT_DATA_DIR").unwrap_or_else(|_| "data".to_string())
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
