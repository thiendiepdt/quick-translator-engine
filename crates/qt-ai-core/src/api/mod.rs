//! Gọi model qua HTTP bằng key của người dùng (động cơ "api" của qt-ai-gui). Port từ
//! qt-web/src/lib/ai-text-client.ts (stream) và completeJson trong ai-client.ts (JSON).
//! Chỉ hai provider: Gemini chính chủ và OpenAI-compatible (OpenAI hoặc hub tự chọn qua base URL).

pub mod gemini;
pub mod openai;
pub mod sse;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::BufReader;
use std::sync::atomic::AtomicBool;
use std::sync::OnceLock;
use std::time::Duration;

pub const DEFAULT_GEMINI_BASE_URL: &str = "https://generativelanguage.googleapis.com";
pub const DEFAULT_OPENAI_BASE_URL: &str = "https://api.openai.com/v1";
pub const DEFAULT_GEMINI_MODEL: &str = "gemini-3.7-flash";
pub const DEFAULT_OPENAI_MODEL: &str = "gpt-5.6-sol";
pub const MAX_OUTPUT_TOKENS: u64 = 65_536;
/// Một lượt gọi model tối đa (stream vẫn phải kết thúc trong khoảng này).
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiProvider {
    Gemini,
    #[serde(rename = "openai")]
    OpenAi,
}

impl ApiProvider {
    pub fn label(self) -> &'static str {
        match self {
            ApiProvider::Gemini => "Gemini",
            ApiProvider::OpenAi => "OpenAI",
        }
    }

    pub fn default_base_url(self) -> &'static str {
        match self {
            ApiProvider::Gemini => DEFAULT_GEMINI_BASE_URL,
            ApiProvider::OpenAi => DEFAULT_OPENAI_BASE_URL,
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            ApiProvider::Gemini => DEFAULT_GEMINI_MODEL,
            ApiProvider::OpenAi => DEFAULT_OPENAI_MODEL,
        }
    }
}

/// Cấu hình đã chốt cho một lượt gọi: model/base URL đã điền mặc định, key đã trim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiConfig {
    pub provider: ApiProvider,
    pub api_key: String,
    pub model: String,
    pub base_url: String,
    /// Gemini: thinkingLevel high ↔ minimal (2.5: budget -1 ↔ 0). OpenAI không dùng.
    pub thinking: bool,
    /// OpenAI: `reasoning_effort` (none…max); rỗng = không gửi.
    pub reasoning_effort: String,
}

impl ApiConfig {
    pub fn resolve(
        provider: ApiProvider,
        api_key: &str,
        model: &str,
        base_url: &str,
        thinking: bool,
        reasoning_effort: &str,
    ) -> ApiConfig {
        let model = model.trim();
        let base_url = base_url.trim();
        ApiConfig {
            provider,
            api_key: api_key.trim().to_string(),
            model: if model.is_empty() { provider.default_model().to_string() } else { model.to_string() },
            base_url: if base_url.is_empty() { provider.default_base_url() } else { base_url }
                .trim_end_matches('/')
                .to_string(),
            thinking,
            reasoning_effort: reasoning_effort.trim().to_string(),
        }
    }

    pub fn label(&self) -> String {
        format!("{} {}", self.provider.label(), self.model)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ApiError {
    /// Model từ chối sinh nội dung (Gemini PROHIBITED_CONTENT/SAFETY…) — chương phải skip, không retry.
    #[error("Model chặn nội dung ({0})")]
    Blocked(String),
    #[error("{provider} trả {status}: {message}")]
    Http { provider: &'static str, status: u16, message: String },
    #[error("{provider} không kết nối được: {message}")]
    Network { provider: &'static str, message: String },
    #[error("Không đọc được stream AI: {0}")]
    Stream(String),
    #[error("{0} không trả về nội dung")]
    Empty(&'static str),
    /// Model trả text nhưng không theo định dạng nhãn [[n]] yêu cầu.
    #[error("Model không trả đúng định dạng: {0}")]
    BadOutput(String),
    #[error("Đã huỷ")]
    Cancelled,
}

/// Lỗi kiểu "thử lại có thể qua" (mạng, 429/5xx, stream đứt) — khác lỗi cấu hình (401/400) hay nội dung.
impl ApiError {
    pub fn is_transient(&self) -> bool {
        match self {
            ApiError::Network { .. } | ApiError::Stream(_) => true,
            ApiError::Http { status, .. } => *status == 408 || *status == 429 || *status >= 500,
            _ => false,
        }
    }
}

/// Model text: vòng dịch chỉ cần hai thao tác này nên test được bằng model giả không HTTP.
pub trait TextModel: Send + Sync {
    fn label(&self) -> String;
    /// Sinh text tự do (stream). `on_progress` nhận tổng ký tự output đã nhận tới lúc đó.
    fn generate(
        &self,
        system: &str,
        user: &str,
        cancel: &AtomicBool,
        on_progress: &mut dyn FnMut(usize),
    ) -> Result<String, ApiError>;
    /// Sinh JSON (không stream) cho tác vụ phụ như trích glossary.
    fn complete_json(&self, system: &str, user: &str) -> Result<String, ApiError>;
}

fn install_crypto_provider() {
    // Workspace có crate khác bật aws-lc-rs cho rustls; cài tường minh ring để không mơ hồ provider.
    let _ = rustls::crypto::ring::default_provider().install_default();
}

pub fn http_client() -> &'static reqwest::blocking::Client {
    static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        install_crypto_provider();
        reqwest::blocking::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .connect_timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client")
    })
}

fn truncate(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

/// `error.message` trong body JSON của provider; không phải JSON thì lấy 500 ký tự đầu.
pub fn error_message(body: &str) -> String {
    let detail = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|value| value.get("error")?.get("message")?.as_str().map(String::from))
        .unwrap_or_else(|| body.to_string());
    truncate(detail.trim(), 500)
}

pub struct HttpModel {
    config: ApiConfig,
}

impl HttpModel {
    pub fn new(config: ApiConfig) -> Self {
        HttpModel { config }
    }

    pub fn config(&self) -> &ApiConfig {
        &self.config
    }

    fn send(&self, url: &str, headers: &[(&str, String)], body: &Value) -> Result<reqwest::blocking::Response, ApiError> {
        let provider = self.config.provider.label();
        let mut request = http_client().post(url).header("content-type", "application/json");
        for (name, value) in headers {
            request = request.header(*name, value.as_str());
        }
        let response = request.body(serde_json::to_vec(body).expect("body serialize")).send().map_err(|error| {
            let message = if error.is_timeout() { "timeout".to_string() } else { error.without_url().to_string() };
            ApiError::Network { provider, message }
        })?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().unwrap_or_default();
            return Err(ApiError::Http { provider, status: status.as_u16(), message: error_message(&text) });
        }
        Ok(response)
    }
}

impl TextModel for HttpModel {
    fn label(&self) -> String {
        self.config.label()
    }

    fn generate(
        &self,
        system: &str,
        user: &str,
        cancel: &AtomicBool,
        on_progress: &mut dyn FnMut(usize),
    ) -> Result<String, ApiError> {
        let config = &self.config;
        match config.provider {
            ApiProvider::Gemini => {
                let response = self.send(
                    &gemini::stream_url(config),
                    &gemini::headers(config),
                    &gemini::stream_body(config, system, user),
                )?;
                gemini::parse_stream(BufReader::new(response), cancel, on_progress)
            }
            ApiProvider::OpenAi => {
                let response = self.send(
                    &openai::url(config),
                    &openai::headers(config, true),
                    &openai::stream_body(config, system, user),
                )?;
                openai::parse_stream(BufReader::new(response), cancel, on_progress)
            }
        }
    }

    fn complete_json(&self, system: &str, user: &str) -> Result<String, ApiError> {
        let config = &self.config;
        let provider = config.provider.label();
        let (url, headers, body) = match config.provider {
            ApiProvider::Gemini => {
                (gemini::json_url(config), gemini::headers(config), gemini::json_body(config, system, user))
            }
            ApiProvider::OpenAi => {
                (openai::url(config), openai::headers(config, false), openai::json_body(config, system, user))
            }
        };
        let response = self.send(&url, &headers, &body)?;
        let payload: Value = response
            .json()
            .map_err(|error| ApiError::Stream(format!("response {provider} không phải JSON: {error}")))?;
        let content = match config.provider {
            ApiProvider::Gemini => gemini::parse_json_response(&payload),
            ApiProvider::OpenAi => openai::parse_json_response(&payload),
        };
        content.filter(|text| !text.trim().is_empty()).ok_or(ApiError::Empty(provider))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_dien_mac_dinh_va_cat_slash_cuoi() {
        let gemini = ApiConfig::resolve(ApiProvider::Gemini, " AIza ", "", "", true, "");
        assert_eq!(gemini.api_key, "AIza");
        assert_eq!(gemini.model, DEFAULT_GEMINI_MODEL);
        assert_eq!(gemini.base_url, DEFAULT_GEMINI_BASE_URL);
        let hub = ApiConfig::resolve(ApiProvider::OpenAi, "sk", "gemini-3.7-flash", "http://192.0.2.10/v1/", false, "high");
        assert_eq!(hub.base_url, "http://192.0.2.10/v1");
        assert_eq!(hub.model, "gemini-3.7-flash");
        assert_eq!(hub.label(), "OpenAI gemini-3.7-flash");
        assert_eq!(ApiConfig::resolve(ApiProvider::OpenAi, "sk", "", "", true, "").model, DEFAULT_OPENAI_MODEL);
    }

    #[test]
    fn provider_serialize_thanh_gemini_openai() {
        assert_eq!(serde_json::to_string(&ApiProvider::OpenAi).unwrap(), "\"openai\"");
        assert_eq!(serde_json::from_str::<ApiProvider>("\"gemini\"").unwrap(), ApiProvider::Gemini);
    }

    #[test]
    fn error_message_lay_error_message_json_hoac_body() {
        assert_eq!(error_message(r#"{"error":{"message":"bad key","code":401}}"#), "bad key");
        assert_eq!(error_message("<html>oops</html>"), "<html>oops</html>");
    }

    #[test]
    fn transient_chi_mang_429_5xx_stream() {
        let http = |status| ApiError::Http { provider: "Gemini", status, message: String::new() };
        assert!(http(429).is_transient() && http(503).is_transient() && http(408).is_transient());
        assert!(!http(401).is_transient() && !http(400).is_transient());
        assert!(ApiError::Network { provider: "Gemini", message: String::new() }.is_transient());
        assert!(!ApiError::Blocked("SAFETY".into()).is_transient());
        assert!(!ApiError::BadOutput(String::new()).is_transient());
    }
}
