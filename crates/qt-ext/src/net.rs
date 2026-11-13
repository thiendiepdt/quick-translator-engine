//! HTTP cho extension. Tự đi redirect để kiểm `HostPolicy` ở từng bước (redirect sang domain lạ hay mạng nội
//! bộ đều bị chặn), cắt phản hồi quá 20 MB, giới hạn số luồng và giãn cách theo từng host, chờ 429 theo
//! `Retry-After`. Lỗi không kèm query của URL vì query có thể chứa khoá.

use crate::text::charset_from_content_type;
use crate::{ExtError, HostPolicy, Limits};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, LOCATION, RETRY_AFTER};
use reqwest::{Method, StatusCode};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use url::Url;

pub const MAX_BODY: usize = 20 * 1024 * 1024;
const MAX_REDIRECTS: usize = 10;
const MAX_429_RETRIES: usize = 3;
const MAX_RETRY_AFTER: Duration = Duration::from_secs(120);
const DEFAULT_RETRY_AFTER: Duration = Duration::from_secs(5);
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DESKTOP_UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Default)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    /// URL cuối sau redirect.
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub charset: Option<String>,
}

struct HostGate {
    permits: tokio::sync::Semaphore,
    last: tokio::sync::Mutex<Option<Instant>>,
}

pub struct HttpClient {
    client: reqwest::Client,
    policy: HostPolicy,
    limits: Limits,
    gates: Mutex<HashMap<String, Arc<HostGate>>>,
}

fn install_crypto_provider() {
    // Workspace có crate khác bật aws-lc-rs cho rustls; cài tường minh ring như qt-ai-core.
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// `host/path` không query — dùng trong thông báo lỗi.
fn safe_url(url: &Url) -> String {
    format!("{}{}", url.host_str().unwrap_or(""), url.path())
}

fn network_error(url: &Url, error: reqwest::Error) -> ExtError {
    if error.is_timeout() {
        return ExtError::Network(format!("{}: hết thời gian chờ", safe_url(url)));
    }
    ExtError::Network(format!("{}: {}", safe_url(url), error.without_url()))
}

fn retry_after(headers: &HeaderMap) -> Duration {
    headers
        .get(RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.trim().parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_RETRY_AFTER)
        .min(MAX_RETRY_AFTER)
}

impl HttpClient {
    pub fn new(policy: HostPolicy, limits: Limits) -> Result<Self, ExtError> {
        install_crypto_provider();
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .cookie_store(true)
            .user_agent(DESKTOP_UA)
            .gzip(true)
            .brotli(true)
            .deflate(true)
            .connect_timeout(Duration::from_secs(20))
            .build()
            .map_err(|e| ExtError::Network(e.to_string()))?;
        Ok(HttpClient { client, policy, limits, gates: Mutex::new(HashMap::new()) })
    }

    pub fn policy(&self) -> &HostPolicy {
        &self.policy
    }

    fn gate(&self, url: &Url) -> Arc<HostGate> {
        let key = url.host_str().unwrap_or("").to_ascii_lowercase();
        let mut gates = self.gates.lock().expect("gates");
        gates
            .entry(key)
            .or_insert_with(|| Arc::new(HostGate { permits: tokio::sync::Semaphore::new(self.limits.concurrency.max(1)), last: tokio::sync::Mutex::new(None) }))
            .clone()
    }

    pub async fn send(&self, request: HttpRequest) -> Result<HttpResponse, ExtError> {
        let mut url = Url::parse(request.url.trim()).map_err(|e| ExtError::Network(format!("url không hợp lệ: {e}")))?;
        if !request.query.is_empty() {
            url.query_pairs_mut().extend_pairs(&request.query);
        }
        let method_name = if request.method.trim().is_empty() { "GET".to_string() } else { request.method.trim().to_ascii_uppercase() };
        let method = Method::from_bytes(method_name.as_bytes()).map_err(|_| ExtError::Network(format!("method lạ: {method_name}")))?;
        let mut headers = HeaderMap::new();
        for (key, value) in &request.headers {
            let name = HeaderName::from_bytes(key.trim().as_bytes()).map_err(|_| ExtError::Network(format!("header lạ: {key}")))?;
            let value = HeaderValue::from_str(value).map_err(|_| ExtError::Network(format!("giá trị header {key} không hợp lệ")))?;
            headers.append(name, value);
        }
        let timeout = request.timeout_ms.map(Duration::from_millis).unwrap_or(DEFAULT_TIMEOUT);
        let mut waited = 0;
        loop {
            let (response, retry) = self.follow(method.clone(), url.clone(), &headers, request.body.clone(), timeout).await?;
            match retry {
                Some(wait) if waited < MAX_429_RETRIES => {
                    waited += 1;
                    tokio::time::sleep(wait).await;
                }
                _ => return Ok(response),
            }
        }
    }

    /// Một lượt gọi kèm redirect. Trả thêm thời gian chờ nếu phản hồi là 429.
    async fn follow(&self, mut method: Method, mut url: Url, headers: &HeaderMap, mut body: Option<Vec<u8>>, timeout: Duration) -> Result<(HttpResponse, Option<Duration>), ExtError> {
        for _ in 0..=MAX_REDIRECTS {
            self.policy.check(&url)?;
            let gate = self.gate(&url);
            let _permit = gate.permits.acquire().await.map_err(|_| ExtError::Cancelled)?;
            {
                let mut last = gate.last.lock().await;
                if let Some(previous) = *last {
                    let gap = Duration::from_millis(self.limits.delay_ms);
                    let elapsed = previous.elapsed();
                    if elapsed < gap {
                        tokio::time::sleep(gap - elapsed).await;
                    }
                }
                *last = Some(Instant::now());
            }
            let mut builder = self.client.request(method.clone(), url.clone()).headers(headers.clone()).timeout(timeout);
            if let Some(bytes) = &body {
                builder = builder.body(bytes.clone());
            }
            let mut response = builder.send().await.map_err(|e| network_error(&url, e))?;
            let status = response.status();
            if status.is_redirection() {
                if let Some(location) = response.headers().get(LOCATION).and_then(|v| v.to_str().ok()) {
                    url = url.join(location).map_err(|e| ExtError::Network(format!("{}: Location không hợp lệ: {e}", safe_url(&url))))?;
                    let to_get = status == StatusCode::SEE_OTHER || ((status == StatusCode::MOVED_PERMANENTLY || status == StatusCode::FOUND) && method == Method::POST);
                    if to_get {
                        method = Method::GET;
                        body = None;
                    }
                    continue;
                }
            }
            let retry = (status == StatusCode::TOO_MANY_REQUESTS).then(|| retry_after(response.headers()));
            let charset = response.headers().get(CONTENT_TYPE).and_then(|v| v.to_str().ok()).and_then(charset_from_content_type);
            let response_headers: Vec<(String, String)> =
                response.headers().iter().map(|(k, v)| (k.as_str().to_string(), String::from_utf8_lossy(v.as_bytes()).into_owned())).collect();
            if response.content_length().is_some_and(|n| n as usize > MAX_BODY) {
                return Err(ExtError::TooLarge(MAX_BODY));
            }
            let mut buffer = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(|e| network_error(&url, e))? {
                if buffer.len() + chunk.len() > MAX_BODY {
                    return Err(ExtError::TooLarge(MAX_BODY));
                }
                buffer.extend_from_slice(&chunk);
            }
            let result = HttpResponse { status: status.as_u16(), url: url.to_string(), headers: response_headers, body: buffer, charset };
            return Ok((result, retry));
        }
        Err(ExtError::Network(format!("{}: quá {MAX_REDIRECTS} lần chuyển hướng", safe_url(&url))))
    }
}
