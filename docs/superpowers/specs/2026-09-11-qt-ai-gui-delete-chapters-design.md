# qt-ai: xoá chương trong app + quét gỡ chương raw đã mất (giữ out/)

Ngày: 2026-09-11. Phạm vi: `crates/qt-ai-core` (init, lệnh delete mới, CLI), `apps/qt-ai-gui` (Rust command +
frontend). Không đổi format `state.json`.

## Vấn đề

1. **Bug user báo:** xoá file trong `raw/` rồi mở app → toast "Lỗi IO …raw\0024….txt: The system cannot find
   the file specified", bấm Quét lại cũng không gỡ chương. Nguyên nhân: `run_init` cố ý giữ chương **done**
   khi raw mất (lý do cũ: bản dịch trong out/ vẫn dùng được), và `chapter_view` đọc raw để hiện tab Gốc nên nổ
   lỗi khi bấm vào chương đó.
2. **Thiếu tính năng:** không xoá được chương từ trong app; phải ra Explorer xoá file rồi quét.

## Nguyên tắc chốt với user

- **Bản dịch trong `out/` không bao giờ bị xoá** bởi hai luồng này. Quét lại và Xoá chương chỉ đụng
  `state.json`, `raw/`, `work/`.
- Xoá chương là xoá hẳn file raw (không có thùng rác), có dialog xác nhận, chặn khi phiên đang chạy.
- Có cả xoá lẻ (trong reader) và xoá theo khoảng Từ/Đến (dialog giống Dịch lại).

## Thiết kế

### Core: `run_init` gỡ luôn chương done khi raw mất

`crates/qt-ai-core/src/commands/init.rs`: bỏ điều kiện `status != Done` khi tính `gone`. Chương raw mất →
gỡ khỏi state, dọn `work/`, **không** đụng `out/<id>.txt`. Thông báo đổi thành
`"… N gỡ vì raw đã mất (bản dịch trong out/ giữ nguyên)"` khi có chương gỡ. Test hiện có
`init_go_chuong_raw_da_mat_tru_chuong_done_va_don_work` đổi tên và kỳ vọng: 0001 done cũng bị gỡ, file
`out/0001.txt` vẫn còn.

Hệ quả: `open_story` (gọi `run_init`) và Quét lại đều tự dọn chương mất → hết lỗi IO khi vào app.

### Core: `chapter_view` chịu được raw mất

Trong khoảng giữa hai lần quét vẫn có thể gặp raw mất (xoá khi app đang mở). `chapter_view` trả `raw: ""`
và `raw_missing: true` thay vì lỗi. Frontend: schema thêm `rawMissing: z.boolean().default(false)`; tab Gốc
hiện dòng "File raw/<id>.txt đã mất — bấm Quét lại để gỡ chương." thay cho nội dung.

### Core: lệnh `delete` mới (`crates/qt-ai-core/src/commands/delete.rs`)

```rust
pub struct DeleteOutcome { pub removed: Vec<String>, pub kept_outputs: Vec<String> }
pub fn run_delete(root: &Path, ids: &[String]) -> Result<DeleteOutcome>
```

- Kiểm mọi `id` có trong state trước, thiếu id nào → `StoryNotFound`, không xoá gì.
- Với từng id: `state.chapters.shift_remove`, `fs::remove_file(raw/<id>.txt)` (bỏ qua NotFound), xoá 5 file
  `work/` (bỏ qua lỗi). `out/<id>.txt` giữ nguyên; nếu file đó tồn tại thì ghi vào `kept_outputs`.
- Ghi state một lần ở cuối. `removed` theo thứ tự tự nhiên.
- Không kiểm trạng thái translating: caller (GUI) chặn khi phiên chạy; CLI tự chịu.
- CLI `qt-ai delete <id> [<id>…]` in "Đã xoá N chương (M bản dịch trong out/ giữ nguyên)."

### GUI Rust: command `chapters_delete(root, ids: Vec<String>) -> DeleteOutcomeView { removed, keptOutputs }`

Chặn `session_locked` như `chapters_retry`. Đăng ký trong `lib.rs`.

### Frontend

- `api.ts`: `chaptersDelete(root, ids)`; schema `deleteOutcomeSchema`.
- **Xoá lẻ** ([chapter-reader.tsx](../../../apps/qt-ai-gui/src/components/chapter-reader.tsx)): nút
  "Xoá chương" (icon Trash2, variant ghost, tông destructive) cạnh "Chốt --force", disabled khi phiên chạy.
  Bấm → `AlertDialog`/`Dialog` xác nhận: "Xoá chương #N <id>? File raw/<id>.txt sẽ bị xoá, không hoàn tác.
  Bản dịch trong out/ (nếu có) giữ nguyên." Xác nhận → gọi `chaptersDelete(root, [id])`, nạp lại snapshot,
  toast; chương đang chọn biến mất nên reader hiện "Chọn một chương bên trái" (store `select(undefined)`
  hoặc TranslatePage tự thấy `row` undefined — chọn cách thứ hai, không thêm state).
- **Xoá theo khoảng**: `DeleteRangeDialog` (`src/components/delete-range-dialog.tsx`), cùng khuôn
  `RetryRangeDialog`: hai ô Từ/Đến nhận số thứ tự hoặc mã, `ChapterRefHint`, xem trước "Sẽ xoá N chương,
  trong đó M chương đã dịch xong (bản dịch trong out/ giữ nguyên)". Khác retry: **không cho để trống cả
  hai** (xoá cả truyện phải gõ rõ 1–N). Nút xác nhận `destructive` "Xoá N chương". Tính danh sách id trong
  khoảng ở frontend (helper `idsInRange(chapters, fromId, toId)` trong `lib/export-range.ts`, tái dùng logic
  `previewRange`) rồi gọi `chaptersDelete(root, ids)`.
- Mở dialog từ toolbar: nút "Xoá…" (variant outline, icon Trash2) cạnh "Dịch lại…", disabled khi phiên chạy.

## Test

- Core (`crates/qt-ai-core/tests/commands.rs`): init gỡ chương done raw mất nhưng out/ còn; `run_delete`
  xoá raw + work + state, giữ out/, báo `kept_outputs`; id lạ → StoryNotFound không xoá gì.
- GUI Rust (`story_cmds.rs` tests, chạy bằng `cargo clippy --all-targets` vì `cargo test` src-tauri hỏng trên
  máy này): `chapter_view` với raw mất trả `raw_missing`.
- Frontend vitest: `delete-range-dialog.test.tsx` (xem trước, chặn trống cả hai, gọi `chaptersDelete` đúng
  ids); `chapter-reader.test.tsx` thêm ca xoá lẻ (dialog xác nhận → gọi API → snapshot nạp lại) và tab Gốc
  khi `rawMissing`; `translate-toolbar.test.tsx` thêm nút "Xoá…" khoá khi phiên chạy.

## Ngoài phạm vi

Thùng rác/khôi phục, xoá bản dịch trong out/, xoá theo bộ lọc trạng thái.
