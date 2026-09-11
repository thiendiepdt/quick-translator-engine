# qt-ai-gui: chọn khoảng export bằng số thứ tự + danh sách chương

Ngày: 2026-09-11. Phạm vi: `apps/qt-ai-gui`, chỉ frontend. Không đụng Rust, không đổi lệnh `export_chapters`.

## Vấn đề

Trang Export nhận cả số thứ tự lẫn mã chương (dùng chung `resolveChapterRef` với dialog Dịch lại), nhưng
người dùng không biết vì:

1. Hai ô Từ/Đến được điền sẵn **nguyên mã chương** của chương done đầu/cuối, nên trông như bắt buộc gõ mã.
2. Gõ số xong không có gợi ý "→ mã chương nào" (dialog Dịch lại có `RefHint`, trang Export không).
3. Không có danh sách chương trên trang để biết số mấy là chương nào, đã done hay chưa.

## Thiết kế (phương án B đã chốt)

### Bố cục

Trang Export chuyển thành hai cột giống trang Dịch: `grid-cols-[300px_1fr]`.

- **Trái:** danh sách chương (`ExportChapterList`, component mới).
- **Phải:** form Từ/Đến, ô xem trước, nút export, khối kết quả — giữ nguyên nội dung hiện có.

### Ô nhập Từ/Đến

- Điền sẵn **số thứ tự** (1-based, như cột `#`) của chương done đầu và cuối; không còn điền mã.
- `inputMode="numeric"`, vẫn nhận nguyên mã chương như trước.
- Dưới mỗi ô có `ChapterRefHint`: "→ <mã>" khi khớp, "không có chương này" khi không khớp, ẩn khi ô trống.
- Mô tả trang ghi rõ: gõ số thứ tự 1–N như cột `#`, hoặc bấm Từ/Đến trong danh sách; để trống = đầu/cuối.
- Tên file gợi ý khi "Chọn nơi lưu…" dùng **mã chương** đã resolve (`<mã từ>-<mã đến>.txt`), không dùng chuỗi gõ.

### `ExportChapterList` (`src/components/export-chapter-list.tsx`)

Props: `rows`, `fromIndex`, `toIndex` (vị trí đã resolve, -1 nếu trống/không khớp), `onPickFrom(ordinal)`,
`onPickTo(ordinal)`.

- Ô tìm + chip lọc trạng thái, tái dùng `filterChapters`/`countByFilter`/`FILTER_ORDER` từ `lib/chapters`.
  Trạng thái lọc/tìm là state cục bộ của component, **không** dùng `statusFilter`/`searchQuery` trong store để
  không làm lệch bộ lọc của trang Dịch.
- Mỗi dòng: chấm trạng thái, `#ordinal`, mã chương (như `ChapterList`). Không bấm để mở chương.
- Mỗi dòng có hai nút nhỏ "Từ" và "Đến" (hiện rõ khi hover/focus, luôn có trong DOM để bàn phím dùng được),
  `aria-label` "Từ chương #N" / "Đến chương #N". Bấm gọi `onPickFrom(N)` / `onPickTo(N)`; trang Export ghi
  `String(N)` vào ô tương ứng.
- Dòng trong khoảng `[fromIndex, toIndex]` (khi cả hai hợp lệ và from ≤ to) tô `bg-accent`; dòng trong khoảng
  mà chưa done thêm nhãn "hổng" (màu `text-status-warning`). Khi chỉ một đầu hợp lệ thì khoảng tính từ đầu
  danh sách hoặc tới cuối, khớp với `previewRange`.
- Không ảo hoá danh sách: trang Dịch đã vẽ toàn bộ chương cùng cách, chấp nhận như nhau.

### `ChapterRefHint` (`src/components/chapter-ref-hint.tsx`)

Tách `RefHint` hiện nằm trong `retry-range-dialog.tsx` ra file chung, giữ nguyên hành vi; dialog Dịch lại và
trang Export cùng import.

## Luồng dữ liệu

```
from/to (chuỗi) --resolveChapterRef--> fromIndex/toIndex
                                        ├─> fromId/toId ("\0" khi có chữ mà không khớp) --previewRange--> preview
                                        └─> ExportChapterList tô khoảng
ExportChapterList onPickFrom/onPickTo --> setFrom(String(N)) / setTo(String(N))
```

## Test (vitest + testing-library)

- `export-chapter-list.test.tsx`: dòng trong khoảng có đánh dấu, dòng chưa done trong khoảng có nhãn "hổng",
  bấm nút Từ/Đến gọi callback đúng số thứ tự, lọc/tìm hoạt động.
- `export-page.test.tsx` (mock `@/lib/api`, nạp `useStoryStore`): ô Từ/Đến điền sẵn dạng số thứ tự; gợi ý
  "→ mã" hiện; bấm "Đến chương #N" trong danh sách đổi ô Đến; gõ số không tồn tại báo lỗi và khoá nút;
  export gọi `exportChapters` với `from/to` là mã chương.
- Test retry dialog hiện có vẫn phải xanh sau khi tách `RefHint`.

## Ngoài phạm vi

- Ảo hoá danh sách dài, kéo chọn khoảng bằng chuột, export nhiều khoảng một lần.
