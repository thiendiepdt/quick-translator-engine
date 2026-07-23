# Quick Translator Engine

Quick Translator Engine là bản tái hiện bằng Rust của engine Quick Translator (QT2025),
phục vụ dịch Trung → Việt theo từ điển qua thư viện, CLI và HTTP API. Mục tiêu của dự án
là giữ hành vi dịch sát engine .NET gốc, đồng thời chạy headless và dễ tích hợp vào các
công cụ xử lý văn bản.

> Dự án đang ở giai đoạn phát triển. Ba mode dịch chính đã hoạt động; tra nghĩa Lạc Việt,
> quản lý từ điển và bộ golden test đối chiếu trực tiếp với ứng dụng QT vẫn chưa hoàn tất.

## Tính năng

- Ba mode: `hanviet`, `vietphrase`, `vietphrase-one`.
- Longest-match, ưu tiên tên riêng, Luật Nhân và chuyển số Hán.
- Chuẩn hóa input theo pipeline QT2025: phồn thể → giản thể, HTML entity, full-width,
  dấu câu, khoảng trắng và ignored phrases.
- Ánh xạ source ↔ target bằng offset UTF-16 để dùng trực tiếp trong JavaScript UI.
- CLI đọc UTF-8 từ `stdin`, ghi bản dịch ra `stdout`.
- HTTP API hỗ trợ dịch đơn, dịch batch, tùy chọn engine và ranges.
- Lọc name theo hai mode: QT-compatible hoặc hybrid rules + memory theo truyện; có thể
  bật thêm token-classification ONNX NER và Gemini fallback cho candidate mơ hồ.
- Ba runtime lọc name chuyên dụng: `qt-ner-cli`, `qt-ner-api` và `qt-ner-lambda`, dùng
  chung pipeline và JSON contract với `/names/filter`.
- Native AWS Lambda entrypoint dùng lại nguyên Axum router, kèm SAM template ARM64.
- Cloudflare Worker gateway ký SigV4 tới Lambda Function URL dùng `AWS_IAM`.
- React web app để dịch theo chương, tùy biến dictionary và đối chiếu source/output bằng
  UTF-16 ranges.
- Cho phép thay thế Names, Names2, Luật Nhân và các từ điển phụ theo từng lần gọi CLI/API;
  VietPhrase và ChinesePhienAmWords luôn dùng bản cố định đã nạp lúc khởi động.

## Yêu cầu

- Rust stable và Cargo.
- Một thư mục dữ liệu theo cấu trúc QT. Chỉ hai file bắt buộc khi khởi động là:
  - `Resources/ChinesePhienAmWords.txt`
  - `VietPhrase/VietPhrase.txt`

`Names.txt`, `Names2/123.txt`, `LuatNhan.txt`, `IgnoredChinesePhrases.txt`, `Pronouns.txt`,
`DanhTu.txt`, `HoNguoi.txt` và `HauTu.txt` là mặc định optional. Caller có thể thay thế
từng file trong số này cho một lần dịch; file không được truyền vẫn dùng bản trong data
directory. Trong checkout hiện tại, có thể dùng `QT2025` làm data directory.

## Chạy CLI

```bash
cargo build --release
echo "他的眼球很好。" | cargo run -q -p qt-cli -- translate \
  --data-dir QT2025 \
  --mode vietphrase-one
```

PowerShell:

```powershell
"他的眼球很好。" | cargo run -q -p qt-cli -- translate `
  --data-dir QT2025 `
  --mode vietphrase-one
```

CLI mặc định dùng `mode=vietphrase`, `data-dir=data` và các option tương thích QT2025:
`scanRange=30`, `translationAlgorithm=1`, `prioritizedName=true`.

```text
qt translate [--mode <hanviet|vietphrase|vietphrase-one>]
             [--data-dir DIR] [--wrap]
             [--scan-range 1..=100]
             [--translation-algorithm 0|1|2]
             [--prioritized-name true|false]
             [--names-file PATH] [--names2-file PATH]
             [--luat-nhan-file PATH] [--pronouns-file PATH]
             [--danh-tu-file PATH] [--ho-nguoi-file PATH]
             [--hau-tu-file PATH]
             [--ignored-chinese-phrases-file PATH]
```

Ví dụ thay Names và Pronouns cho đúng một lần dịch:

```bash
echo "萧炎看着她。" | cargo run -q -p qt-cli -- translate \
  --data-dir QT2025 \
  --mode vietphrase-one \
  --names-file ./user/Names.txt \
  --pronouns-file ./user/Pronouns.txt
```

Mỗi option file thay thế đúng dictionary tương ứng. Muốn vô hiệu dictionary mặc định,
truyền đường dẫn tới một file rỗng.

Lọc name từ `stdin`; mặc định output là các dòng tương thích Names2:

```bash
cat chapter.txt | cargo run -q -p qt-cli -- names filter \
  --data-dir QT2025 \
  --mode hybrid \
  --known-names-file ./book/accepted.txt \
  --rejected-names-file ./book/rejected.txt
```

Thêm `--json` để lấy score, loại entity, occurrences và nguồn phát hiện. Mode `qt` giữ
cửa sổ token 2–5 ký tự và ngưỡng tần suất kiểu QT; mode `hybrid` bổ sung overlapping
n-gram, ngữ cảnh giới thiệu tên, họ/hậu tố và memory xuyên chương.

## Chạy name filter chuyên dụng

`qt-ner-cli` mặc định bật ONNX NER nếu bốn biến `QT_NER_*`/`ORT_DYLIB_PATH` hợp lệ. Input
đọc từ `stdin` hoặc `--input`; output mặc định là Names2-compatible:

```bash
cat chapter.txt | cargo run -q -p qt-ner-cli -- filter \
  --data-dir QT2025 \
  --known-names-file ./book/accepted.txt \
  --rejected-names-file ./book/rejected.txt
```

Dùng `--json` để lấy nguyên response `/names/filter`, `--no-ner` để chỉ chạy rules và
`--ai-fallback` để bật Gemini reviewer đã cấu hình ở environment.

HTTP server chuyên dụng chỉ expose `GET /health`, `GET /capabilities` và
`POST /names/filter`:

```bash
QT_NER_DATA_DIR=QT2025 QT_NER_PORT=3001 \
  cargo run -q -p qt-ner-api --bin qt-ner-api
```

Các package chuyên dụng bật feature ONNX mặc định, nhưng chỉ load native runtime/model khi
`QT_NER_MODEL` được cấu hình. Xem quy trình chuẩn bị artifact tại
[docs/engine/name-filter.md](docs/engine/name-filter.md#onnx-ner) và deploy
`qt-ner-lambda` tại [deploy/aws-lambda/README.md](deploy/aws-lambda/README.md#lambda-lọc-name-chuyên-dụng).

## Chạy HTTP server

Bash:

```bash
QT_DATA_DIR=QT2025 QT_PORT=3000 cargo run -q -p qt-api --bin qt-server
```

PowerShell:

```powershell
$env:QT_DATA_DIR = "QT2025"
$env:QT_PORT = "3000"
cargo run -q -p qt-api --bin qt-server
```

Thử request:

```bash
curl -X POST http://localhost:3000/translate \
  -H "content-type: application/json" \
  -d '{"text":"萧炎看着她。","mode":"vietphrase-one","dictionaries":{"names":"萧炎=Tiêu Viêm","pronouns":"她=nàng"}}'
```

Server hiện bind `0.0.0.0`, giới hạn JSON body ở 5 MiB nhưng không có authentication hay
rate limit. Chỉ nên dùng trong mạng tin cậy hoặc đặt sau reverse proxy có các lớp bảo vệ
phù hợp; xem [SECURITY.md](SECURITY.md).

API contract đầy đủ nằm tại [docs/api.md](docs/api.md).

Để bật provider tùy chọn cho `/names/filter`, build server với `--features onnx` và cấu
hình `QT_NER_MODEL`, `QT_NER_TOKENIZER`, `QT_NER_CONFIG`; Gemini dùng
`QT_GEMINI_API_KEY` và `QT_GEMINI_MODEL`. Không cấu hình provider thì rules vẫn chạy độc
lập, không phát sinh network call. Xem hướng dẫn chọn model, export, tải ONNX Runtime và
xử lý lỗi tại [docs/engine/name-filter.md](docs/engine/name-filter.md#onnx-ner).

## Chạy web app

`apps/qt-web` dùng Vite, React, TypeScript, Tailwind CSS, shadcn/ui, React Hook Form,
TanStack Query, Zustand và Zod. Khi development, Vite proxy `/api` tới `qt-server` ở
`http://localhost:3000`, nên không cần bật CORS cho local Rust server:

```bash
QT_DATA_DIR=QT2025 QT_PORT=3000 cargo run -q -p qt-api --bin qt-server

cd apps/qt-web
npm ci
npm run dev
```

Để gọi Cloudflare gateway đã deploy, đặt `VITE_QT_API_URL` theo
[`apps/qt-web/.env.example`](apps/qt-web/.env.example) và thêm origin của web app vào
`CORS_ALLOWED_ORIGINS` của Worker. Xem [hướng dẫn qt-web](apps/qt-web/README.md).

## Kiểm tra chất lượng

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Đo tốc độ cùng precision/recall/F1 trên một chương đã gán nhãn:

```bash
cargo run --release -p qt-core --example name_filter_bench -- \
  QT2025 chapter.txt gold-names.txt 20 hybrid
```

## Cấu trúc repository

```text
crates/qt-core/   Engine và public Rust API
crates/qt-cli/    Binary qt
crates/qt-api/    Binary qt-server và HTTP handlers
crates/qt-lambda/ Native AWS Lambda entrypoint
crates/qt-ner-cli/    CLI lọc name với rules, ONNX và AI
crates/qt-ner-api/    HTTP API chỉ dành cho name filter
crates/qt-ner-lambda/ Lambda chỉ dành cho name filter
apps/qt-web/      React web app dịch và đối chiếu theo range
deploy/           Infrastructure và hướng dẫn deploy
docs/             Kiến trúc, API và đặc tả thuật toán
QT2025/           Dữ liệu/config tham chiếu từ bộ QT gốc
reference/        Source C# decompile dùng để đối chiếu hành vi
```

Đọc tiếp:

- [Kiến trúc hệ thống](docs/architecture.md)
- [HTTP API](docs/api.md)
- [Deploy lên AWS Lambda](deploy/aws-lambda/README.md)
- [Cloudflare gateway cho Lambda](deploy/cloudflare-worker/README.md)
- [Web app](apps/qt-web/README.md)
- [Đặc tả engine](docs/engine/README.md)
- [Cách tái tạo source decompile](docs/dev/decompile.md)
- [Hướng dẫn đóng góp](CONTRIBUTING.md)

## Độ tương thích

Các test hiện tại kiểm tra engine, CLI và API ở source level. Chưa có bộ golden test được
thu trực tiếp từ ứng dụng QT2025 cho toàn bộ Luật Nhân và trường hợp ghép câu phức tạp, vì
vậy chưa nên hiểu “tương thích QT2025” là cam kết khớp 100% với mọi input.

Ranges dùng đơn vị UTF-16, nhưng được chia theo phrase hoặc Unicode scalar của Rust. Ví dụ
một emoji fallback tạo một range có `length=2`; .NET có thể biểu diễn nó bằng hai entry
surrogate. HanViet ranges là contract hai chiều mở rộng của bản Rust, không phải bản sao
của mapping target-only trong QT2025.

## Giấy phép và dữ liệu tham chiếu

Phần code Rust của Quick Translator Engine được phát hành theo
[GNU General Public License v3.0 only](LICENSE) (`GPL-3.0-only`).

Dữ liệu trong `QT2025/` và source decompile trong `reference/` có nguồn gốc riêng, không
mặc nhiên được bao phủ bởi GPL-3.0 của code. Trước khi phát hành công khai hoặc phân phối
lại các artifact này, cần xác nhận quyền phân phối; xem
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
