mod agy_cmds;
mod app_config;
mod error;
mod library_cmds;
mod sidecar;
mod session_cmds;
mod session_registry;
mod story_cmds;

use app_config::AppConfig;
use session_registry::SessionRegistry;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Manager;

pub struct AppState {
    pub config_path: PathBuf,
    pub config: Mutex<AppConfig>,
    /// Phiên dịch theo folder truyện — nhiều truyện chạy song song (xem session_registry).
    pub sessions: Mutex<SessionRegistry>,
}

/// Bản portable: có file đánh dấu `portable` cạnh exe → config.json nằm cạnh exe, copy folder là mang
/// theo cấu hình. Không có thì dùng thư mục config của app (%APPDATA%/io.quicktranslator.ai-gui).
pub fn resolve_config_path(exe_dir: Option<&Path>, app_config_dir: Option<PathBuf>) -> PathBuf {
    if let Some(dir) = exe_dir.filter(|dir| dir.join("portable").is_file()) {
        return dir.join("config.json");
    }
    app_config_dir.map(|dir| dir.join("config.json")).unwrap_or_else(|| PathBuf::from("config.json"))
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let exe_dir = std::env::current_exe().ok().and_then(|exe| exe.parent().map(Path::to_path_buf));
            let config_path = resolve_config_path(exe_dir.as_deref(), app.path().app_config_dir().ok());
            let config = AppConfig::load(&config_path);
            app.manage(AppState {
                config_path,
                config: Mutex::new(config),
                sessions: Mutex::new(SessionRegistry::new()),
            });
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
        .expect("không khởi động được QT AI Translator");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_canh_exe_khi_co_file_portable_con_lai_dung_app_config_dir() {
        let dir = tempfile::tempdir().unwrap();
        let app_dir = dir.path().join("appdata");
        assert_eq!(resolve_config_path(Some(dir.path()), Some(app_dir.clone())), app_dir.join("config.json"));
        std::fs::write(dir.path().join("portable"), "").unwrap();
        assert_eq!(resolve_config_path(Some(dir.path()), Some(app_dir.clone())), dir.path().join("config.json"));
        assert_eq!(resolve_config_path(None, None), PathBuf::from("config.json"));
    }
}
