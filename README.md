# QT-CLI

Quick Translator (dịch Trung → Việt theo từ điển) dưới dạng **CLI + HTTP API**, reimplement
bằng Rust từ engine gốc `QT2025` (.NET WinForms). Mục tiêu: kết quả dịch **y hệt** bản gốc,
nhưng chạy headless, dễ tích hợp vào tool/server dịch truyện, và mở rộng được.

## Trạng thái

🚧 Giai đoạn **tài liệu** (docs-first). Xem [docs/](docs/):

- [docs/architecture.md](docs/architecture.md) — thiết kế hệ thống, lộ trình.
- [docs/engine/](docs/engine/) — đặc tả thuật toán engine "y hệt" (từ decompile).

Code (`crates/qt-core`, `qt-cli`, `qt-api`) sẽ triển khai theo spec ở giai đoạn sau.

## Nguồn gốc

`QT2025/` chứa app gốc để tham chiếu (engine `TranslatorEngine.dll`, bộ từ điển). Không build.
