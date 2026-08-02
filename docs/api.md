# HTTP API

`qt-server` nạp VietPhrase, ChinesePhienAmWords và các dictionary mặc định một lần khi
khởi động, sau đó dùng chung một `Engine` cho mọi request. Request có thể thay thế các
dictionary phụ hoặc gửi patch nhỏ cho hai dictionary cố định mà không sửa state dùng
chung. Request và response dùng JSON UTF-8.

## Khởi động

| Biến môi trường | Mặc định | Ý nghĩa |
|---|---:|---|
| `QT_DATA_DIR` | `data` | Thư mục chứa bộ từ điển |
| `QT_PORT` | `3000` | Cổng HTTP |

Server không gọi AI và không nhận API key AI nào: client tự gọi DeepSeek/Gemini (hoặc
proxy của mình) rồi gửi kết quả dưới dạng dữ liệu trơ trong `aiEntities` (xem
`POST /names/filter`). Server không bao giờ fetch URL do request cung cấp.

```bash
QT_DATA_DIR=QT2025 QT_PORT=3000 cargo run -q -p qt-api --bin qt-server
```

Server bind trên `0.0.0.0`. JSON body được giới hạn 5 MiB. Bản hiện tại không có TLS,
authentication hay rate limit; không nên expose trực tiếp ra Internet.

## Mode và option dùng chung

| Field | Kiểu | Mặc định | Giá trị |
|---|---|---:|---|
| `mode` | string | `vietphrase-one` | `hanviet`, `vietphrase`, `vietphrase-one` |
| `wrap` | boolean | `false` | Bọc từng cụm dịch trong `[...]` |
| `pretty` | boolean | `true` | Bỏ whitespace đầu output và viết hoa ký tự đầu; gửi `false` để giữ output QT nguyên bản |
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

Request tối thiểu chỉ cần `text` — `mode` mặc định `vietphrase-one` và response chỉ có
`translated`; các field khác (ranges, option engine, dictionary) là opt-in khi client
cần bản đầy đủ:

```json
{
  "text": "他的眼球很好。"
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
  },
  "dictionaryPatches": {
    "vietPhrase": {
      "很好": "rất ổn/rất tốt"
    },
    "chinesePhienAmWords": {
      "他": "hắn"
    }
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
- `vietPhrase` và `chinesePhienAmWords` không thuộc object `dictionaries`, nên không thể
  thay toàn bộ file qua request.
- Override chỉ sống trong request, không được lưu và không xuất hiện trong request khác.
- Luật Nhân có regex không hợp lệ trả `400 Bad Request` thay vì làm dừng server.

### Patch VietPhrase và Phiên Âm

`dictionaryPatches` là optional và chỉ chứa các entry user sửa. Base
`VietPhrase.txt`/`ChinesePhienAmWords.txt` vẫn được nạp cố định trong engine:

- `vietPhrase` là object `cụm tiếng Trung -> nghĩa tiếng Việt`.
- `chinesePhienAmWords` là object `một ký tự Hán -> âm đọc`. Key rỗng hoặc nhiều hơn một
  Unicode character trả `400 Bad Request`.
- Patch thắng entry cùng key trong dictionary cố định, nhưng Name vẫn thắng VietPhrase
  theo priority gốc của QT2025.
- Patch chỉ áp dụng cho request hiện tại. API không lưu patch, không ghi file và không
  mutate `Engine`.
- Client nên chỉ gửi các entry đã sửa; không gửi lại toàn bộ file VietPhrase.

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

`mode` cũng mặc định `vietphrase-one` như `/translate`. Khi `ranges=true`,
`sourceRanges` và `targetRanges` là hai ma trận song song với `texts`.
Nội dung dịch phụ thuộc bộ từ điển đang nạp. Batch được xử lý tuần tự trong một request;
các request độc lập vẫn có thể được runtime phục vụ đồng thời. `dictionaries` và
`dictionaryPatches` có cùng schema như `/translate` và được áp dụng cho toàn bộ phần tử
trong `texts`.

## `POST /names/filter`

Lọc candidate name cho một chương bằng deterministic rules. Server **không gọi AI**:
client nào muốn AI (trích entity, duyệt ứng viên) thì tự gọi DeepSeek/Gemini/proxy bằng
key của mình — qt-web làm việc này ngay trong trình duyệt — rồi gửi entity đã trích cho
server dưới dạng dữ liệu trơ trong `aiEntities` để merge với ứng viên từ rules. Không có
field credentials nào; server không bao giờ thấy API key và không fetch URL nào từ
request.

```json
{
  "text": "来人名为萧炎。萧炎看向云韵。",
  "mode": "hybrid",
  "minOccurrences": 2,
  "minConfidence": 0.60,
  "maxCandidates": 200,
  "knownNames": { "云韵": "Vân Vận" },
  "rejectedNames": ["看向"],
  "aiEntities": {
    "minConfidence": 0.65,
    "entities": [
      { "text": "萧炎", "entityType": "person", "suggested": "Tiêu Viêm", "confidence": 0.9 }
    ]
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
| `includeKnown` | `true` | Trả lại name đã duyệt trong memory với score `1.0`; cũng áp cho merge `aiEntities` |
| `knownNames` | `{}` | Map name đã duyệt → bản dịch, do client lưu theo truyện |
| `rejectedNames` | `[]` | Candidate đã loại, không đề xuất lại ở chương sau |
| `aiEntities` | không có | Entity do client trích bằng AI của mình: `entities[]` (tối đa 500; mỗi entity `text` ≤ 100 ký tự, `suggested` ≤ 200 ký tự, `confidence` 0–1, mặc định 0.75) + `minConfidence` ngưỡng merge (mặc định 0.65) |
| `ner` | deprecated | Bị bỏ qua; bật chỉ sinh warning (ONNX NER đã gỡ) |
| `dictionaries` | mặc định | Cùng schema override như `/translate` |

Payload `aiEntities` sai (quá 500 entity, text rỗng/quá dài, confidence ngoài 0–1, field
lạ) trả `400`/`422` trước khi rules chạy. Server chỉ merge entity xuất hiện nguyên văn
trong chương và tự định vị occurrence; entity trùng dictionary, bị reject, hay dưới
ngưỡng merge bị bỏ qua lặng lẽ. Rules và merge dùng chung scan document đã áp dụng
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
    "aiMergedCandidates": 0
  }
}
```

`ranges` dùng UTF-16 như endpoint dịch. `stats.aiMergedCandidates` đếm số entity từ
`aiEntities` được merge (thêm mới hoặc xác nhận ứng viên rules có sẵn — các ứng viên đó
mang source `ai-fallback`). `warnings` chỉ xuất hiện khi có cảnh báo (hiện tại: `ner`
deprecated).

Client tham khảo cách gọi AI phía mình trong `apps/qt-web/src/lib/ai-client.ts`
(chia chương thành chunk ≤ 15k ký tự, tối đa 4 request song song, prompt/schema cho cả
DeepSeek lẫn Gemini, và bước AI duyệt ứng viên mơ hồ chạy hoàn toàn phía client trên
response của endpoint này).

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
