//! `qt-lite-server`: quick convert — chỉ dịch VietPhrase một nghĩa, từ điển
//! lưu gọn. Không có `/names/filter`, `/meanings`, `/dictionaries/defaults`
//! và range; tên đã lọc thì gửi kèm request qua `dictionaries.names`.

use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use qt_api::{
    build_lite_router_with_body_limit, configure_allocator, data_dir_from_env,
    max_body_bytes_from_env, serve, LiteAppState,
};
use qt_core::LiteEngine;

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
    let engine = match LiteEngine::load(Path::new(&data_dir)) {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("error: failed to load dictionaries from {data_dir}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let stats = engine.stats();
    println!(
        "qt-lite-server: {} VietPhrase entries, {} names (vietphrase-one only), max request body {} MiB",
        stats.vietphrase_entries,
        stats.name_entries,
        max_body_bytes >> 20
    );
    let state = Arc::new(LiteAppState {
        engine: Arc::new(engine),
    });
    serve(
        build_lite_router_with_body_limit(state, max_body_bytes),
        "qt-lite-server",
        &data_dir,
    )
    .await
}
