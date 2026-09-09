# qt-ai-gui: thư viện truyện, tạo truyện mới, quét lại, kéo thả chương

Ngày 2026-09-09. Bổ sung cho spec `2026-09-01-qt-ai-gui-design.md` và redesign `2026-09-03`.

## Mục tiêu

Người dùng không phải tự tạo folder rồi "Mở folder truyện": chọn một **folder cha (thư viện)** một lần,
tạo truyện mới ngay trong app, thả file chương vào cửa sổ để nạp `raw/`. Cách mở folder thủ công giữ nguyên.

## 1. Thư viện (folder cha)

- `AppConfig.libraryRoot: string | null` (Rust `library_root`, zod `libraryRoot`), mặc định `null`.
- Chọn ở màn picker (nút "Chọn thư viện", dialog chọn folder) hoặc trong Cài đặt → App.
- Lệnh `library_list()` → `Vec<RecentSummary>` (tái dùng kiểu `{root, name, done, total}`):
  thư mục con **trực tiếp** của `libraryRoot`; folder chưa init (`state.json` không đọc được) có
  `name = null, total = null`. Sắp theo mtime của `state.json` mới nhất lên đầu; folder chưa init xếp cuối
  theo tên. `libraryRoot` null hoặc không tồn tại → danh sách rỗng, không lỗi.

## 2. Tạo truyện mới

- Nút "Tạo truyện mới" trên picker, chỉ bật khi có `libraryRoot`.
- Dialog: **Tên truyện tiếng Việt** (bắt buộc), **Tên folder** tự sinh từ tên qua lệnh `slugify(name)`
  và sửa tay được (khi người dùng đã sửa tay thì không tự sinh nữa), **Link truyện** (tuỳ chọn).
  Hiện đường dẫn đầy đủ `<libraryRoot>/<slug>` sẽ tạo.
- `slugify` (Rust, có test): NFD bỏ dấu, `đ→d`, chữ thường, mọi ký tự ngoài `[a-z0-9]` → `-`, gộp `-`
  liên tiếp, cắt `-` hai đầu, tối đa 80 ký tự. Rỗng sau khi chuẩn hoá → `"truyen"`.
- Lệnh `create_story(name, slug, source_url)` → `StorySnapshot`:
  - từ chối `invalid_state` nếu chưa có `libraryRoot`, slug rỗng, slug chứa ký tự ngoài `[a-z0-9-]`,
    hoặc `<libraryRoot>/<slug>` đã tồn tại;
  - tạo `<libraryRoot>/<slug>/raw/`, `run_init`, ghi `name`, `sourceUrl` vào `story.json`;
  - `touch_recent` + lưu config, trả snapshot; UI `openStory(snapshot)` như mở thường.
- Trang Dịch khi `counts.total == 0`: thay danh sách bằng ghi chú "Chưa có chương — thả file .txt vào đây
  hoặc copy vào raw/ rồi bấm Quét lại" + nút "Mở folder raw/" (`reveal_folder`).

## 3. Quét chương mới

- `open_story` gọi `run_init` (idempotent: chỉ thêm chương raw mới vào hàng đợi, không đè template đã có)
  trước khi dựng snapshot. Folder không có `story.json`/`state.json` vẫn trả `story_not_found` như cũ để
  giữ luồng "Khởi tạo?" — tức chỉ chạy `run_init` khi `state.json` đã tồn tại.
- Lệnh `rescan_story(root)` → `StorySnapshot`: `run_init` rồi snapshot. Nút "Quét lại" trên toolbar
  trang Dịch (tắt khi phiên đang chạy), toast số chương mới.

## 4. Kéo thả chương

- Lắng nghe `getCurrentWebview().onDragDropEvent` trong hook `useChapterDrop(root)` ở trang Dịch
  (chỉ khi `screen === "workbench"`). `enter/over` → overlay "Thả để thêm vào raw/" phủ vùng danh sách chương;
  `leave` → tắt; `drop` → gọi `import_chapters(root, paths)`.
- Lệnh `import_chapters(root, paths: Vec<String>)` → `ImportOutcome { added: Vec<String>, skipped_existing:
  Vec<String>, ignored: Vec<String>, snapshot: StorySnapshot }`:
  - mỗi path: file `.txt` → ứng viên; thư mục → các `.txt` ngay trong nó (không đệ quy); còn lại → `ignored`;
  - tên đích = tên file gốc; đã có trong `raw/` → `skipped_existing`, không ghi đè;
  - copy xong `run_init` rồi snapshot. Đang chạy phiên vẫn cho thả (runner đọc `state.json` lúc lấy chương kế).
- Toast: "Đã thêm N chương" + ", bỏ qua M trùng tên" + ", K file không phải .txt" khi khác 0.

## 5. Picker

Thứ tự từ trên xuống: tiêu đề; hàng nút **Tạo truyện mới** + **Mở folder truyện**; mục **Thư viện**
(đường dẫn + nút "Đổi"; chưa chọn thì ô mời "Chọn thư viện"; danh sách truyện, mỗi dòng bấm để mở —
folder chưa init đi qua luồng hỏi khởi tạo hiện có); mục **Mở gần đây** ẩn các root đã có trong thư viện,
vẫn có nút X bỏ khỏi danh sách.

## Không làm

Crawl từ link, import khi tạo truyện, đệ quy thư mục con, đổi tên/xoá truyện, thư viện nhiều folder cha.

## Kiểm thử

- Rust (Tauri crate, tempdir): `slugify`; `create_story` tạo đúng cấu trúc, ghi name, từ chối trùng/slug xấu;
  `library_list` phân biệt init/chưa init và thứ tự; `open_story` thấy chương raw mới; `import_chapters`
  copy/bỏ trùng/bỏ đuôi khác/lấy `.txt` trong folder một cấp và state có chương mới `queued`.
- GUI (vitest): dialog tạo truyện (slug tự sinh qua mock `slugify`, ngừng tự sinh khi sửa tay, submit đúng
  tham số); picker liệt kê thư viện, ẩn trùng ở gần đây, nút tạo tắt khi chưa có thư viện; toolbar Quét lại;
  hook kéo thả gọi `import_chapters` với đúng paths và toast đúng số.
