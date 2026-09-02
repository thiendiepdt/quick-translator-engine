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
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig { agy_path: None, model: None, max_sessions: 50, recent: vec![] }
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
}
