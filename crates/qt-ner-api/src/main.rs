use std::net::SocketAddr;
use std::path::Path;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let data_dir = std::env::var("QT_NER_DATA_DIR")
        .or_else(|_| std::env::var("QT_DATA_DIR"))
        .unwrap_or_else(|_| "data".to_string());
    let port = std::env::var("QT_NER_PORT")
        .or_else(|_| std::env::var("QT_PORT"))
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3001);

    let state = match qt_ner_api::load_state(Path::new(&data_dir)) {
        Ok(state) => state,
        Err(error) => {
            eprintln!("error: failed to load dictionaries from {data_dir}: {error}");
            return ExitCode::FAILURE;
        }
    };
    let app = qt_ner_api::build_router(state);
    let address = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = match tokio::net::TcpListener::bind(address).await {
        Ok(listener) => listener,
        Err(error) => {
            eprintln!("error: failed to bind {address}: {error}");
            return ExitCode::FAILURE;
        }
    };

    println!("qt-ner-api listening on http://{address} (data-dir: {data_dir})");
    if let Err(error) = axum::serve(listener, app).await {
        eprintln!("error: server failed: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
