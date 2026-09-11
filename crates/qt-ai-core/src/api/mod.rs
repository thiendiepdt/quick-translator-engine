//! Gọi model qua HTTP bằng key của người dùng (động cơ "api" của qt-ai-gui). Port từ
//! qt-web/src/lib/ai-text-client.ts (stream) và completeJson trong ai-client.ts (JSON).
//! Chỉ hai provider: Gemini chính chủ và OpenAI-compatible (OpenAI hoặc hub tự chọn qua base URL).

pub mod gemini;
pub mod openai;
pub mod sse;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufReader, Read};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, RecvTimeoutError};
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

/// Mức "nghĩ" cho một lượt gọi. Dịch dùng mức người dùng chọn; soát và trích glossary là việc cơ học,
/// thinking cao chỉ đốt output token (Gemini tính thought vào output).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effort {
    /// Theo cấu hình người dùng (Gemini `thinking`, OpenAI `reasoning_effort`).
    Full,
    /// Gemini 3: low; 2.5: budget 1024; OpenAI: low.
    Low,
    /// Gemini 3: minimal; 2.5: budget 0; OpenAI: low (hub lạ thường không nhận "minimal").
    Minimal,
}

/// Token một lượt gọi theo provider báo (Gemini usageMetadata / OpenAI usage). `thoughts` tính riêng
/// với `output`; `cached` là phần của `input` được cache.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub input: u64,
    pub cached: u64,
    pub output: u64,
    pub thoughts: u64,
}

impl Usage {
    pub fn total(&self) -> u64 {
        self.input + self.output + self.thoughts
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }

    /// "vào 13.5k (cache 9.9k) · ra 3.7k · nghĩ 2.1k" — bỏ phần bằng 0.
    pub fn summary(&self) -> String {
        let mut parts = vec![format!("vào {}", format_tokens(self.input))];
        if self.cached > 0 {
            parts[0].push_str(&format!(" (cache {})", format_tokens(self.cached)));
        }
        parts.push(format!("ra {}", format_tokens(self.output)));
        if self.thoughts > 0 {
            parts.push(format!("nghĩ {}", format_tokens(self.thoughts)));
        }
        parts.join(" · ")
    }
}

impl std::ops::AddAssign for Usage {
    fn add_assign(&mut self, other: Usage) {
        self.input += other.input;
        self.cached += other.cached;
        self.output += other.output;
        self.thoughts += other.thoughts;
    }
}

/// "812", "13.5k", "1.2M".
pub fn format_tokens(count: u64) -> String {
    if count >= 1_000_000 {
        format!("{:.1}M", count as f64 / 1_000_000.0)
    } else if count >= 1000 {
        format!("{:.1}k", count as f64 / 1000.0)
    } else {
        count.to_string()
    }
}

/// Kết quả một lượt gọi: text và usage nếu provider báo (hub lạ có thể không báo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generated {
    pub text: String,
    pub usage: Option<Usage>,
}

impl Generated {
    pub fn text(text: impl Into<String>) -> Self {
        Generated { text: text.into(), usage: None }
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
        effort: Effort,
        cancel: &AtomicBool,
        on_progress: &mut dyn FnMut(usize),
    ) -> Result<Generated, ApiError>;
    /// Sinh JSON (không stream) cho tác vụ phụ như trích glossary. `cancel` bật → `ApiError::Cancelled`.
    fn complete_json(&self, system: &str, user: &str, effort: Effort, cancel: &AtomicBool) -> Result<Generated, ApiError>;
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

    /// Gửi request rồi đọc body trên thread phụ; phía gọi nhận từng khúc qua channel và nhìn cờ huỷ mỗi
    /// `CANCEL_POLL`. reqwest blocking không abort được, nên huỷ = bỏ receiver: thread phụ thấy channel
    /// đóng thì drop response → kết nối bị cắt, hub ngừng sinh. Nhờ vậy Dừng phản hồi ngay cả khi model
    /// đang "nghĩ" và chưa gửi byte nào.
    fn send<'a>(
        &self,
        url: &str,
        headers: &[(&str, String)],
        body: &Value,
        cancel: &'a AtomicBool,
    ) -> Result<CancellableBody<'a>, ApiError> {
        let provider = self.config.provider.label();
        let mut request = http_client().post(url).header("content-type", "application/json");
        for (name, value) in headers {
            request = request.header(*name, value.as_str());
        }
        let request = request.body(serde_json::to_vec(body).expect("body serialize"));
        let (tx, rx) = sync_channel::<Result<Vec<u8>, ApiError>>(64);
        std::thread::spawn(move || {
            let mut response = match request.send() {
                Ok(response) => response,
                Err(error) => {
                    let message = if error.is_timeout() { "timeout".to_string() } else { error.without_url().to_string() };
                    let _ = tx.send(Err(ApiError::Network { provider, message }));
                    return;
                }
            };
            let status = response.status();
            if !status.is_success() {
                let text = response.text().unwrap_or_default();
                let _ = tx.send(Err(ApiError::Http { provider, status: status.as_u16(), message: error_message(&text) }));
                return;
            }
            // Header OK: báo một khúc rỗng để phía gọi biết đã kết nối, rồi stream body.
            if tx.send(Ok(Vec::new())).is_err() {
                return;
            }
            let mut chunk = [0u8; 8192];
            loop {
                match response.read(&mut chunk) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(Ok(chunk[..n].to_vec())).is_err() {
                            break; // phía gọi đã huỷ — drop response, cắt kết nối
                        }
                    }
                    Err(error) => {
                        let _ = tx.send(Err(ApiError::Stream(error.to_string())));
                        break;
                    }
                }
            }
        });
        let mut body = CancellableBody { rx, cancel, buffer: Vec::new(), pos: 0, done: false };
        body.wait_headers()?;
        Ok(body)
    }
}

/// Nhịp nhìn cờ huỷ khi đang chờ byte từ hub.
pub const CANCEL_POLL: Duration = Duration::from_millis(100);

/// Body HTTP đọc qua channel từ thread phụ — `Read` trả lỗi "đã huỷ" ngay khi cờ bật.
pub struct CancellableBody<'a> {
    rx: Receiver<Result<Vec<u8>, ApiError>>,
    cancel: &'a AtomicBool,
    buffer: Vec<u8>,
    pos: usize,
    done: bool,
}

impl CancellableBody<'_> {
    /// Chờ tín hiệu header OK (khúc rỗng đầu tiên) hoặc lỗi gửi/HTTP; huỷ giữa chừng → Cancelled.
    fn wait_headers(&mut self) -> Result<(), ApiError> {
        match self.recv()? {
            Some(_) => Ok(()),
            None => Err(ApiError::Stream("kết nối đóng trước khi có response".to_string())),
        }
    }

    /// `Ok(None)` = hết body. Chờ theo nhịp `CANCEL_POLL` để cờ huỷ có tác dụng ngay.
    fn recv(&mut self) -> Result<Option<Vec<u8>>, ApiError> {
        loop {
            if self.cancel.load(Ordering::SeqCst) {
                return Err(ApiError::Cancelled);
            }
            match self.rx.recv_timeout(CANCEL_POLL) {
                Ok(Ok(chunk)) => return Ok(Some(chunk)),
                Ok(Err(error)) => return Err(error),
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => return Ok(None),
            }
        }
    }

    /// Đọc trọn body thành text (đường JSON không stream).
    pub fn read_all(mut self) -> Result<String, ApiError> {
        let mut bytes = self.buffer.split_off(self.pos);
        while let Some(chunk) = self.recv()? {
            bytes.extend_from_slice(&chunk);
        }
        String::from_utf8(bytes).map_err(|error| ApiError::Stream(format!("body không phải UTF-8: {error}")))
    }
}

impl Read for CancellableBody<'_> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        while self.pos >= self.buffer.len() {
            if self.done {
                return Ok(0);
            }
            match self.recv() {
                Ok(Some(chunk)) => {
                    self.buffer = chunk;
                    self.pos = 0;
                }
                Ok(None) => {
                    self.done = true;
                    return Ok(0);
                }
                Err(error) => return Err(std::io::Error::other(error.to_string())),
            }
        }
        let n = out.len().min(self.buffer.len() - self.pos);
        out[..n].copy_from_slice(&self.buffer[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
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
        effort: Effort,
        cancel: &AtomicBool,
        on_progress: &mut dyn FnMut(usize),
    ) -> Result<Generated, ApiError> {
        let config = &self.config;
        match config.provider {
            ApiProvider::Gemini => {
                let response = self.send(
                    &gemini::stream_url(config),
                    &gemini::headers(config),
                    &gemini::stream_body(config, system, user, effort),
                    cancel,
                )?;
                gemini::parse_stream(BufReader::new(response), cancel, on_progress)
            }
            ApiProvider::OpenAi => {
                let response = self.send(
                    &openai::url(config),
                    &openai::headers(config, true),
                    &openai::stream_body(config, system, user, effort),
                    cancel,
                )?;
                openai::parse_stream(BufReader::new(response), cancel, on_progress)
            }
        }
    }

    /// JSON mode trước; hub/model không nhận JSON mode (400/404/422, trả rỗng, không phải JSON…) thì
    /// gọi lại bằng lượt text thường rồi bóc object JSON ra — glossary/AI điền không vì hub lạ mà câm.
    fn complete_json(&self, system: &str, user: &str, effort: Effort, cancel: &AtomicBool) -> Result<Generated, ApiError> {
        match self.complete_json_strict(system, user, effort, cancel) {
            Ok(generated) => Ok(generated),
            Err(error) if error.is_transient() || matches!(error, ApiError::Blocked(_) | ApiError::Cancelled) => Err(error),
            Err(error) => {
                let system = format!("{system}\n\nChỉ trả về đúng một JSON object hợp lệ, không giải thích, không markdown.");
                let generated = self.generate(&system, user, effort, cancel, &mut |_| {})?;
                let text = extract_json_object(&generated.text)
                    .ok_or_else(|| ApiError::BadOutput(format!("model không trả JSON (JSON mode lỗi: {error})")))?;
                Ok(Generated { text, usage: generated.usage })
            }
        }
    }
}

impl HttpModel {
    fn complete_json_strict(&self, system: &str, user: &str, effort: Effort, cancel: &AtomicBool) -> Result<Generated, ApiError> {
        let config = &self.config;
        let provider = config.provider.label();
        let (url, headers, body) = match config.provider {
            ApiProvider::Gemini => {
                (gemini::json_url(config), gemini::headers(config), gemini::json_body(config, system, user, effort))
            }
            ApiProvider::OpenAi => {
                (openai::url(config), openai::headers(config, false), openai::json_body(config, system, user, effort))
            }
        };
        let text = self.send(&url, &headers, &body, cancel)?.read_all()?;
        let payload: Value = serde_json::from_str(&text)
            .map_err(|error| ApiError::Stream(format!("response {provider} không phải JSON: {error}")))?;
        let (content, usage) = match config.provider {
            ApiProvider::Gemini => (gemini::parse_json_response(&payload), gemini::usage_of(&payload)),
            ApiProvider::OpenAi => (openai::parse_json_response(&payload), openai::usage_of(&payload)),
        };
        let text = content.filter(|text| !text.trim().is_empty()).ok_or(ApiError::Empty(provider))?;
        // Một số hub bật JSON mode nhưng vẫn bọc ```json hoặc nói thêm một câu — bóc luôn cho chắc.
        Ok(Generated { text: extract_json_object(&text).unwrap_or(text), usage })
    }
}

/// Bóc object JSON đầu-cuối khỏi text model trả (chịu rào ```json và chữ thừa quanh). Phải parse được.
pub fn extract_json_object(text: &str) -> Option<String> {
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    if end < start {
        return None;
    }
    let candidate = &text[start..=end];
    serde_json::from_str::<Value>(candidate).ok().filter(Value::is_object).map(|_| candidate.to_string())
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
    fn usage_cong_don_va_tom_tat_bo_phan_bang_0() {
        let mut total = Usage { input: 13_500, cached: 9_900, output: 3_700, thoughts: 2_100 };
        assert_eq!(total.summary(), "vào 13.5k (cache 9.9k) · ra 3.7k · nghĩ 2.1k");
        total += Usage { input: 500, ..Usage::default() };
        assert_eq!(total.input, 14_000);
        assert_eq!(Usage { input: 812, output: 40, ..Usage::default() }.summary(), "vào 812 · ra 40");
        assert_eq!(format_tokens(1_250_000), "1.2M");
        assert!(Usage::default().is_empty());
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
