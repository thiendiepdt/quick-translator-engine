# QT Engine Reference

Đặc tả **độc lập ngôn ngữ triển khai** của engine dịch Quick Translator, được phân tích từ
`TranslatorEngine.dll`. Tài liệu này mô tả hành vi tham chiếu; source Rust hiện hành nằm
trong `crates/qt-core` và có thể ghi rõ một contract mở rộng khi cần cho CLI/API.

## Đọc theo thứ tự

1. [overview.md](overview.md) — pipeline tổng quát, tham số, thứ tự ưu tiên.
2. [translation-algorithm.md](translation-algorithm.md) — main loop `TranslateAll`, longest-match,
   name-priority, ghép chuỗi & viết hoa. **Trọng tâm.**
3. [dictionaries.md](dictionaries.md) — mọi từ điển: format, load order, priority merge, cache.
4. [han-viet.md](han-viet.md) — phiên âm Hán Việt từng chữ + `isChinese` + `ToNarrow`.
5. [luat-nhan.md](luat-nhan.md) — luật văn phạm `{n}` / `{s}` / `{h}{t}`.
6. [number-conversion.md](number-conversion.md) — số Hán → Việt (đơn vị, dải, thập phân).
7. [meanings-lacviet.md](meanings-lacviet.md) — tra cứu nghĩa chi tiết (mode phụ).

## Quy ước

- Pseudocode bám sát tên biến/hàm trong bản decompile để dễ đối chiếu
  (`reference/decompiled/TranslatorEngine.decompiled.cs`).
- Mọi hành vi "dễ sai khi tái tạo" được đánh dấu bằng blockquote hoặc **in đậm**.
- Ưu tiên verify: HanViet → VietPhraseOneMeaning → VietPhrase → LuatNhan/số.

## Phạm vi và độ tin cậy

Đặc tả dựa trên **đọc trực tiếp code decompile**. Các giá trị cấu hình đã được xác nhận từ
`QT2025/Resources/QuickTranslatorMain.config`: `scanRange=30`,
`TranslationAlgorithm=1`, `PrioritizedName=true`. Source-level regression tests đã bao phủ
các nhánh chính, nhưng golden corpus lấy trực tiếp từ ứng dụng QT thật vẫn chưa đầy đủ cho
Luật Nhân, chuyển số và các trường hợp ghép câu phức tạp.

Những khác biệt có chủ đích của Rust—đặc biệt non-BMP và HanViet ranges—được ghi ngay tại
tài liệu tương ứng, không được trình bày như hành vi nguyên bản của QT2025.
