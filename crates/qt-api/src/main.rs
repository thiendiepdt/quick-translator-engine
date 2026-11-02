use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;

use qt_api::{build_router, data_dir_from_env, serve, AppState};
use qt_core::{Dictionaries, Engine};

// mimalloc thay glibc malloc: glibc giữ lại bộ nhớ đã free theo từng arena
// của thread nên RSS chỉ tăng (bench Linux: sau tải đồng thời 1383 MB so với
// 696 MB), mà tốc độ dịch / lọc tên ngang hoặc nhanh hơn.
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> ExitCode {
    let data_dir = data_dir_from_env();
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
    serve(build_router(state), "qt-server", &data_dir).await
}
