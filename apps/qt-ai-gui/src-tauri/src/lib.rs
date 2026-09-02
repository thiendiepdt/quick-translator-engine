mod agy_cmds;
mod app_config;
mod error;
mod sidecar;
// mod story_cmds;   // Task 3
// mod session_cmds; // Task 4

use app_config::AppConfig;
use qt_ai_core::session::SessionHandle;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub config_path: PathBuf,
    pub config: Mutex<AppConfig>,
    pub session: Mutex<Option<SessionHandle>>,
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let config_path = app
                .path()
                .app_config_dir()
                .map(|dir| dir.join("config.json"))
                .unwrap_or_else(|_| PathBuf::from("config.json"));
            let config = AppConfig::load(&config_path);
            app.manage(AppState { config_path, config: Mutex::new(config), session: Mutex::new(None) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            agy_cmds::agy_status,
            agy_cmds::app_config_get,
            agy_cmds::app_config_set,
        ])
        .run(tauri::generate_context!())
        .expect("không khởi động được QT AI Translator");
}
