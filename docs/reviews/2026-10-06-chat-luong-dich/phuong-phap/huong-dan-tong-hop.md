# Hướng dẫn tổng hợp một nhóm lỗi

Bối cảnh: đã review chất lượng bản dịch AI (app qt-ai-gui, repo `D:\Work\projects\quick-translator-engine`) của 61 truyện trong `D:\Work\projects\bookshelf\books\common`. Mỗi truyện có báo cáo `scratchpad\reviews\<slug>.md/.json`. Tổng 899 lỗi đã gắn thẻ. Nhiệm vụ của bạn: tổng hợp MỘT nhóm lỗi thành một chương của bản đánh giá cuối, kèm truy nguyên vào code/prompt và đề xuất sửa cụ thể. Chỉ đọc; chỉ ghi đúng 1 file đầu ra.

Thư mục: `C:\Users\Chiro\AppData\Local\Temp\claude\d--Work-projects-quick-translator-engine\d47bac56-20f0-4192-82dd-d8e4ed70ac57\scratchpad\`
- `synth\<nhóm>.md` — danh sách lỗi của nhóm bạn, mỗi dòng: `[THẺ|mức|HT=hệ thống/le=lẻ|đếm] slug (setting) chương:dòng RAW「…」 → 「bản dịch」 ⇒ 「đề xuất」 — vì sao`. Đọc HẾT file này.
- `synth\goi_y_va_lam_tot.md` — gợi ý + điểm làm tốt của từng truyện (lớn; grep theo từ khoá của nhóm thay vì đọc hết).
- `reviews\<slug>.md` — đọc thêm khi cần chi tiết.

Code liên quan (đọc để xác định NGUYÊN NHÂN GỐC, ghi `file:dòng`):
- Prompt dịch: `crates/qt-ai-core/prompts/prompts.json` (bases theo `setting/names`, `tones`, `suffix`; file sinh từ qt-web qua gen-golden — tìm nguồn thật trong `apps/qt-web` hoặc `apps/qt-ai-cli` và ghi rõ chỗ phải sửa). Bản xem nhanh đã tách: `scratchpad\base_ancient_han.md`, `scratchpad\base_modern_mixed.md`.
- Lắp prompt: `crates/qt-ai-core/src/prompt.rs`; xưng hô/cast: `src/cast.rs`, `src/cast_scan.rs`; glossary tự sinh: `src/glossary.rs`, chỉ dẫn trích glossary trong `src/api_session.rs` (≈ dòng 30–70) và `src/api_fill.rs`; hồ sơ: `src/story.rs`; kiểm tra: `src/check.rs`, `src/commands/check.rs`; accept: `src/commands/accept.rs`; tiêu đề: `src/titles.rs`.
- Một truyện chỉ có MỘT `genre.setting` (ancient/modern/mixed) trong story.json.

Sự thật đã được xác minh (dùng, đừng mâu thuẫn):
1. Prompt cổ đại luật 11 bắt `嗯`→"Ân", `嗯?`→"A?"; tone "witty" trong cùng prompts.json lại xếp "Ân, cả bàn…" vào cột "Khô (tránh)". Đếm toàn thư viện ~3.200 "Ân".
2. Luật "Lời kể gián tiếp" đưa `bản thân hắn` làm ví dụ cho 自己; ~5.100 "bản thân hắn/cô/nàng" toàn thư viện.
3. Tiêu đề viết hoa mọi chữ (moi-ngay 1043–1045) do model, KHÔNG do code titles (titles.json chỉ ghi lại dòng đầu đã dịch). Tiêu đề là câu thoại mà dùng "ta" là do luật người kể "ta" áp cả vào tiêu đề.
4. phuong-thon-dao-chu: raw/ được nhập lại 06/10 07:39, mất ch382 nên số file lùi; out/0392 (dịch ch382) nằm cạnh raw/0392 (ch383) mà state vẫn "done" — app không phát hiện raw đã dịch bị đổi nội dung.
5. Toàn thư viện (regex): "đem " ~10.000, "người phụ nữ" ~4.100, "Hừ" ~4.300.

## Đầu ra: `synth\<nhóm>_report.md` (tiếng Việt, ~1500–2500 chữ)

1. Tổng quan nhóm: số lỗi, số truyện dính, mức nặng.
2. Các VẤN ĐỀ HỆ THỐNG (gộp các lỗi cùng gốc), xếp theo ảnh hưởng. Mỗi vấn đề:
   - Mô tả + số truyện dính (đếm slug) + số liệu đếm tiêu biểu.
   - 2–3 ví dụ tiêu biểu: RAW → hiện tại → đề xuất (lấy nguyên văn từ digest, ghi slug + chương).
   - Nguyên nhân gốc: prompt / glossary tự sinh / cast / setting / model — CHỈ khẳng định khi đã thấy trong code hoặc prompt (ghi file:dòng); không chắc thì ghi "nghi".
   - Đề xuất sửa cụ thể: câu chữ prompt mới, thay đổi code (hàm nào, làm gì), luật check mới (regex/điều kiện). Ước lượng công sức (nhỏ/vừa/lớn) và ưu tiên (P0/P1/P2).
3. Lỗi lẻ nhưng đáng chú ý (ngắn).
4. Điều đang làm tốt trong nhóm này (để không sửa nhầm).

Không bịa số liệu: chỉ dùng số trong digest/báo cáo hoặc tự đếm lại bằng grep. Trả lời cho người gọi ≤100 chữ: đã ghi file + 3 vấn đề lớn nhất.
