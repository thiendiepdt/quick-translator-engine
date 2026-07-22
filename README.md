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

Input đi qua đầy đủ pipeline `StandardizeInput` của QT2025: giản thể hóa, HTML decode,
đổi dấu câu/full-width, chèn khoảng trắng và loại ignored phrases; source ranges vẫn trỏ
về đúng offset UTF-16 của input gốc.

Luật Nhân tổng quát (`{s}`, `{n}`, `{h}{t}`) và chuyển số Hán đã có; tra nghĩa LacViet vẫn
nằm trong lộ trình tiếp theo. Xem
[kiến trúc](docs/architecture.md) và [đặc tả engine](docs/engine/).

Chạy HTTP server từ thư mục repo:

```powershell
$env:QT_DATA_DIR="QT2025"
$env:QT_PORT="3000"
cargo run -p qt-api --bin qt-server
```

Các tham số engine là optional và mặc định vẫn là QT2025 (`scanRange=30`,
`translationAlgorithm=1`, `prioritizedName=true`):

```powershell
Get-Content input.txt | cargo run -p qt-cli -- translate --mode vietphrase-one `
  --data-dir QT2025 --scan-range 30 --translation-algorithm 1 `
  --prioritized-name true

curl.exe -X POST http://localhost:3000/translate `
  -H "content-type: application/json" `
  -d '{"text":"他很好","mode":"vietphrase","scanRange":30,"translationAlgorithm":1,"prioritizedName":true}'
```

## Nguồn gốc

`QT2025/` chứa bộ từ điển và config của app gốc; code engine đã decompile nằm trong
`reference/decompiled/` để đối chiếu hành vi.
