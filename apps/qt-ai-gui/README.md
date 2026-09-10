# VNCVT AI Translator (qt-ai-gui)

App desktop dịch truyện Trung → Việt hàng loạt: điều khiển Antigravity CLI (`agy`) theo vòng phiên, hoặc gọi thẳng Gemini / API OpenAI-compatible bằng key của người dùng; logic dịch/kiểm tra/glossary nằm trong crate `qt-ai-core` (port 1-1 từ `apps/qt-ai-cli` + qt-web).

## Thư viện, tạo truyện mới, kéo thả chương

- **Thư viện** (`libraryRoot` trong config): một folder cha chứa mọi truyện. Chọn ở màn chọn truyện hoặc Cài đặt.
  Màn chọn truyện liệt kê thư mục con trực tiếp (truyện đã init có tiến độ; folder lạ bấm vào sẽ hỏi khởi tạo).
- **Tạo truyện mới**: tên tiếng Việt → tên folder tự sinh (`ta-tuyet-the-chi-dau`, sửa được) → tạo `<thư viện>/<slug>/raw/`,
  init, ghi tên + link vào `story.json`, mở luôn. Chương thêm sau.
- **Kéo thả**: đang mở truyện, thả file `.txt` (hoặc folder chứa `.txt`, không đệ quy) vào cửa sổ → copy vào `raw/`
  (không ghi đè file trùng tên, đuôi hạ thường) rồi đưa chương mới vào hàng đợi. Copy tay vào `raw/` thì bấm **Quét lại** (cũng gỡ chương có file raw đã mất, trừ chương đã dịch xong)
  trên trang Dịch; mở lại truyện cũng tự quét.
- "Mở folder truyện" thủ công vẫn còn.

## Dịch nhiều truyện song song

Mỗi truyện một phiên, tối đa `maxParallel` truyện cùng lúc (Cài đặt → App → "Số truyện dịch song song", mặc định 2;
API hub dễ trả 429 nếu để cao). Bấm Bắt đầu dịch ở một truyện rồi "Về danh sách truyện" mở truyện khác và bấm tiếp;
màn chọn truyện hiện "Đang dịch N/M truyện", dòng đang dịch có tiến độ live và nút Dừng. Tiến độ/log tách theo truyện
(`session-event` từ Rust mang `root`). Lock `work/.session.lock` vẫn chặn hai bản app dịch trùng một truyện.

**Chuyển nhanh**: cột "Phiên này" ngoài cùng bên phải liệt kê mọi truyện đã mở trong phiên app dưới dạng card (chữ cái
đầu + tên cắt 2 dòng, hover có tooltip đủ tên/đường dẫn), đang dịch xếp trên kèm chấm nhấp nháy và thanh tiến độ, còn lại
theo lần mở gần nhất; dài quá thì nút "Xem thêm N" nạp thêm 12 card mỗi lần. Bấm ô để chuyển, giữ nguyên trang đang xem. Nút lưới đầu cột (hoặc **Ctrl+K**) mở dialog
"Chuyển truyện": tìm theo tên/folder, thẻ theo lưới (đang dịch → đang mở → thư viện → gần đây), "Xem thêm" 24 thẻ mỗi
lần; folder chưa khởi tạo hiện mờ, mở từ màn chọn truyện.

## Hai động cơ dịch

Chọn ở Cài đặt → **Động cơ dịch** (lưu trong `config.json` của app):

- **Antigravity CLI (agy)** — agent chạy trong `agy -p` theo vòng phiên, tự dịch/check/accept
  bằng quota Antigravity. Cần cài agy; app chỉ dò agy khi bạn chọn động cơ này.
- **API key** — mặc định cho người dùng mới: app tự gọi model qua HTTP bằng key của bạn rồi chạy cùng
  vòng next → dịch → check → accept trong `qt-ai-core` (`api_session`). Không cần agy. Provider: **Gemini** chính chủ, hoặc
  **OpenAI-compatible** (OpenAI, hay hub riêng qua Base URL, ví dụ `http://192.0.2.10/v1` với model
  `gemini-3.7-flash`). Gemini có công tắc Thinking; OpenAI có Mức nghĩ `reasoning_effort`
  (none…max, mặc định high). Key lưu plain trong `config.json`.

Glossary tự động và AI điền dùng lượt "JSON mode" (`response_format`/`responseMimeType`); hub không nhận JSON mode
(400/404/422, trả rỗng) thì app tự gọi lại bằng lượt text thường rồi bóc object JSON ra — log trang Dịch ghi
"bỏ qua trích glossary — …" khi cả hai đường đều hỏng, và dòng "chốt (… +N glossary)" cho biết mỗi chương thêm bao nhiêu.

Cùng folder truyện, cùng `state.json`; đổi động cơ giữa chừng vẫn tiếp được. Ở chế độ API, chương
model từ chối được skip kèm lý do; lỗi mạng/429/5xx thử lại một lần rồi skip chương, hai chương liên
tiếp lỗi thì dừng phiên (`api_failed`); lỗi cấu hình (400 model không có, 401/403 key sai) dừng ngay
không skip. Phiên dừng vì lỗi thì chương đang dịch trả về hàng đợi, không kẹt "đang dịch". "AI điền hồ sơ" đi theo động cơ đang chọn: agy tra web + đọc
chương đầu qua workflow `setup-story.md`; API key cho model đọc 3 chương đầu trong `raw/` (không tra web)
rồi đề xuất hồ sơ — cả hai chỉ hiện diff, không ghi gì cho tới khi bấm Áp dụng.

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

`npm run tauri build` → `src-tauri/target/release/bundle/{nsis,msi}/`. Sidecar `qt-ai.exe` được đặt cạnh app exe. AGENTS.md và workflow trong folder truyện chỉ ghi lệnh `qt-ai` trần; app chèn folder sidecar vào PATH của agy khi chạy phiên, nên folder truyện init ở máy này mang sang máy khác vẫn dịch được. Mỗi lần mở truyện, file template nào chưa sửa tay còn chứa đường dẫn cũ sẽ được render lại.

`npm run build:portable` → `dist-portable/VNCVT-AI-Translator-<version>-portable/` (+ `.zip` trên Windows): `qt-ai-gui.exe`,
`qt-ai.exe` và file đánh dấu `portable`. Có file này cạnh exe thì app đọc/ghi `config.json` ngay cạnh exe thay vì
`%APPDATA%\com.vn-converter.qt-ai-gui\` (bản cài trước 2026-09-11 dùng `io.quicktranslator.ai-gui`; lần đầu chạy bản mới tự copy config.json cũ sang) — copy folder đi đâu cũng mang theo cấu hình. Máy đích vẫn cần WebView2.

## Folder truyện

Cùng format với `apps/qt-ai-cli` và Antigravity IDE: `raw/`, `out/`, `work/`, `story.json`, `state.json`, `AGENTS.md`, `.agent/workflows/`. Mở truyện đang dịch dở bằng bản nào cũng tiếp được.

## Giao diện

Ba bộ màu (Editorial / Studio / Soft) × sáng / tối / theo hệ thống, chọn ở trang Cài đặt hoặc nút mặt trăng trên rail; lưu trong `config.json` của app. Token nằm trong `src/index.css`, mỗi tổ hợp đánh dấu `/* palette: <id> <mode> */`; `src/lib/theme-tokens.test.ts` kiểm đủ token và tương phản ≥ 4.5:1, `src/lib/no-hardcoded-colors.test.ts` chặn class màu cứng ngoài `components/ui`. Font đóng gói offline (`@fontsource-variable`).

## Trang đọc và hồ sơ truyện

- Chiều ngang vùng đọc (hẹp / vừa / rộng / toàn màn) chọn ngay trên thanh tab của trang đọc hoặc ở
  Cài đặt → Giao diện; lưu `readingWidth` trong `config.json`. Nút chương trước / sau nằm cố định ở
  đầu trang, cặp nút cuối bài vẫn giữ.
- Glossary: nhóm dài thu gọn sẵn, bảng hiện theo khúc 50 dòng; nút **Sửa dạng văn bản** mở textarea
  mỗi dòng `Hán=Việt` để sửa hàng loạt rồi Áp dụng một lần. Nhóm **Xưng hô theo cặp** (`addressing`)
  dùng key `甲→乙`, value `X–Y` (甲 tự xưng X, gọi 乙 là Y); cả agent lẫn động cơ API trích thêm cặp
  mới sau mỗi chương để chương sau giữ nguyên cách xưng hô.
- Prompt và rule kiểm tra: lệnh `story_defaults` trả prompt gốc + bộ rule mặc định của hệ. Ô prompt
  luôn hiện nội dung đang dùng (sửa trên bản mặc định là thành prompt riêng, **Về mặc định** lưu
  trống); rule trống hiện bộ mặc định chỉ đọc, **Sửa bộ mặc định** sao chép ra để chỉnh.

## Thể loại

`story.json` có `genre: { setting: "ancient" | "modern" | "mixed", names: "han" | "foreign" | "mixed" }`; chọn ở trang Hồ sơ truyện, nhóm Thể loại trong tab Thông tin. Bối cảnh quyết xưng hô, thán từ, từ gia đình, bảng thuật ngữ và bộ rule kiểm tra mặc định; `mixed` (xuyên qua lại, đô thị tu tiên) đưa cả hai bộ xưng hô vào prompt để chọn theo cảnh và chỉ chạy rule trung lập. Tên riêng quyết phiên Hán-Việt hay trả về dạng gốc. Truyện cũ thiếu `genre` chạy như cổ đại/Hán-Việt. Prompt riêng hoặc rule riêng vẫn thắng. Chữ prompt nằm ở qt-web (`src/lib/ai-translation-prompt.ts`), Rust đọc 6 bản ghép sẵn trong `crates/qt-ai-core/prompts/prompts.json` qua golden.

## Drift với qt-web

Prompt/rule của GUI khớp qt-web qua golden fixtures. Trước khi mở PR: `npm --prefix apps/qt-ai-cli run -s golden:check`; đỏ thì chạy `golden` rồi sửa Rust cho `cargo test -p qt-ai-core` xanh.
