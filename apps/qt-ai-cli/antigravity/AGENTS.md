# Workspace dịch truyện — điều khiển bằng CLI qt-ai

Thư mục truyện: `{{STORY_ROOT}}`
Chạy CLI (dùng NGUYÊN VĂN, đừng tự chế đường dẫn khác):

    {{QT_AI}} <lệnh> {{STORY_ROOT}} [chương] [cờ]

Lệnh: `init` · `next` · `check <id>` · `accept <id> [--force]` · `skip <id> --reason <lý do>` · `retry <id>` · `export [--from <id> --to <id>]` · `status`

## Luật bắt buộc

1. **Mọi tiến độ nằm trong file, không nằm trong trí nhớ của mày.** Bắt đầu phiên bằng `status`. Không bao giờ tự sửa `state.json`, `story.json` bằng tay — chỉ qua lệnh CLI (trừ workflow /setup-story được phép điền `story.json`).
2. **Dịch đúng một chương một lúc** theo vòng lặp trong `.agent/workflows/translate.md`. Không dịch gộp, không nhảy chương.
3. **Vệ sinh context:** không đọc lại out/ của các chương đã xong; sau khi `accept`, quên nội dung chương đó đi; chỉ giữ trong đầu chương đang dịch.
4. **Giới hạn phiên:** dịch tối đa số chương/phiên ghi trong dòng cuối của `status` (mặc định 10). Đủ số thì dừng, chạy `status`, báo người dùng mở phiên mới.
5. **Bản dịch phải đủ 100% số đoạn, giữ nhãn [[n]].** `check` sẽ bắt lỗi thiếu — sửa theo `work/<id>.review.md` chứ không cãi.
6. **Nếu bị chính sách nội dung chặn không dịch được chương nào:** KHÔNG chế lại nội dung, không tóm tắt thay thế. Chạy `skip <id> --reason "model từ chối: <mô tả ngắn>"` rồi sang chương kế.
7. Không sửa file trong `raw/`. Không xoá gì trong `out/`.
