# Security Policy

## Phạm vi hỗ trợ

Nhánh `main` là phiên bản duy nhất đang được duy trì. Dự án chưa phát hành stable release.

## Báo cáo lỗ hổng

Không đăng token, dữ liệu từ điển riêng tư, request chứa nội dung nhạy cảm hoặc chi tiết
khai thác lên public issue. Hãy dùng GitHub Private Vulnerability Reporting trong tab
**Security** của repository nếu tính năng đó được bật.

Nếu chưa có kênh báo cáo riêng tư, chỉ mở issue công khai với mô tả tối thiểu rằng cần một
kênh liên hệ bảo mật; không đính kèm proof-of-concept hoặc secret.

## Lưu ý khi vận hành `qt-server`

Server hiện:

- bind `0.0.0.0`;
- không có TLS hoặc authentication;
- không có rate limit;
- giới hạn JSON body ở 5 MiB nhưng chưa giới hạn số phần tử batch hay số entry dictionary;
- xử lý dịch bằng blocking worker.

Vì vậy không expose trực tiếp ra Internet. Khi triển khai ngoài máy cá nhân hoặc mạng tin
cậy, đặt server sau reverse proxy có TLS, authentication, body-size limit, timeout,
concurrency limit và logging đã loại nội dung nhạy cảm.

Dictionary custom là dữ liệu không tin cậy. Đặc biệt, Luật Nhân có biểu thức regex được
compile theo request; nên áp dụng timeout và quota CPU ở lớp triển khai.

Không đặt credential trong `QT2025/`, config được track, command line hoặc log. Dùng secret
manager/environment của nền tảng triển khai và chạy secret scanner trước khi push.
