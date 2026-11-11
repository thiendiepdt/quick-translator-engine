---
description: Vòng lặp dịch batch next → dịch → check → accept qua tool MCP qt-ai
---

# /translate — vòng lặp dịch batch

Mọi bước gọi tool MCP `qt_*` của server qt-ai, không chạy lệnh terminal; đọc, ghi file bằng tool file.
Lặp cho tới khi hết chương hoặc chạm giới hạn chương/phiên (xem AGENTS.md luật 4):

1. `qt_next` → nhận id chương + đường dẫn `work/<id>.prompt.md`.
   - Nếu báo còn chương translating dở: xử lý chương đó trước (bước 2–5) thay vì lấy chương mới.
   - Nếu báo hết hàng đợi: gọi `qt_status`, tổng kết cho người dùng, dừng.
2. Đọc TOÀN BỘ `work/<id>.prompt.md` và làm đúng theo nó: dịch, ghi `work/<id>.draft.md`
   (giữ nhãn [[n]]), ghi `work/<id>.glossary.json`.
3. `qt_check` với id chương
   - FAIL còn lượt sửa: đọc `work/<id>.review.md`, sửa đúng chỗ trong `work/<id>.draft.md`
     (dịch bổ sung đoạn thiếu / thay cụm vi phạm, KHÔNG viết lại chỗ khác), rồi gọi lại bước 3.
   - Báo "hết vòng review … chốt kèm cảnh báo": coi như PASS, sang bước 4 (người dùng xem cảnh báo sau).
   - Báo "quá số vòng review → error" (thiếu đoạn/quá ngắn): bỏ chương này, quay lại bước 1.
4. `qt_accept` với id chương — không tự ý đặt `force`; force là quyết định của người dùng.
5. Báo một dòng tiến độ (`x/y chương của phiên`) rồi quay lại bước 1.

Bị `blocked by content safety filters` → sinh lại bản dịch tối đa 3 lượt theo AGENTS.md luật 6; chỉ `qt_skip` (kèm lý do) khi cả 3 lượt đều bị chặn.

Không thấy tool `qt_*` nào: làm đúng các bước trên bằng CLI dự phòng — `{{QT_AI}} next {{STORY_ROOT}}`, `{{QT_AI}} check {{STORY_ROOT}} <id>`, `{{QT_AI}} accept {{STORY_ROOT}} <id>`, `{{QT_AI}} status {{STORY_ROOT}}`.
