# Tải raw — mốc 2: link tải raw, mẫu tên file, khớp chương, trang Tải raw

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Từ trang "Tải raw" của app, tải mục lục và chương qua extension đã đặt trong thư mục extension, ghi `raw/` theo mẫu tên file, khớp chương đã có, rồi đưa chương mới vào hàng đợi dịch.

**Architecture:** Logic nằm ở `qt-ai-core::raw_download` (phụ thuộc `qt-ext`): cấu hình `download.json`, mẫu tên, chuẩn hoá nội dung, khớp chương, danh sách extension, và hàm chạy một lượt tải dùng `qt_ext::Session`. App (`raw_cmds.rs`) chỉ gọi các hàm đó trên thread nền, phát event tiến độ, rồi quét `raw/` bằng `run_init` như nút "Quét raw/" đang làm. Auth, cài đặt, cache, session data, webview là mốc 3–4; cài extension từ file/link là mốc 5. Ở mốc này extension được nạp từ `<config>/extensions/<id>/pkg/`.

**Tech Stack:** Rust (qt-ai-core, qt-ext, Tauri 2), React + zod + vitest (apps/qt-ai-gui).

Spec: [docs/superpowers/specs/2026-11-07-tai-raw-extension-design.md](../specs/2026-11-07-tai-raw-extension-design.md) mục 3, 10.

## Global Constraints

- Không chạy `cargo fmt`. Không đưa tên app đọc truyện bên thứ ba vào repo.
- Không động vào file untracked của task khác (`QT-hop-nhat/`, `tools/`, `vp-*`, `corpus/`, `1`, `docs/reviews/2026-10-21-hop-nhat-tu-dien/`).
- `download.json` không chứa khoá, mật khẩu, cookie, session.
- Mẫu tên mặc định `{chuong:4}_{chapter-id}`; mẫu phải có `{chuong}` hoặc `{chuong:N}`; tên tối đa 120 ký tự; bỏ `\/:*?"<>|` và ký tự điều khiển; trùng thì `-2`, `-3`.
- Chỉ nội dung trống (không còn chữ nào sau dòng tiêu đề) là lỗi; dưới 50 ký tự gắn nhãn "Ngắn".
- File raw: dòng đầu tiêu đề, một dòng trống, rồi các đoạn mỗi đoạn một dòng.
- Thử lại lỗi tạm 2 lần, cách 2 rồi 6 giây. Tải lại chương đã có thì chép bản cũ vào `work/raw-backup/<id>.<thời điểm>.txt`.
- App: không có test lib chạy được (binary test Tauri lỗi 0xc0000139 từ trước) → kiểm Rust app bằng `cargo clippy --all-targets`; GUI bằng `npm run check`.

## File structure

```
crates/qt-ext/src/testkit.rs                 máy chủ HTTP giả dùng chung cho test (chuyển từ tests/common)
crates/qt-ai-core/src/raw_download/mod.rs    DownloadConfig (download.json), effective_raw_url, re-export
crates/qt-ai-core/src/raw_download/template.rs   kiểm/render mẫu tên, đoán mẫu từ tên cũ, kiểm thứ tự
crates/qt-ai-core/src/raw_download/content.rs    html → text, chèn tiêu đề, trống/ngắn
crates/qt-ai-core/src/raw_download/matching.rs   khớp mục lục với raw đã có (sổ / thứ tự / mốc), kiểm tiêu đề
crates/qt-ai-core/src/raw_download/registry.rs   đọc extension trong thư mục, chọn theo link
crates/qt-ai-core/src/raw_download/run.rs        lượt tải: toc, chọn chương theo chế độ, ghi file, sổ, thử lại
crates/qt-ai-core/tests/raw_download.rs          chạy trọn vòng với máy chủ giả + extension mẫu
apps/qt-ai-gui/src-tauri/src/raw_cmds.rs     lệnh Tauri: raw_info, raw_save, raw_toc, raw_download, raw_download_cancel
apps/qt-ai-gui/src/lib/raw.ts (+ test)       kiểm mẫu tên phía GUI (cùng luật Rust), nhãn trạng thái
apps/qt-ai-gui/src/components/pages/raw-page.tsx (+ test)
```

---

### Task 1: `download.json`

**Files:** Create `crates/qt-ai-core/src/raw_download/mod.rs`; Modify `crates/qt-ai-core/src/lib.rs`, `crates/qt-ai-core/Cargo.toml` (thêm `qt-ext = { path = "../qt-ext" }`).

**Interfaces (Produces):**
- `pub const DOWNLOAD_JSON: &str = "download.json";`
- `#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)] #[serde(rename_all = "camelCase", default)] pub struct DownloadConfig { pub raw_url: String, pub extension: String, pub file_template: String, pub settings: IndexMap<String, serde_json::Map<String, Value>>, pub chapters: IndexMap<String, String> }`
- `pub fn load_download(paths: &StoryPaths) -> DownloadConfig` (thiếu hoặc hỏng → mặc định; hỏng thì chép sang `download.json.bad`).
- `pub fn save_download(paths: &StoryPaths, config: &DownloadConfig) -> Result<()>` (ghi an toàn, JSON thụt 2, xuống dòng cuối).
- `pub fn effective_raw_url(config: &DownloadConfig, story: &StoryConfig) -> String` (rỗng → `story.source_url`).

Test: ghi rồi đọc lại bằng nhau; file hỏng → mặc định + có `.bad`; `effective_raw_url` lấy link nguồn khi rỗng, khoảng trắng tính là rỗng.

Commit: `feat(core): download.json — link tải raw, extension, mẫu tên, sổ chương của truyện`.

### Task 2: Mẫu tên file

**Files:** Create `crates/qt-ai-core/src/raw_download/template.rs`.

**Interfaces:**
- `pub const DEFAULT_TEMPLATE: &str = "{chuong:4}_{chapter-id}";`
- `pub fn validate_template(template: &str) -> Result<(), String>` — token hợp lệ: `{chuong}`, `{chuong:N}` (N 1–9), `{chapter-id}`, `{chapterTitle}`, `{volume}`; thiếu `{chuong…}` hoặc token lạ → lỗi tiếng Việt nêu token.
- `pub struct NameParts<'a> { pub index: usize, pub id: &'a str, pub title: &'a str, pub volume: Option<&'a str> }`
- `pub fn render(template: &str, parts: &NameParts) -> String` — thay token, bỏ ký tự cấm và điều khiển, gộp khoảng trắng, bỏ khoảng trắng và dấu chấm cuối, cắt 120 ký tự (theo ký tự, không cắt giữa ký tự).
- `pub fn unique_stem(stem: String, taken: &HashSet<String>) -> String`
- `pub fn guess_template(existing: &[String]) -> Option<String>` — theo spec 10.3 (toàn `chuong-\d+`; toàn `\d+_\d+`; toàn `<tiền tố chung>_\d+_…`), `W` = số chữ số dài nhất.
- `pub fn sorts_after(candidate: &str, last: &str) -> bool` — `natural_chapter_compare(candidate, last) == Greater`.

Test: render `{chuong:4}_{chapter-id}` (5, "7445") → `0005_7445`; `{chuong}` 12 → `12`; `{chuong:2}` 123 → `123`; tiêu đề có `?:*` bị bỏ; tên 300 ký tự còn 120; `unique_stem` thêm `-2`, `-3`; `validate_template` từ chối `{chapterTitle}`, `{foo}`, `{chuong:0}`; `guess_template` cho `chuong-0001..chuong-1619` → `chuong-{chuong:4}`, `0001_744…` → `{chuong:4}_{chapter-id}`, `142631_0001_第1章 …` → `142631_{chuong:4}_{chapterTitle}`, lẫn lộn → `None`; `sorts_after("0975_x", "chuong-0974")` false (chữ số đứng trước chữ), `sorts_after("chuong-0975", "chuong-0974")` true.

Commit: `feat(core): mẫu tên file raw — token, ký tự cấm, đoán mẫu từ tên cũ, kiểm thứ tự`.

### Task 3: Chuẩn hoá nội dung

**Files:** Create `crates/qt-ai-core/src/raw_download/content.rs`.

**Interfaces:**
- `pub struct Prepared { pub text: String, pub body_chars: usize, pub short: bool }`
- `pub fn html_to_text(html: &str) -> String` — `<br>`, `</p>`, `</div>`, `<p>`, `<div>` thành xuống dòng; bỏ mọi thẻ; giải `&amp; &lt; &gt; &quot; &#39; &apos; &nbsp;`, `&#NNN;`, `&#xHH;`.
- `pub fn prepare(content: &str, html: bool, title: &str) -> Option<Prepared>` — chuẩn hoá xuống dòng (`\r\n`, `\r`), cắt khoảng trắng đầu cuối từng dòng (kể cả khoảng trắng toàn khổ `\u{3000}`), bỏ dòng trống; dòng đầu bằng tiêu đề (so sau khi bỏ khoảng trắng) thì giữ, không thì chèn tiêu đề; thân rỗng → `None`; kết quả `"{title}\n\n{đoạn}\n{đoạn}…\n"`; `short` = thân dưới 50 ký tự.

Test: html có `<p>`, `<br/>`, thực thể; nội dung đã có tiêu đề không bị chèn lặp; chương xin nghỉ "今晚事务繁忙，先请个假。" → `Some`, `short: true`; chỉ có tiêu đề hoặc toàn khoảng trắng → `None`.

Commit: `feat(core): chuẩn hoá nội dung chương tải về — html thành dòng, chèn tiêu đề, chỉ trống mới là lỗi`.

### Task 4: Khớp chương đã có

**Files:** Create `crates/qt-ai-core/src/raw_download/matching.rs`.

**Interfaces:**
- `pub struct TocRef<'a> { pub id: &'a str, pub title: &'a str }`
- `pub struct Existing { pub id: String, pub title: String }` (id file raw theo thứ tự tự nhiên, title = dòng đầu file).
- `pub enum MatchedBy { Ledger, Order, Empty }`
- `pub struct Mismatch { pub index: usize, pub toc_title: String, pub file_id: String, pub file_title: String }`
- `pub struct Matching { pub files: Vec<Option<String>> /* theo vị trí mục lục */, pub by: MatchedBy, pub last_index: Option<usize>, pub mismatches: Vec<Mismatch>, pub unmatched_files: usize }`
- `pub fn match_existing(toc: &[TocRef], ledger: &IndexMap<String, String>, existing: &[Existing], anchor: Option<usize>) -> Matching`
  - Sổ có ít nhất một mã khớp mục lục → dùng sổ (chỉ mục có file còn tồn tại), `by = Ledger`, không kiểm tiêu đề.
  - Không có sổ: `anchor` = vị trí (từ 1) của chương cuối đã có, mặc định `existing.len()`; file thứ j (0-based) ↔ mục `j + anchor - existing.len()`; file rơi ra ngoài mục lục đếm vào `unmatched_files`.
  - Kiểm tiêu đề ở chương cuối và 5 vị trí rải đều: hai bên bỏ khoảng trắng và dấu câu, chuyển phồn sang giản (`zh_convert::to_simplified`), khớp khi bằng nhau hoặc bên này chứa bên kia; lệch thì đưa vào `mismatches`.
- `pub fn title_matches(a: &str, b: &str) -> bool`

Test: sổ khớp theo mã kể cả khi mục lục chèn thêm chương ở giữa; thứ tự mặc định; `anchor` dời đúng; lệch tiêu đề bị báo, phồn/giản và thiếu tiền tố "第N章" không bị báo; truyện mới (không file) → `Empty`, `last_index None`.

Commit: `feat(core): khớp mục lục với raw đã có — theo sổ, theo thứ tự có mốc, báo lệch tiêu đề`.

### Task 5: Danh sách extension

**Files:** Create `crates/qt-ai-core/src/raw_download/registry.rs`.

**Interfaces:**
- `pub struct Installed { pub package: qt_ext::Package, pub dir: PathBuf, pub dev: bool }`
- `pub fn list_installed(extensions_dir: &Path) -> (Vec<Installed>, Vec<String> /* lỗi nạp, mỗi dòng "<thư mục>: <lỗi>" */)` — đọc `<dir>/<id>/pkg/` (allow_local = false), bỏ thư mục không có `pkg/`.
- `pub fn candidates<'a>(installed: &'a [Installed], url: &str) -> Vec<&'a Installed>` — `manifest.matches_url`.
- `pub fn pick<'a>(installed: &'a [Installed], url: &str, chosen: &str) -> Option<&'a Installed>` — `chosen` còn khớp thì lấy, không thì đúng một ứng viên thì lấy, nhiều thì `None`.

Test: thư mục tạm có 2 extension hợp lệ + 1 manifest hỏng; ứng viên theo link; `pick` khi 0/1/2 ứng viên và khi đã chọn.

Commit: `feat(core): đọc extension đã cài, chọn theo link tải raw`.

### Task 6: Lượt tải

**Files:** Create `crates/qt-ai-core/src/raw_download/run.rs`, `crates/qt-ext/src/testkit.rs` (chuyển `TestServer`, `Route` từ `crates/qt-ext/tests/common/mod.rs`, tests của qt-ext dùng lại qua `qt_ext::testkit`); Test `crates/qt-ai-core/tests/raw_download.rs`.

**Interfaces:**
- `pub enum Mode { New, Selected(Vec<String>), Single(String) }`
- `pub enum RowStatus { Have, Missing, Locked }`
- `pub struct TocRow { pub index: usize, pub id: String, pub title: String, pub locked: bool, pub file: Option<String>, pub status: RowStatus }`
- `pub struct TocView { pub rows: Vec<TocRow>, pub by: MatchedBy, pub last_index: Option<usize>, pub mismatches: Vec<Mismatch>, pub unmatched_files: usize, pub next_name: String, pub next_sorts_after: bool }`
- `pub struct RunOptions { pub extension: Installed, pub session: qt_ext::SessionOptions, pub anchor: Option<usize>, pub retry_delays: Vec<Duration> /* mặc định [2s, 6s] */ }`
- `pub enum Event { Toc { total: usize }, Chapter { done: usize, total: usize, index: usize, title: String, outcome: ChapterResult }, Log(String) }`
- `pub enum ChapterResult { Saved { file: String, short: bool, replaced: bool }, Locked, Failed(String) }`
- `pub struct Summary { pub saved: Vec<usize>, pub short: Vec<usize>, pub locked: Vec<usize>, pub failed: Vec<(usize, String, String)> /* vị trí, tiêu đề, lý do */, pub cancelled: bool }`
- `pub fn load_toc(root: &Path, opts: &RunOptions, cancel: Arc<AtomicBool>) -> Result<(Vec<qt_ext::TocItem>, TocView)>` — dựng `ctx`, gọi `toc`, khớp, trả cả mục lục để lượt tải dùng lại.
- `pub fn download(root: &Path, opts: &RunOptions, toc: Option<Vec<qt_ext::TocItem>>, mode: Mode, cancel: Arc<AtomicBool>, on_event: &mut dyn FnMut(Event)) -> Result<Summary>`
  - Chương cần tải: `New` = sau `last_index`, bỏ chương khoá; `Selected` / `Single` = đúng mã (chương khoá vẫn thử, extension có thể có auth sau này).
  - Mỗi kết quả: `Locked` → Khoá; lỗi → giữ để thử lại nếu là `Network`, `Timeout`, `Extension { retry: true }`, `TooLarge` không; nội dung → `content::prepare`; `None` → Lỗi "nội dung trống".
  - Ghi: tên = mục đã có trong sổ/khớp thì giữ tên cũ (ghi đè, có backup); mới thì `render` + `unique_stem` trên tập id raw hiện có. `write_atomic` vào `raw/<tên>.txt`, cập nhật `chapters` trong `download.json` ngay sau mỗi chương.
  - Lượt đầu với truyện cũ: ghi cả các cặp đã khớp theo thứ tự vào sổ.
  - Thử lại: sau lượt chính, các chương lỗi tạm chạy lại sau `retry_delays[0]`, rồi `retry_delays[1]`.
  - `ctx`: `{ sourceUrl, rawUrl, story: { name, chapterCount, current }, request: { mode, ids }, auth: null, session: null, settings: <mặc định manifest đè bởi download.json> }`.
- Test trọn vòng (máy chủ giả + `qt-ext` fixture demo chép vào thư mục extension tạm, cài như `<dir>/<id>/pkg`):
  - truyện mới: tải chương mới → 4 file `0001_1`…`0004_4`, chương 5 Khoá, chương 6 Lỗi sau 2 lần thử lại (delay 0 trong test), sổ có 4 mục, nội dung đúng định dạng (tiêu đề, dòng trống, đoạn);
  - lần 2 "Tải chương mới" không tải lại gì; `Single("2")` ghi đè có backup trong `work/raw-backup/`;
  - truyện cũ có `chuong-0001`, `chuong-0002` tiêu đề khớp: mẫu đoán `chuong-{chuong:4}`, tải tiếp từ chương 3, sổ có cả 2 chương cũ;
  - truyện cũ tiêu đề lệch → `mismatches` khác rỗng; với `anchor` đúng thì hết lệch;
  - chương ngắn được ghi và nằm trong `short`.

Commit: `feat(core): lượt tải raw qua extension — chọn chương theo chế độ, ghi file theo mẫu, sổ chương, thử lại, backup`.

### Task 7: Lệnh Tauri

**Files:** Create `apps/qt-ai-gui/src-tauri/src/raw_cmds.rs`; Modify `lib.rs` (đăng ký lệnh, danh sách heavy), `library_cmds.rs` (`create_story` nhận `raw_url`, khác link nguồn thì ghi `download.json`).

**Interfaces (JSON camelCase):**
- `raw_info(root) -> { rawUrl, rawUrlIsDefault, sourceUrl, extension: Ext | null, candidates: Ext[], loadErrors: string[], fileTemplate, defaultTemplate, guessedTemplate: string | null, extensionsDir }`, `Ext = { id, name, version, description, developer: { name, url?, email? }, hosts: string[] }`.
- `raw_save(root, rawUrl, extension, fileTemplate) -> RawInfo` (kiểm mẫu bằng `validate_template`; `rawUrl` bằng link nguồn thì lưu rỗng).
- `raw_toc(root, anchor?) -> TocView` (cache mục lục theo truyện trong `AppState`).
- `raw_download(root, mode: "new" | "selected" | "single", ids: string[], anchor?) -> Summary`; event `raw-download-progress` `{ root, kind: "toc" | "chapter" | "log", done, total, index, title, status: "saved" | "short" | "locked" | "failed", message }`; xong thì `run_init` + `refresh_quietly` như `rescan_story`.
- `raw_download_cancel(root)`.
- Một lượt tải mỗi truyện; lượt thứ hai cùng truyện → lỗi "Đang tải raw".

Kiểm: `cargo clippy --all-targets` trong `apps/qt-ai-gui/src-tauri` không thêm cảnh báo.

Commit: `feat(gui): lệnh Tauri tải raw — thông tin, lưu, mục lục, tải, huỷ`.

### Task 8: Giao diện

**Files:** Modify `src/lib/schema.ts`, `src/lib/api.ts`, `src/store/story.ts` (`Page` thêm `"raw"`), `src/components/app-rail.tsx` (mục "Tải raw", icon `CloudDownload`, giữa Hồ sơ truyện và Kiểm tra VN), `src/App.tsx` (`PAGES.raw`), `src/components/create-story-dialog.tsx` (+ test: ô "Link tải raw" chép theo link nguồn tới khi sửa tay, gửi `rawUrl`), `src/components/pages/story-page.tsx` (ô "Link tải raw" dưới "Link nguồn", lưu qua `rawSave` khi bấm Lưu); Create `src/lib/raw.ts` (+ test: `validateTemplate`, `renderPreview` cùng luật Rust), `src/components/pages/raw-page.tsx` (+ test).

Trang Tải raw:
- Đầu trang: ô link (mặc định link nguồn, có chữ "theo link nguồn"), chọn extension (một / nhiều / không có — không có thì chỉ đường dẫn thư mục extension), ô mẫu tên có kiểm ngay và xem trước tên chương kế tiếp; có mẫu đoán khác mẫu đang dùng thì nút "Dùng mẫu theo tên cũ".
- "Tải mục lục" → bảng: vị trí, tiêu đề, trạng thái (Đã có / Chưa có / Khoá), ô tick, nút tải lẻ / tải lại. Lệch tiêu đề → hộp cảnh báo liệt kê cặp lệch + ô "chương cuối đã có là mục số" + nút Khớp lại.
- "Tải chương mới", "Tải đã chọn (N)", thanh tiến độ, log, Huỷ; xong thì bảng tổng kết (đã tải, ngắn, khoá, lỗi kèm lý do) và gọi `rescanStory`.

Test (vitest): chưa có extension → hướng dẫn; tải mục lục hiển thị trạng thái; mẫu sai chặn nút lưu; lệch tiêu đề hiện ô mốc; bấm tải gọi `rawDownload` đúng chế độ và id.

Kiểm: `npm run check` trong `apps/qt-ai-gui`.

Commit: `feat(gui): trang Tải raw, ô Link tải raw khi tạo truyện và trong Hồ sơ truyện`.

### Task 9: CHANGELOG

Thêm mục vào "## Chưa phát hành" của `apps/qt-ai-gui/CHANGELOG.md`: trang Tải raw qua extension (chưa có kho/cài từ app — đặt gói vào thư mục extension), ô Link tải raw. Commit `docs(gui): CHANGELOG — tải raw qua extension`.
