# Kho extension (phần 2)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Thêm kho extension bằng link `index.json`, cài / cập nhật extension từ kho (kiểm sha256), auth dùng chung của kho; dựng repo kho chính thức (build + deploy R2) với extension Qimao gọi API book-downloader.

**Architecture:** `qt-ext::store` tách phần auth thành `AuthStore` (của extension hoặc của kho); `ExtStore` dùng auth của kho khi extension được kho cho dùng chung. `qt-ai-core::raw_download::repository` đọc / kiểm `index.json`, lưu danh sách kho, tải gói từ kho rồi đưa vào `Registry::inspect` như cài từ file, và quyết `ExtStore` cho từng extension. App thêm lệnh `repo_*` và phần Kho trong Cài đặt → Extension tải raw. Repo kho chính thức ở `D:\Work\projects\vncvt-ai-translator-officical-extentions`.

**Tech Stack:** Rust (qt-ext, qt-ai-core, Tauri 2), React + vitest; repo kho: Node 22 (zip tự viết bằng `zlib`), wrangler.

Spec: [docs/superpowers/specs/2026-11-14-kho-extension-design.md](../specs/2026-11-14-kho-extension-design.md).

## Global Constraints

- Không chạy `cargo fmt`. Không đưa tên app đọc truyện bên thứ ba vào cả hai repo.
- Không động vào file untracked của task khác trong repo chính.
- `index.json` và gói chỉ tải qua `https://` công khai, `index.json` tối đa 2 MB; sha256 gói phải khớp kho.
- Chỉ extension cài từ chính kho và được kho đánh dấu `sharedAuth` mới đọc auth chung của kho.
- Deploy lên R2 là việc người dùng chạy (cần `wrangler login`); không tự deploy.
- Ổ C: đầy: chạy npm / cargo với `TEMP`, `TMP`, `npm_config_cache` trỏ `D:/Work/projects/quick-translator-engine/target/tmp-check`; `CARGO_INCREMENTAL=0`.

---

### Task 1: Repo kho chính thức + Qimao qua API book-downloader

**Files (repo kho):** `README.md`, `repository.json`, `package.json`, `.gitignore`, `extensions/qimao/{manifest.json,main.js}`, `scripts/{zip.mjs,build.mjs,deploy.mjs}`.

- Qimao: `hosts` `qimao-api.khotangtruyen.org`, auth `apiKey` bắt buộc, `toc` → `GET /book/<mã>`, `chapters` → `GET /content?id=&chapterIds=`; 401 → lỗi "API key sai", 5xx → thử lại, `missing` → lỗi từng chương (thử lại).
- `build.mjs`: kiểm manifest, nén deflate thứ tự cố định (zip giống nhau giữa hai lần build), sha256, `dist/index.json`.
- `deploy.mjs`: `wrangler r2 object put --remote`, zip trước, index sau; `--dry-run` in lệnh.
- Kiểm: build hai lần ra sha256 giống nhau; gói qua `qt_ext::Package` (giải nén, nạp); chạy Qimao với máy chủ giả mô phỏng worker; gọi worker thật không key ra 401.

### Task 2: `AuthStore` và auth dùng chung (qt-ext)

**Files:** `crates/qt-ext/src/store/{mod.rs,auth.rs}`, `src/manifest.rs` (`parse_auth_value`), `src/lib.rs`.

**Interfaces:** `AuthStore::extension(config_root, id, secrets)`, `AuthStore::repository(config_root, repo_id, secrets, spec)`; `values/state/save/clear/version`; `ExtStore::with_shared_auth(AuthStore)`, `ExtStore::shared_auth() -> Option<&AuthStore>`; `pub fn parse_auth_value(Value) -> Result<AuthSpec, ExtError>`.

Test: hai `ExtStore` cùng `AuthStore` kho đọc chung key; đổi key kho đổi mã phiên bản → session data cũ trống; `ExtStore` thường không thấy key kho.

### Task 3: Kho ở core

**Files:** Create `crates/qt-ai-core/src/raw_download/repository.rs`; Modify `registry.rs` (`Source::Repository { repository, url }`), `mod.rs`.

**Interfaces:** `RepoIndex`, `RepoEntry`, `parse_index(text, base_url) -> (RepoIndex, warnings)`; `Repositories::new(extensions_dir)` (+ `with_fetcher` cho test); `list() -> Vec<RepoView>`, `add(url)`, `refresh(id)`, `refresh_all()`, `remove(id, config_root)`; `inspect_entry(&Registry, repo_id, ext_id) -> (Inspection, Source)`; `ext_store(config_root, extensions_dir, id) -> ExtStore`; `repo_auth(config_root, repo_id) -> Option<AuthStore>`.

Test (máy chủ giả qua `with_fetcher`): thêm kho; mục lỗi bỏ qua có cảnh báo; `formatVersion` 2 từ chối; trùng id kho từ chối; cài từ kho sha256 lệch bị từ chối; manifest khác id/version bị từ chối; cài xong nguồn là kho; cập nhật thêm host cần đồng ý; `ext_store` của extension `sharedAuth` dùng auth kho, cài từ file thì không; gỡ kho thì extension về auth riêng.

### Task 4: Lệnh Tauri và giao diện

**Files:** Create `apps/qt-ai-gui/src-tauri/src/repo_cmds.rs`, `apps/qt-ai-gui/src/components/ext-repositories.tsx` (+ test); Modify `ext_cmds.rs` (`store_for` qua `ext_store`, `ExtConfigView.sharedAuth`), `ext_manage_cmds.rs` (cập nhật nguồn kho), `lib.rs`, `lib/schema.ts`, `lib/api.ts`, `components/ext-manager.tsx`, `components/ext-config-dialog.tsx` (xuất `AuthForm`, dòng auth chung), `components/ext-permission-dialog.tsx`.

Lệnh: `repo_list`, `repo_add(url)`, `repo_refresh(id?)`, `repo_remove(id)`, `repo_auth(id)`, `repo_save_auth(id, scheme, values)`, `repo_clear_auth(id)`, `repo_inspect(repoId, extId)`.

Test: thêm kho hiện dòng kho và danh sách; trạng thái Chưa cài / Đã cài / Có bản mới; Cài mở hộp xin quyền có dòng "API key chung"; Cập nhật hết gọi đúng các mục; form API key của kho.

### Task 5: Tài liệu, CHANGELOG

**Files:** `docs/extensions/README.md` (mục "Đăng kho"), `apps/qt-ai-gui/CHANGELOG.md`.
