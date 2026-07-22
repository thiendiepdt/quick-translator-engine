# HTTP API

`qt-server` nạp VietPhrase, ChinesePhienAmWords và các dictionary mặc định một lần khi
khởi động, sau đó dùng chung một `Engine` cho mọi request. Request có thể thay thế các
dictionary phụ mà không sửa state dùng chung. Request và response dùng JSON UTF-8.

## Khởi động

| Biến môi trường | Mặc định | Ý nghĩa |
|---|---:|---|
| `QT_DATA_DIR` | `data` | Thư mục chứa bộ từ điển |
| `QT_PORT` | `3000` | Cổng HTTP |

```bash
QT_DATA_DIR=QT2025 QT_PORT=3000 cargo run -q -p qt-api --bin qt-server
```

Server bind trên `0.0.0.0`. JSON body được giới hạn 5 MiB. Bản hiện tại không có TLS,
authentication hay rate limit; không nên expose trực tiếp ra Internet.

## Mode và option dùng chung

| Field | Kiểu | Mặc định | Giá trị |
|---|---|---:|---|
| `mode` | string | bắt buộc | `hanviet`, `vietphrase`, `vietphrase-one` |
| `wrap` | boolean | `false` | Bọc từng cụm dịch trong `[...]` |
| `pretty` | boolean | `false` | Bỏ whitespace đầu output và viết hoa ký tự đầu |
| `ranges` | boolean | `false` | Trả thêm `sourceRanges` và `targetRanges` |
| `scanRange` | integer | `30` | Độ dài quét tối đa, từ `1` đến `100` |
| `translationAlgorithm` | integer | `1` | Thuật toán `0`, `1` hoặc `2` |
| `prioritizedName` | boolean | `true` | Ưu tiên Name trước cụm VietPhrase chồng lấn |

Option engine không ảnh hưởng mode `hanviet`, ngoại trừ `wrap` không được áp dụng cho
đường dịch HanViet độc lập của QT.

## `GET /health`

Response `200 OK`:

```json
{"status":"ok"}
```

Endpoint chỉ cho biết process đang phục vụ request; nó không reload hoặc kiểm tra lại dữ
liệu sau khi server đã khởi động.

## `GET /modes`

Response `200 OK`:

```json
{"modes":["hanviet","vietphrase","vietphrase-one"]}
```

## `POST /translate`

Request tối thiểu:

```json
{
  "text": "他的眼球很好。",
  "mode": "vietphrase-one"
}
```

Request với đầy đủ option:

```json
{
  "text": "他的眼球很好。",
  "mode": "vietphrase-one",
  "wrap": false,
  "pretty": true,
  "ranges": true,
  "scanRange": 30,
  "translationAlgorithm": 1,
  "prioritizedName": true,
  "dictionaries": {
    "names": "萧炎=Tiêu Viêm\n云韵=Vân Vận",
    "names2": "",
    "pronouns": "他=hắn\n她=nàng",
    "luatNhan": "在{n}身后=sau lưng {n}",
    "danhTu": "",
    "hoNguoi": "萧=Tiêu",
    "hauTu": "先生=tiên sinh",
    "ignoredChinesePhrases": "本章完"
  }
}
```

### Dictionary theo request

`dictionaries` là optional. Mỗi field chứa nguyên nội dung UTF-8 của file text tương ứng:

| Field | File tương ứng | Tác dụng |
|---|---|---|
| `names` | `Names.txt` | Name chính |
| `names2` | `Names2/123.txt` | Name phụ, ưu tiên hơn `names` |
| `luatNhan` | `LuatNhan.txt` | Thay toàn bộ tập Luật Nhân và compile cho request |
| `pronouns` | `Resources/Pronouns.txt` | Đại từ dùng cho luật `{n}` |
| `danhTu` | `Resources/DanhTu.txt` | Được nhận/parse; đường dịch hiện tại chưa sử dụng |
| `hoNguoi` | `Resources/HoNguoi.txt` | Họ dùng cho luật `{h}{t}` |
| `hauTu` | `Resources/HauTu.txt` | Hậu tố dùng cho luật `{h}{t}` |
| `ignoredChinesePhrases` | `IgnoredChinesePhrases.txt` | Cụm bị bỏ ở bước chuẩn hóa |

Semantics của từng field:

- Không truyền field: dùng dictionary mặc định đã nạp cùng engine.
- Truyền chuỗi rỗng: thay dictionary đó bằng tập rỗng.
- Truyền nội dung: thay toàn bộ file tương ứng trong request hiện tại.
- `names2` vẫn ghi đè `names`; Name vẫn ưu tiên hơn VietPhrase khi trùng key.
- `vietphrase` và `chinesePhienAmWords` không thuộc schema và không thể thay qua request.
- Override chỉ sống trong request, không được lưu và không xuất hiện trong request khác.
- Luật Nhân có regex không hợp lệ trả `400 Bad Request` thay vì làm dừng server.

Response khi `ranges=false`:

```json
{
  "translated": "Ánh mắt của hắn rất tốt."
}
```

Ví dụ tối giản, độc lập với nội dung dictionary, khi `ranges=true`:

```json
{
  "translated": "😀",
  "sourceRanges": [{"start": 0, "length": 2}],
  "targetRanges": [{"start": 0, "length": 2}]
}
```

Request tương ứng dùng `{"text":"😀","mode":"vietphrase-one","ranges":true}`.

## `POST /translate/batch`

```json
{
  "texts": ["他很好。", "她很好。"],
  "mode": "vietphrase-one",
  "pretty": true
}
```

Response giữ nguyên thứ tự input:

```json
{
  "translated": ["Hắn rất tốt.", "Nàng rất tốt."]
}
```

Khi `ranges=true`, `sourceRanges` và `targetRanges` là hai ma trận song song với `texts`.
Nội dung dịch phụ thuộc bộ từ điển đang nạp. Batch được xử lý tuần tự trong một request;
các request độc lập vẫn có thể được runtime phục vụ đồng thời. `dictionaries` có cùng
schema như `/translate` và được áp dụng cho toàn bộ phần tử trong `texts`.

## Range contract

Mỗi range là một đoạn half-open được biểu diễn bằng `{start, length}`. Offset và length
dùng số code unit UTF-16, phù hợp với indexing của JavaScript và .NET.

- `sourceRanges[i]` ánh xạ tới `targetRanges[i]`.
- Range nguồn trỏ về input gốc, kể cả sau khi chuẩn hóa HTML entity hoặc dấu câu.
- Entry được chia theo phrase hoặc Unicode scalar của Rust, không bắt buộc một entry trên
  mỗi code unit UTF-16.
- Emoji fallback là một entry `length=2`.
- Mode HanViet dùng contract source/target hai chiều của Rust; mapping này có chủ đích khác
  mảng target-only của QT2025.
- Khi `pretty=true`, target ranges được điều chỉnh theo output sau khi trim/viết hoa.

## Lỗi

Mode, option engine hoặc Luật Nhân custom không hợp lệ trả `400 Bad Request`:

```json
{"error":"translationAlgorithm must be 0, 1, or 2"}
```

JSON sai schema do extractor của Axum xử lý và có thể trả `4xx` ở dạng text. Panic trong
blocking task trả `500 Internal Server Error` với JSON `{ "error": "..." }`.
