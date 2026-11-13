# Tải raw — mốc 4: webview

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extension có quyền `webview` mở được cửa sổ trình duyệt riêng (hiện để người dùng đăng nhập / giải captcha, hoặc ẩn để trang chạy JS), app tự lấy cookie, localStorage, sessionStorage, HTML rồi trả cho extension.

**Architecture:** `qt-ext::webview` lo phần không phụ thuộc Tauri: đọc và kiểm tuỳ chọn, quyền `webview`, domain trong `hosts`, giới hạn số cửa sổ mỗi extension, lọc kết quả theo `hosts`, nạp cookie vào hũ cookie của `http`. Phần mở cửa sổ là trait `WebviewHost` (gọi chặn, chạy trên blocking pool); app cài bằng `WebviewWindowBuilder` của Tauri với hồ sơ riêng (`data_directory`) cho từng extension. Không có host (CLI về sau) thì `webview.open` ném `NeedsInteraction`.

**Tech Stack:** Rust (qt-ext, qt-ai-core, Tauri 2.11: `data_directory`, `on_navigation`, `on_new_window`, `on_page_load`, `cookies`, `eval_with_callback`).

Spec: [docs/superpowers/specs/2026-11-07-tai-raw-extension-design.md](../specs/2026-11-07-tai-raw-extension-design.md) mục 7, 8, 9.

## Global Constraints

- Không chạy `cargo fmt`. Không đưa tên app đọc truyện bên thứ ba vào repo (grep trước khi commit).
- Không động vào file untracked của task khác.
- Extension không chèn được script vào trang; script lấy dữ liệu là script cố định của app.
- Cửa sổ nhãn `ext-<id>-<n>` (dấu chấm trong id đổi thành `_`), không thuộc capability nào. Tauri chặn origin từ xa gọi lệnh của app khi không có capability `remote`.
- Điều hướng khung chính chỉ tới domain trong `hosts` (cùng `HostPolicy` với `http`); popup bị chặn, link popup thuộc `hosts` thì mở ngay trong cửa sổ đó.
- `url` và mọi thứ `collect` trả về chỉ thuộc `hosts`; cookie của domain khác bị lọc.
- Thanh tiêu đề: `<title> — <tên extension> · <domain đang mở>`.
- Mỗi extension 1 cửa sổ hiện và 2 cửa sổ ẩn cùng lúc (giữa mọi lượt tải); quá thì chờ.
- `timeout` mặc định 300 giây, tối đa 900. Thời gian chờ người dùng ở cửa sổ hiện không tính vào giới hạn của `toc` / `chapter` / `login`.
- Hồ sơ `<config>/extension-data/<id>/webview/`; "Đăng xuất" và "Xoá auth" xoá thư mục này.
- Build với `CARGO_INCREMENTAL=0`.

## Quy ước API (bổ sung spec mục 8)

- `visible` mặc định `true`.
- `until`: khớp một điều kiện là đóng. `url` là regex trên URL hiện tại, `cookie` là tên cookie (thuộc `hosts`), `selector` là CSS có phần tử trong trang. Chỉ kiểm sau khi trang tải xong lần đầu.
- Cửa sổ ẩn không có `until` thì đóng khi trang tải xong: `closedBy: "load"`.
- `collect` mặc định `{ cookies: true }`. `localStorage` / `sessionStorage`: `true` = mọi khoá, mảng = các khoá đó, `false` = không lấy.
- Cookie lấy được cũng nạp vào hũ cookie của `http` (lưu trong session data), nên request sau đó gửi kèm luôn.
- Người dùng đóng cửa sổ hiện = xong (`closedBy: "user"`): app chặn lệnh đóng, lấy dữ liệu rồi mới đóng.
- Lượt tải bị huỷ thì cửa sổ đóng ngay, `webview.open` ném lỗi huỷ.

---

### Task 1: `qt-ext::webview` — tuỳ chọn, giới hạn, lọc, API JS

**Files:** Create `crates/qt-ext/src/webview.rs`, `crates/qt-ext/src/engine/bind_webview.rs`, `crates/qt-ext/tests/webview.rs`; Modify `src/lib.rs`, `src/engine/mod.rs` (`Stores.webview`, giới hạn thời gian cộng thời gian chờ cửa sổ hiện), `src/engine/prelude.js` (không đổi API công khai khác), `src/session.rs` (`SessionOptions.webview`), `src/cookies.rs` (`Jar::insert`), `src/store/mod.rs` (`webview_dir()`, `logout` xoá hồ sơ).

**Interfaces (Produces):**
- `pub enum StorageKeys { None, All, Keys(Vec<String>) }`, `pub struct Collect { cookies, local_storage, session_storage, html }`, `pub struct Until { url: Option<Regex>, cookie: Option<String>, selector: Option<String> }`.
- `pub struct WebviewRequest { extension_id, extension_name, url: Url, title, visible, until, timeout: Duration, collect, policy: HostPolicy, profile_dir: Option<PathBuf>, cancel: Arc<AtomicBool> }` + `fn allows(&self, url: &Url) -> bool`.
- `pub struct WebviewCookie { name, value, domain, path, expires: Option<i64>, http_only, secure }`.
- `pub enum ClosedBy { Until, User, Timeout, Load }`.
- `pub struct WebviewOutcome { url: String, cookies, local_storage: Map, session_storage: Map, html: Option<String>, closed_by }` (dữ liệu thô từ host; qt-ext lọc).
- `pub trait WebviewHost: Send + Sync { fn open(&self, request: WebviewRequest) -> Result<WebviewOutcome, ExtError>; }` — gọi chặn, chạy trong `spawn_blocking`.
- `SessionOptions.webview: Option<Arc<dyn WebviewHost>>`.

Test (host giả): thiếu quyền `webview` → lỗi; không có host → `NeedsInteraction`; `url` ngoài `hosts` → bị chặn trước khi mở; kết quả bị lọc (cookie domain lạ, URL cuối ngoài `hosts` thì `url` null và không có storage/html); cookie lấy được gửi kèm request `http` sau đó và còn ở lượt sau; 3 cửa sổ ẩn song song thì tối đa 2 cùng lúc; `login` giới hạn 200 ms vẫn xong khi cửa sổ hiện mở 600 ms; huỷ lượt thì host thấy cờ huỷ; tuỳ chọn sai (`until.url` không phải regex, `timeout` quá 900 bị chặn trần).

Commit: `feat(ext): webview.open cho extension — quyền, hosts, giới hạn cửa sổ, lọc kết quả`.

### Task 2: Host Tauri

**Files:** Create `apps/qt-ai-gui/src-tauri/src/ext_webview.rs`; Modify `raw_cmds.rs`, `ext_cmds.rs` (gắn host vào `RunOptions`), `lib.rs`.

- `TauriWebviews<R>` cài `WebviewHost`: dựng `WebviewWindowBuilder` từ thread blocking (không phải main thread), `data_directory` = hồ sơ của extension (không có thì `incognito`), `on_navigation` kiểm `hosts`, `on_new_window` chặn popup, `on_page_load` đổi tiêu đề và bật cờ đã tải, `CloseRequested` chặn đóng rồi bật cờ "người dùng xong".
- Vòng kiểm 400 ms: huỷ, người dùng đóng, hết giờ, `until` (URL hiện tại, `cookies()`, `eval_with_callback` kiểm selector), cửa sổ ẩn không `until` thì đóng khi tải xong.
- Lấy dữ liệu: `cookies()`, một lần `eval_with_callback` script cố định trả JSON `{ local, session, html }`; xong thì `destroy()`.
- Tự kiểm (chỉ bản debug): biến môi trường `QT_EXT_WEBVIEW_SELFTEST=<url>` mở cửa sổ ẩn tới URL đó, ghi kết quả ra file, rồi thoát. Dùng để kiểm `data_directory`, cookie, localStorage, selector trên WebView2 thật với máy chủ giả.

Kiểm: `cargo clippy -p qt-ai-gui --all-targets`; chạy tự kiểm với máy chủ giả trên máy.

Commit: `feat(gui): mở webview cho extension — hồ sơ riêng, chặn điều hướng ngoài hosts, lấy cookie và storage`.

### Task 3: Tài liệu thay đổi

**Files:** Modify `apps/qt-ai-gui/CHANGELOG.md`.

Commit: `docs(gui): CHANGELOG — webview cho extension`.

## Kiểm tay (spec mục 12)

- Đăng nhập thật trên một trang có form đăng nhập; `until.cookie` đóng cửa sổ đúng lúc.
- `collect` trả cookie, localStorage đúng khoá yêu cầu; cookie domain ngoài `hosts` không có.
- Trong cửa sổ extension, `window.__TAURI_INTERNALS__.invoke("raw_info", …)` bị từ chối.
- Bấm link tới domain ngoài `hosts`: không đi.
- Đăng xuất xoá hồ sơ: mở lại cửa sổ thì phải đăng nhập lại.
