//! `webview.open` (spec mục 8): extension có quyền `webview` mở cửa sổ trình duyệt riêng — hiện để người dùng
//! đăng nhập / giải captcha, hoặc ẩn để trang chạy JS — rồi nhận cookie, localStorage, sessionStorage, HTML.
//!
//! Crate này lo phần không phụ thuộc giao diện: đọc tuỳ chọn, kiểm `hosts`, giới hạn số cửa sổ mỗi extension,
//! script cố định để lấy dữ liệu, lọc kết quả theo `hosts`. Mở cửa sổ là việc của `WebviewHost` (app cài bằng
//! Tauri). Extension không chèn được script vào trang: script lấy dữ liệu là của app.

use crate::{ExtError, HostPolicy};
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::Duration;
use url::Url;

pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);
pub const MAX_TIMEOUT: Duration = Duration::from_secs(900);
/// Mỗi extension: số cửa sổ hiện / ẩn mở cùng lúc (giữa mọi lượt tải).
pub const MAX_VISIBLE: usize = 1;
pub const MAX_HIDDEN: usize = 2;
const MAX_SELECTOR: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageKeys {
    None,
    All,
    Keys(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collect {
    pub cookies: bool,
    pub local_storage: StorageKeys,
    pub session_storage: StorageKeys,
    pub html: bool,
}

/// Khớp một điều kiện là đóng cửa sổ.
#[derive(Debug, Clone, Default)]
pub struct Until {
    /// Regex trên URL hiện tại.
    pub url: Option<Regex>,
    /// Tên cookie (thuộc `hosts`).
    pub cookie: Option<String>,
    /// CSS có phần tử trong trang.
    pub selector: Option<String>,
}

impl Until {
    pub fn is_empty(&self) -> bool {
        self.url.is_none() && self.cookie.is_none() && self.selector.is_none()
    }
}

#[derive(Debug, Clone)]
pub struct WebviewRequest {
    pub extension_id: String,
    pub extension_name: String,
    pub url: Url,
    pub title: String,
    pub visible: bool,
    pub until: Until,
    pub timeout: Duration,
    pub collect: Collect,
    /// Domain được mở / điều hướng tới (cùng luật với `http`).
    pub policy: HostPolicy,
    /// Hồ sơ trình duyệt riêng của extension; `None` = không lưu gì (incognito).
    pub profile_dir: Option<PathBuf>,
    /// Lượt tải bị huỷ: host đóng cửa sổ ngay.
    pub cancel: Arc<AtomicBool>,
}

impl WebviewRequest {
    pub fn allows(&self, url: &Url) -> bool {
        self.policy.check(url).is_ok()
    }

    pub fn allows_cookie(&self, domain: &str) -> bool {
        self.policy.allows_cookie_domain(domain)
    }

    /// Tiêu đề cửa sổ: `<title> — <tên extension> · <domain đang mở>`.
    pub fn window_title(&self, current: Option<&Url>) -> String {
        let domain = current.or(Some(&self.url)).and_then(Url::host_str).unwrap_or("");
        let head = if self.title.is_empty() { String::new() } else { format!("{} — ", self.title) };
        format!("{head}{} · {domain}", self.extension_name)
    }

    /// Script cố định của app chạy trong trang để lấy storage và HTML. Trả chuỗi JSON `{ local, session, html }`.
    pub fn collect_script(&self) -> String {
        let keys = |k: &StorageKeys| match k {
            StorageKeys::None => "false".to_string(),
            StorageKeys::All => "true".to_string(),
            StorageKeys::Keys(list) => serde_json::to_string(list).unwrap_or_else(|_| "false".into()),
        };
        format!(
            r#"(() => {{
  const pick = (name, keys) => {{
    const out = {{}};
    if (keys === false) return out;
    try {{
      const store = window[name];
      const list = keys === true ? Array.from({{ length: store.length }}, (_, i) => store.key(i)) : keys;
      for (const k of list) {{ const v = store.getItem(k); if (v !== null) out[k] = v; }}
    }} catch (e) {{}}
    return out;
  }};
  let html = null;
  if ({html}) {{ try {{ html = document.documentElement.outerHTML; }} catch (e) {{}} }}
  return JSON.stringify({{ local: pick("localStorage", {local}), session: pick("sessionStorage", {session}), html }});
}})()"#,
            html = self.collect.html,
            local = keys(&self.collect.local_storage),
            session = keys(&self.collect.session_storage),
        )
    }

    /// Script kiểm `until.selector`; trả true/false.
    pub fn selector_script(&self) -> Option<String> {
        let selector = self.until.selector.as_ref()?;
        let literal = serde_json::to_string(selector).ok()?;
        Some(format!("(() => {{ try {{ return !!document.querySelector({literal}); }} catch (e) {{ return false; }} }})()"))
    }
}

/// Đọc kết quả `collect_script` (host trả nguyên chuỗi JSON). Sai dạng thì coi như trống.
pub fn parse_collected(text: &str) -> (Map<String, Value>, Map<String, Value>, Option<String>) {
    let value: Value = serde_json::from_str(text).unwrap_or(Value::Null);
    let map = |key: &str| value.get(key).and_then(Value::as_object).cloned().unwrap_or_default();
    (map("local"), map("session"), value.get("html").and_then(Value::as_str).map(str::to_string))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebviewCookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    /// Giây từ 1970; `None` = cookie phiên.
    pub expires: Option<i64>,
    pub http_only: bool,
    pub secure: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClosedBy {
    Until,
    User,
    Timeout,
    /// Cửa sổ ẩn không có `until`: đóng khi trang tải xong.
    Load,
}

impl ClosedBy {
    pub fn as_str(self) -> &'static str {
        match self {
            ClosedBy::Until => "until",
            ClosedBy::User => "user",
            ClosedBy::Timeout => "timeout",
            ClosedBy::Load => "load",
        }
    }
}

/// Dữ liệu thô host lấy được; qt-ext lọc theo `hosts` trước khi đưa extension.
#[derive(Debug, Clone)]
pub struct WebviewOutcome {
    /// URL cuối của trang.
    pub url: String,
    pub cookies: Vec<WebviewCookie>,
    pub local_storage: Map<String, Value>,
    pub session_storage: Map<String, Value>,
    pub html: Option<String>,
    pub closed_by: ClosedBy,
}

/// Mở cửa sổ. Gọi chặn (app chạy trên blocking pool), trả khi cửa sổ đóng. Phải đóng ngay khi
/// `request.cancel` bật.
pub trait WebviewHost: Send + Sync {
    fn open(&self, request: WebviewRequest) -> Result<WebviewOutcome, ExtError>;
}

/// Phần webview của một lượt: host (nếu có) và thông tin extension.
#[derive(Clone)]
pub(crate) struct WebviewContext {
    pub host: Option<Arc<dyn WebviewHost>>,
    pub extension_id: String,
    pub extension_name: String,
    /// Manifest có quyền `webview`.
    pub allowed: bool,
    pub profile_dir: Option<PathBuf>,
}

impl WebviewContext {
    pub fn none() -> WebviewContext {
        WebviewContext { host: None, extension_id: String::new(), extension_name: String::new(), allowed: false, profile_dir: None }
    }
}

pub(crate) struct Gates {
    pub visible: tokio::sync::Semaphore,
    pub hidden: tokio::sync::Semaphore,
}

static GATES: LazyLock<Mutex<HashMap<String, Arc<Gates>>>> = LazyLock::new(Default::default);

pub(crate) fn gates(extension_id: &str) -> Arc<Gates> {
    GATES
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .entry(extension_id.to_string())
        .or_insert_with(|| Arc::new(Gates { visible: tokio::sync::Semaphore::new(MAX_VISIBLE), hidden: tokio::sync::Semaphore::new(MAX_HIDDEN) }))
        .clone()
}

#[derive(Deserialize)]
struct RawUntil {
    url: Option<String>,
    cookie: Option<String>,
    selector: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawCollect {
    cookies: Option<bool>,
    local_storage: Option<Value>,
    session_storage: Option<Value>,
    html: Option<bool>,
}

#[derive(Deserialize)]
struct RawOptions {
    url: Option<String>,
    title: Option<String>,
    visible: Option<bool>,
    until: Option<RawUntil>,
    timeout: Option<f64>,
    collect: Option<RawCollect>,
}

fn storage_keys(value: Option<Value>, name: &str) -> Result<StorageKeys, String> {
    match value {
        None | Some(Value::Null) | Some(Value::Bool(false)) => Ok(StorageKeys::None),
        Some(Value::Bool(true)) => Ok(StorageKeys::All),
        Some(Value::Array(list)) => list
            .into_iter()
            .map(|v| v.as_str().map(str::to_string).ok_or_else(|| format!("collect.{name} phải là true/false hoặc mảng tên khoá")))
            .collect::<Result<Vec<_>, _>>()
            .map(StorageKeys::Keys),
        Some(_) => Err(format!("collect.{name} phải là true/false hoặc mảng tên khoá")),
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Đọc tuỳ chọn JS (JSON). Lỗi là lỗi kiểu (TypeError phía JS).
pub(crate) fn parse_request(json: &str, web: &WebviewContext, policy: HostPolicy, cancel: Arc<AtomicBool>) -> Result<WebviewRequest, String> {
    let raw: RawOptions = serde_json::from_str(json).map_err(|e| e.to_string())?;
    let url = raw.url.as_deref().map(str::trim).filter(|u| !u.is_empty()).ok_or("thiếu url")?;
    let url = Url::parse(url).map_err(|e| format!("url không hợp lệ: {e}"))?;
    let until = match raw.until {
        None => Until::default(),
        Some(u) => {
            let selector = non_empty(u.selector);
            if selector.as_ref().is_some_and(|s| s.chars().count() > MAX_SELECTOR) {
                return Err(format!("until.selector dài quá {MAX_SELECTOR} ký tự"));
            }
            Until {
                url: non_empty(u.url).map(|p| Regex::new(&p).map_err(|e| format!("until.url không phải regex: {e}"))).transpose()?,
                cookie: non_empty(u.cookie),
                selector,
            }
        }
    };
    let timeout = match raw.timeout {
        None => DEFAULT_TIMEOUT,
        Some(t) if t.is_finite() && t > 0.0 => Duration::from_secs_f64(t).min(MAX_TIMEOUT),
        Some(_) => return Err("timeout phải là số giây lớn hơn 0".into()),
    };
    let collect = match raw.collect {
        None => Collect { cookies: true, local_storage: StorageKeys::None, session_storage: StorageKeys::None, html: false },
        Some(c) => Collect {
            cookies: c.cookies.unwrap_or(true),
            local_storage: storage_keys(c.local_storage, "localStorage")?,
            session_storage: storage_keys(c.session_storage, "sessionStorage")?,
            html: c.html.unwrap_or(false),
        },
    };
    Ok(WebviewRequest {
        extension_id: web.extension_id.clone(),
        extension_name: web.extension_name.clone(),
        url,
        title: raw.title.map(|t| t.trim().to_string()).unwrap_or_default(),
        visible: raw.visible.unwrap_or(true),
        until,
        timeout,
        collect,
        policy,
        profile_dir: web.profile_dir.clone(),
        cancel,
    })
}

fn pick(map: Map<String, Value>, keys: &StorageKeys) -> Map<String, Value> {
    match keys {
        StorageKeys::None => Map::new(),
        StorageKeys::All => map,
        StorageKeys::Keys(list) => map.into_iter().filter(|(k, _)| list.contains(k)).collect(),
    }
}

/// Lọc kết quả theo `hosts` → JSON cho extension, kèm cookie được giữ (để nạp vào hũ cookie của `http`).
/// URL cuối ngoài `hosts` thì `url` là null và không có storage, HTML.
pub(crate) fn finish(request: &WebviewRequest, raw: WebviewOutcome) -> (Value, Vec<WebviewCookie>) {
    let url_ok = Url::parse(&raw.url).map(|u| request.allows(&u)).unwrap_or(false);
    let cookies: Vec<WebviewCookie> = if request.collect.cookies { raw.cookies.into_iter().filter(|c| request.allows_cookie(&c.domain)).collect() } else { Vec::new() };
    let (local, session, html) = if url_ok {
        (pick(raw.local_storage, &request.collect.local_storage), pick(raw.session_storage, &request.collect.session_storage), raw.html.filter(|_| request.collect.html))
    } else {
        (Map::new(), Map::new(), None)
    };
    let cookie_json: Vec<Value> = cookies.iter().map(|c| json!({ "name": c.name, "value": c.value, "domain": c.domain, "path": c.path, "expires": c.expires })).collect();
    let value = json!({
        "url": if url_ok { Value::String(raw.url) } else { Value::Null },
        "cookies": cookie_json,
        "localStorage": local,
        "sessionStorage": session,
        "html": html,
        "closedBy": raw.closed_by.as_str(),
    });
    (value, cookies)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn web() -> WebviewContext {
        WebviewContext { host: None, extension_id: "io.github.dev.w".into(), extension_name: "W".into(), allowed: true, profile_dir: None }
    }

    fn request(json: &str) -> Result<WebviewRequest, String> {
        parse_request(json, &web(), HostPolicy::new(&["demo.test".into(), "*.cdn.test".into()], false), Arc::default())
    }

    #[test]
    fn doc_tuy_chon() {
        let r = request(r#"{ "url": "https://demo.test/login" }"#).unwrap();
        assert!(r.visible);
        assert_eq!(r.timeout, DEFAULT_TIMEOUT);
        assert_eq!(r.collect, Collect { cookies: true, local_storage: StorageKeys::None, session_storage: StorageKeys::None, html: false });
        assert!(r.until.is_empty());
        assert_eq!(r.window_title(None), "W · demo.test");

        let r = request(r#"{ "url": "https://demo.test/", "title": " Đăng nhập ", "visible": false, "timeout": 5000,
            "until": { "url": "^https://demo\\.test/main", "cookie": "sid", "selector": ".avatar" },
            "collect": { "cookies": false, "localStorage": ["token"], "sessionStorage": true, "html": true } }"#)
        .unwrap();
        assert_eq!(r.timeout, MAX_TIMEOUT);
        assert!(!r.visible);
        assert!(r.until.url.as_ref().unwrap().is_match("https://demo.test/main/1"));
        assert_eq!(r.collect.local_storage, StorageKeys::Keys(vec!["token".into()]));
        assert_eq!(r.collect.session_storage, StorageKeys::All);
        assert_eq!(r.window_title(Some(&Url::parse("https://img.cdn.test/x").unwrap())), "Đăng nhập — W · img.cdn.test");

        assert!(request(r#"{}"#).unwrap_err().contains("url"));
        assert!(request(r#"{ "url": "https://demo.test/", "until": { "url": "(" } }"#).unwrap_err().contains("regex"));
        assert!(request(r#"{ "url": "https://demo.test/", "timeout": -1 }"#).is_err());
        assert!(request(r#"{ "url": "https://demo.test/", "collect": { "localStorage": "token" } }"#).is_err());
    }

    #[test]
    fn script_co_dinh_nhung_khoa_an_toan() {
        let r = request(r#"{ "url": "https://demo.test/", "until": { "selector": "a[href=\"x\"]" }, "collect": { "localStorage": ["t\"); alert(1); (\""], "html": true } }"#).unwrap();
        let script = r.collect_script();
        assert!(script.contains(r#"pick("localStorage", ["t\"); alert(1); (\""])"#), "{script}");
        assert!(script.contains("if (true)"));
        assert!(r.selector_script().unwrap().contains(r#"document.querySelector("a[href=\"x\"]")"#));
        let (local, session, html) = parse_collected(r#"{ "local": { "t": "1" }, "session": {}, "html": "<p>" }"#);
        assert_eq!((local.len(), session.len(), html.as_deref()), (1, 0, Some("<p>")));
        assert_eq!(parse_collected("rác").0.len(), 0);
    }

    #[test]
    fn loc_ket_qua_theo_hosts() {
        let r = request(r#"{ "url": "https://demo.test/", "collect": { "localStorage": ["token"], "html": true } }"#).unwrap();
        let cookie = |name: &str, domain: &str| WebviewCookie { name: name.into(), value: "v".into(), domain: domain.into(), path: "/".into(), expires: None, http_only: false, secure: true };
        let raw = |url: &str| WebviewOutcome {
            url: url.into(),
            cookies: vec![cookie("sid", "demo.test"), cookie("ga", ".google.com"), cookie("img", ".cdn.test")],
            local_storage: serde_json::from_str(r#"{ "token": "t", "khac": "x" }"#).unwrap(),
            session_storage: Map::new(),
            html: Some("<html>".into()),
            closed_by: ClosedBy::Until,
        };
        let (value, kept) = finish(&r, raw("https://demo.test/home"));
        assert_eq!(kept.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["sid", "img"]);
        assert_eq!(value["url"], json!("https://demo.test/home"));
        assert_eq!(value["localStorage"], json!({ "token": "t" }));
        assert_eq!(value["html"], json!("<html>"));
        assert_eq!(value["closedBy"], json!("until"));

        let (value, _) = finish(&r, raw("https://accounts.google.com/x"));
        assert_eq!((value["url"].clone(), value["localStorage"].clone(), value["html"].clone()), (Value::Null, json!({}), Value::Null));
    }
}
