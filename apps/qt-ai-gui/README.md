# QT AI Translator (qt-ai-gui)

App desktop dịch truyện Trung → Việt hàng loạt bằng quota Antigravity: điều khiển Antigravity CLI (`agy`) theo vòng phiên, logic dịch/kiểm tra/glossary nằm trong crate `qt-ai-core` (port 1-1 từ `apps/qt-ai-cli` + qt-web).

## Yêu cầu máy người dùng

- Windows 10/11, WebView2 (installer tự tải).
- Antigravity CLI: `irm https://antigravity.google/cli/install.ps1 | iex`, chạy `agy` một lần để đăng nhập Google.

## Dev

```bash
cd apps/qt-ai-gui
npm install
npm run build:sidecar     # build crates/qt-ai-core --bin qt-ai → src-tauri/binaries/ (bắt buộc trước dev/build)
npm run tauri dev
npm run check             # typecheck + lint + vitest + vite build
cargo test -p qt-ai-gui   # Tauri commands
```

## Build

`npm run tauri build` → `src-tauri/target/release/bundle/{nsis,msi}/`. Sidecar `qt-ai.exe` được đặt cạnh app exe; AGENTS.md trong folder truyện trỏ tới nó.

## Folder truyện

Cùng format với `apps/qt-ai-cli` và Antigravity IDE: `raw/`, `out/`, `work/`, `story.json`, `state.json`, `AGENTS.md`, `.agent/workflows/`. Mở truyện đang dịch dở bằng bản nào cũng tiếp được.

## Drift với qt-web

Prompt/rule của GUI khớp qt-web qua golden fixtures. Trước khi mở PR: `npm --prefix apps/qt-ai-cli run -s golden:check`; đỏ thì chạy `golden` rồi sửa Rust cho `cargo test -p qt-ai-core` xanh.
