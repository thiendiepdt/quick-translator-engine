# QT-CLI

Quick Translator (dịch Trung → Việt theo từ điển) dưới dạng **CLI + HTTP API**, reimplement
bằng Rust từ engine gốc `QT2025` (.NET WinForms). Mục tiêu: kết quả dịch **y hệt** bản gốc,
nhưng chạy headless, dễ tích hợp vào tool/server dịch truyện, và mở rộng được.

## Trạng thái

MVP đã có đủ ba crate:

- `qt-core`: HanViet, VietPhrase, VietPhraseOneMeaning, chuyển số Hán và ánh xạ
  source↔target theo UTF-16.
- `qt-cli`: đọc văn bản từ stdin và ghi bản dịch ra stdout.
- `qt-api`: HTTP server với health check, danh sách mode, dịch đơn và dịch batch.

Các luật số `{s}` độc lập đã có; Luật Nhân tổng quát và tra nghĩa LacViet vẫn nằm trong
lộ trình tiếp theo. Xem
[kiến trúc](docs/architecture.md) và [đặc tả engine](docs/engine/).

Chạy HTTP server từ thư mục repo:

```powershell
$env:QT_DATA_DIR="QT2025"
$env:QT_PORT="3000"
cargo run -p qt-api --bin qt-server
```

## Nguồn gốc

`QT2025/` chứa bộ từ điển và config của app gốc; code engine đã decompile nằm trong
`reference/decompiled/` để đối chiếu hành vi.
