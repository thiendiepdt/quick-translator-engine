mod agy_cmds;
mod app_config;
mod base_cmds;
mod error;
mod library_cmds;
mod sidecar;
mod session_cmds;
mod session_registry;
mod story_cmds;
mod summary_cache;

use app_config::AppConfig;
use session_registry::SessionRegistry;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub config_path: PathBuf,
    /// `<app_config_dir>/base` — bản mặc định sửa được (qt_ai_core::base). Cũng đặt vào env
    /// QT_AI_BASE_DIR lúc khởi động để phiên API trong app và tiến trình agy con cùng thấy.
    pub base_dir: PathBuf,
    pub config: Mutex<AppConfig>,
    /// Phiên dịch theo folder truyện — nhiều truyện chạy song song (xem session_registry).
    pub sessions: Mutex<SessionRegistry>,
    /// Tóm tắt truyện (tên, done/total) theo mtime — màn chọn truyện không parse lại state.json mỗi lần.
    pub summaries: summary_cache::SummaryCache,
}

impl AppState {
    pub fn base_store(&self) -> qt_ai_core::base::BaseStore {
        qt_ai_core::base::BaseStore::at(&self.base_dir)
    }
}

/// Identifier cũ (trước 2026-09-11) — thư mục config cũ trong %APPDATA% để migrate.
const LEGACY_IDENTIFIER: &str = "io.quicktranslator.ai-gui";

/// config.json luôn nằm trong thư mục config của app (%APPDATA%/com.vn-converter.qt-ai-gui), kể cả bản
/// portable — cài đặt hay portable dùng chung một cấu hình. Không lấy được thư mục thì rơi về cwd.
pub fn resolve_config_path(app_config_dir: Option<PathBuf>) -> PathBuf {
    app_config_dir.map(|dir| dir.join("config.json")).unwrap_or_else(|| PathBuf::from("config.json"))
}

/// Đổi identifier làm %APPDATA% đổi thư mục: chưa có config mới mà thư mục identifier cũ (cùng cha)
/// còn config.json thì copy sang, người dùng không mất recent/API key. Lỗi copy bỏ qua (app vẫn chạy).
pub fn migrate_legacy_config(config_path: &Path) {
    if config_path.exists() {
        return;
    }
    let Some(legacy) = config_path
        .parent()
        .and_then(Path::parent)
        .map(|base| base.join(LEGACY_IDENTIFIER).join("config.json"))
        .filter(|path| path.is_file())
    else {
        return;
    };
    if let Some(dir) = config_path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::copy(&legacy, config_path);
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // Updater chỉ có trên desktop; đăng ký trong setup như novelkit nên mock_builder trong test không cần nó.
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;
            let config_path = resolve_config_path(app.path().app_config_dir().ok());
            migrate_legacy_config(&config_path);
            let base_dir = config_path.parent().map(|dir| dir.join("base")).unwrap_or_else(|| PathBuf::from("base"));
            std::env::set_var(qt_ai_core::base::BASE_DIR_ENV, &base_dir);
            let config = AppConfig::load(&config_path);
            app.manage(AppState {
                config_path,
                base_dir,
                config: Mutex::new(config),
                sessions: Mutex::new(SessionRegistry::new()),
                summaries: summary_cache::SummaryCache::new(),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            agy_cmds::agy_status,
            agy_cmds::app_config_get,
            agy_cmds::app_config_set,
            base_cmds::base_get,
            base_cmds::base_save,
            base_cmds::base_reset,
            story_cmds::open_story,
            story_cmds::init_story,
            story_cmds::story_snapshot,
            story_cmds::read_chapter,
            story_cmds::save_story,
            story_cmds::save_chapter_output,
            story_cmds::save_settings,
            story_cmds::chapter_retry,
            story_cmds::chapters_retry,
            story_cmds::chapter_skip,
            story_cmds::chapter_force_accept,
            story_cmds::export_chapters,
            story_cmds::reveal_folder,
            story_cmds::recent_summaries,
            story_cmds::story_defaults,
            session_cmds::session_state,
            session_cmds::session_start,
            session_cmds::session_stop,
            session_cmds::ai_fill_story,
            library_cmds::slugify_name,
            library_cmds::library_list,
            library_cmds::create_story,
            library_cmds::rescan_story,
            library_cmds::import_chapters,
        ])
        .run(tauri::generate_context!())
        .expect("không khởi động được VNCVT AI Translator");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_luon_trong_app_config_dir() {
        let app_dir = PathBuf::from("appdata");
        assert_eq!(resolve_config_path(Some(app_dir.clone())), app_dir.join("config.json"));
        assert_eq!(resolve_config_path(None), PathBuf::from("config.json"));
    }

    #[test]
    fn migrate_copy_config_tu_thu_muc_identifier_cu_khi_chua_co_config_moi() {
        let dir = tempfile::tempdir().unwrap();
        let legacy = dir.path().join(LEGACY_IDENTIFIER);
        std::fs::create_dir_all(&legacy).unwrap();
        std::fs::write(legacy.join("config.json"), "config-cu").unwrap();
        let new_path = dir.path().join("com.vn-converter.qt-ai-gui").join("config.json");
        migrate_legacy_config(&new_path);
        assert_eq!(std::fs::read_to_string(&new_path).unwrap(), "config-cu");
        // Đã có config mới thì không đè.
        std::fs::write(&new_path, "moi").unwrap();
        migrate_legacy_config(&new_path);
        assert_eq!(std::fs::read_to_string(&new_path).unwrap(), "moi");
        // Không có thư mục cũ: không tạo gì.
        let lonely = dir.path().join("x").join("y").join("config.json");
        migrate_legacy_config(&lonely);
        assert!(!lonely.exists());
    }

    /// Lệnh Tauri đồng bộ chạy ngay trên luồng nhận IPC — trên Linux là luồng GTK vẽ cửa sổ, nên lệnh
    /// tốn thời gian (HTTP, spawn tiến trình, quét raw/) làm cả app đơ. Nhưng `#[tauri::command(async)]`
    /// trên hàm sync lại chạy thân hàm ngay trên worker tokio: 12 lệnh chặn cùng lúc là mọi lệnh async
    /// khác treo. Những lệnh này phải là `async fn` và bọc thân bằng `error::blocking` (spawn_blocking).
    #[test]
    fn lenh_nang_phai_la_async_fn_boc_blocking() {
        let sources = [
            include_str!("agy_cmds.rs"),
            include_str!("base_cmds.rs"),
            include_str!("library_cmds.rs"),
            include_str!("session_cmds.rs"),
            include_str!("story_cmds.rs"),
        ]
        .join("\n");
        let heavy = [
            "agy_status",
            "base_get",
            "base_save",
            "base_reset",
            "library_list",
            "create_story",
            "rescan_story",
            "import_chapters",
            "session_start",
            "session_stop",
            "ai_fill_story",
            "recent_summaries",
            "open_story",
            "init_story",
            "chapter_force_accept",
            "export_chapters",
            "reveal_folder",
        ];
        for name in heavy {
            // Có AppHandle thì generic theo Runtime (mock runtime trong test gọi được), không thì không.
            let markers = [
                format!("#[tauri::command]\npub async fn {name}<R: Runtime>("),
                format!("#[tauri::command]\npub async fn {name}("),
            ];
            let Some(body_start) = markers.iter().find_map(|m| sources.find(m)) else {
                panic!("{name} phải là #[tauri::command] pub async fn");
            };
            let body = &sources[body_start..body_start + sources[body_start..].find("\n}\n").unwrap()];
            assert!(body.contains("blocking(move ||"), "{name} phải bọc thân bằng blocking(move || …)");
        }
    }

    /// Gọi lệnh async qua IPC giả của Tauri: phải trả lời (không treo), kể cả khi lệnh có `State`.
    #[test]
    fn lenh_async_tra_loi_qua_ipc() {
        use tauri::test::{mock_builder, mock_context, noop_assets};
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");
        let app = mock_builder()
            .manage(AppState {
                config_path: config_path.clone(),
                base_dir: dir.path().join("base"),
                config: Mutex::new(AppConfig::load(&config_path)),
                sessions: Mutex::new(SessionRegistry::new()),
                summaries: summary_cache::SummaryCache::new(),
            })
            .invoke_handler(tauri::generate_handler![story_cmds::recent_summaries, agy_cmds::app_config_get])
            .build(mock_context(noop_assets()))
            .expect("mock app");
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
        for cmd in ["app_config_get", "recent_summaries"] {
            let (tx, rx) = std::sync::mpsc::sync_channel(1);
            let webview = webview.clone();
            std::thread::spawn(move || {
                let response = tauri::test::get_ipc_response(
                    &webview,
                    tauri::webview::InvokeRequest {
                        cmd: cmd.into(),
                        callback: tauri::ipc::CallbackFn(0),
                        error: tauri::ipc::CallbackFn(1),
                        url: "tauri://localhost".parse().unwrap(),
                        body: tauri::ipc::InvokeBody::Json(serde_json::json!({})),
                        headers: Default::default(),
                        invoke_key: tauri::test::INVOKE_KEY.to_string(),
                    },
                );
                let _ = tx.send(response.map(|b| b.deserialize::<serde_json::Value>().unwrap()));
            });
            let response = rx
                .recv_timeout(std::time::Duration::from_secs(10))
                .unwrap_or_else(|_| panic!("{cmd}: IPC không trả lời trong 10 giây (treo)"));
            assert!(response.is_ok(), "{cmd}: {response:?}");
        }
    }
}
