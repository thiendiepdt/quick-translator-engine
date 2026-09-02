---
description: Vòng lặp dịch batch next → dịch → check → accept qua CLI qt-ai
---

# /translate — vòng lặp dịch batch

Lặp cho tới khi hết chương hoặc chạm giới hạn chương/phiên (xem AGENTS.md luật 4):

1. `{{QT_AI}} next {{STORY_ROOT}}` → nhận id chương + đường dẫn `work/<id>.prompt.md`.
   - Nếu báo còn chương translating dở: xử lý chương đó trước (bước 2–5) thay vì lấy chương mới.
   - Nếu báo hết hàng đợi: chạy `status`, tổng kết cho người dùng, dừng.
2. Đọc TOÀN BỘ `work/<id>.prompt.md` và làm đúng theo nó: dịch, ghi `work/<id>.draft.md`
   (giữ nhãn [[n]]), ghi `work/<id>.glossary.json`.
3. `{{QT_AI}} check {{STORY_ROOT}} <id>`
   - FAIL còn lượt sửa: đọc `work/<id>.review.md`, sửa đúng chỗ trong `work/<id>.draft.md`
     (dịch bổ sung đoạn thiếu / thay cụm vi phạm, KHÔNG viết lại chỗ khác), rồi chạy lại bước 3.
   - Báo "hết vòng review … chốt kèm cảnh báo": coi như PASS, sang bước 4 (người dùng xem cảnh báo sau).
   - Báo "quá số vòng review → error" (thiếu đoạn/quá ngắn): bỏ chương này, quay lại bước 1.
4. `{{QT_AI}} accept {{STORY_ROOT}} <id>` — không tự ý dùng `--force`; force là quyết định của người dùng.
5. Báo một dòng tiến độ (`x/y chương của phiên`) rồi quay lại bước 1.

Model từ chối dịch vì chính sách nội dung → làm theo AGENTS.md luật 6 (skip kèm lý do).
