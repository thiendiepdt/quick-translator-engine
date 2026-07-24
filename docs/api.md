# HTTP API

`qt-server` nạp VietPhrase, ChinesePhienAmWords và các dictionary mặc định một lần khi
khởi động, sau đó dùng chung một `Engine` cho mọi request. Request có thể thay thế các
dictionary phụ mà không sửa state dùng chung. Request và response dùng JSON UTF-8.

## Khởi động

| Biến môi trường | Mặc định | Ý nghĩa |
|---|---:|---|
| `QT_DATA_DIR` | `data` | Thư mục chứa bộ từ điển |
| `QT_PORT` | `3000` | Cổng HTTP |
| `QT_NER_MODEL` | không có | Path model token-classification ONNX; cần build feature `onnx` |
| `QT_NER_TOKENIZER` | không có | Path `tokenizer.json` tương ứng model |
| `QT_NER_CONFIG` | không có | Path `config.json` có `id2label` BIO |
| `ORT_DYLIB_PATH` | theo ONNX Runtime | Path dynamic library ONNX Runtime khi bật feature |
| `QT_GEMINI_API_KEY` | không có | API key cho AI fallback; chỉ dùng ở server |
| `QT_GEMINI_MODEL` | không có | Model Gemini dùng cho structured output |
| `QT_GEMINI_BASE_URL` | Google API | Base URL tùy chọn cho test/proxy |

```bash
QT_DATA_DIR=QT2025 QT_PORT=3000 cargo run -q -p qt-api --bin qt-server
```

Server bind trên `0.0.0.0`. JSON body được giới hạn 5 MiB. Bản hiện tại không có TLS,
authentication hay rate limit; không nên expose trực tiếp ra Internet.

### Name-filter API chuyên dụng

`qt-ner-api` dùng cùng request/response `POST /names/filter` nhưng không expose endpoint
dịch hoặc raw dictionary:

```bash
QT_NER_DATA_DIR=QT2025 QT_NER_PORT=3001 \
  cargo run -q -p qt-ner-api --bin qt-ner-api
```

`QT_NER_DATA_DIR` fallback sang `QT_DATA_DIR`; `QT_NER_PORT` fallback sang `QT_PORT` rồi
mặc định `3001`. Router chuyên dụng gồm:

- `GET /health`
- `GET /capabilities`
- `POST /names/filter`

`GET /capabilities` trả trạng thái provider đã khởi tạo:

```json
{
  "nerConfigured": true,
  "aiConfigured": false
}
```

Nếu provider cấu hình lỗi, response có thêm `warnings`; chi tiết lỗi vẫn được ghi vào
stderr/CloudWatch lúc process khởi động.

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

## `GET /dictionaries/defaults`

Trả nguyên nội dung tám file mặc định được phép tùy biến. `vietphrase` và
`chinesePhienAmWords` không xuất hiện vì hai bộ này cố định ở engine.

```json
{
  "names": "萧炎=Tiêu Viêm\n...",
  "names2": "药老=Dược Lão\n...",
  "luatNhan": "在{n}身后=sau lưng {n}\n...",
  "pronouns": "他=hắn\n她=nàng\n...",
  "danhTu": "...",
  "hoNguoi": "...",
  "hauTu": "...",
  "ignoredChinesePhrases": "本章完\n..."
}
```

Response thành công có `Cache-Control: public, max-age=3600`. Web dùng nội dung này làm
bản gốc để sửa; nếu một file không thay đổi thì không cần gửi lại trong request dịch.

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

## `POST /names/filter`

Lọc candidate name cho một chương. Request mặc định chạy deterministic rules; ONNX và AI
fallback chỉ chạy khi vừa được cấu hình ở server vừa có `enabled=true` trong request.

```json
{
  "text": "来人名为萧炎。萧炎看向云韵。",
  "mode": "hybrid",
  "minOccurrences": 2,
  "minConfidence": 0.60,
  "maxCandidates": 200,
  "knownNames": { "云韵": "Vân Vận" },
  "rejectedNames": ["看向"],
  "ner": { "enabled": true, "minConfidence": 0.65 },
  "aiFallback": {
    "enabled": true,
    "minConfidence": 0.65,
    "minRuleConfidence": 0.4,
    "maxRuleConfidence": 0.82,
    "maxCandidates": 25
  },
  "dictionaries": {
    "names2": "云韵=Vân Vận",
    "hoNguoi": "萧=Tiêu"
  }
}
```

| Field | Mặc định | Ý nghĩa |
|---|---:|---|
| `mode` | `hybrid` | `qt` dùng Jieba + giới hạn 2–5 + frequency; `hybrid` thêm rules mới |
| `minOccurrences` | `2` | Số lần xuất hiện tối thiểu; context/họ/hậu tố có thể giữ name hiếm |
| `minConfidence` | `0.55`/`0.60` | Ngưỡng rules từ `0` đến `1` cho QT/hybrid |
| `maxCandidates` | `200` | Giới hạn `1..=1000` |
| `maxNameLength` | `8` | Hybrid n-gram, giới hạn `2..=8` |
| `includeKnown` | `true` | Trả lại name đã duyệt trong memory với score `1.0` |
| `knownNames` | `{}` | Map name đã duyệt → bản dịch, do client lưu theo truyện |
| `rejectedNames` | `[]` | Candidate đã loại, không đề xuất lại ở chương sau |
| `ner` | disabled | Xác nhận/tạo candidate từ token-classification ONNX |
| `aiFallback` | disabled | Chỉ gửi nhóm candidate mơ hồ sang Gemini để duyệt |
| `dictionaries` | mặc định | Cùng schema override như `/translate` |

Rules, ONNX NER và AI fallback dùng chung scan document đã áp dụng
`ignoredChinesePhrases`. Phrase bị ignore không sinh occurrence/candidate; range của phần
còn lại vẫn dùng offset UTF-16 trên raw `text`. Nếu request gửi
`dictionaries.ignoredChinesePhrases`, nội dung đó thay toàn bộ default của engine.

Response:

```json
{
  "candidates": [{
    "text": "萧炎",
    "suggested": "Tiêu Viêm",
    "entityType": "person",
    "score": 0.99,
    "occurrences": 2,
    "ranges": [{ "start": 4, "length": 2 }],
    "contexts": ["来人名为【萧炎】。萧炎看向云韵。"],
    "reasons": ["xuất hiện sau ngữ cảnh giới thiệu tên"],
    "sources": ["qt-jieba", "context-rule", "surname-rule"],
    "known": false
  }],
  "stats": {
    "scannedCharacters": 14,
    "ruleCandidates": 1,
    "nerCandidates": 0,
    "aiReviewed": 0
  },
  "capabilities": { "nerConfigured": false, "aiConfigured": false },
  "warnings": ["NER was requested but no ONNX model is configured"]
}
```

`ranges` dùng UTF-16 như endpoint dịch. Lỗi provider tùy chọn không làm hỏng kết quả
rules: response vẫn là `200`, kèm `warnings`. API key Gemini không bao giờ nhận từ client.
Server chỉ gửi tối đa 50 candidate cùng context ngắn và xác thực output AI phải thuộc tập
candidate đầu vào.

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
