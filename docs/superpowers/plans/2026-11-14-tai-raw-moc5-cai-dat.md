# Tải raw — mốc 5: cài extension, trang quản lý, chế độ developer, tài liệu

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Người dùng cài extension từ file zip hoặc link, xem và quản lý ở Cài đặt → Extension tải raw (bật/tắt, kiểm tra cập nhật, auth, gỡ); nhà phát triển nạp thư mục đang viết, chạy thử `toc` / `chapter` xem request và log; có tài liệu tiếng Việt, `qt-ext.d.ts`, `manifest.schema.json` và 2 extension mẫu chạy được.

**Architecture:** `qt-ext` thêm giải nén zip an toàn (chặn zip slip, symlink, giới hạn cỡ) và tải zip qua https có chặn mạng nội bộ; ghi lại request của từng lượt (cho Chạy thử) và gỡ sạch dữ liệu extension. `qt-ai-core::raw_download::registry` thêm sổ `installed.json` (nguồn cài, sha256, quyền đã cấp, bật/tắt), cài qua thư mục tạm rồi đổi chỗ, so quyền khi cập nhật, nạp thư mục dev. Core thêm lượt chạy thử không ghi gì vào truyện. App thêm lệnh `ext_*`, hộp xin quyền, Card "Extension tải raw" trong Cài đặt, hộp Chạy thử; log và tiến độ của extension hiện ở trang Tải raw.

**Tech Stack:** Rust (qt-ext, qt-ai-core, Tauri 2), `zip` 4.6 (đã có ở core), React + vitest.

Spec: [docs/superpowers/specs/2026-11-07-tai-raw-extension-design.md](../specs/2026-11-07-tai-raw-extension-design.md) mục 4, 6, 7, 9, 11, 12.

## Global Constraints

- Không chạy `cargo fmt`. Không đưa tên app đọc truyện bên thứ ba vào repo (grep trước khi commit, kể cả docs và extension mẫu).
- Không động vào file untracked của task khác.
- Zip tối đa 5 MB, tối đa 200 file, giải nén tối đa 5 MB; đường dẫn thoát thư mục, đường dẫn tuyệt đối, symlink bị từ chối. Zip có một thư mục gốc chứa `manifest.json` thì bỏ thư mục đó.
- Link cài chỉ `https://`, không tới mạng nội bộ.
- Cài: kiểm manifest → hộp xin quyền (hosts, quyền `webview`, auth bắt buộc) → giải nén. Cập nhật thêm domain hoặc quyền thì hỏi lại; chưa đồng ý thì giữ bản cũ. Cập nhật không thêm gì thì cài luôn.
- Gỡ: xoá gói, cài đặt, auth, session data, cache, hồ sơ webview; `download.json` của truyện giữ nguyên.
- Chỉ extension nạp ở chế độ developer mới khai được `localhost` / `127.0.0.1`. Thư mục dev đọc lại mỗi lượt chạy, sửa file là lượt sau dùng bản mới.
- Chạy thử không ghi gì vào truyện.
- `log.*` và `progress()` của extension hiện ở log trang Tải raw; extension dev thêm từng request (method, url, status, ms).
- Thư mục cũ đặt tay ở `extensions/<id>/pkg/` (mốc 2) vẫn chạy: coi như đã cài, nguồn "đặt tay".
- Build với `CARGO_INCREMENTAL=0`; vitest chạy từ `D:/Work/...`.

## Chỗ lưu

```
<config>/extensions/installed.json     [{ id, version, source: {kind: file|link|dev|manual, path|url|dir}, sha256, hosts, permissions, enabled, installedAt, updatedAt }]
<config>/extensions/<id>/pkg/          gói đã giải nén
<config>/extensions/.staging/<sha256>/ gói vừa kiểm, chờ người dùng đồng ý (xoá khi cài hoặc sau 1 giờ)
```

---

### Task 1: Giải nén zip, tải zip, gỡ sạch dữ liệu (qt-ext)

**Files:** Modify `crates/qt-ext/Cargo.toml` (`zip`, `flate2` như core), `src/package.rs`, `src/store/mod.rs`, `src/lib.rs`.

**Interfaces:** `pub fn extract_zip(bytes: &[u8], dest: &Path) -> Result<(), ExtError>`; `pub fn sha256_hex(bytes: &[u8]) -> String`; `pub fn fetch_zip(url: &str) -> Result<Vec<u8>, ExtError>` (https, chặn mạng nội bộ, ≤ 5 MB); `ExtStore::remove_all(&Manifest)`.

Test: zip thường giải nén rồi `Package::load_dir` được; zip có thư mục gốc bỏ được; `../x`, `/x`, symlink, quá 200 file, quá 5 MB bị từ chối và không ghi gì ra ngoài; `fetch_zip` từ chối `http://` và `https://127.0.0.1`; `remove_all` xoá khoá, `settings.json`, thư mục dữ liệu.

Commit: `feat(ext): giải nén gói zip an toàn, tải zip qua https, gỡ sạch dữ liệu extension`.

### Task 2: Sổ cài đặt (core)

**Files:** Modify `crates/qt-ai-core/src/raw_download/registry.rs` (+ test).

**Interfaces:**
- `Source { File { path }, Link { url }, Dev { dir }, Manual }`, `InstalledEntry { id, version, source, sha256, hosts, permissions, enabled, installed_at, updated_at }`.
- `Inspection { package, sha256, current: Option<InstalledEntry>, new_hosts, new_permissions, newer, needs_consent }`.
- `Registry::new(extensions_dir)`, `entries()`, `list_all() -> (Vec<(InstalledEntry, Result<Installed, String>)>)`, `list_enabled() -> (Vec<Installed>, Vec<String>)`, `inspect(bytes) -> Inspection` (giải nén vào `.staging/<sha>`), `install(sha256, source) -> InstalledEntry` (đổi chỗ `pkg`), `uninstall(id, store)`, `set_enabled(id, bool)`, `add_dev(dir)`.
- `list_installed` giữ chữ ký, trả extension đang bật.

Test: cài từ zip, mở lại thấy; cập nhật thêm host thì `needs_consent`; bản cũ hơn hoặc bằng thì `newer = false`; tắt thì không còn là ứng viên; gỡ xoá thư mục và khoá; thư mục đặt tay vẫn hiện; thư mục dev được khai `127.0.0.1`, id trùng bản đã cài bị từ chối.

Commit: `feat(core): sổ extension đã cài — cài từ zip, cập nhật hỏi lại quyền, bật/tắt, gỡ, nạp thư mục dev`.

### Task 3: Chạy thử, ghi request, log extension (qt-ext + core)

**Files:** Modify `crates/qt-ext/src/engine/{mod.rs,bind_http.rs}` (`EngineOptions.on_request`), `crates/qt-ai-core/src/raw_download/run.rs`.

**Interfaces:** `pub struct RequestLog { method, url (đã che khoá), status: Option<u16>, ms, error: Option<String> }`; `EngineOptions.on_request: Option<RequestSink>`; core `test_toc(opts, url) -> TestReport`, `test_chapter(opts, url, item, index) -> TestReport` với `TestReport { ok, ms, result: Value, error, requests, logs }`.

Test: chạy thử `toc` của extension mẫu với máy chủ giả: có kết quả, có request kèm status; chạy thử `chapter`; lỗi extension hiện trong `error`; không ghi file nào vào truyện (không có truyện).

Commit: `feat(core): chạy thử extension — toc, chapter, request và log, không ghi vào truyện`.

### Task 4: Lệnh Tauri và giao diện

**Files:** Modify `apps/qt-ai-gui/src-tauri/src/{ext_cmds.rs,raw_cmds.rs,lib.rs}`; Create `apps/qt-ai-gui/src/components/{ext-manager.tsx,ext-permission-dialog.tsx,ext-test-dialog.tsx}` (+ test); Modify `lib/schema.ts`, `lib/api.ts`, `components/pages/settings-page.tsx`, `components/pages/raw-page.tsx`.

- Lệnh: `ext_list`, `ext_inspect_file(path)`, `ext_inspect_url(url)`, `ext_install(sha256, source)`, `ext_uninstall(id)`, `ext_set_enabled(id, enabled)`, `ext_check_update(id)`, `ext_add_dev(dir)`, `ext_test_toc(id, url)`, `ext_test_chapter(id, url, item, index)`.
- `raw_toc` / `raw_download` / `ext_login`: log và tiến độ của extension phát qua event `raw-download-progress` (`kind: "log"`); extension dev thêm dòng request.
- Card "Extension tải raw" trong Cài đặt: nút "Cài từ file…", ô link + "Cài", "Nạp thư mục (developer)…"; mỗi dòng: icon, tên, phiên bản, nhãn Dev / Tắt, nhà phát triển (tên, link, email), giấy phép, domain, quyền, nguồn cài; nút Bật/Tắt, Kiểm tra cập nhật (cài từ link), Auth và cài đặt, Chạy thử, Gỡ.
- Hộp xin quyền: tên, phiên bản (cũ → mới), nhà phát triển, domain (đánh dấu domain mới), quyền webview, auth bắt buộc; "Cài" / "Huỷ".
- Hộp Chạy thử: ô link, "Chạy toc" → số mục, 50 mục đầu, thời gian, bảng request, log; chọn một mục → "Chạy chapter" → nội dung.

Test: danh sách vẽ từ `ext_list`; cài từ link mở hộp xin quyền, đồng ý thì gọi `ext_install`; cập nhật đánh dấu domain mới; Gỡ hỏi lại; Chạy thử hiện request.

Commit: `feat(gui): Cài đặt → Extension tải raw — cài từ file/link, xin quyền, bật/tắt, cập nhật, gỡ, nạp thư mục dev, chạy thử`.

### Task 5: Tài liệu và 2 extension mẫu

**Files:** Create `docs/extensions/README.md`, `docs/extensions/qt-ext.d.ts`, `docs/extensions/manifest.schema.json`, `docs/extensions/examples/html-gbk/{manifest.json,main.js,lib/parse.js}`, `docs/extensions/examples/api-key/{manifest.json,main.js}`, `crates/qt-ext/tests/examples.rs`.

- Mẫu 1: trang HTML, mục lục nhiều trang, GBK, `progress`, chương khoá, tiêu đề chương.
- Mẫu 2: API JSON có API key, `login` (token có hạn), `AuthExpired` khi 401, `session.set` mã thiết bị, `cache` mục lục 10 phút, cài đặt chung `apiBase`.
- Test chạy 2 mẫu với máy chủ giả (manifest đổi `hosts` / `match` sang `127.0.0.1` lúc test).

Commit: `docs(ext): hướng dẫn viết extension, qt-ext.d.ts, manifest.schema.json, 2 extension mẫu`.

### Task 6: CHANGELOG

Commit: `docs(gui): CHANGELOG — cài và quản lý extension`.
