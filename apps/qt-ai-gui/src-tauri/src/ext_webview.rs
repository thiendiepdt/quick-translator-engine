//! Cửa sổ trình duyệt cho `webview.open` của extension (spec mục 8), cài `qt_ext::WebviewHost` bằng Tauri.
//!
//! - Nhãn `ext-<id>-<n>`, không thuộc capability nào; trang ngoài là origin từ xa nên Tauri từ chối mọi lệnh app.
//! - Hồ sơ WebView2 riêng của extension (`data_directory`), không có thì incognito.
//! - Khung chính chỉ điều hướng tới domain trong `hosts`; popup bị chặn, link popup thuộc `hosts` mở ngay trong cửa
//!   sổ đó. Tiêu đề luôn là `<title> — <tên extension> · <domain>`, trang không đổi được.
//! - Người dùng đóng cửa sổ = xong: chặn lệnh đóng, lấy dữ liệu rồi mới đóng.
//! - Dữ liệu lấy bằng script cố định của `qt-ext`; `cookies()` và chờ script chạy trên thread blocking (gọi trên
//!   main thread thì WebView2 deadlock).

use qt_ext::webview::parse_collected;
use qt_ext::{ClosedBy, ExtError, StorageKeys, WebviewCookie, WebviewHost, WebviewOutcome, WebviewRequest};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::webview::{Cookie, NewWindowResponse, PageLoadEvent};
use tauri::{AppHandle, Runtime, Url, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

const POLL: Duration = Duration::from_millis(400);
const EVAL_WAIT: Duration = Duration::from_secs(5);

static COUNTER: AtomicU64 = AtomicU64::new(1);

pub struct TauriWebviews<R: Runtime> {
    app: AppHandle<R>,
}

/// Host webview gắn vào `SessionOptions.webview` của lượt tải / đăng nhập.
pub fn host<R: Runtime>(app: &AppHandle<R>) -> Arc<dyn WebviewHost> {
    Arc::new(TauriWebviews { app: app.clone() })
}

/// Nhãn cửa sổ: Tauri chỉ nhận chữ, số, `-`, `/`, `:`, `_`.
pub fn label_for(extension_id: &str) -> String {
    let id: String = extension_id.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
    format!("ext-{id}-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Chạy script, chờ kết quả (chuỗi JSON của giá trị trả về).
fn eval<R: Runtime>(window: &WebviewWindow<R>, script: String) -> Option<String> {
    let (tx, rx) = mpsc::channel();
    window
        .eval_with_callback(script, move |result| {
            let _ = tx.send(result);
        })
        .ok()?;
    rx.recv_timeout(EVAL_WAIT).ok()
}

fn to_cookie(cookie: &Cookie<'static>) -> WebviewCookie {
    WebviewCookie {
        name: cookie.name().to_string(),
        value: cookie.value().to_string(),
        domain: cookie.domain().unwrap_or("").to_string(),
        path: cookie.path().unwrap_or("/").to_string(),
        expires: cookie.expires_datetime().map(|t| t.unix_timestamp()),
        http_only: cookie.http_only().unwrap_or(false),
        secure: cookie.secure().unwrap_or(false),
    }
}

fn until_matched<R: Runtime>(window: &WebviewWindow<R>, request: &WebviewRequest) -> bool {
    let until = &request.until;
    if let Some(pattern) = &until.url {
        if window.url().is_ok_and(|url| pattern.is_match(url.as_str())) {
            return true;
        }
    }
    if let Some(name) = &until.cookie {
        if window.cookies().is_ok_and(|list| list.iter().any(|c| c.name() == name && request.allows_cookie(c.domain().unwrap_or("")))) {
            return true;
        }
    }
    if let Some(script) = request.selector_script() {
        if eval(window, script).as_deref() == Some("true") {
            return true;
        }
    }
    false
}

fn collect<R: Runtime>(window: &WebviewWindow<R>, request: &WebviewRequest, closed_by: ClosedBy) -> WebviewOutcome {
    let url = window.url().map(|u| u.to_string()).unwrap_or_default();
    let cookies = if request.collect.cookies { window.cookies().unwrap_or_default().iter().map(to_cookie).collect() } else { Vec::new() };
    let wants_script = request.collect.html || request.collect.local_storage != StorageKeys::None || request.collect.session_storage != StorageKeys::None;
    let (local_storage, session_storage, html) = if wants_script {
        // Script trả một chuỗi JSON; WebView2 bọc kết quả thêm một lớp JSON nữa.
        eval(window, request.collect_script()).and_then(|raw| serde_json::from_str::<String>(&raw).ok()).map(|text| parse_collected(&text)).unwrap_or_default()
    } else {
        Default::default()
    };
    WebviewOutcome { url, cookies, local_storage, session_storage, html, closed_by }
}

impl<R: Runtime> WebviewHost for TauriWebviews<R> {
    fn open(&self, request: WebviewRequest) -> Result<WebviewOutcome, ExtError> {
        let fail = |e: tauri::Error| ExtError::Extension { message: format!("không mở được cửa sổ: {e}"), retry: false };
        let loaded = Arc::new(AtomicBool::new(false));
        let user_done = Arc::new(AtomicBool::new(false));
        let popup: Arc<Mutex<Option<Url>>> = Arc::default();
        let shared = Arc::new(request.clone());

        let navigation = shared.clone();
        let new_window = (shared.clone(), popup.clone());
        let page_load = (shared.clone(), loaded.clone());
        let title = shared.clone();
        let mut builder = WebviewWindowBuilder::new(&self.app, label_for(&request.extension_id), WebviewUrl::External(request.url.clone()))
            .title(request.window_title(None))
            .inner_size(1024.0, 768.0)
            .center()
            .visible(request.visible)
            .focused(request.visible)
            .skip_taskbar(!request.visible)
            .on_navigation(move |url| navigation.allows(url))
            .on_new_window(move |url, _| {
                if new_window.0.allows(&url) {
                    *new_window.1.lock().unwrap_or_else(|e| e.into_inner()) = Some(url);
                }
                NewWindowResponse::Deny
            })
            .on_page_load(move |window, payload| {
                let _ = window.set_title(&page_load.0.window_title(Some(payload.url())));
                if payload.event() == PageLoadEvent::Finished {
                    page_load.1.store(true, Ordering::SeqCst);
                }
            })
            .on_document_title_changed(move |window, _| {
                let _ = window.set_title(&title.window_title(window.url().ok().as_ref()));
            });
        builder = match &request.profile_dir {
            Some(dir) => builder.data_directory(dir.clone()),
            None => builder.incognito(true),
        };
        let window = builder.build().map_err(fail)?;
        {
            let user_done = user_done.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    user_done.store(true, Ordering::SeqCst);
                }
            });
        }

        let start = Instant::now();
        let closed_by = loop {
            if request.cancel.load(Ordering::SeqCst) {
                let _ = window.destroy();
                return Err(ExtError::Cancelled);
            }
            if user_done.load(Ordering::SeqCst) {
                break ClosedBy::User;
            }
            if start.elapsed() >= request.timeout {
                break ClosedBy::Timeout;
            }
            if let Some(url) = popup.lock().unwrap_or_else(|e| e.into_inner()).take() {
                let _ = window.navigate(url);
            }
            if loaded.load(Ordering::SeqCst) {
                if until_matched(&window, &request) {
                    break ClosedBy::Until;
                }
                if !request.visible && request.until.is_empty() {
                    break ClosedBy::Load;
                }
            }
            std::thread::sleep(POLL);
        };
        let outcome = collect(&window, &request, closed_by);
        let _ = window.destroy();
        Ok(outcome)
    }
}

/// Tự kiểm trên WebView2 thật, chỉ ở bản debug: `QT_EXT_WEBVIEW_SELFTEST=<url>` mở cửa sổ (ẩn, trừ khi
/// `QT_EXT_WEBVIEW_SELFTEST_VISIBLE=1`) tới URL đó, chờ `#done`, ghi kết quả ra `QT_EXT_WEBVIEW_SELFTEST_OUT` rồi
/// thoát app. Hồ sơ ở `QT_EXT_WEBVIEW_SELFTEST_PROFILE`.
#[cfg(debug_assertions)]
pub fn selftest_from_env<R: Runtime>(app: &AppHandle<R>) {
    use qt_ext::{Collect, HostPolicy, Until};
    let Ok(url) = std::env::var("QT_EXT_WEBVIEW_SELFTEST") else {
        return;
    };
    let out = std::env::var("QT_EXT_WEBVIEW_SELFTEST_OUT").unwrap_or_else(|_| "webview-selftest.json".into());
    let profile = std::env::var("QT_EXT_WEBVIEW_SELFTEST_PROFILE").ok().map(std::path::PathBuf::from);
    let visible = std::env::var("QT_EXT_WEBVIEW_SELFTEST_VISIBLE").is_ok_and(|v| v == "1");
    let app = app.clone();
    std::thread::spawn(move || {
        let report = match Url::parse(&url) {
            Err(e) => serde_json::json!({ "error": e.to_string() }),
            Ok(parsed) => {
                let domain = parsed.host_str().unwrap_or("").to_string();
                let request = WebviewRequest {
                    extension_id: "io.github.dev.selftest".into(),
                    extension_name: "Tự kiểm".into(),
                    url: parsed,
                    title: "Tự kiểm webview".into(),
                    visible,
                    until: Until { selector: Some("#done".into()), ..Default::default() },
                    timeout: Duration::from_secs(30),
                    collect: Collect { cookies: true, local_storage: StorageKeys::All, session_storage: StorageKeys::All, html: true },
                    policy: HostPolicy::new(&[domain], true),
                    profile_dir: profile,
                    cancel: Arc::default(),
                };
                let start = Instant::now();
                match host(&app).open(request) {
                    Err(e) => serde_json::json!({ "error": e.to_string() }),
                    Ok(o) => serde_json::json!({
                        "ms": start.elapsed().as_millis() as u64,
                        "url": o.url,
                        "closedBy": o.closed_by.as_str(),
                        "cookies": o.cookies.iter().map(|c| serde_json::json!({ "name": c.name, "value": c.value, "domain": c.domain, "httpOnly": c.http_only, "expires": c.expires })).collect::<Vec<_>>(),
                        "localStorage": o.local_storage,
                        "sessionStorage": o.session_storage,
                        "html": o.html.map(|h| h.chars().take(300).collect::<String>()),
                    }),
                }
            }
        };
        let _ = std::fs::write(&out, serde_json::to_string_pretty(&report).unwrap_or_default());
        app.exit(0);
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn nhan_cua_so_hop_le() {
        let label = super::label_for("io.github.dev.fan-qie");
        assert!(label.starts_with("ext-io_github_dev_fan-qie-"), "{label}");
        assert!(label.chars().all(|c| c.is_ascii_alphanumeric() || "-_".contains(c)));
    }
}
