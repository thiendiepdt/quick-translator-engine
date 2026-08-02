# Kiến trúc hệ thống

Tài liệu này mô tả trạng thái hiện tại của Quick Translator Engine. Đặc tả chi tiết thuật
toán gốc nằm trong [engine/](engine/README.md); HTTP contract nằm trong [api.md](api.md).

## Mục tiêu thiết kế

- Tái hiện hành vi dịch của Quick Translator/QT2025 bằng Rust.
- Giữ engine độc lập với transport để dùng chung cho CLI, server, Lambda, web và desktop.
- Nạp VietPhrase/Hán Việt một lần; áp dụng dictionary phụ theo request mà không mutate
  state dùng chung.
- Giữ source ↔ target ranges theo UTF-16 cho JavaScript/.NET consumers.
- Tách rõ output “faithful” của engine và chuẩn hóa trình bày `pretty` của API.

## Cargo workspace

```text
quick-translator-engine/
├── apps/
│   ├── qt-web/    # React web client
│   └── qt-gui/    # React + Tauri desktop client
├── crates/
│   ├── qt-core/   # thư viện dịch đồng bộ
│   ├── qt-cli/    # binary qt, stdin -> stdout
│   ├── qt-api/    # binary qt-server, Axum HTTP API
│   ├── qt-lambda/ # native AWS Lambda entrypoint đầy đủ
├── deploy/
│   ├── aws-lambda/       # SAM template, event và hướng dẫn deploy
│   └── cloudflare-worker/ # edge gateway ký SigV4
├── docs/
│   ├── api.md
│   ├── architecture.md
│   ├── dev/
│   └── engine/
├── QT2025/        # dữ liệu/config tham chiếu
└── reference/     # source decompile để đối chiếu
```

### `qt-core`

`qt-core` không phụ thuộc HTTP hay CLI. Public API chính:

```rust
let dictionaries = Dictionaries::load(data_dir)?;
let engine = Engine::from_dicts(dictionaries);

let text = engine.translate(input, mode, &options);
let mapped = engine.translate_with_ranges(input, mode, &options);

let overrides = DictionaryOverrides::from_sources(DictionarySourceOverrides {
    names: Some("萧炎=Tiêu Viêm"),
    ..Default::default()
});
let custom = engine.translate_with_overrides(input, mode, &options, &overrides)?;

let names = engine.filter_names(
    input,
    &NameFilterOptions::default(),
    &NameFilterMemory::default(),
    Some(&overrides),
);
```

`Engine` giữ ba nhóm state chỉ đọc:

- các dictionary đã parse/merge;
- cache Luật Nhân và regex;
- bảng chuẩn hóa input, HTML entities và ignored phrases.

Bộ lọc name deterministic nằm trong core. `qt-api` mới orchestration AI provider bên
ngoài (DeepSeek hoặc Gemini): AI extract đọc cả chương để trích entity, còn AI fallback
chỉ nhận candidate mơ hồ. Cả hai chạy async.

`DictionaryOverrides` là state đã parse chỉ sống trong caller/request. Lookup custom dùng
view ưu tiên trên dictionary cố định, không clone VietPhrase và không sửa `Engine`.

### `qt-cli`

CLI parse argument, nạp dictionary, đọc các file override nếu được chỉ định, đọc toàn bộ
UTF-8 từ `stdin`, gọi engine và ghi nguyên output vào `stdout`. Lỗi cấu hình/argument được
ghi vào `stderr` và trả exit code khác 0. CLI không chứa thuật toán dịch.

`qt names filter` gọi cùng core rules, nhận file accepted/rejected làm book memory và có
thể xuất Names2-compatible hoặc JSON. AI provider thuộc HTTP server để CLI mặc định
không phát sinh network call.

### `qt-gui`

Desktop app giữ một `Engine` chỉ đọc trong Tauri managed state. Lúc khởi động, app ưu
tiên `QT_DATA_DIR`, sau đó tìm `QT2025`/`data` từ working directory và executable; người
dùng cũng có thể chọn data directory trong UI. Translation và deterministic name filter
chạy ở blocking task qua commands, không bind socket và không khởi động `qt-api`.

Frontend dùng lại stack và interaction model của `qt-web`, nhưng transport là Tauri IPC.
Compact quick-update entries và book memory được persist trong WebView local storage;
raw dictionary draft chỉ sống trong session. Override được parse riêng cho từng lần dịch
nên không mutate engine hoặc ghi đè dữ liệu QT2025. File open/save dùng native dialog,
còn đọc/ghi UTF-8 đi qua commands có giới hạn 16 MiB.

### `qt-api`

Server nạp một `Engine` và giữ raw customizable defaults lúc khởi động. Nội dung
dictionary custom được parse thành state riêng cho request. Mỗi lần dịch chạy trong
`tokio::task::spawn_blocking` vì engine là code đồng bộ, CPU-bound. Router cung cấp:

- `GET /health`
- `GET /modes`
- `GET /dictionaries/defaults`
- `POST /translate`
- `POST /translate/batch`
- `POST /names/filter`

Batch giữ thứ tự và xử lý lần lượt từng item để không làm đầy blocking pool trong một
request. Xem schema tại [api.md](api.md).

`/names/filter` thuần rules — server không gọi AI và không nhận API key nào. Tính năng
AI của lọc tên chạy ở client: qt-web gọi thẳng DeepSeek/Gemini (hoặc proxy người dùng
cấu hình) từ trình duyệt rồi gửi entity đã trích trong `aiEntities` như dữ liệu trơ.
Server không bao giờ fetch URL từ request, nên không có bề mặt SSRF lẫn credit phía
server để bị lạm dụng.

### `qt-lambda`

Lambda binary dùng `lambda_http` để chạy lại nguyên Axum router của `qt-api`. Toàn bộ
dictionary mặc định QT2025 được nhúng vào executable bằng `include_str!`, vì vậy execution
environment không cần filesystem dữ liệu hay bước download lúc cold start. Tám raw
dictionary được phép tùy biến được giữ để web tải về; bản custom vẫn được parse riêng theo
từng request và không làm thay đổi state dùng chung của warm instance.

SAM template deploy runtime `provided.al2023` trên ARM64, tạo Function URL mặc định dùng
`AWS_IAM` và giới hạn concurrency. Xem [hướng dẫn deploy](../deploy/aws-lambda/README.md).

Cloudflare Worker trong `deploy/cloudflare-worker` là public gateway tùy chọn. Worker
giới hạn route/body ở edge, ký request bằng SigV4 với IAM principal least-privilege rồi
proxy tới Function URL. Lambda vẫn dùng `AWS_IAM`, nên request gọi thẳng origin mà không
có chữ ký hợp lệ bị AWS từ chối trước khi engine chạy.

## Pipeline dịch

```text
UTF-8 input
    |
    v
StandardizeInput
  - phồn thể -> giản thể
  - HTML decode
  - chuẩn hóa dấu câu/full-width/khoảng trắng
  - bỏ ignored phrases
  - giữ mapping về UTF-16 input gốc
    |
    +---------------------- mode=hanviet ----------------------+
    |                                                          |
    v                                                          v
PreScanForNumbers                                      Hán Việt từng scalar
    |                                                          |
NumberModifier                                                |
    |                                                          |
TranslateAll                                                   |
  1. longest dictionary phrase                                |
  2. Luật Nhân                                                |
  3. chuyển số                                                |
  4. fallback Hán Việt                                        |
    |                                                          |
    +-----------------------------+----------------------------+
                                  v
             TranslationResult { text, source_ranges, target_ranges }
```

`VietPhrase` và `VietPhraseOneMeaning` dùng chung `TranslateAll`, chỉ khác dictionary.
`HanViet` đi qua đường dịch từng Unicode scalar riêng.

## Module `qt-core`

| Module | Trách nhiệm | Đối chiếu QT2025 |
|---|---|---|
| `dict` | Parse file, merge Names/VietPhrase, dựng one-meaning | `LoadDictionaries`, `load*Dictionary` |
| `standardize` | Chuẩn hóa input và giữ source mapping | `StandardizeInput` |
| `translate` | Main loop, longest phrase, name priority | `TranslateAll`, `ProcessTranslation` |
| `han_viet` | Phiên âm từng ký tự và fallback | `ChineseToHanViet`, `ToNarrow` |
| `luat_nhan` | `{n}`, `{s}`, `{h}{t}` và regex cache | `TransLuatNhan`, `HandleNhanBy` |
| `number` | Nhận diện/chuyển số và dải số | `PreScanForNumbers`, `NumberModifier` |
| `name_filter` | QT-compatible extraction, hybrid rules và book memory | `LocNameQT`, contract Rust mở rộng |
| `text` | Nối từ, spacing, capitalization, wrapping | `appendTranslatedWord`, `WrapTranslation` |

## Dictionary lifecycle

`Dictionaries::load` đọc các tên file chuẩn dưới một data directory. Chỉ VietPhrase và
ChinesePhienAmWords là bắt buộc; các file còn lại được load làm mặc định nếu tồn tại.
Engine không parse đường dẫn tùy biến từ `Dictionaries.config`.

CLI có thể thay mỗi file phụ bằng option `--*-file`; API nhận nguyên nội dung file trong
object `dictionaries`. `None` giữ bản mặc định, nội dung rỗng thay bằng dictionary rỗng.
Override không mutate state chung và không thể thay VietPhrase hay ChinesePhienAmWords.

Merge order quan trọng:

1. `Names.txt` được nạp first-wins.
2. Dòng đầu tiên trong `Names2/123.txt` ghi đè `Names.txt` khi trùng key.
3. Names được merge trước VietPhrase, nên Name thắng VietPhrase khi cùng key.
4. OneMeaning lấy phần đầu trước `/` hoặc `|`.

Chi tiết nằm trong [engine/dictionaries.md](engine/dictionaries.md).

## Options và mode

| Option | Default | Ghi chú |
|---|---:|---|
| `wrap_type` / `wrap` | `0` / `false` | `1` bọc phrase trong `[...]` |
| `translation_algorithm` | `1` | Chỉ nhận `0`, `1`, `2` ở CLI/API |
| `prioritized_name` | `true` | Bảo vệ Name bị phrase khác che |
| `scan_range` | `30` | CLI/API giới hạn `1..=100` |

Các option longest-match/name không được dùng trong mode HanViet. Default lấy từ
`QuickTranslatorMain.config` của bộ tham chiếu.

## Range model

`CharRange { start, length }` dùng UTF-16 code unit. Sau chuẩn hóa, mỗi scalar giữ range
trỏ về phần input gốc đã sinh ra nó; một source range có thể được dùng cho nhiều ký tự đầu
ra khi phép chuẩn hóa nở chuỗi, ví dụ `… -> ...`.

Các cặp `source_ranges[i]` và `target_ranges[i]` biểu diễn một phrase/fallback tương ứng.
Entry không bắt buộc tương ứng từng code unit: emoji là một Unicode scalar nhưng dài hai
UTF-16 unit. Contract HanViet hai chiều là phần mở rộng hữu dụng của Rust và cố ý không sao
chép mảng mapping target-only có trường hợp lệch entry cuối của QT2025.

## Độ tương thích và giới hạn

- Source-level tests bao phủ dictionary priority, HanViet, longest-match, số, Luật Nhân,
  chuẩn hóa, ranges, CLI và HTTP API.
- Chưa có golden corpus đầy đủ lấy trực tiếp từ ứng dụng QT2025; các trường hợp hiếm vẫn có
  thể khác engine gốc.
- `POST /meanings`/Lạc Việt chưa được triển khai.
- Input/output của CLI và API là UTF-8; charset detector cũ không được port.
- Server chưa có cơ chế bảo vệ để expose trực tiếp ra Internet.

## Kiểm chứng thay đổi

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Khi thay đổi thuật toán, nên thêm một regression test nhỏ nhất có thể và ghi rõ điểm nào
được đối chiếu từ source decompile, điểm nào là contract mở rộng của Rust.
