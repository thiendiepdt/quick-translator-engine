# VNCVT AI Translator (qt-ai-gui)

App desktop dịch truyện Trung → Việt hàng loạt: điều khiển Antigravity CLI (`agy`) theo vòng phiên, hoặc gọi thẳng Gemini / API OpenAI-compatible bằng key của người dùng; logic dịch/kiểm tra/glossary nằm trong crate `qt-ai-core` (port 1-1 từ `apps/qt-ai-cli` + qt-web).

## Thư viện, tạo truyện mới, kéo thả chương

- **Thư viện** (`libraryRoot` trong config): một folder cha chứa mọi truyện. Chọn ở màn chọn truyện hoặc Cài đặt.
  Màn chọn truyện liệt kê thư mục con trực tiếp (truyện đã init có tiến độ; folder lạ bấm vào sẽ hỏi khởi tạo).
- **Tạo truyện mới**: tên tiếng Việt → tên folder tự sinh (`ta-tuyet-the-chi-dau`, sửa được) → tạo `<thư viện>/<slug>/raw/`,
  init, ghi tên + link vào `story.json`, mở luôn. Chương thêm sau.
- **Kéo thả**: đang mở truyện, thả file `.txt` (hoặc folder chứa `.txt`, không đệ quy) vào cửa sổ → copy vào `raw/`
  (không ghi đè file trùng tên, đuôi hạ thường) rồi đưa chương mới vào hàng đợi. Copy tay vào `raw/` thì bấm **Quét lại** (cũng gỡ chương có file raw đã mất, trừ chương đã dịch xong). **Dịch lại…** đưa toàn bộ hoặc một khoảng chương về hàng đợi (chương done giữ bản cũ thành `out/<id>.txt.bak`)
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

Hồ sơ truyện → **Thể loại** có ba trục: Bối cảnh (xưng hô, thuật ngữ), Tên riêng (cách phiên) và **Giọng văn**:
Trung tính (mặc định, prompt như trước), Ngôn tình (truyện nữ: ngọt, hài, chớt nhả), Hài hước cợt nhả (truyện nam giọng
đùa, hậu cung nhật thường: giữ punchline, thán từ tự nhiên, bớt Hán-Việt sách vở), Sảng văn dồn dập (chiến đấu, vô địch
lưu: câu ngắn, khí thế), Cổ phong trữ tình (cổ ngôn: giữ hình ảnh, nhịp cân đối), Sắc (sắc hiệp, sắc đô thị: cảnh thân
mật dịch đúng độ trực diện của raw, từ vựng truyện sắc, không nói giảm), Thanh xuân đời thường (đô thị nhẹ nhàng, giải trí
văn, học đường: thoại người trẻ tự nhiên, lời kể ấm, đoạn tả nhạc/cảm xúc giữ độ bay bổng). Giọng khác Trung tính thì prompt được
chèn thêm mục "Giọng văn: …" ngay trước "Đại từ nhân xưng", kèm ví dụ khô → đúng giọng; vẫn cấm thêm ý, và có điều
khoản cứng: không đụng bảng đại từ, không đổi cặp xưng hô trong glossary `addressing`. Mục này không hiện ở tab Prompt
(base theo bối cảnh/tên riêng vẫn sửa được như cũ); prompt riêng của truyện thì không chèn. AI điền đề xuất giọng từ
3 chương đầu. Nguồn mục ở `apps/qt-web/src/lib/ai-translation-prompt.ts` (`TONE_*`), Rust chèn qua `prompts.json` → `tones`.

Chọn ở Cài đặt → **Động cơ dịch** (lưu trong `config.json` của app):

- **Antigravity CLI (agy)** — agent chạy trong `agy -p` theo vòng phiên, tự dịch/check/accept
  bằng quota Antigravity. Cần cài agy; app chỉ dò agy khi bạn chọn động cơ này.
  Mỗi phiên chạy `agy -p … --print-timeout 3h` (agy ≥ 1.2; mặc định 5 phút của agy cắt lượt dịch nhiều chương giữa chừng).
- **API key** — mặc định cho người dùng mới: app tự gọi model qua HTTP bằng key của bạn rồi chạy cùng
  vòng next → dịch → check → accept trong `qt-ai-core` (`api_session`). Không cần agy. Provider: **Gemini** chính chủ, hoặc
  **OpenAI-compatible** (OpenAI, hay hub riêng qua Base URL, ví dụ `http://192.0.2.10/v1` với model
  `gemini-3.7-flash`). Mức nghĩ (Gemini `thinkingLevel`, OpenAI `reasoning_effort`) cấu hình riêng cho từng
  bước: dịch, soát, trích glossary, AI điền hồ sơ. Key lưu plain trong `config.json`.

Token mỗi chương (động cơ API): lượt dịch gửi base prompt + raw, model trả bản dịch kèm khối `[[glossary]]`
nên không cần lượt trích riêng (model bỏ khối thì app mới gọi lượt trích như cũ). Chỉ soát khi check còn vi phạm; model
giữ nguyên bản dịch thì chốt kèm cảnh báo ngay thay vì lặp đủ `maxReviewRounds`. Tab Log ghi token từng lượt
("dịch — vào 13.5k (cache 9.9k) · ra 3.7k · nghĩ 2.1k") và tổng ở dòng "chốt (…) · 21.3k token" khi provider báo usage
(OpenAI-compatible cần hub chuyển `stream_options.include_usage`; hub không báo thì không có dòng này).

Glossary tự động và AI điền dùng lượt "JSON mode" (`response_format`/`responseMimeType`); hub không nhận JSON mode
(400/404/422, trả rỗng) thì app tự gọi lại bằng lượt text thường rồi bóc object JSON ra — log trang Dịch ghi
"bỏ qua trích glossary — …" khi cả hai đường đều hỏng, và dòng "chốt (… +N glossary)" cho biết mỗi chương thêm bao nhiêu.

Cùng folder truyện, cùng `state.json`; đổi động cơ giữa chừng vẫn tiếp được. Ở chế độ agy, lỗi
`blocked by content safety filters` là bộ lọc đầu ra của Antigravity nhảy ngẫu nhiên theo lượt (cùng
chương lượt sau thường qua, bản dịch không bị nhạt đi): AGENTS.md luật 6 bảo agent sinh lại tối đa 3 lượt,
cả 3 đều bị chặn mới skip. AGENTS.md/workflows trong folder truyện có dòng dấu `<!-- qt-ai-template … -->`
ở cuối: file chưa sửa tay được app làm mới khi mở truyện, kể cả khi luật đổi lời. Ở chế độ API, chương
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

`npm run build:portable` → `dist-portable/VNCVT-AI-Translator-<version>-portable/` (+ `.zip` trên Windows): `VNCVT-AI-Translator.exe`
và `qt-ai.exe`. Bản portable và bản cài đặt dùng chung `config.json` ở `%APPDATA%\com.vn-converter.qt-ai-gui\`
(bản cài trước 2026-09-11 dùng `io.quicktranslator.ai-gui`; lần đầu chạy bản mới tự copy config.json cũ sang). Máy đích vẫn cần WebView2.

## Phát hành và cập nhật

Tag `qt-ai-gui-v<x.y.z>` kích hoạt `.github/workflows/release-qt-ai-gui.yml` (Windows): tauri-action build NSIS + MSI, ký bằng
secret `TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` của repo, tạo GitHub Release kèm `latest.json` và `.sig`;
bước sau gom zip portable (`PORTABLE_SKIP_BUILD=1 npm run build:portable`) và upload thêm. App đã cài dò
`releases/latest/download/latest.json` mỗi lần mở (tắt ở dev), có bản mới thì hỏi bằng hộp thoại native, tải, cài rồi khởi động lại
(`src/hooks/use-update-check.ts`). Bản portable cũng cập nhật bằng NSIS nên sau đó thành bản cài đặt.

```
npm run set-version 0.2.0     # package.json, package-lock.json, tauri.conf.json, Cargo.toml + cargo update → Cargo.lock
git commit -am "release(qt-ai-gui): 0.2.0"
git tag qt-ai-gui-v0.2.0 && git push origin HEAD --tags
```

Khoá ký sinh bằng `npx tauri signer generate -w ~/.tauri/qt-ai-gui.key --ci` (không mật khẩu, secret password để rỗng); public key
nằm ở `plugins.updater.pubkey` trong `tauri.conf.json`. Mất private key thì bản đã phát hành không nhận bản mới nữa, phải đổi pubkey
và người dùng cài tay một lần.

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
- Tab Glossary có **Export Names.txt…**: chọn nhóm/dòng rồi **Chép** (clipboard) hoặc **Lưu…** thành
  `Names.txt` / `Names2.txt` cho bản convert QT, mỗi dòng `Hán=Việt` (CRLF, UTF-8 có BOM khi lưu file).
  Mặc định tick hết trừ Cụm từ đặc trưng; Xưng hô theo cặp không export; dòng có dấu `=`, thiếu một
  cột hay trùng Hán bị bỏ vì QT vứt dòng / chỉ giữ dòng đầu.

## Thể loại

`story.json` có `genre: { setting: "ancient" | "modern" | "mixed", names: "han" | "foreign" | "mixed" }`; chọn ở trang Hồ sơ truyện, nhóm Thể loại trong tab Thông tin. Bối cảnh quyết xưng hô, thán từ, từ gia đình, bảng thuật ngữ và bộ rule kiểm tra mặc định; `mixed` (xuyên qua lại, đô thị tu tiên) đưa cả hai bộ xưng hô vào prompt để chọn theo cảnh và chỉ chạy rule trung lập. Tên riêng quyết phiên Hán-Việt hay trả về dạng gốc. Truyện cũ thiếu `genre` chạy như cổ đại/Hán-Việt. Prompt riêng hoặc rule riêng vẫn thắng. Chữ prompt nằm ở qt-web (`src/lib/ai-translation-prompt.ts`), Rust đọc 6 bản ghép sẵn trong `crates/qt-ai-core/prompts/prompts.json` qua golden.

## Bản mặc định sửa được

Cài đặt → **Bản mặc định**: sửa base prompt từng genre, bộ rule từng bối cảnh và kho glossary chung từng bối cảnh.
File rời trong thư mục cấu hình app: `base/prompts/<setting>-<names>.md`, `base/rules/<setting>.json`,
`base/glossary/<setting>.json`; không có file = bản cứng trong binary; "Về mặc định" = xoá file. Core đọc qua env
`QT_AI_BASE_DIR` (app đặt lúc khởi động, phiên agy thừa hưởng) nên app, phiên API và `qt-ai next` dùng cùng bản.
Ưu tiên: prompt/rule riêng của truyện > file base > bản cứng; glossary chung làm nền, glossary truyện đè key trùng.
Tab Prompt/Rule của truyện ghi "mặc định của app (đã sửa)" khi base đang là file.

## Drift với qt-web

Prompt/rule của GUI khớp qt-web qua golden fixtures. Trước khi mở PR: `npm --prefix apps/qt-ai-cli run -s golden:check`; đỏ thì chạy `golden` rồi sửa Rust cho `cargo test -p qt-ai-core` xanh.
