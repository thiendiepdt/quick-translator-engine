# Nút "Dịch lại cả N" cho chương hổng — thiết kế

## Vấn đề

Dòng cảnh báo "N chương trước #X chưa dịch" ở toolbar Dịch liệt kê chương skip/lỗi nằm trước chương
done cuối. Muốn dịch lại phải mở từng chương bấm Dịch lại, hoặc dùng "Dịch lại…" theo khoảng — nhưng
khoảng sẽ kéo cả chương done nằm giữa về hàng đợi (đổi out/ thành .bak).

## Thiết kế

- **UI**: nút `Dịch lại cả N` ở cuối dòng cảnh báo hổng, cùng hàng chip. N = số chương hổng đang
  `skipped`/`error` (chương hổng đang `queued` đã ở hàng đợi, không đếm). Không có chương nào như vậy
  thì không hiện nút. Mờ khi phiên đang chạy (như Xoá…) hoặc đang gửi lệnh.
- **Hành vi**: bấm là đưa các chương đó về hàng đợi ngay, không dialog xác nhận (không mất dữ liệu —
  chỉ chương done mới bị đổi .bak, chương hổng chưa done). Xong tải lại snapshot, toast
  "Đã đưa N chương về hàng đợi — bấm Bắt đầu để dịch". Phiên mới lấy chương queued theo thứ tự tự
  nhiên nên các chương này chạy trước.
- **Core**: `run_retry_ids(root, ids) -> RetryRangeOutcome` trong `commands/retry.rs`, cùng luật
  với `run_retry_range` (queued → already_queued, done → backed_up) và dùng chung vòng lặp
  `requeue_many`. Id lạ → `StoryNotFound`, không đổi gì. Id trùng gộp lại, kết quả theo thứ tự tự nhiên.
- **Tauri**: lệnh `chapters_retry_ids(root, ids)` trả `RetryRangeOutcomeView`, chặn khi phiên đang
  chạy như `chapters_retry`.
- **Frontend**: `chaptersRetryIds(root, ids)` trong `lib/api.ts`; toolbar lọc `gapInfo.gaps` bỏ
  `queued` rồi gọi.

## Không làm

- Không tự khởi động phiên sau khi đưa về hàng đợi.
- Không sửa nội dung raw hay đổi model: chương skip vì model từ chối sẽ bị từ chối lại nếu không đổi gì.

## Kiểm thử

- Core: `run_retry_ids` với hỗn hợp error/skipped/queued/done, id lạ, id trùng.
- GUI: nút hiện đúng N, gọi API đúng danh sách id, khoá khi phiên chạy, không hiện khi hổng toàn queued.
