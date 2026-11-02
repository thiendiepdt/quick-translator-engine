//! `qt-lite-server`: quick convert — chỉ dịch VietPhrase một nghĩa, từ điển
//! lưu gọn. Không có `/names/filter`, `/meanings`, `/dictionaries/defaults`;
//! tên đã lọc thì gửi kèm request qua `dictionaries.names`.

use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use qt_api::{build_lite_router, data_dir_from_env, serve, LiteAppState};
use qt_core::LiteEngine;

// Xem ghi chú allocator trong `qt-server`.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> ExitCode {
    let data_dir = data_dir_from_env();
    let engine = match LiteEngine::load(Path::new(&data_dir)) {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("error: failed to load dictionaries from {data_dir}: {e}");
            return ExitCode::FAILURE;
        }
    };
    let stats = engine.stats();
    println!(
        "qt-lite-server: {} VietPhrase entries, {} names (vietphrase-one only)",
        stats.vietphrase_entries, stats.name_entries
    );
    let state = Arc::new(LiteAppState {
        engine: Arc::new(engine),
    });
    serve(build_lite_router(state), "qt-lite-server", &data_dir).await
}
