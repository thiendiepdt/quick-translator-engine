# Tải raw — mốc 3: auth, cài đặt, cache, session data, login

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extension dùng được auth (API key, tài khoản…), cài đặt do extension khai (chung và theo truyện), `cache`, `session` có `ttl`, hàm `login` và lỗi `AuthExpired`; người dùng nhập auth và cài đặt ngay trên trang Tải raw.

**Architecture:** Phần lưu trữ của extension nằm ở `qt-ext::store` (crate này đã có HTTP và sandbox; CLI về sau dùng lại được): `ExtStore` (auth, cài đặt chung, xoá phiên, xoá cache) với khoá bí mật ở Windows Credential Manager (`keyring`), `Cache` trên SQLite, `SessionData` trên file JSON, hũ cookie lưu trong session data. `Session` tự nạp auth từ `ExtStore`, tự chèn `ctx.auth` và `ctx.session`, gọi `login` khi cần và xử lý `AuthExpired`. Core chỉ gộp cài đặt (mặc định → chung → truyện) vào `ctx.settings`. App thêm lệnh `ext_*` và form vẽ từ manifest.

**Tech Stack:** Rust (qt-ext, qt-ai-core, Tauri 2), `keyring` 3.6 (`windows-native`), `rusqlite` 0.40 bundled (đã có ở core), `cookie_store` 0.22 (đã có qua reqwest), React + zod + vitest.

Spec: [docs/superpowers/specs/2026-11-07-tai-raw-extension-design.md](../specs/2026-11-07-tai-raw-extension-design.md) mục 5, 6, 7, 9, 10.1, 11.

## Global Constraints

- Không chạy `cargo fmt`. Không đưa tên app đọc truyện bên thứ ba vào repo (grep trước khi commit).
- Không động vào file untracked của task khác (`QT-hop-nhat/`, `tools/`, `vp-*`, `corpus/`, `1`, `docs/reviews/2026-10-21-hop-nhat-tu-dien/`, `docs/superpowers/specs/2026-11-06-quy-tac/__pycache__/`).
- Khoá, mật khẩu, token, cookie không nằm trong thư mục truyện và không gửi lên giao diện: form chỉ biết trường bí mật "đã lưu" hay chưa; ô trống khi lưu = giữ giá trị cũ.
- Log và lỗi che mọi giá trị bí mật (auth `secret`, cài đặt `secret`, token của `login`) thành `***`.
- Cache tối đa 50 MB, đầy thì xoá mục dùng lâu nhất; một mục tối đa 5 MB. Session data tối đa 1 MB. `ttl` tính bằng giây.
- Session data mất khi hết `ttl`, khi đổi hoặc xoá auth, khi `session.clear()`, khi bấm "Đăng xuất".
- Thời gian tối đa của `login`: 2 phút.
- Build với `CARGO_INCREMENTAL=0` (ổ D: gần đầy). vitest chạy từ `D:/Work/...` (chữ hoa).
- App: kiểm Rust app bằng `cargo clippy --all-targets`; GUI bằng `npm run check`.

## Chỗ lưu (spec mục 7)

```
<config>/extensions/<id>/pkg/             gói (mốc 2)
<config>/extensions/<id>/settings.json    cài đặt chung, trừ trường secret
<config>/extension-data/<id>/auth.json    kiểu auth đã chọn, trường text, mã phiên bản auth
<config>/extension-data/<id>/session.json session data, phiên login, hũ cookie, mã phiên bản auth
<config>/extension-data/<id>/cache.sqlite cache
Credential Manager: qt-ext/<id>/auth/<field>, qt-ext/<id>/setting/<key>
```

Tiền tố `auth/` và `setting/` tách hai nhóm khoá để trường auth và trường cài đặt trùng tên không đè nhau. Máy
không phải Windows thì khoá ghi vào `<config>/extension-data/<id>/secrets.json` và form hiện cảnh báo.

Đổi auth thì sinh mã phiên bản mới trong `auth.json`. `session.json` ghi mã phiên bản lúc tạo; lệch thì coi như
trống. Cách này không lưu hash của khoá ra đĩa và vẫn đúng khi một lượt tải cũ còn đang ghi session.

## File structure

```
crates/qt-ext/src/store/mod.rs           ExtStore: auth, cài đặt chung, đăng xuất, xoá cache, trạng thái
crates/qt-ext/src/store/secrets.rs       SecretBackend: Keyring | File | Memory
crates/qt-ext/src/store/cache.rs         Cache SQLite: ttl, quota, LRU
crates/qt-ext/src/store/session_data.rs  SessionData: giá trị có ttl, phiên login, cookie, trần 1 MB
crates/qt-ext/src/cookies.rs             Jar: CookieStore của reqwest trên cookie_store, cờ đã đổi
crates/qt-ext/src/engine/bind_store.rs   JS: cache.*, session.*
crates/qt-ext/src/manifest.rs            check_setting (kiểm kiểu giá trị cài đặt)
crates/qt-ext/src/session.rs             auth, login, AuthExpired, lệnh Login
crates/qt-ai-core/src/raw_download/run.rs  gộp cài đặt, lưu cài đặt truyện, login
apps/qt-ai-gui/src-tauri/src/ext_cmds.rs   lệnh ext_config / ext_save_auth / ext_clear_auth / ext_save_settings / ext_logout / ext_clear_cache / ext_login
apps/qt-ai-gui/src/components/ext-fields.tsx (+ test)        ô nhập vẽ từ manifest
apps/qt-ai-gui/src/components/ext-config-dialog.tsx (+ test) hộp "Auth và cài đặt"
```

---

### Task 1: Kho khoá và `ExtStore` (auth, cài đặt chung)

**Files:** Create `crates/qt-ext/src/store/{mod.rs,secrets.rs}`; Modify `crates/qt-ext/Cargo.toml` (`[target.'cfg(windows)'.dependencies] keyring = { version = "3.6", features = ["windows-native"] }`, `rusqlite = { version = "0.40.2", features = ["bundled"] }`, `cookie_store = "0.22"`), `src/lib.rs`, `src/error.rs` (thêm `Store(String)`, `AuthMissing`, `LoginFailed(String)`), `src/manifest.rs`.

**Interfaces (Produces):**
- `pub fn check_setting(field: &SettingField, value: &Value) -> Result<Value, String>`: text/secret nhận chuỗi; number nhận số hoặc chuỗi số; boolean nhận true/false; select nhận chuỗi trong `choices`.
- `pub enum SecretBackend { Keyring, File(PathBuf), Memory(Arc<Mutex<HashMap<String, String>>>) }` với `platform(file)`, `memory()`, `kind() -> &'static str` ("keyring" | "file" | "memory"), `get/set/delete(name)`.
- `pub struct AuthValues { pub scheme: String, pub values: Map<String, Value> }`, `secret_values(&Manifest) -> Vec<String>`.
- `pub struct AuthState { pub scheme: Option<String>, pub text: Map<String, Value>, pub filled: Vec<String> }`.
- `pub struct SessionStatus { pub logged_in: bool, pub expires_at: Option<i64>, pub has_data: bool }`.
- `ExtStore::new(config_root, id)`, `with_backend(config_root, id, backend)`, `data_dir()`, `secret_kind()`, `auth(&m) -> Result<Option<AuthValues>>`, `auth_state(&m)`, `auth_version() -> String`, `save_auth(&m, scheme, &values)`, `clear_auth(&m)`, `settings(&m) -> Result<Map>` (chung, kể cả secret), `settings_state(&m) -> (Map, Vec<String>)`, `save_settings(&m, &values)`, `logout()`, `clear_cache()`, `cache_bytes()`, `session_status()`.

Luật lưu auth: kiểu phải có trong manifest; trường text lưu vào `auth.json`; trường secret khác rỗng thì ghi kho khoá, rỗng thì giữ giá trị cũ; trường của kiểu khác bị xoá. Có gì đổi thì sinh mã phiên bản mới. `clear_auth` xoá mọi khoá, `auth.json`, `session.json`.

Test (Memory và File backend): lưu rồi đọc lại; ô secret rỗng giữ giá trị cũ; đổi kiểu thì khoá của kiểu cũ mất; lưu y nguyên không đổi mã phiên bản, đổi giá trị thì đổi; `clear_auth` xoá `session.json`; cài đặt sai kiểu bị từ chối, secret tách khỏi `settings.json`; test `#[ignore]` ghi/đọc/xoá thật trên Credential Manager với id riêng.

Commit: `feat(ext): kho khoá và ExtStore — auth, cài đặt chung của extension`.

### Task 2: Cache SQLite

**Files:** Create `crates/qt-ext/src/store/cache.rs`.

**Interfaces:** `pub const CACHE_QUOTA: u64 = 50 * 1024 * 1024; pub const MAX_CACHE_ENTRY: usize = 5 * 1024 * 1024;` `Cache::open(path)`, `Cache::memory()`, `with_quota(u64)`, `get(key) -> Result<Option<String>>`, `set(key, value, ttl: Option<u64>)`, `delete(key)`, `total_bytes()`. Bảng `entries(key PRIMARY KEY, value, size, expires, used)`; `used` là bộ đếm tăng dần (đúng thứ tự LRU, không phụ thuộc đồng hồ). `get_at` / `set_at` nhận thời điểm cho test.

Test: hết `ttl` thì `get` trả `None` và mục bị xoá; vượt quota thì mục dùng lâu nhất bị xoá, mục vừa `get` được giữ; mục quá 5 MB bị từ chối; khoá rỗng hoặc dài quá 512 bị từ chối; mở lại file thấy dữ liệu cũ.

Commit: `feat(ext): cache SQLite cho extension — ttl, quota 50 MB, xoá mục dùng lâu nhất`.

### Task 3: Session data và hũ cookie

**Files:** Create `crates/qt-ext/src/store/session_data.rs`, `crates/qt-ext/src/cookies.rs`; Modify `crates/qt-ext/src/net.rs`.

**Interfaces:**
- `pub const SESSION_LIMIT: usize = 1024 * 1024;` `pub struct LoginRecord { pub token: Option<String>, pub expires_at: Option<i64>, pub data: Value }` (`expires_at` ms).
- `SessionData::open(path: Option<PathBuf>, auth_version: &str)`, `get(key)`, `set(key, value, ttl)`, `delete(key)`, `clear()`, `login() -> Option<LoginRecord>` (chỉ khi còn hạn, chừa 30 giây), `set_login(record)`, `clear_login()` (xoá cả cookie), `cookies()`, `set_cookies(list)`, `status()`.
- Ghi: khoá theo đường dẫn trong process → đọc lại file → áp đúng thay đổi → bỏ mục hết hạn → kiểm trần 1 MB → ghi an toàn. Hai lượt tải cùng extension không xoá giá trị của nhau.
- `pub(crate) struct Jar` (reqwest `CookieStore` trên `cookie_store::CookieStore`): `from_cookies`, `snapshot()`, `take_dirty()`, `clear()`.
- `HttpClient::with_jar(policy, limits, Arc<Jar>)`, `jar()`; `HttpRequest.cookies: bool` (mặc định true; false thì gửi qua client không cookie).

Test: ttl; trần 1 MB; lệch mã phiên bản auth thì trống; hai handle cùng file không mất khoá của nhau; login hết hạn thì `login()` là `None`; cookie máy chủ giả đặt ở lượt này được gửi lại ở `Session` sau; `cookies: false` không gửi cookie.

Commit: `feat(ext): session data có ttl và hũ cookie lưu giữa các lượt`.

### Task 4: API `cache` và `session` cho JS, che khoá động

**Files:** Create `crates/qt-ext/src/engine/bind_store.rs`; Modify `engine/mod.rs` (`HostState.secrets: RefCell<Vec<String>>`, `redact()`, `add_secret()`, `Stores { cache: Option<PathBuf>, session: Rc<SessionData> }`), `bind_http.rs` (`cookies`, lưu cookie khi hũ đổi), `bind_misc.rs`.

JS: `cache.get(key)`, `cache.set(key, value, { ttl })`, `cache.delete(key)`, `session.get/set/delete/clear`. Giá trị là chuỗi hoặc JSON; `undefined` bị từ chối; `get` thiếu trả `null`. Mọi hàm trả Promise.

Test (qua `testing::call` và `Session` với `ExtStore` tạm): set/get/delete, ttl, giá trị giữ qua hai `Session`, lỗi quá trần hiện thành exception JS bắt được.

Commit: `feat(ext): API cache và session cho extension`.

### Task 5: Auth, `login`, `AuthExpired` trong `Session`

**Files:** Modify `crates/qt-ext/src/session.rs`, `src/lib.rs`.

**Interfaces:** `SessionOptions.store: Option<ExtStore>`, `login_limit: Duration` (120 giây). `Session::login(&ctx) -> Result<Option<LoginRecord>, ExtError>` (`None` = extension không có `login`).

Luật:
- Mở lượt: nạp auth từ `ExtStore`; manifest `auth.required` mà chưa có auth thì `AuthMissing`. Giá trị secret của auth thêm vào danh sách che.
- Mỗi lệnh: chèn `ctx.auth = { scheme, values } | null`, `ctx.session = { token, expiresAt, data } | null`; giá trị cài đặt secret trong `ctx.settings` thêm vào danh sách che.
- Có `login` mà phiên thiếu hoặc hết hạn thì gọi `login` trước. `expiresAt` nhỏ hơn 1e11 coi là giây. Token thêm vào danh sách che. `login` lỗi thì `LoginFailed`.
- `toc` hoặc chương ném `AuthExpired`: có `login` thì xoá phiên (login và cookie, giữ giá trị `session.set`), gọi `login`, chạy lại đúng các lệnh đó một lần; không có `login` thì trả lỗi luôn.

Test (máy chủ giả + gói tạm): `login` gọi một lần cho cả lượt và lượt sau dùng lại phiên; phiên hết hạn thì gọi lại; 3 chương song song cùng `AuthExpired` → đăng nhập lại đúng một lần, cả 3 chạy lại thành công; `AuthExpired` lần hai thành lỗi; không có `login` thì lỗi đi thẳng; `auth.required` thiếu auth thì `AuthMissing`; log chứa API key và token hiện `***`.

Commit: `feat(ext): auth, login và AuthExpired trong lượt chạy extension`.

### Task 6: Core — gộp cài đặt, cài đặt theo truyện, login

**Files:** Modify `crates/qt-ai-core/src/raw_download/{run.rs,mod.rs}`, `crates/qt-ai-core/tests/raw_download.rs`.

**Interfaces:**
- `settings_for(extension, download, global: &Map)`: mặc định → chung → truyện (chỉ trường `scope: "story"`).
- `pub fn set_story_settings(config: &mut DownloadConfig, manifest: &Manifest, values: &Map) -> std::result::Result<(), String>`.
- `pub fn login(root: Option<&Path>, opts: &RunOptions, cancel) -> Result<SessionStatus>`.
- `start_session` thêm giá trị cài đặt secret vào danh sách che; `build_ctx` bỏ `auth` / `session` (do `Session` chèn).

Test trọn vòng: extension có `login` + API key, chương thứ 2 ném `AuthExpired` một lần → tải đủ, `login` gọi 2 lần; `ctx.settings` đúng thứ tự gộp; cài đặt truyện sai kiểu bị từ chối.

Commit: `feat(core): tải raw dùng auth, cài đặt chung và theo truyện của extension`.

### Task 7: Lệnh Tauri

**Files:** Create `apps/qt-ai-gui/src-tauri/src/ext_cmds.rs`; Modify `raw_cmds.rs`, `lib.rs`, `Cargo.toml` không đổi.

**Interfaces:** `ExtConfigView { id, name, auth: Option<AuthView>, settings: Vec<SettingView>, session: SessionView, secretStore, cacheBytes }`, `AuthView { required, schemes, scheme, values (chỉ text), filled }`, `SettingView { key, label, kind, choices, default, scope, value (không có với secret), filled }`. Lệnh `ext_config(id)`, `ext_save_auth(id, scheme, values)`, `ext_clear_auth(id)`, `ext_save_settings(id, values)`, `ext_logout(id)`, `ext_clear_cache(id)`, `ext_login(root, id)`. `RawInfo` thêm `extConfig`, `storySettings`; `raw_save` thêm `storySettings`. `options_for` gắn `ExtStore`.

Kiểm: `cargo clippy -p qt-ai-gui --all-targets`.

Commit: `feat(gui): lệnh Tauri cho auth, cài đặt, đăng nhập, xoá phiên, xoá cache của extension`.

### Task 8: Giao diện

**Files:** Create `apps/qt-ai-gui/src/components/ext-fields.tsx`, `ext-config-dialog.tsx` (+ test); Modify `lib/schema.ts`, `lib/api.ts`, `components/pages/raw-page.tsx` (+ test), `CHANGELOG.md`.

- `ExtField`: text, number, boolean (checkbox), select, secret (ô mật khẩu, placeholder "Đã lưu — để trống để giữ").
- Hộp "Auth và cài đặt": chọn kiểu auth theo thứ tự manifest, các trường, "Lưu auth", "Xoá auth"; cài đặt chung + "Lưu cài đặt"; "Đăng xuất", "Xoá cache (N MB)"; cảnh báo khi khoá lưu ở file.
- Trang Tải raw: dòng trạng thái auth ("Chưa nhập auth" / "Đã nhập: API key" / "Đã đăng nhập, hết hạn …") + nút "Đăng nhập" + "Auth và cài đặt…"; các trường `scope: "story"` lưu cùng nút Lưu.

Test: form vẽ đúng từ manifest; ô secret trống gửi chuỗi rỗng; đổi kiểu auth đổi trường; trang Tải raw gửi `storySettings`.

Commit: `feat(gui): auth và cài đặt extension trên trang Tải raw`.
