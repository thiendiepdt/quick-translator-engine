use crate::error::{CmdResult, CommandError};
use qt_ai_core::api::{ApiConfig, ApiProvider, DEFAULT_GEMINI_MODEL, DEFAULT_OPENAI_MODEL};
use serde::{Deserialize, Serialize};
use std::path::Path;

const MAX_RECENT: usize = 10;

/// Động cơ dịch: agy điều khiển Antigravity CLI; api gọi model thẳng bằng key của người dùng.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Engine {
    #[default]
    Agy,
    Api,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProviderCredentials {
    pub api_key: String,
    pub model: String,
    /// Trống = endpoint chính thức; OpenAI đổi sang hub OpenAI-compatible bất kỳ.
    pub base_url: String,
}

/// Key/model tách riêng theo provider như qt-web: đổi provider không mang key bên này sang bên kia.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ApiSettings {
    pub provider: ApiProvider,
    pub gemini: ProviderCredentials,
    pub openai: ProviderCredentials,
    /// Gemini: thinkingLevel high ↔ minimal.
    pub thinking: bool,
    /// OpenAI: none|low|medium|high|xhigh|max.
    pub reasoning_effort: String,
}

impl Default for ApiSettings {
    fn default() -> Self {
        ApiSettings {
            provider: ApiProvider::Gemini,
            gemini: ProviderCredentials { model: DEFAULT_GEMINI_MODEL.to_string(), ..Default::default() },
            openai: ProviderCredentials { model: DEFAULT_OPENAI_MODEL.to_string(), ..Default::default() },
            thinking: true,
            reasoning_effort: "high".to_string(),
        }
    }
}

impl ApiSettings {
    pub fn active(&self) -> &ProviderCredentials {
        match self.provider {
            ApiProvider::Gemini => &self.gemini,
            ApiProvider::OpenAi => &self.openai,
        }
    }

    /// Cấu hình đã điền mặc định cho provider đang chọn.
    pub fn resolve(&self) -> ApiConfig {
        let active = self.active();
        ApiConfig::resolve(self.provider, &active.api_key, &active.model, &active.base_url, self.thinking, &self.reasoning_effort)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AppConfig {
    pub engine: Engine,
    pub api: ApiSettings,
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
    /// Chiều ngang vùng đọc: narrow | normal | wide | full (UI kiểm tra giá trị, Rust chỉ lưu).
    pub reading_width: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            engine: Engine::Agy,
            api: ApiSettings::default(),
            agy_path: None,
            model: None,
            max_sessions: 50,
            recent: vec![],
            palette: "editorial".to_string(),
            theme_mode: "system".to_string(),
            reading_width: "normal".to_string(),
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
        assert_eq!(json["readingWidth"], "normal");
        assert_eq!(old.reading_width, "normal");
    }

    #[test]
    fn config_cu_thieu_engine_api_ra_agy_va_api_mac_dinh() {
        let old: AppConfig = serde_json::from_str(r#"{"agyPath":null,"model":null,"maxSessions":7,"recent":[]}"#).unwrap();
        assert_eq!(old.engine, Engine::Agy);
        assert_eq!(old.api, ApiSettings::default());
        assert_eq!(old.api.gemini.model, "gemini-3.7-flash");
        assert_eq!(old.api.openai.model, "gpt-5.6-sol");
        assert_eq!(old.api.reasoning_effort, "high");
        let json = serde_json::to_value(&old).unwrap();
        assert_eq!(json["engine"], "agy");
        assert_eq!(json["api"]["provider"], "gemini");
        assert_eq!(json["api"]["gemini"]["apiKey"], "");
    }

    #[test]
    fn api_settings_resolve_theo_provider_dang_chon() {
        let mut api = ApiSettings::default();
        api.gemini.api_key = " AIza ".into();
        api.openai = ProviderCredentials { api_key: "sk-hub".into(), model: "".into(), base_url: "http://192.0.2.10/v1/".into() };
        let gemini = api.resolve();
        assert_eq!(gemini.provider, ApiProvider::Gemini);
        assert_eq!(gemini.api_key, "AIza");
        assert_eq!(gemini.base_url, "https://generativelanguage.googleapis.com");
        api.provider = ApiProvider::OpenAi;
        api.reasoning_effort = "xhigh".into();
        let openai = api.resolve();
        assert_eq!(openai.api_key, "sk-hub");
        assert_eq!(openai.model, "gpt-5.6-sol");
        assert_eq!(openai.base_url, "http://192.0.2.10/v1");
        assert_eq!(openai.reasoning_effort, "xhigh");
        let round: AppConfig = serde_json::from_str(&serde_json::to_string(&AppConfig { api, engine: Engine::Api, ..Default::default() }).unwrap()).unwrap();
        assert_eq!(round.engine, Engine::Api);
        assert_eq!(round.api.openai.api_key, "sk-hub");
    }
}
