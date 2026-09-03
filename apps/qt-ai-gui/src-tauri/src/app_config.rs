use crate::error::{CmdResult, CommandError};
use serde::{Deserialize, Serialize};
use std::path::Path;

const MAX_RECENT: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    /// Đường dẫn agy chỉnh tay; None = tự tìm trong PATH.
    pub agy_path: Option<String>,
    pub model: Option<String>,
    pub max_sessions: u32,
    /// Folder truyện mở gần đây, mới nhất đứng đầu.
    pub recent: Vec<String>,
    /// Bộ màu: editorial | studio | soft (UI kiểm tra giá trị, Rust chỉ lưu).
    pub palette: String,
    /// light | dark | system.
    pub theme_mode: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            agy_path: None,
            model: None,
            max_sessions: 50,
            recent: vec![],
            palette: "editorial".to_string(),
            theme_mode: "system".to_string(),
        }
    }
}

impl AppConfig {
    /// File thiếu/hỏng → default; không bao giờ chặn app khởi động vì config.
    pub fn load(path: &Path) -> AppConfig {
        std::fs::read_to_string(path).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> CmdResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| CommandError::new("io", e.to_string()))?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| CommandError::new("internal", e.to_string()))?;
        std::fs::write(path, json).map_err(|e| CommandError::new("io", e.to_string()))
    }

    pub fn touch_recent(&mut self, root: &str) {
        self.recent.retain(|item| item != root);
        self.recent.insert(0, root.to_string());
        self.recent.truncate(MAX_RECENT);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_thieu_file_ra_default_save_roi_load_lai_khop() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut config = AppConfig::load(&path);
        assert_eq!(config.max_sessions, 50);
        assert!(config.recent.is_empty());
        config.model = Some("gemini".into());
        for i in 0..12 {
            config.touch_recent(&format!("D:\\truyen{i}"));
        }
        config.touch_recent("D:\\truyen3");
        assert_eq!(config.recent.len(), 10);
        assert_eq!(config.recent[0], "D:\\truyen3");
        config.save(&path).unwrap();
        assert_eq!(AppConfig::load(&path), config);
        std::fs::write(&path, "hỏng").unwrap();
        assert_eq!(AppConfig::load(&path), AppConfig::default()); // file hỏng → default, không panic
    }

    #[test]
    fn default_co_palette_editorial_va_theme_system_va_doc_config_cu_thieu_truong() {
        let config = AppConfig::default();
        assert_eq!(config.palette, "editorial");
        assert_eq!(config.theme_mode, "system");
        let old: AppConfig =
            serde_json::from_str(r#"{"agyPath":null,"model":null,"maxSessions":7,"recent":[]}"#).unwrap();
        assert_eq!(old.max_sessions, 7);
        assert_eq!(old.palette, "editorial");
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["themeMode"], "system");
    }
}
