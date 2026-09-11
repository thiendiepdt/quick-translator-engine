# QT AI Translator — thiết kế lại giao diện + hệ theme

Ngày: 2026-09-03. Phạm vi: tầng giao diện `apps/qt-ai-gui/src` và 2 trường cấu hình mới trong Rust `AppConfig`. Không đụng `crates/qt-ai-core`, `api.ts`, `schema.ts`, `story-form.ts`.

## Mục tiêu

Bản MVP (plan 2026-09-01) dựng UI để chạy được: ui kit copy nguyên từ qt-gui, chỉ có token sáng, bố cục nhét dialog, màu hardcode. Người dùng đánh giá xấu ở cả bố cục, màu, chữ. Thiết kế lại toàn bộ lớp giao diện, giữ nguyên logic store/api, và cho người dùng chọn theme.

## Quyết định đã chốt

- Ba bộ màu người dùng tự chọn: **Editorial** (giấy ấm, đọc bằng serif, nhấn đất nung), **Studio** (graphite + teal, số liệu monospace, mật độ cao), **Soft** (bo tròn, tím-hồng, nhiều khoảng trắng). Mỗi bộ có **sáng / tối / theo hệ thống**.
- Bố cục: **rail trái, mỗi mục là một trang**, bỏ toàn bộ dialog Hồ sơ / Cài đặt / Export (dialog nhỏ như "Bỏ qua chương", "Khởi tạo folder" vẫn giữ).

## Hệ theme

- Hai trục độc lập: `palette: "editorial" | "studio" | "soft"` gắn `data-palette` lên `<html>`; `themeMode: "light" | "dark" | "system"` do next-themes quản (`attribute="class"`, bỏ `forcedTheme`, `enableSystem`).
- `src/index.css` viết lại: mỗi bộ màu định nghĩa **đủ** bộ token shadcn (`--background --foreground --card --card-foreground --popover --popover-foreground --primary --primary-foreground --secondary --secondary-foreground --muted --muted-foreground --accent --accent-foreground --destructive --border --input --ring --radius`) cho sáng và tối (`:root[data-palette=X]` và `.dark[data-palette=X]`). Token riêng của app: `--status-done --status-error --status-warning --status-queued --status-translating`, `--font-ui --font-reading --font-mono`, `--reading-size`. Component **chỉ dùng token**; cấm class màu cứng kiểu `bg-amber-50`, `text-zinc-100`.
- Bán kính: Editorial 6px, Studio 4px, Soft 12px (qua `--radius`).
- Font đóng gói offline qua `@fontsource-variable`: Inter (UI, tất cả bộ), Noto Serif (vùng đọc của Editorial), JetBrains Mono (số liệu/log của Studio, log của mọi bộ). Soft dùng Inter cho cả UI và đọc. Không tải font qua mạng.
- Lưu trong `AppConfig` (Rust: `palette: String` default `"editorial"`, `theme_mode: String` default `"system"`; camelCase `palette`, `themeMode`). Lúc khởi động app đọc config rồi gắn `data-palette` trước khi render để không nháy màu; next-themes nhận `defaultTheme` từ config và ghi lại khi đổi.
- Chức năng theme là module thuần `src/lib/theme.ts`: `PALETTES` (id, tên, mô tả, màu xem trước), `applyPalette(document, palette)`, `isPalette(x)`, `isThemeMode(x)`; test được không cần Tauri.

## Khung app

- Rail trái 56px cố định khi đã mở truyện: logo, 4 mục **Dịch / Hồ sơ / Export / Cài đặt** (icon + tooltip), dưới cùng nút đổi sáng↔tối nhanh và nút "Về danh sách truyện". Trang hiện tại là `page` trong store.
- Store thêm `page: "translate" | "story" | "export" | "settings"` và `setPage`. `openStory` đặt `page = "translate"`.
- Màn kiểm tra agy và màn chọn truyện thiết kế lại cùng ngôn ngữ: một cột giữa rộng tối đa 640px, tiêu đề lớn, mô tả ngắn. "Mở gần đây" là danh sách thẻ hiện **tên truyện + tiến độ** (đọc từ `story.json`/`state.json` qua command `recent_summaries` mới, trả `{root, name, done, total}`; folder không đọc được thì hiện đường dẫn và mờ).

## Trang Dịch

- Thanh trên: tên truyện, thanh tiến độ mỏng + `done/tổng`, chip đếm chờ / lỗi / bỏ qua / cảnh báo theo màu trạng thái, chọn model, nút Bắt đầu / Dừng nổi bật. Đang chạy: viền thanh đổi màu nhấn, hiện "Phiên N · đang dịch 0046", nút thành Dừng (destructive).
- Cột trái 280px: ô tìm chương (lọc theo mã, không phân biệt hoa thường), chip lọc trạng thái **Tất cả / Chờ / Đang dịch / Xong / Cảnh báo / Lỗi / Bỏ qua** (Cảnh báo = done có warnings). Hàng: chấm màu trạng thái, mã chương (mono), "soát N" khi N > 0, huy hiệu số cảnh báo. Hàng chọn có nền `accent` và vạch trái `primary`.
- Cột phải: dải trạng thái chương (mã, trạng thái, lý do, số cảnh báo) cùng hàng với **Dịch lại / Bỏ qua / Chốt --force / Mở folder**; điều kiện bật/tắt như hiện tại. Cảnh báo liệt kê dưới dải, nền `warning/10`. Tab **Bản dịch / Nháp / Yêu cầu sửa / Gốc / Log**. Vùng đọc: `max-width: 70ch`, font `--font-reading`, cỡ `--reading-size` (16px), line-height 1.75; cuối bài có nút chương trước / sau.
- Log: nền tối ở cả hai chế độ (token `--log-bg/--log-fg` riêng), mono, tự cuộn, nút xoá, đếm dòng; stderr màu warning.

## Trang Hồ sơ

- Hai cột: trái là mục lục neo (Thông tin, Style, Glossary → 7 nhóm, Rule, Prompt) cuộn theo; phải là form dài một trang, mỗi mục là một card. Thanh dưới dính đáy: **Nhập JSON · Xuất JSON · AI điền · Lưu** (Lưu disabled khi form không dirty hoặc đang có phiên).
- Glossary mỗi nhóm: ô tìm nhanh lọc theo CN/VN, đếm số dòng, nút Thêm; dòng: input CN (mono) · input VN · xoá.
- AI điền giữ là dialog (chạy agy, diff, Áp dụng / Bỏ) vì là tác vụ modal.
- Logic form (`story-form.ts`, RHF, zod) giữ nguyên.

## Trang Export

- Hai ô "Từ chương / Đến chương" dạng combobox có gợi ý theo mã; mặc định chương done đầu và cuối. Dưới đó hiện **xem trước**: số chương sẽ gộp và danh sách hổng (tính từ snapshot, không gọi Rust) trước khi bấm. Hai nút: "Export vào export/" và "Chọn nơi lưu…". Kết quả: đường dẫn + Mở folder.

## Trang Cài đặt

- Ba khối card: **Giao diện** (3 thẻ xem trước bộ màu chọn bằng click; chọn chế độ Sáng / Tối / Theo hệ thống), **App** (đường dẫn agy, model mặc định, số phiên tối đa), **Truyện này** (chương/phiên, vòng soát, tỉ lệ). Giao diện áp ngay khi chọn và lưu `AppConfig`; hai khối kia lưu bằng nút Lưu như hiện tại.

## Rust

- `AppConfig` thêm `palette`, `theme_mode` (default như trên, `#[serde(default)]` đã có ở struct).
- Command mới `recent_summaries() -> Vec<RecentSummary { root, name: Option<String>, done: Option<usize>, total: Option<usize> }>` đọc từng folder trong `config.recent`, lỗi thì trả `None`.

## Kiểm

- Test tự động (vitest): `theme.ts` (applyPalette gắn attribute, guard kiểu); store `page` và `openStory` reset về `translate`; chapter-list lọc chip + tìm (thay test chapter-table); `index.css` có đủ token: test đọc file CSS và kiểm mỗi `data-palette` × sáng/tối khai báo đủ danh sách token bắt buộc.
- Rust: `AppConfig` default có `palette = "editorial"`, `theme_mode = "system"`; `recent_summaries` với 1 folder hợp lệ + 1 folder hỏng.
- Kiểm tay: chụp từng trang ở 3 bộ × sáng/tối; script `scripts/contrast-check.mjs` đọc token từ `index.css`, tính tương phản `foreground/background`, `muted-foreground/background`, `primary-foreground/primary` ≥ 4.5:1 cho mọi bộ × chế độ, chạy trong `npm run check`.

## Ngoài phạm vi

- Không đổi hợp đồng folder truyện, command Rust hiện có, hay core.
- Không làm theme tuỳ biến màu tự do; chỉ 3 bộ cố định.
- Không đổi ui kit sang thư viện khác; vẫn shadcn (radix-ui), chỉ chỉnh token và biến thể.
