use std::net::SocketAddr;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use qt_core::{Dictionaries, Engine};
use qt_api::{build_router, AppState};

#[tokio::main]
async fn main() -> ExitCode {
    let data_dir = std::env::var("QT_DATA_DIR").unwrap_or_else(|_| "data".to_string());
    let port: u16 = std::env::var("QT_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    let dicts = match Dictionaries::load(Path::new(&data_dir)) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: failed to load dictionaries from {data_dir}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let state = Arc::new(AppState { engine: Arc::new(Engine::from_dicts(dicts)) });
    let app = build_router(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: failed to bind {addr}: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("qt-server listening on http://{addr} (data-dir: {data_dir})");
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("error: server failed: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
