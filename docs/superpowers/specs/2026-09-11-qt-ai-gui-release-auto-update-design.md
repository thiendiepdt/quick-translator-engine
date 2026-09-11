# qt-ai-gui: phát hành trên GitHub Releases và tự cập nhật

Ngày: 2026-09-11. Mẫu tham chiếu: repo novelkit (`.github/workflows/release.yml`, `tauri-plugin-updater`, hộp thoại native khi mở app).

## Mục tiêu

- Đẩy tag `qt-ai-gui-v<x.y.z>` → GitHub Actions build Windows, tạo release kèm NSIS, MSI, chữ ký `.sig`, `latest.json` và zip portable.
- App đã cài tự dò bản mới khi mở, hỏi bằng hộp thoại native, tải và cài rồi khởi động lại.
- Không đụng `apps/qt-gui` hay crate khác của monorepo.

## Quyết định đã chốt với người dùng

| Điểm | Chọn |
| --- | --- |
| Nền tảng build | Chỉ Windows (`windows-latest`) |
| Bản portable | Vẫn đưa zip lên release; cập nhật chạy NSIS như bản cài, nên máy portable sau cập nhật thành bản cài đặt (ghi rõ trong README.txt của portable) |
| Giao diện cập nhật | Native `ask` / `message` của plugin-dialog, y hệt novelkit; không có nút kiểm tra tay |
| Tag | `qt-ai-gui-v<x.y.z>`; workflow chỉ chạy với tiền tố này |

## 1. Workflow CI

File `.github/workflows/release-qt-ai-gui.yml`, trigger `push.tags: ['qt-ai-gui-v*']`, một job `release` trên `windows-latest`, `permissions.contents: write`.

Bước:

1. `actions/checkout@v6`.
2. `actions/setup-node@v6` Node 20, cache npm theo `apps/qt-ai-gui/package-lock.json`.
3. `dtolnay/rust-toolchain@stable`.
4. `swatinem/rust-cache@v2` với `workspaces: '. -> target'` (target nằm ở gốc workspace cargo).
5. `npm ci` trong `apps/qt-ai-gui`.
6. `tauri-apps/tauri-action@v0`: `projectPath: apps/qt-ai-gui`, `tagName: qt-ai-gui-v__VERSION__`, `releaseName: 'VNCVT AI Translator v__VERSION__'`, `releaseBody: 'Xem changelog bên dưới.'`, `releaseDraft: false`, `prerelease: false`, `updaterJsonPreferNsis: true`. Env: `GITHUB_TOKEN`, `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Sidecar `qt-ai.exe` được build vì `beforeBuildCommand` của app đã gọi `npm run build:sidecar`.
7. `npm run build:portable` với `PORTABLE_SKIP_BUILD=1` → `dist-portable/VNCVT-AI-Translator-<version>-portable.zip`.
8. `gh release upload qt-ai-gui-v<version> <zip> --clobber` (pwsh, đọc version từ `tauri.conf.json`).

## 2. Khoá ký

- Sinh một lần bằng `npx tauri signer generate -w ~/.tauri/qt-ai-gui.key` (ngoài repo). Public key ghi vào `plugins.updater.pubkey` trong `tauri.conf.json`.
- Người dùng tự thêm secret `TAURI_SIGNING_PRIVATE_KEY` (nội dung file key) và `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` trên GitHub; README hướng dẫn. Mất private key thì app đã phát hành không nhận bản mới nữa, phải phát hành lại pubkey mới bằng tay.

## 3. Cấu hình app

- `tauri.conf.json`: `bundle.createUpdaterArtifacts: true`; giữ `targets: ["nsis","msi"]`; thêm `plugins.updater.endpoints = ["https://github.com/thiendiepdt/quick-translator-engine/releases/latest/download/latest.json"]` và `pubkey`. CSP giữ nguyên: updater tải từ phía Rust, không qua webview.
- `Cargo.toml`: thêm `tauri-plugin-updater = "2"`, `tauri-plugin-process = "2"`.
- `lib.rs`: trong `setup`, dưới `#[cfg(desktop)]`, `app.handle().plugin(tauri_plugin_updater::Builder::new().build())?` và `.plugin(tauri_plugin_process::init())` ở builder. Đăng ký trong `setup` nên `tauri::test::mock_builder` trong test hiện có không bị ảnh hưởng.
- `capabilities/main.json` thêm `updater:default`, `process:allow-restart`, `dialog:allow-ask`, `dialog:allow-message`.
- `package.json`: thêm `@tauri-apps/plugin-updater`, `@tauri-apps/plugin-process`.

## 4. Trong app

`src/hooks/use-update-check.ts`:

- Hàm thuần `runUpdateCheck(deps)` với `deps = { check, ask, message, relaunch, warn }` (mặc định lấy từ plugin, `console.warn`). Luồng:
  1. `update = await check()`; không có → kết thúc, trả `"none"`.
  2. `ask("Phiên bản mới <version> đã có! Bạn có muốn cập nhật ngay không?", { title: "Cập nhật VNCVT AI Translator", kind: "info" })`; từ chối → trả `"declined"`.
  3. `await update.downloadAndInstall()`; `await message("Cập nhật hoàn tất. Ứng dụng sẽ khởi động lại.", { title: "Cập nhật thành công" })`; `await relaunch()`; trả `"installed"`.
  4. Bất kỳ bước nào ném lỗi → `warn("Kiểm tra cập nhật thất bại:", error)`, trả `"failed"`. Không toast, không chặn app.
- Hook `useUpdateCheck()` gọi `runUpdateCheck()` một lần khi mount, bỏ qua khi `import.meta.env.DEV` (dev server không có bản cài để cập nhật). Plugin import động để test không kéo `@tauri-apps/*`.
- `app.tsx` gọi `useUpdateCheck()` cạnh `useUndoFallback()`.

## 5. Bump version

`apps/qt-ai-gui/scripts/set-version.mjs <x.y.z>`:

- Kiểm tra định dạng `^\d+\.\d+\.\d+$`, sai thì thoát mã 1.
- Ghi `version` vào `package.json`, `package-lock.json` (cả `version` gốc và `packages[""].version`), `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` (`[package] version`, chỉ dòng đầu tiên khớp `^version = "…"`).
- Chạy `cargo update -p qt-ai-gui --offline` ở gốc workspace để `Cargo.lock` khớp.
- Logic đọc/ghi tách thành hàm `applyVersion(dir, version)` export để test trên thư mục tạm; phần gọi cargo chỉ chạy khi script là entry point.

`build-portable.mjs`: nếu `process.env.PORTABLE_SKIP_BUILD` được đặt thì bỏ qua `tauri build --no-bundle`, chỉ kiểm tra binary đã có. README.txt của portable thêm dòng: cập nhật tự động sẽ cài bản đầy đủ; muốn giữ portable thì tải zip mới bằng tay.

Quy trình phát hành (ghi vào README của app):

```
node scripts/set-version.mjs 0.2.0
git commit -am "release(qt-ai-gui): 0.2.0"
git tag qt-ai-gui-v0.2.0
git push origin HEAD --tags
```

## 6. Kiểm thử

- `use-update-check.test.ts`: `runUpdateCheck` với deps giả — có bản mới + đồng ý (gọi đủ downloadAndInstall → message → relaunch, trả `installed`), từ chối (không tải, trả `declined`), không có bản mới (`none`, không hỏi), `check` ném lỗi (`failed`, có `warn`, không ném ra ngoài).
- `set-version.test.ts`: `applyVersion` trên thư mục tạm chứa 4 file mẫu; kiểm tra cả 4 đổi đúng và phần khác giữ nguyên; version sai định dạng bị từ chối.
- Rust: test hiện có vẫn qua; `cargo check` trên Linux phải qua với plugin mới.
- Workflow: kiểm bằng `actionlint` nếu có sẵn trên máy, không thì đọc tay.

## Ngoài phạm vi

- Build Linux/macOS, nút "Kiểm tra cập nhật" trong Cài đặt, hiện phiên bản trong UI, changelog tự động, kênh beta.
