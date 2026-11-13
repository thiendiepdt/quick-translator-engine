# Tải raw — mốc 1: crate `qt-ext` (sandbox JS) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Crate `crates/qt-ext` chạy được `toc` và `chapter` của một extension trong sandbox QuickJS, với API `http`, `html`, `crypto`, `encoding`, `log`, `sleep`, `progress`, đúng các giới hạn ở mục 6 và 9 của spec.

**Architecture:** Mỗi lượt (`Session`) chạy trên một thread riêng với tokio current-thread và một `AsyncRuntime` của `rquickjs` (không `Send`). Caller gọi đồng bộ qua channel. Module JS nạp từ bộ nhớ (file trong gói đọc sẵn lúc nạp gói), nên extension không có đường đọc ổ đĩa. Mọi request mạng đi qua `HostPolicy` ở từng bước redirect.

**Tech Stack:** Rust 2021, `rquickjs` 0.14 (`futures`, `loader`, `macro`, `classes`), tokio 1 (rt, time), reqwest 0.13 (async, rustls), scraper 0.29, encoding_rs + chardetng, RustCrypto (md-5, sha1, sha2, hmac, aes, cbc, ecb, aes-gcm), base64 0.22, hex, getrandom 0.3, futures-util.

Spec: [docs/superpowers/specs/2026-11-07-tai-raw-extension-design.md](../specs/2026-11-07-tai-raw-extension-design.md)

## Global Constraints

- Không chạy `cargo fmt`. Không đưa tên app đọc truyện bên thứ ba vào repo (file, code, comment, commit).
- `qt-ext` không phụ thuộc `qt-ai-core`; về sau core và app phụ thuộc `qt-ext`.
- RAM mỗi lượt 256 MB; stack 1 MB; JS chạy liên tục không chờ I/O 30 giây thì ngắt.
- Hạn: `toc` 300 giây, `chapter` 120 giây, `login` 120 giây.
- Phản hồi `http` tối đa 20 MB; redirect tối đa 10 bước, kiểm `hosts` ở từng bước.
- Chặn IP nội bộ, loopback, link-local, và đuôi `.localhost`, `.local`, `.internal`, `.intranet`, `.lan`, `.home`, `.corp`; `localhost` chỉ được khi `allow_local` (chế độ developer, test).
- `concurrency` mặc định 1, trần 8; `delayMs` mặc định 0. 429 chờ `Retry-After` tối đa 120 giây, tối đa 3 lần.
- `manifest.id` khớp `^[a-z0-9]+(\.[a-z0-9-]+)+$`; `version` là semver; `apiVersion` hiện chỉ có 1.
- Gói tối đa 5 MB, tối đa 200 file.
- Comment và thông báo lỗi viết tiếng Việt như phần còn lại của repo.

## File structure

```
crates/qt-ext/
  Cargo.toml
  src/lib.rs              khai mô-đun, re-export kiểu công khai
  src/error.rs            ExtError
  src/manifest.rs         Manifest + parse + kiểm hợp lệ
  src/policy.rs           HostPolicy: domain được phép, chặn nội bộ
  src/text.rs             đoán mã hoá, decode, encode
  src/net.rs              HttpClient async: redirect thủ công, giới hạn kích thước, giới hạn theo host, 429
  src/package.rs          Package: manifest + file JS trong bộ nhớ, nạp từ thư mục
  src/engine/mod.rs       dựng runtime (giới hạn, interrupt, loader), gọi hàm export, đổi lỗi JS → ExtError
  src/engine/loader.rs    Resolver/Loader từ bộ nhớ, chặn thoát gói
  src/engine/prelude.js   lớp lỗi, bọc http Response, đóng băng ctx
  src/engine/bind_http.rs
  src/engine/bind_html.rs
  src/engine/bind_crypto.rs
  src/engine/bind_misc.rs log, sleep, progress, encoding
  src/session.rs          Session: thread + channel; toc, chapters (song song, gộp, hạn giờ, huỷ)
  tests/common/mod.rs     máy chủ HTTP giả (std::net::TcpListener) + dựng gói tạm
  tests/fixtures/demo/    extension mẫu cho test (manifest.json, main.js, lib/parse.js)
  tests/*.rs
```

---

### Task 1: Khung crate, `ExtError`, `Manifest`

**Files:**
- Create: `crates/qt-ext/Cargo.toml`, `crates/qt-ext/src/lib.rs`, `crates/qt-ext/src/error.rs`, `crates/qt-ext/src/manifest.rs`
- Modify: `Cargo.toml` (workspace members thêm `crates/qt-ext`)

**Interfaces:**
- Produces:
  - `pub enum ExtError { Manifest(String), Package(String), HostDenied(String), Network(String), TooLarge(usize), Timeout(&'static str), Interrupted, OutOfMemory, Cancelled, Script(String), AuthExpired(String), Locked(String), NeedsInteraction(String), Extension { message: String, retry: bool }, BadResult(String) }` (thiserror, `Clone`, `PartialEq`).
  - `pub struct Manifest { id, name, version: semver::Version, api_version: u32, description, developer: Developer, license: Option<String>, homepage: Option<String>, matches: Vec<String>, hosts: Vec<String>, permissions: Vec<Permission>, auth: Option<AuthSpec>, settings: Vec<SettingField>, limits: Limits }`
  - `pub struct Limits { pub concurrency: usize, pub delay_ms: u64 }` (đã chặn trần 8).
  - `pub fn parse_manifest(json: &str, allow_local: bool) -> Result<Manifest, ExtError>`
  - `impl Manifest { pub fn matches_url(&self, url: &str) -> bool }`

- [ ] **Step 1: Test manifest hợp lệ và các lỗi** — `crates/qt-ext/src/manifest.rs` (mod tests):

```rust
const OK: &str = r#"{
  "id": "io.github.dev.demo", "name": "Demo", "version": "1.2.0", "apiVersion": 1,
  "description": "Mẫu", "developer": { "name": "dev", "url": "https://x.dev" },
  "match": ["^https://demo\\.test/book/\\d+"], "hosts": ["demo.test", "*.cdn.test"],
  "permissions": ["webview"],
  "auth": { "required": false, "schemes": [ { "id": "apiKey", "label": "API key",
    "fields": [ { "key": "apiKey", "label": "API key", "type": "secret" } ] } ] },
  "settings": [ { "key": "proxy", "label": "Proxy", "type": "text", "scope": "story" } ],
  "limits": { "concurrency": 20, "delayMs": 300 }
}"#;

#[test]
fn doc_manifest_day_du_va_chan_tran_concurrency() {
    let m = parse_manifest(OK, false).unwrap();
    assert_eq!(m.id, "io.github.dev.demo");
    assert_eq!(m.version.to_string(), "1.2.0");
    assert_eq!(m.limits.concurrency, 8);
    assert_eq!(m.limits.delay_ms, 300);
    assert!(m.matches_url("https://demo.test/book/12"));
    assert!(!m.matches_url("https://other.test/book/12"));
    assert_eq!(m.permissions, vec![Permission::Webview]);
}

#[test]
fn tu_choi_manifest_sai() {
    let bad = |patch: &str, needle: &str| {
        let mut v: serde_json::Value = serde_json::from_str(OK).unwrap();
        let p: serde_json::Value = serde_json::from_str(patch).unwrap();
        for (k, val) in p.as_object().unwrap() { v[k] = val.clone(); }
        let err = parse_manifest(&v.to_string(), false).unwrap_err().to_string();
        assert!(err.contains(needle), "{patch} → {err}");
    };
    bad(r#"{"id": "Demo"}"#, "id");
    bad(r#"{"version": "1.2"}"#, "version");
    bad(r#"{"apiVersion": 2}"#, "app mới hơn");
    bad(r#"{"match": ["("]}"#, "match");
    bad(r#"{"hosts": ["http://demo.test"]}"#, "hosts");
    bad(r#"{"hosts": ["127.0.0.1"]}"#, "hosts");
    bad(r#"{"hosts": ["localhost"]}"#, "hosts");
    bad(r#"{"permissions": ["fs"]}"#, "permissions");
    bad(r#"{"settings": [{"key":"k","label":"K","type":"secret","scope":"story"}]}"#, "secret");
    bad(r#"{"developer": {"name": ""}}"#, "developer");
}

#[test]
fn localhost_chi_duoc_khi_allow_local() {
    let mut v: serde_json::Value = serde_json::from_str(OK).unwrap();
    v["hosts"] = serde_json::json!(["localhost", "127.0.0.1"]);
    assert!(parse_manifest(&v.to_string(), true).is_ok());
}
```

- [ ] **Step 2: Chạy, thấy lỗi biên dịch** — `cargo test -p qt-ext` → FAIL (chưa có crate).

- [ ] **Step 3: Viết `Cargo.toml`, `error.rs`, `manifest.rs`**

`Cargo.toml` của crate:

```toml
[package]
name = "qt-ext"
version = "0.1.0"
edition = "2021"
license = "GPL-3.0-or-later"

[dependencies]
rquickjs = { version = "0.14", features = ["futures", "loader", "macro", "classes"] }
tokio = { version = "1", features = ["rt", "time", "sync", "macros"] }
futures-util = "0.3"
reqwest = { version = "0.13", default-features = false, features = ["rustls-no-provider", "cookies", "gzip", "brotli", "deflate"] }
rustls = { version = "0.23", default-features = false, features = ["ring", "std", "tls12", "logging"] }
url = "2"
serde = { version = "1", features = ["derive"] }
serde_json = { version = "1", features = ["preserve_order"] }
thiserror = "2"
regex = "1"
semver = "1"
scraper = "0.29"
encoding_rs = "0.8"
chardetng = "1"
md-5 = "0.10"
sha1 = "0.10"
sha2 = "0.10"
hmac = "0.12"
aes = "0.8"
cbc = { version = "0.1", features = ["alloc"] }
ecb = { version = "0.1", features = ["alloc"] }
aes-gcm = "0.10"
base64 = "0.22"
hex = "0.4"
getrandom = "0.3"

[dev-dependencies]
tempfile = "3"
```

`manifest.rs`: serde struct thô (`RawManifest`, camelCase) → hàm `parse_manifest` kiểm từng trường theo bảng mục 4 của spec, trả thông báo tiếng Việt có tên trường (`"id …"`, `"version …"`, `"apiVersion 2: cần app mới hơn"`, `"match[0] …"`, `"hosts[0] …"`, `"permissions …"`, `"settings[0]: trường secret không được scope story"`, `"developer.name …"`). Host hợp lệ: chữ thường, `*.` chỉ ở đầu, không `://`, không `:`, không phải IP (`ip.parse::<IpAddr>()`), không phải `localhost`/đuôi nội bộ trừ khi `allow_local`. `concurrency = raw.unwrap_or(1).clamp(1, 8)`. `matches_url` biên dịch `regex::Regex` một lần (lưu `Vec<Regex>` trong field `#[serde(skip)]`).

- [ ] **Step 4: Chạy test** — `cargo test -p qt-ext manifest` → PASS.
- [ ] **Step 5: Commit** — `feat(qt-ext): khung crate, manifest extension có kiểm hợp lệ`.

---

### Task 2: `HostPolicy`

**Files:** Create `crates/qt-ext/src/policy.rs`

**Interfaces:**
- Produces: `pub struct HostPolicy { patterns: Vec<String>, allow_local: bool }`, `HostPolicy::new(hosts: &[String], allow_local: bool)`, `pub fn check(&self, url: &url::Url) -> Result<(), ExtError>` (sai thì `ExtError::HostDenied(host)`).

- [ ] **Step 1: Test**

```rust
#[test]
fn chi_cho_host_da_khai() {
    let p = HostPolicy::new(&["demo.test".into(), "*.cdn.test".into()], false);
    let ok = |u: &str| p.check(&url::Url::parse(u).unwrap()).is_ok();
    assert!(ok("https://demo.test/a"));
    assert!(ok("http://demo.test/a"));
    assert!(ok("https://img.cdn.test/a.jpg"));
    assert!(ok("https://a.b.cdn.test/x"));
    assert!(!ok("https://cdn.test/x"), "*. không gồm chính tên gốc");
    assert!(!ok("https://www.demo.test/a"), "không tự gồm www");
    assert!(!ok("https://evil.test/a"));
    assert!(!ok("https://demo.test.evil.com/a"));
    assert!(!ok("ftp://demo.test/a"));
    assert!(ok("https://user:pw@demo.test/a"), "userinfo không đổi host");
}

#[test]
fn chan_noi_bo_tru_khi_allow_local() {
    let p = HostPolicy::new(&["localhost".into(), "127.0.0.1".into(), "box.lan".into()], false);
    for u in ["http://localhost:8080/", "http://127.0.0.1/", "http://box.lan/", "http://10.0.0.2/", "http://[::1]/", "http://169.254.1.1/"] {
        assert!(p.check(&url::Url::parse(u).unwrap()).is_err(), "{u}");
    }
    let dev = HostPolicy::new(&["localhost".into(), "127.0.0.1".into()], true);
    assert!(dev.check(&url::Url::parse("http://127.0.0.1:9000/x").unwrap()).is_ok());
    assert!(dev.check(&url::Url::parse("http://localhost:9000/x").unwrap()).is_ok());
}
```

- [ ] **Step 2:** `cargo test -p qt-ext policy` → FAIL.
- [ ] **Step 3: Viết `policy.rs`** — scheme chỉ `http`/`https`; lấy `url.host()`; `Host::Ipv4/Ipv6` → chặn nếu private/loopback/link-local/unspecified/broadcast/unique-local (IPv6 `fc00::/7`, `fe80::/10`) trừ khi `allow_local` và IP đó có trong `patterns`; `Host::Domain` → chặn nếu `localhost` hoặc có đuôi nội bộ (trừ `allow_local`); rồi khớp: bằng đúng pattern, hoặc pattern `*.x` và host kết thúc bằng `.x`.
- [ ] **Step 4:** `cargo test -p qt-ext policy` → PASS.
- [ ] **Step 5: Commit** — `feat(qt-ext): HostPolicy — chỉ domain đã khai, chặn mạng nội bộ`.

---

### Task 3: Mã hoá văn bản

**Files:** Create `crates/qt-ext/src/text.rs`

**Interfaces:**
- Produces: `pub fn decode(bytes: &[u8], label: Option<&str>, header_charset: Option<&str>) -> Result<String, ExtError>` (label `"auto"` hoặc `None` = đoán: header → `<meta charset>` trong 2 KB đầu → BOM → chardetng); `pub fn encode(text: &str, label: &str) -> Result<Vec<u8>, ExtError>`; `pub fn charset_from_content_type(ct: &str) -> Option<String>`.

- [ ] **Step 1: Test**

```rust
#[test]
fn doan_ma_hoa_theo_thu_tu() {
    let (gbk, _, _) = encoding_rs::GBK.encode("第一章 天下");
    assert_eq!(decode(&gbk, None, Some("gbk")).unwrap(), "第一章 天下");
    let page = [b"<html><head><meta charset=\"gbk\"></head><body>".as_slice(), &gbk, b"</body>"].concat();
    assert!(decode(&page, None, None).unwrap().contains("第一章 天下"));
    assert_eq!(decode("天下".as_bytes(), None, None).unwrap(), "天下");
    assert_eq!(decode(&gbk, Some("gb18030"), Some("utf-8")).unwrap(), "第一章 天下", "label ép thắng header");
    let long = "他走进了房间，看见桌上放着一封信。".repeat(20);
    let (g, _, _) = encoding_rs::GBK.encode(&long);
    assert_eq!(decode(&g, Some("auto"), None).unwrap(), long, "chardetng");
    assert!(decode(b"x", Some("klingon"), None).is_err());
}

#[test]
fn encode_va_content_type() {
    assert_eq!(encode("天", "gbk").unwrap(), vec![0xcc, 0xec]);
    assert_eq!(charset_from_content_type("text/html; charset=GBK").as_deref(), Some("gbk"));
    assert_eq!(charset_from_content_type("text/html"), None);
}
```

- [ ] **Step 2:** FAIL. **Step 3:** viết bằng `encoding_rs::Encoding::for_label`, regex `(?i)<meta[^>]+charset=["']?([\w-]+)` trên 2048 byte đầu (lossy ASCII), `Encoding::for_bom`, `chardetng::EncodingDetector`. **Step 4:** PASS.
- [ ] **Step 5: Commit** — `feat(qt-ext): đoán và giải mã GBK/Big5/UTF-8 cho trang nguồn`.

---

### Task 4: `HttpClient`

**Files:** Create `crates/qt-ext/src/net.rs`, `crates/qt-ext/tests/common/mod.rs`, `crates/qt-ext/tests/net.rs`

**Interfaces:**
- Produces:
  - `pub struct HttpRequest { pub method: String, pub url: String, pub headers: Vec<(String, String)>, pub query: Vec<(String, String)>, pub body: Option<Vec<u8>>, pub timeout_ms: Option<u64> }`
  - `pub struct HttpResponse { pub status: u16, pub url: String, pub headers: Vec<(String, String)>, pub body: Vec<u8>, pub charset: Option<String> }`
  - `pub struct HttpClient` — `HttpClient::new(policy: HostPolicy, limits: Limits) -> Result<Self, ExtError>`; `pub async fn send(&self, req: HttpRequest) -> Result<HttpResponse, ExtError>`.
  - `tests/common::TestServer` — `TestServer::start(routes: Vec<Route>) -> TestServer` (`.url(path)`, `.hits(path)`), `Route { path, status, headers, body, delay_ms }`.

- [ ] **Step 1: Máy chủ giả** `tests/common/mod.rs`: `std::net::TcpListener` ở `127.0.0.1:0`, mỗi kết nối một thread, đọc request line + header + body (theo `Content-Length`), tra route theo path (bỏ query), ghi status, header, body, `Connection: close`; đếm số lần gọi mỗi path (`Arc<Mutex<HashMap>>`). Route có thể trả danh sách phản hồi theo lượt gọi (để mô phỏng 429 rồi 200).

- [ ] **Step 2: Test** `tests/net.rs`:

```rust
mod common;
use common::{Route, TestServer};
use qt_ext::{HostPolicy, HttpClient, HttpRequest, Limits, ExtError};

fn client(allow: &[&str]) -> HttpClient {
    let hosts: Vec<String> = allow.iter().map(|s| s.to_string()).collect();
    HttpClient::new(HostPolicy::new(&hosts, true), Limits { concurrency: 2, delay_ms: 0 }).unwrap()
}
fn get(url: String) -> HttpRequest { HttpRequest { method: "GET".into(), url, headers: vec![], query: vec![], body: None, timeout_ms: None } }
fn rt() -> tokio::runtime::Runtime { tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap() }

#[test]
fn redirect_kiem_host_tung_buoc() {
    let s = TestServer::start(vec![
        Route::redirect("/a", "/b"), Route::ok("/b", "xong"),
        Route::redirect("/out", "http://evil.test/x"),
    ]);
    let c = client(&["127.0.0.1"]);
    let r = rt().block_on(c.send(get(s.url("/a")))).unwrap();
    assert_eq!((r.status, r.body.as_slice()), (200, b"xong".as_slice()));
    assert!(r.url.ends_with("/b"));
    let e = rt().block_on(c.send(get(s.url("/out")))).unwrap_err();
    assert!(matches!(e, ExtError::HostDenied(h) if h == "evil.test"));
}

#[test]
fn chan_host_ngoai_danh_sach_va_qua_kich_thuoc() {
    let s = TestServer::start(vec![Route::ok("/big", &"x".repeat(21 * 1024 * 1024))]);
    let c = client(&["127.0.0.1"]);
    assert!(matches!(rt().block_on(c.send(get("https://evil.test/".into()))), Err(ExtError::HostDenied(_))));
    assert!(matches!(rt().block_on(c.send(get(s.url("/big")))), Err(ExtError::TooLarge(_))));
}

#[test]
fn cho_429_theo_retry_after() {
    let s = TestServer::start(vec![Route::sequence("/r", vec![(429, vec![("Retry-After".into(), "1".into())], "".into()), (200, vec![], "ok".into())])]);
    let c = client(&["127.0.0.1"]);
    let t = std::time::Instant::now();
    let r = rt().block_on(c.send(get(s.url("/r")))).unwrap();
    assert_eq!(r.status, 200);
    assert!(t.elapsed().as_millis() >= 900);
    assert_eq!(s.hits("/r"), 2);
}

#[test]
fn query_header_body_va_charset() {
    let s = TestServer::start(vec![Route::echo("/echo")]);
    let c = client(&["127.0.0.1"]);
    let mut req = get(s.url("/echo"));
    req.method = "POST".into();
    req.query = vec![("q".into(), "天下".into())];
    req.headers = vec![("X-Test".into(), "1".into())];
    req.body = Some(b"abc".to_vec());
    let r = rt().block_on(c.send(req)).unwrap();
    let echo = String::from_utf8(r.body).unwrap();
    assert!(echo.contains("POST /echo?q=%E5%A4%A9%E4%B8%8B"));
    assert!(echo.to_lowercase().contains("x-test: 1"));
    assert!(echo.ends_with("abc"));
    assert_eq!(r.charset.as_deref(), Some("utf-8"));
}
```

- [ ] **Step 3:** `cargo test -p qt-ext --test net` → FAIL.
- [ ] **Step 4: Viết `net.rs`** — `reqwest::Client::builder().redirect(Policy::none()).cookie_store(true).user_agent(<UA desktop>).gzip(true).brotli(true).deflate(true).timeout(30s mặc định)`; cài crypto provider ring một lần (`OnceLock`, giống `qt-ai-core::api::install_crypto_provider`). `send`: parse URL, gắn query, `policy.check` trước mỗi bước; vòng redirect ≤ 10 (301/302/303 → GET bỏ body; 307/308 giữ method), `Location` ghép tương đối; đọc body bằng `chunk()` và cộng dồn, quá 20 MB → `TooLarge`; 429 → đọc `Retry-After` (giây, mặc định 5, trần 120), `tokio::time::sleep`, tối đa 3 lần; giới hạn theo host: `HashMap<host, (Arc<Semaphore>, Mutex<Instant lần cuối>)>`, giữ permit trong lúc gọi, chờ `delay_ms` kể từ lần gọi trước. Lỗi reqwest → `Network(msg)` (không kèm URL có query để tránh lộ khoá; chỉ host + path).
- [ ] **Step 5:** PASS. **Step 6: Commit** — `feat(qt-ext): HttpClient — redirect kiểm host từng bước, trần 20 MB, giới hạn theo host, chờ 429`.

---

### Task 5: `Package` và loader trong bộ nhớ

**Files:** Create `crates/qt-ext/src/package.rs`, `crates/qt-ext/src/engine/loader.rs`

**Interfaces:**
- Produces: `pub struct Package { pub manifest: Manifest, pub files: Arc<HashMap<String, String>>, pub icon: Option<Vec<u8>> }`; `Package::load_dir(dir: &Path, allow_local: bool) -> Result<Package, ExtError>` (đọc `manifest.json`, mọi `*.js` / `*.mjs` theo đường dẫn tương đối dùng `/`, `icon.png`; bỏ qua thư mục ẩn và `node_modules`; > 200 file hoặc > 5 MB tổng → `Package`; thiếu `main.js` → `Package`). `engine::loader::{MemResolver, MemLoader}` — `MemResolver::new(files)`; chỉ nhận đường dẫn tương đối `./`, `../` từ module khác và `main.js` ở gốc; không cho thoát gói, không cho tên trần.

- [ ] **Step 1: Test** (`package.rs` mod tests, dùng `tempfile::tempdir`): nạp thư mục có `manifest.json`, `main.js`, `lib/a.js`, `node_modules/x.js`, `.git/y.js` → `files` có đúng `main.js`, `lib/a.js`; thiếu `main.js` → lỗi; 201 file → lỗi. Test `normalize` của loader:

```rust
assert_eq!(normalize("", "main.js").as_deref(), Some("main.js"));
assert_eq!(normalize("main.js", "./lib/a.js").as_deref(), Some("lib/a.js"));
assert_eq!(normalize("lib/a.js", "../b.js").as_deref(), Some("b.js"));
assert_eq!(normalize("lib/a.js", "./c/../d.js").as_deref(), Some("lib/d.js"));
assert_eq!(normalize("main.js", "../x.js"), None);
assert_eq!(normalize("main.js", "fs"), None);
assert_eq!(normalize("main.js", "/etc/passwd"), None);
```

- [ ] **Step 2:** FAIL. **Step 3:** viết (`normalize` như bản thử ở scratchpad). **Step 4:** PASS.
- [ ] **Step 5: Commit** — `feat(qt-ext): nạp gói vào bộ nhớ, loader module không thoát gói`.

---

### Task 6: Engine — runtime, prelude, gọi hàm, đổi lỗi

**Files:** Create `crates/qt-ext/src/engine/mod.rs`, `crates/qt-ext/src/engine/prelude.js`, `crates/qt-ext/src/engine/bind_misc.rs`; Test `crates/qt-ext/tests/engine.rs`

**Interfaces:**
- Produces:
  - `pub struct EngineOptions { pub memory_limit: usize, pub cpu_slice: Duration, pub cancel: Arc<AtomicBool>, pub secrets: Vec<String>, pub log: Arc<dyn Fn(LogLevel, String) + Send + Sync>, pub progress: Arc<dyn Fn(u64, u64, Option<String>) + Send + Sync> }` (`Default`: 256 MB, 30 s).
  - `pub enum LogLevel { Info, Warn, Error }`
  - `pub(crate) struct Engine` — `Engine::new(package: &Package, http: Rc<HttpClient>, opts: EngineOptions) -> Result<Engine, ExtError>` (async); `pub(crate) async fn call(&self, name: &str, args: Vec<serde_json::Value>, limit: Duration) -> Result<serde_json::Value, ExtError>`; `pub(crate) fn has_export(&self, name: &str) -> bool`.
  - `pub fn redact(text: &str, secrets: &[String]) -> String` (thay mỗi secret dài ≥ 4 ký tự bằng `***`).
- Hành vi:
  - Interrupt handler trả `true` khi `cancel` bật, hoặc khi `now - last_activity > cpu_slice`; `last_activity` (`Rc<Cell<Instant>>`) cập nhật lúc bắt đầu `call` và lúc mỗi hàm async của app bắt đầu và xong.
  - `prelude.js` chạy như script toàn cục trước khi import `main.js`: định nghĩa `AuthExpired`, `Locked`, `NeedsInteraction`, `ExtensionError(message, { retry })` (đều `extends Error`, gán `name`), `deepFreeze`, các object `log`, `progress`, `sleep`, `http`, `html`, `crypto`, `encoding` gọi hàm `__host_*` do Rust gắn; sau cùng xoá `__host_*` khỏi `globalThis` (giữ tham chiếu trong closure) để extension không gọi thẳng.
  - `call`: tìm export, `json_parse` từng arg rồi `deepFreeze`, gọi, `tokio::time::timeout(limit, promise.into_future())`; kết quả `json_stringify` → `serde_json::Value` (`undefined` → `Null`).
  - Đổi lỗi: hết hạn → `Timeout(name)`; `cancel` bật → `Cancelled`; message bắt đầu `interrupted` → `Interrupted`; `out of memory` → `OutOfMemory`; exception có `name` là `AuthExpired`/`Locked`/`NeedsInteraction` → biến thể tương ứng; `ExtensionError` → `Extension { message, retry }`; khác → `Script(message + stack)`. Mọi message đi qua `redact`.

- [ ] **Step 1: Test** `tests/engine.rs` — dựng gói tạm bằng helper `common::package(&[("main.js", SRC)])` (manifest mặc định hosts `127.0.0.1`, allow_local) rồi chạy qua `qt_ext::testing::call(&pkg, "fn", args, opts)` (hàm `#[doc(hidden)] pub mod testing` trong lib dựng tokio current-thread + Engine + gọi một lần):

```rust
#[test]
fn goi_ham_export_tra_json_va_ctx_bi_dong_bang() {
    let pkg = common::package(&[("main.js", r#"
        import { twice } from "./lib/u.js";
        export async function toc(ctx) {
            let frozen = false;
            try { ctx.story.name = "x"; } catch (e) { frozen = true; }
            return [{ id: "1", title: ctx.story.name + twice(2), frozen }];
        }"#), ("lib/u.js", "export const twice = (x) => x * 2;")]);
    let v = qt_ext::testing::call(&pkg, "toc", vec![serde_json::json!({"story": {"name": "A"}})], Default::default()).unwrap();
    assert_eq!(v, serde_json::json!([{ "id": "1", "title": "A4", "frozen": true }]));
}

#[test]
fn doi_loi_cua_extension() {
    let pkg = common::package(&[("main.js", r#"
        export async function a() { throw new AuthExpired("hết phiên"); }
        export async function b() { throw new Locked("VIP"); }
        export async function c() { throw new ExtensionError("tạm lỗi", { retry: true }); }
        export async function d() { throw new NeedsInteraction("cần đăng nhập"); }
        export async function e() { null.x; }"#)]);
    let call = |f| qt_ext::testing::call(&pkg, f, vec![], Default::default()).unwrap_err();
    assert_eq!(call("a"), qt_ext::ExtError::AuthExpired("hết phiên".into()));
    assert_eq!(call("b"), qt_ext::ExtError::Locked("VIP".into()));
    assert_eq!(call("c"), qt_ext::ExtError::Extension { message: "tạm lỗi".into(), retry: true });
    assert_eq!(call("d"), qt_ext::ExtError::NeedsInteraction("cần đăng nhập".into()));
    assert!(matches!(call("e"), qt_ext::ExtError::Script(m) if m.contains("TypeError")));
}

#[test]
fn ngat_vong_lap_vo_han_va_vuot_ram() {
    let pkg = common::package(&[("main.js", r#"
        export async function spin() { while (true) {} }
        export async function hog() { const a = []; while (true) a.push(new Array(1e5).fill(1)); }"#)]);
    let mut opts = qt_ext::EngineOptions::default();
    opts.cpu_slice = std::time::Duration::from_millis(300);
    opts.memory_limit = 32 * 1024 * 1024;
    let t = std::time::Instant::now();
    assert_eq!(qt_ext::testing::call(&pkg, "spin", vec![], opts.clone()).unwrap_err(), qt_ext::ExtError::Interrupted);
    assert!(t.elapsed().as_millis() < 2000);
    assert_eq!(qt_ext::testing::call(&pkg, "hog", vec![], opts).unwrap_err(), qt_ext::ExtError::OutOfMemory);
}

#[test]
fn khong_co_api_he_thong_va_che_khoa_trong_log() {
    let pkg = common::package(&[("main.js", r#"
        export async function probe() {
            log.info("khoá là sk-SECRET-123");
            return [typeof require, typeof process, typeof std, typeof os, typeof __host_http, typeof fetch];
        }
        export async function boom() { throw new Error("lộ sk-SECRET-123"); }"#)]);
    let logs = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let sink = logs.clone();
    let mut opts = qt_ext::EngineOptions::default();
    opts.secrets = vec!["sk-SECRET-123".into()];
    opts.log = std::sync::Arc::new(move |_lvl, msg| sink.lock().unwrap().push(msg));
    let v = qt_ext::testing::call(&pkg, "probe", vec![], opts.clone()).unwrap();
    assert_eq!(v, serde_json::json!(["undefined"; 6]));
    assert_eq!(logs.lock().unwrap().as_slice(), ["khoá là ***"]);
    let e = qt_ext::testing::call(&pkg, "boom", vec![], opts).unwrap_err().to_string();
    assert!(e.contains("***") && !e.contains("SECRET"));
}

#[test]
fn het_han_va_huy() {
    let pkg = common::package(&[("main.js", "export async function slow() { await sleep(5000); return 1; }")]);
    let e = qt_ext::testing::call_with_limit(&pkg, "slow", vec![], Default::default(), std::time::Duration::from_millis(200)).unwrap_err();
    assert_eq!(e, qt_ext::ExtError::Timeout("call"));
}
```

(`Timeout` mang `&'static str`: engine đổi tên hàm sang hằng `"toc" | "chapter" | "chapters" | "login"`, tên khác thành `"call"`.)

- [ ] **Step 2:** FAIL. **Step 3:** viết `engine/mod.rs`, `prelude.js`, `bind_misc.rs` (log → `opts.log` sau `redact`, `progress`, `sleep` bằng `tokio::time::sleep`, cập nhật `last_activity`). **Step 4:** PASS.
- [ ] **Step 5: Commit** — `feat(qt-ext): engine QuickJS — giới hạn RAM/CPU, lỗi riêng của extension, che khoá`.

---

### Task 7: Bind `http`

**Files:** Create `crates/qt-ext/src/engine/bind_http.rs`; Modify `prelude.js`; Test `crates/qt-ext/tests/js_http.rs`

**Interfaces:**
- JS: `http.request({ method, url, headers, query, body, timeout, encoding })`, `http.get(url, opts?)`, `http.post(url, body, opts?)` → `Response { status, ok, url, headers (object, khoá chữ thường), text(label?), json(), bytes() }`. `body` là chuỗi, `Uint8Array`, hoặc object (đổi sang JSON, tự thêm `Content-Type: application/json` nếu chưa có).
- Rust: class `HttpBody` (`#[rquickjs::class]`) giữ `Rc<Vec<u8>>` + charset; method `text(label: Opt<String>)`, `bytes()` (trả `TypedArray<u8>`).

- [ ] **Step 1: Test** — server giả có `/gbk` (body GBK, header `text/html; charset=gbk`), `/json`, `/echo`; extension gọi `http.get`, `res.text()`, `res.json()`, `http.post(url, {a: 1})`, `http.get("https://evil.test")` trong `try/catch` trả `e.message`. Kỳ vọng: text đúng tiếng Trung, json đúng, echo có `content-type: application/json` và body `{"a":1}`, thông báo chặn có `evil.test`.
- [ ] **Step 2:** FAIL. **Step 3:** viết; hàm host `__host_http(optsJson: String, body: Opt<TypedArray<u8>>)` chép body ra `Vec<u8>` trước khi tạo future, future gọi `HttpClient::send`, trả `HttpResponseJs` (class) gồm status/url/headers/`HttpBody`. Lỗi trả về JS dưới dạng exception `Error` với message đã `redact`. **Step 4:** PASS.
- [ ] **Step 5: Commit** — `feat(qt-ext): API http cho extension`.

---

### Task 8: Bind `html`

**Files:** Create `crates/qt-ext/src/engine/bind_html.rs`; Test `crates/qt-ext/tests/js_html.rs`

**Interfaces:**
- JS: `html.parse(text)` → `HtmlNode` (nút gốc); `HtmlNode.select(css)` → mảng `HtmlNode`; `selectFirst(css)` → `HtmlNode | null`; `text()` (gộp text con, chuẩn hoá khoảng trắng), `html()` (inner), `outerHtml()`, `attr(name)` → chuỗi hoặc `null`, `children()` → mảng phần tử con.
- Rust: `#[rquickjs::class] struct HtmlNode { #[qjs(skip_trace)] doc: Rc<scraper::Html>, #[qjs(skip_trace)] id: ego_tree::NodeId }`. CSS sai → exception `Error("css không hợp lệ: …")`.

- [ ] **Step 1: Test**

```rust
let src = r#"export async function run() {
  const d = html.parse(`<div id="toc"><ul><li><a href="/c/1">第1章 起</a></li><li><a href="/c/2">第2章  承 </a></li></ul></div><p class="x">a<br>b</p>`);
  const links = d.select("#toc li a");
  let bad = null; try { d.select("##"); } catch (e) { bad = e.message; }
  return {
    n: links.length, first: links[0].text(), second: links[1].text(), href: links[1].attr("href"),
    none: links[0].attr("nope"), p: d.selectFirst("p.x").html(), miss: d.selectFirst(".none"),
    kids: d.selectFirst("ul").children().length, bad: bad !== null
  };
}"#;
// kỳ vọng
serde_json::json!({ "n": 2, "first": "第1章 起", "second": "第2章 承", "href": "/c/2", "none": null,
                    "p": "a<br>b", "miss": null, "kids": 2, "bad": true })
```

- [ ] **Step 2:** FAIL. **Step 3:** viết. **Step 4:** PASS.
- [ ] **Step 5: Commit** — `feat(qt-ext): API html (bóc HTML bằng CSS selector)`.

---

### Task 9: Bind `crypto` và `encoding`

**Files:** Create `crates/qt-ext/src/engine/bind_crypto.rs`; Modify `bind_misc.rs` (encoding); Test `crates/qt-ext/tests/js_crypto.rs`

**Interfaces (JS):**
- `crypto.md5(data)`, `crypto.sha1(data)`, `crypto.sha256(data)` → hex; `crypto.hmac("sha256" | "sha1" | "md5", key, data)` → hex; `data`/`key` là chuỗi (UTF-8) hoặc `Uint8Array`.
- `crypto.aes.encrypt({ mode: "cbc" | "ecb" | "gcm", key, iv?, data, padding?: "pkcs7" | "none" })` → `Uint8Array`; `crypto.aes.decrypt(...)` tương tự. Khoá 16/24/32 byte; `gcm` tag nối cuối dữ liệu mã hoá.
- `crypto.randomBytes(n)` → `Uint8Array` (n ≤ 1024).
- `encoding.decode(bytes, label)`, `encoding.encode(text, label)`, `encoding.base64.encode(bytes|string)`, `encoding.base64.decode(str)` → `Uint8Array`, `encoding.hex.encode`, `encoding.hex.decode`, `encoding.utf8.decode(bytes)`.

- [ ] **Step 1: Test** với vector chuẩn:

```js
return [
  crypto.md5("abc"),                       // 900150983cd24fb0d6963f7d28e17f72
  crypto.sha1("abc"),                      // a9993e364706816aba3e25717850c26c9cd0d89d
  crypto.sha256("abc"),                    // ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
  crypto.hmac("sha256", "key", "The quick brown fox jumps over the lazy dog"),
                                           // f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8
  encoding.hex.encode(crypto.aes.encrypt({ mode: "ecb", key: encoding.hex.decode("2b7e151628aed2a6abf7158809cf4f3c"),
                       data: encoding.hex.decode("6bc1bee22e409f96e93d7e117393172a"), padding: "none" })),
                                           // 3ad77bb40d7a3660a89ecaf32466ef97
  encoding.utf8.decode(crypto.aes.decrypt({ mode: "cbc", key: "0123456789abcdef", iv: "fedcba9876543210",
                       data: crypto.aes.encrypt({ mode: "cbc", key: "0123456789abcdef", iv: "fedcba9876543210", data: "第一章" }) })),
                                           // 第一章
  encoding.utf8.decode(crypto.aes.decrypt({ mode: "gcm", key: "0123456789abcdef", iv: "012345678901",
                       data: crypto.aes.encrypt({ mode: "gcm", key: "0123456789abcdef", iv: "012345678901", data: "天下" }) })),
                                           // 天下
  encoding.base64.encode("天"),            // 5aSp
  encoding.decode(encoding.encode("天下", "gbk"), "gbk"),   // 天下
  crypto.randomBytes(16).length,           // 16
];
```

- [ ] **Step 2:** FAIL. **Step 3:** viết. **Step 4:** PASS.
- [ ] **Step 5: Commit** — `feat(qt-ext): API crypto (md5, sha, hmac, AES) và encoding`.

---

### Task 10: `Session` — toc, chapters song song, gộp, huỷ

**Files:** Create `crates/qt-ext/src/session.rs`; Create `crates/qt-ext/tests/fixtures/demo/{manifest.json,main.js,lib/parse.js}`; Test `crates/qt-ext/tests/session.rs`

**Interfaces:**
- Produces:
  - `pub struct TocItem { pub id: String, pub title: String, pub locked: bool, pub volume: Option<String>, pub data: Option<serde_json::Value> }` (Serialize/Deserialize; `id` số được đổi sang chuỗi).
  - `pub enum ChapterFormat { Text, Html }`
  - `pub struct ChapterContent { pub content: String, pub title: Option<String>, pub format: ChapterFormat }`
  - `pub struct ChapterOutcome { pub id: String, pub index: usize, pub result: Result<ChapterContent, ExtError> }`
  - `pub struct Session` — `Session::start(package: Package, opts: SessionOptions) -> Result<Session, ExtError>`; `pub fn toc(&self, ctx: &serde_json::Value) -> Result<Vec<TocItem>, ExtError>`; `pub fn chapters(&self, ctx: &serde_json::Value, items: &[(usize, TocItem)], on_each: &mut dyn FnMut(ChapterOutcome))` (`index` là vị trí 1-based); `pub fn cancel(&self)`; `Drop` dừng thread.
  - `pub struct SessionOptions { pub engine: EngineOptions, pub allow_local: bool, pub toc_limit: Duration (300 s), pub chapter_limit: Duration (120 s) }`.
- Hành vi:
  - Thread riêng: tokio current-thread + `Engine`; lệnh qua `std::sync::mpsc`, kết quả từng chương gửi về ngay khi xong (caller ghi file luôn).
  - `toc`: kiểm kết quả là mảng, mỗi mục có `id` (chuỗi hoặc số) không rỗng, không trùng, `title` chuỗi → nếu sai `BadResult("toc[3]: …")`.
  - `chapters`: nếu extension export `chapters` thì gom theo nhóm 20 mục, gọi song song tối đa `limits.concurrency` nhóm; kết quả mảng `{id, content, title?, format?, error?}`; mục thiếu trong kết quả hoặc có `error` → `Extension { message, retry: true }`. Không có thì gọi `chapter(ctx, {...item, index})` song song tối đa `concurrency` (`FuturesUnordered`). Mỗi lệnh có hạn `chapter_limit` (nhóm thì `chapter_limit × số mục`, trần 10 phút).
  - Kiểm `ChapterContent`: `content` là chuỗi (rỗng vẫn trả về, việc coi trống là lỗi để mốc 2 quyết), `format` thiếu = `Text`, giá trị lạ → `BadResult`.
  - Huỷ: `cancel()` bật cờ; chương chưa chạy trả `Cancelled`; chương đang chạy bị ngắt.
  - Retry chưa làm ở đây (mốc 2 quyết theo spec 10.5).

- [ ] **Step 1: Extension mẫu** `tests/fixtures/demo/` — manifest hosts `127.0.0.1`, `limits.concurrency: 3`; `main.js`: `toc` tải 2 trang mục lục GBK (`/book/1/list?page=1`, `page=2`, trang 2 không có link "下一页"), bóc `#list a` bằng `lib/parse.js`, gắn `locked` khi có class `vip`, `data: { href }`; `chapter` tải `href`, lấy `#content` dạng html, trả `{ content, format: "html" }`; chương href `/c/err` ném `ExtensionError("hỏng", { retry: true })`.

- [ ] **Step 2: Test** `tests/session.rs`:

```rust
#[test]
fn toc_nhieu_trang_gbk_va_chapters_song_song() {
    let s = common::demo_server();       // trang mục lục GBK 2 trang, 6 chương, /c/3 chậm 400 ms, chương 5 vip, chương 6 /c/err
    let pkg = common::demo_package();    // tests/fixtures/demo, allow_local
    let session = qt_ext::Session::start(pkg, common::session_opts()).unwrap();
    let ctx = serde_json::json!({ "rawUrl": s.url("/book/1"), "story": { "name": "Demo", "current": null } });
    let toc = session.toc(&ctx).unwrap();
    assert_eq!(toc.iter().map(|t| t.title.as_str()).collect::<Vec<_>>(),
               ["第1章 起", "第2章 承", "第3章 转", "第4章 合", "第5章 VIP", "第6章 坏"]);
    assert!(toc[4].locked && !toc[0].locked);
    let items: Vec<(usize, qt_ext::TocItem)> = toc.iter().cloned().enumerate().map(|(i, t)| (i + 1, t)).collect();
    let mut got = Vec::new();
    let t = std::time::Instant::now();
    session.chapters(&ctx, &items[..4], &mut |o| got.push(o));
    assert!(t.elapsed().as_millis() < 1500, "song song 3 luồng");
    got.sort_by_key(|o| o.index);
    assert_eq!(got.len(), 4);
    let c1 = got[0].result.as_ref().unwrap();
    assert_eq!(c1.format, qt_ext::ChapterFormat::Html);
    assert!(c1.content.contains("天下大势"));
    let mut bad = Vec::new();
    session.chapters(&ctx, &items[5..6], &mut |o| bad.push(o));
    assert_eq!(bad[0].result, Err(qt_ext::ExtError::Extension { message: "hỏng".into(), retry: true }));
}

#[test]
fn toc_sai_dang_bi_tu_choi() {
    let pkg = common::package(&[("main.js", r#"export async function toc() { return [{ id: "1", title: "a" }, { id: "1", title: "b" }]; }
                                             export async function chapter() { return { content: "" }; }"#)]);
    let session = qt_ext::Session::start(pkg, common::session_opts()).unwrap();
    assert!(matches!(session.toc(&serde_json::json!({})), Err(qt_ext::ExtError::BadResult(m)) if m.contains("trùng")));
}

#[test]
fn ham_chapters_gop_nhom() {
    let pkg = common::package(&[("main.js", r#"
        globalThis.calls = 0;
        export async function toc() { return Array.from({ length: 45 }, (_, i) => ({ id: String(i + 1), title: "c" + (i + 1) })); }
        export async function chapter() { throw new Error("không được gọi"); }
        export async function chapters(ctx, list) {
            calls++;
            return list.filter(c => c.id !== "7").map(c => ({ id: c.id, content: "nội dung " + c.id + " nhóm " + calls }));
        }"#)]);
    let session = qt_ext::Session::start(pkg, common::session_opts()).unwrap();
    let toc = session.toc(&serde_json::json!({})).unwrap();
    let items: Vec<_> = toc.into_iter().enumerate().map(|(i, t)| (i + 1, t)).collect();
    let mut got = Vec::new();
    session.chapters(&serde_json::json!({}), &items, &mut |o| got.push(o));
    assert_eq!(got.len(), 45);
    assert_eq!(got.iter().filter(|o| o.result.is_err()).count(), 1, "mục 7 thiếu trong kết quả");
    let groups: std::collections::BTreeSet<String> = got.iter().filter_map(|o| o.result.as_ref().ok())
        .map(|c| c.content.rsplit(' ').next().unwrap().to_string()).collect();
    assert_eq!(groups.len(), 3, "45 mục = 3 nhóm 20/20/5");
}

#[test]
fn huy_giua_chung() {
    let pkg = common::package(&[("main.js", r#"
        export async function toc() { return Array.from({ length: 10 }, (_, i) => ({ id: String(i), title: "c" })); }
        export async function chapter(ctx, ch) { await sleep(300); return { content: "x" }; }"#)]);
    let session = std::sync::Arc::new(qt_ext::Session::start(pkg, common::session_opts()).unwrap());
    let toc = session.toc(&serde_json::json!({})).unwrap();
    let items: Vec<_> = toc.into_iter().enumerate().map(|(i, t)| (i + 1, t)).collect();
    let s2 = session.clone();
    std::thread::spawn(move || { std::thread::sleep(std::time::Duration::from_millis(450)); s2.cancel(); });
    let mut got = Vec::new();
    session.chapters(&serde_json::json!({}), &items, &mut |o| got.push(o));
    assert_eq!(got.len(), 10);
    assert!(got.iter().any(|o| o.result.is_ok()));
    assert!(got.iter().any(|o| o.result == Err(qt_ext::ExtError::Cancelled)));
}
```

- [ ] **Step 3:** FAIL. **Step 4:** viết `session.rs` + fixture. **Step 5:** PASS cả crate: `cargo test -p qt-ext`; `cargo clippy -p qt-ext --all-targets` không cảnh báo mới.
- [ ] **Step 6: Commit** — `feat(qt-ext): Session — toc, tải chương song song, hàm gộp, huỷ giữa chừng`.

---

## Sau mốc 1

Mốc 2 (`download.json`, ô Link tải raw, mẫu tên file, khớp chương, ghi file, đồng bộ `state.json`, trang Tải raw) có kế hoạch riêng, viết sau khi mốc 1 xong vì nó dùng `Session` và `ChapterOutcome` đúng như mốc 1 làm ra.
