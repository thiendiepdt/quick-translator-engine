# QT AI Translator (qt-ai-gui)

App desktop dịch truyện Trung → Việt hàng loạt: điều khiển Antigravity CLI (`agy`) theo vòng phiên, hoặc gọi thẳng Gemini / API OpenAI-compatible bằng key của người dùng; logic dịch/kiểm tra/glossary nằm trong crate `qt-ai-core` (port 1-1 từ `apps/qt-ai-cli` + qt-web).

## Hai động cơ dịch

Chọn ở Cài đặt → **Động cơ dịch** (lưu trong `config.json` của app):

- **Antigravity CLI (agy)** — mặc định: agent chạy trong `agy -p` theo vòng phiên, tự dịch/check/accept
  bằng quota Antigravity. Cần cài agy.
- **API key** — app tự gọi model qua HTTP bằng key của bạn rồi chạy cùng vòng next → dịch → check →
  accept trong `qt-ai-core` (`api_session`). Không cần agy. Provider: **Gemini** chính chủ, hoặc
  **OpenAI-compatible** (OpenAI, hay hub riêng qua Base URL, ví dụ `http://192.0.2.10/v1` với model
  `gemini-3.7-flash`). Gemini có công tắc Thinking; OpenAI có Mức nghĩ `reasoning_effort`
  (none…max, mặc định high). Key lưu plain trong `config.json`.

Cùng folder truyện, cùng `state.json`; đổi động cơ giữa chừng vẫn tiếp được. Ở chế độ API, chương
model từ chối được skip kèm lý do; lỗi mạng/HTTP thử lại một lần rồi skip chương, hai chương liên
tiếp lỗi thì dừng phiên (`api_failed`). "AI điền hồ sơ" vẫn chỉ chạy bằng agy.

## Yêu cầu máy người dùng

- Windows 10/11, WebView2 (installer tự tải).
- Động cơ agy: Antigravity CLI `irm https://antigravity.google/cli/install.ps1 | iex`, chạy `agy` một lần để đăng nhập Google.
- Động cơ API key: chỉ cần mạng tới provider (HTTPS qua rustls, không cần OpenSSL).

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

## Giao diện

Ba bộ màu (Editorial / Studio / Soft) × sáng / tối / theo hệ thống, chọn ở trang Cài đặt hoặc nút mặt trăng trên rail; lưu trong `config.json` của app. Token nằm trong `src/index.css`, mỗi tổ hợp đánh dấu `/* palette: <id> <mode> */`; `src/lib/theme-tokens.test.ts` kiểm đủ token và tương phản ≥ 4.5:1, `src/lib/no-hardcoded-colors.test.ts` chặn class màu cứng ngoài `components/ui`. Font đóng gói offline (`@fontsource-variable`).

## Trang đọc và hồ sơ truyện

- Chiều ngang vùng đọc (hẹp / vừa / rộng / toàn màn) chọn ngay trên thanh tab của trang đọc hoặc ở
  Cài đặt → Giao diện; lưu `readingWidth` trong `config.json`. Nút chương trước / sau nằm cố định ở
  đầu trang, cặp nút cuối bài vẫn giữ.
- Glossary: nhóm dài thu gọn sẵn, bảng hiện theo khúc 50 dòng; nút **Sửa dạng văn bản** mở textarea
  mỗi dòng `Hán=Việt` để sửa hàng loạt rồi Áp dụng một lần.
- Prompt và rule kiểm tra: lệnh `story_defaults` trả prompt gốc + bộ rule mặc định của hệ. Ô prompt
  luôn hiện nội dung đang dùng (sửa trên bản mặc định là thành prompt riêng, **Về mặc định** lưu
  trống); rule trống hiện bộ mặc định chỉ đọc, **Sửa bộ mặc định** sao chép ra để chỉnh.

## Thể loại

`story.json` có `genre: { setting: "ancient" | "modern", names: "han" | "foreign" | "mixed" }`; chọn ở trang Hồ sơ truyện, mục Thể loại. Bối cảnh quyết xưng hô, thán từ, từ gia đình, bảng thuật ngữ và bộ rule kiểm tra mặc định; tên riêng quyết phiên Hán-Việt hay trả về dạng gốc. Truyện cũ thiếu `genre` chạy như cổ đại/Hán-Việt. Prompt riêng hoặc rule riêng vẫn thắng. Chữ prompt nằm ở qt-web (`src/lib/ai-translation-prompt.ts`), Rust đọc 6 bản ghép sẵn trong `crates/qt-ai-core/prompts/prompts.json` qua golden.

## Drift với qt-web

Prompt/rule của GUI khớp qt-web qua golden fixtures. Trước khi mở PR: `npm --prefix apps/qt-ai-cli run -s golden:check`; đỏ thì chạy `golden` rồi sửa Rust cho `cargo test -p qt-ai-core` xanh.
