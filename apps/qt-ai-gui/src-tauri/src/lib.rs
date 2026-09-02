mod agy_cmds;
mod app_config;
mod error;
mod sidecar;
mod session_cmds;
mod story_cmds;

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
            story_cmds::open_story,
            story_cmds::init_story,
            story_cmds::story_snapshot,
            story_cmds::read_chapter,
            story_cmds::save_story,
            story_cmds::save_settings,
            story_cmds::chapter_retry,
            story_cmds::chapter_skip,
            story_cmds::chapter_force_accept,
            story_cmds::export_chapters,
            story_cmds::reveal_folder,
            session_cmds::session_state,
            session_cmds::session_start,
            session_cmds::session_stop,
            session_cmds::ai_fill_story,
        ])
        .run(tauri::generate_context!())
        .expect("không khởi động được QT AI Translator");
}
