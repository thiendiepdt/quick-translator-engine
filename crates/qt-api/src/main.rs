use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use qt_api::{
    build_router_with_body_limit, configure_allocator, data_dir_from_env, max_body_bytes_from_env,
    serve, AppState,
};
use qt_core::{Dictionaries, Engine};

#[tokio::main]
async fn main() -> ExitCode {
    configure_allocator();
    let data_dir = data_dir_from_env();
    let max_body_bytes = match max_body_bytes_from_env() {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let (dicts, dictionary_defaults) = match Dictionaries::load_with_defaults(Path::new(&data_dir))
    {
        Ok(loaded) => loaded,
        Err(e) => {
            eprintln!("error: failed to load dictionaries from {data_dir}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let state = Arc::new(AppState {
        engine: Arc::new(Engine::from_dicts(dicts)),
        dictionary_defaults: Arc::new(dictionary_defaults),
    });
    println!("qt-server: max request body {} MiB", max_body_bytes >> 20);
    serve(
        build_router_with_body_limit(state, max_body_bytes),
        "qt-server",
        &data_dir,
    )
    .await
}
