//! Mốc 4: `webview.open` với host giả — quyền, hosts, lọc kết quả, cookie vào hũ `http`, giới hạn cửa sổ,
//! thời gian chờ người dùng, huỷ.

mod common;

use common::{package, Route, TestServer};
use qt_ext::{ClosedBy, EngineOptions, ExtError, ExtStore, SecretBackend, Session, SessionOptions, TocItem, WebviewCookie, WebviewHost, WebviewOutcome, WebviewRequest};
use serde_json::{json, Map, Value};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

type Reply = dyn Fn(&WebviewRequest) -> WebviewOutcome + Send + Sync;

struct FakeHost {
    delay: Duration,
    /// Chờ tới khi lượt bị huỷ thay vì `delay`.
    until_cancel: bool,
    reply: Box<Reply>,
    requests: Mutex<Vec<WebviewRequest>>,
    active: AtomicUsize,
    max_active: AtomicUsize,
    saw_cancel: AtomicBool,
}

impl FakeHost {
    fn new(delay_ms: u64, reply: impl Fn(&WebviewRequest) -> WebviewOutcome + Send + Sync + 'static) -> Arc<FakeHost> {
        Arc::new(FakeHost {
            delay: Duration::from_millis(delay_ms),
            until_cancel: false,
            reply: Box::new(reply),
            requests: Mutex::default(),
            active: AtomicUsize::new(0),
            max_active: AtomicUsize::new(0),
            saw_cancel: AtomicBool::new(false),
        })
    }
}

impl WebviewHost for FakeHost {
    fn open(&self, request: WebviewRequest) -> Result<WebviewOutcome, ExtError> {
        let now = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_active.fetch_max(now, Ordering::SeqCst);
        self.requests.lock().unwrap().push(request.clone());
        let start = Instant::now();
        while (self.until_cancel || start.elapsed() < self.delay) && start.elapsed() < Duration::from_secs(5) {
            if request.cancel.load(Ordering::SeqCst) {
                self.saw_cancel.store(true, Ordering::SeqCst);
                self.active.fetch_sub(1, Ordering::SeqCst);
                return Err(ExtError::Cancelled);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        self.active.fetch_sub(1, Ordering::SeqCst);
        Ok((self.reply)(&request))
    }
}

fn manifest(id: &str, permission: bool, concurrency: usize) -> String {
    json!({ "id": id, "name": "Web", "version": "1.0.0", "apiVersion": 1, "description": "d", "developer": { "name": "dev" },
        "match": ["^http"], "hosts": ["127.0.0.1", "demo.test"], "permissions": if permission { json!(["webview"]) } else { json!([]) },
        "limits": { "concurrency": concurrency } })
    .to_string()
}

fn outcome(url: &str, cookies: Vec<WebviewCookie>) -> WebviewOutcome {
    WebviewOutcome {
        url: url.into(),
        cookies,
        local_storage: serde_json::from_str::<Map<String, Value>>(r#"{ "token": "tk", "khac": "x" }"#).unwrap(),
        session_storage: Map::new(),
        html: Some("<html><body>ok</body></html>".into()),
        closed_by: ClosedBy::Until,
    }
}

fn cookie(name: &str, domain: &str) -> WebviewCookie {
    WebviewCookie { name: name.into(), value: "abc".into(), domain: domain.into(), path: "/".into(), expires: None, http_only: true, secure: false }
}

fn opts(host: Option<Arc<FakeHost>>, store: Option<ExtStore>) -> SessionOptions {
    SessionOptions {
        allow_local: true,
        store,
        webview: host.map(|h| h as Arc<dyn WebviewHost>),
        engine: EngineOptions::default(),
        ..Default::default()
    }
}

fn title(session: &Session, ctx: &Value) -> Value {
    serde_json::from_str(&session.toc(ctx).unwrap().remove(0).title).unwrap()
}

const OPEN_MAIN: &str = r#"
    export async function toc(ctx) {
      try {
        const r = await webview.open(ctx.options);
        let checked = null;
        if (ctx.check) checked = (await (await http.get(ctx.check)).text()).includes("sid=abc");
        return [{ id: "1", title: JSON.stringify({ r, checked }) }];
      } catch (e) {
        return [{ id: "1", title: JSON.stringify({ error: e.name, message: e.message }) }];
      }
    }
    export async function chapter() { return { content: "x" }; }"#;

#[test]
fn thieu_quyen_khong_co_host_va_ngoai_hosts() {
    let host = FakeHost::new(0, |_| outcome("https://demo.test/", vec![]));
    let m = manifest("io.github.dev.web1", false, 1);
    let files = [("manifest.json", m.as_str()), ("main.js", OPEN_MAIN)];
    let ctx = json!({ "options": { "url": "https://demo.test/login" } });
    let r = title(&Session::start(package(&files), opts(Some(host.clone()), None)).unwrap(), &ctx);
    assert!(r["message"].as_str().unwrap().contains("quyền \"webview\""), "{r}");

    let m = manifest("io.github.dev.web1", true, 1);
    let files = [("manifest.json", m.as_str()), ("main.js", OPEN_MAIN)];
    let r = title(&Session::start(package(&files), opts(None, None)).unwrap(), &ctx);
    assert_eq!(r["error"], json!("NeedsInteraction"), "{r}");

    let session = Session::start(package(&files), opts(Some(host.clone()), None)).unwrap();
    let r = title(&session, &json!({ "options": { "url": "https://evil.test/login" } }));
    assert!(r["message"].as_str().unwrap().contains("evil.test"), "{r}");
    let r = title(&session, &json!({ "options": { "url": "https://demo.test/", "until": { "url": "(" } } }));
    assert_eq!(r["error"], json!("TypeError"), "{r}");
    assert!(host.requests.lock().unwrap().is_empty(), "bị chặn trước khi mở cửa sổ");
}

#[test]
fn loc_ket_qua_cookie_vao_hu_http_va_giu_qua_luot() {
    let server = TestServer::start(vec![Route::echo("/check")]);
    let dir = tempfile::tempdir().unwrap();
    let store = ExtStore::with_backend(dir.path(), "io.github.dev.web2", SecretBackend::memory());
    let host = FakeHost::new(0, |_| outcome("https://demo.test/home", vec![cookie("sid", "127.0.0.1"), cookie("ga", ".google.com"), cookie("p", ".demo.test")]));
    let m = manifest("io.github.dev.web2", true, 1);
    let files = [("manifest.json", m.as_str()), ("main.js", OPEN_MAIN)];
    let ctx = json!({
        "check": server.url("/check"),
        "options": { "url": "https://demo.test/login", "title": "Đăng nhập", "until": { "cookie": "sid", "url": "^https://demo\\.test/home" },
                     "collect": { "localStorage": ["token"], "html": true } }
    });
    let r = title(&Session::start(package(&files), opts(Some(host.clone()), Some(store.clone()))).unwrap(), &ctx);
    assert_eq!(r["r"]["url"], json!("https://demo.test/home"));
    assert_eq!(r["r"]["cookies"].as_array().unwrap().iter().map(|c| c["name"].as_str().unwrap()).collect::<Vec<_>>(), ["sid", "p"]);
    assert_eq!(r["r"]["localStorage"], json!({ "token": "tk" }));
    assert_eq!(r["r"]["html"], json!("<html><body>ok</body></html>"));
    assert_eq!(r["r"]["closedBy"], json!("until"));
    assert_eq!(r["checked"], json!(true), "cookie lấy từ webview được gửi kèm http");

    let sent = host.requests.lock().unwrap()[0].clone();
    assert_eq!(sent.window_title(None), "Đăng nhập — Web · demo.test");
    assert_eq!(sent.profile_dir, Some(store.webview_dir()));
    assert!(sent.visible);

    let again = json!({ "check": server.url("/check"), "options": { "url": "https://demo.test/", "collect": { "cookies": false } } });
    let host2 = FakeHost::new(0, |_| outcome("https://other.test/", vec![cookie("x", "127.0.0.1")]));
    let r = title(&Session::start(package(&files), opts(Some(host2), Some(store.clone()))).unwrap(), &again);
    assert_eq!(r["checked"], json!(true), "cookie còn ở lượt sau");
    assert_eq!((r["r"]["url"].clone(), r["r"]["cookies"].clone(), r["r"]["localStorage"].clone(), r["r"]["html"].clone()), (Value::Null, json!([]), json!({}), Value::Null), "URL cuối ngoài hosts");
}

#[test]
fn moi_extension_toi_da_hai_cua_so_an() {
    let host = FakeHost::new(150, |_| outcome("https://demo.test/", vec![]));
    let m = manifest("io.github.dev.web3", true, 5);
    let main = r#"
        export async function toc() { return []; }
        export async function chapter(ctx, ch) {
          const r = await webview.open({ url: "https://demo.test/c/" + ch.id, visible: false });
          return { content: r.closedBy };
        }"#;
    let session = Session::start(package(&[("manifest.json", m.as_str()), ("main.js", main)]), opts(Some(host.clone()), None)).unwrap();
    let items: Vec<(usize, TocItem)> = (1..=5).map(|i| (i, TocItem { id: i.to_string(), title: i.to_string(), locked: false, volume: None, data: None })).collect();
    let mut done = Vec::new();
    session.chapters(&json!({}), &items, &mut |o| done.push(o.result.map(|c| c.content)));
    assert_eq!(done.len(), 5);
    assert!(done.iter().all(|r| r.as_deref() == Ok("until")), "{done:?}");
    assert_eq!(host.max_active.load(Ordering::SeqCst), 2);
}

#[test]
fn cho_nguoi_dung_khong_tinh_vao_gioi_han_login() {
    let main = r#"
        export async function login(ctx) {
          await webview.open({ url: "https://demo.test/login", visible: ctx.visible });
          return { token: "t" };
        }
        export async function toc(ctx) { return [{ id: "1", title: ctx.session.token }]; }
        export async function chapter() { return { content: "x" }; }"#;
    let m = manifest("io.github.dev.web4", true, 1);
    let files = [("manifest.json", m.as_str()), ("main.js", main)];
    let host = FakeHost::new(600, |_| outcome("https://demo.test/", vec![]));
    let mut o = opts(Some(host.clone()), None);
    o.login_limit = Duration::from_millis(200);
    let session = Session::start(package(&files), o.clone()).unwrap();
    assert_eq!(session.toc(&json!({ "visible": true })).unwrap()[0].title, "t", "cửa sổ hiện 600 ms, login giới hạn 200 ms");

    let session = Session::start(package(&files), o).unwrap();
    assert_eq!(session.toc(&json!({ "visible": false })), Err(ExtError::LoginFailed("login chạy quá thời gian cho phép".into())), "cửa sổ ẩn vẫn tính giờ");
}

#[test]
fn huy_luot_thi_dong_cua_so() {
    let mut host = FakeHost::new(0, |_| outcome("https://demo.test/", vec![]));
    Arc::get_mut(&mut host).unwrap().until_cancel = true;
    let m = manifest("io.github.dev.web5", true, 1);
    let files = [("manifest.json", m.as_str()), ("main.js", OPEN_MAIN)];
    let session = Arc::new(Session::start(package(&files), opts(Some(host.clone()), None)).unwrap());
    let runner = session.clone();
    let job = std::thread::spawn(move || runner.toc(&json!({ "options": { "url": "https://demo.test/login" } })));
    std::thread::sleep(Duration::from_millis(200));
    session.cancel();
    assert_eq!(job.join().unwrap(), Err(ExtError::Cancelled));
    let start = Instant::now();
    while !host.saw_cancel.load(Ordering::SeqCst) && start.elapsed() < Duration::from_secs(2) {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(host.saw_cancel.load(Ordering::SeqCst), "host thấy cờ huỷ và đóng cửa sổ");
}
