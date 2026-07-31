# Lọc name

Module `qt-core::name_filter` tách việc **đề xuất candidate** khỏi thao tác ghi Names2.
Core không gọi network và không giữ state theo người dùng; caller truyền book memory vào
mỗi lần lọc. API ghép thêm AI extract và AI fallback (DeepSeek/Gemini).

## Chuẩn hóa input

Trước khi chạy rules hoặc AI, engine match
`IgnoredChinesePhrases` bằng cùng logic chuẩn hóa dùng cho dịch, rồi mask các span tương
ứng trên raw text bằng separator. Hai phía không bị nối lại nên không sinh candidate giả
bắc cầu qua phần bị ignore. Override `ignoredChinesePhrases` trong request thay toàn bộ
default, kể cả chuỗi rỗng.

Scan document giữ mapping về UTF-16 của raw input. Vì vậy `ranges` và context trả về vẫn
trỏ đúng nguyên văn, kể cả khi phía trước candidate có emoji, HTML entity, chữ phồn thể
hoặc phrase đã bị ignore. Entity AI đi qua separator bị loại trước khi merge.

## Mode `qt`

Mode này là port trung thực của `LocNameOff.LocNameQT` trong QuickTranslator.exe
(QT2025), đối chiếu từ bản decompile của GUI:

1. Jieba cắt từ với HMM; giữ token Hán dài 2–5 ký tự có số lần xuất hiện ≥
   `minOccurrences` (QT2025 dùng threshold theo độ dài văn bản: `<50k` chữ → 1,
   `<100k` → 2, `<200k` → 3, còn lại 4 — với một chương lẻ hãy truyền `1`).
2. Loại token chứa stopword cứng của QT2025 (`的`, `了`, `在`, `什么`, …) hoặc chữ số.
3. Trong mỗi nhóm cùng hai ký tự đầu chỉ giữ candidate ngắn nhất
   (`FilterUnnecessaryPhrasesOptimized`).
4. Lọc chuỗi nối Names2 (`FilterUnnecessaryItems`, chạy hai lần khi Names2 khác rỗng).
5. `ValidateAndMergeTerms`: term 2 chữ không thuộc VietPhrase phải bắt đầu bằng
   `HoNguoi` hoặc merge được với segment `DanhTu` liền sau (`冷涯`+`郡`→`冷涯郡`);
   term 4 chữ phải là `<prefix><DanhTu>` và không phải hai từ VietPhrase ghép lại.
6. Loại candidate đã có trong VietPhrase/Names/Names2 (vô điều kiện), candidate
   giao thoa với Names2 (`IsPartOfAnyPhraseInDictionary`), chuỗi số Hán và
   candidate mà value gợi ý chỉ có một âm tiết.
7. Value gợi ý theo `BuildFormattedHanViet`: template `DanhTu` dạng `{0}`
   (`郡={0} quận` → `Lãnh Nhai quận`), VietPhrase một nghĩa, hoặc âm Hán Việt
   title-case.
8. Tách thêm tựa sách `《…》` và sắp toàn bộ kết quả theo vị trí xuất hiện đầu tiên.
   Key tựa sách được xuất ở dạng đã chuẩn hóa dịch (`《 X 》` có dấu cách) — đây là
   dạng duy nhất match được khi dịch, vì `StandardizeInput` chèn dấu cách quanh
   `《》`; QT2025 cũng vậy do panel Trung đã được chuẩn hóa trước khi lọc. Thêm
   entry Names/Names2 cho tựa sách mà viết `《X》=…` (không dấu cách) sẽ không bao
   giờ khớp.

Trên corpus thật (8 chương `phuong-thon-dao-chu`), output mode này trùng byte-for-byte
với replica Python của thuật toán QT2025 chạy cùng bộ từ điển. Khác biệt duy nhất còn
lại là phiên bản resource Jieba (jieba-rs dùng dict mặc định tương thích) và book
memory của engine (rejected suppress, known giữ value đã duyệt) — QT2025 không có
khái niệm này. `minConfidence` không có tác dụng trong mode `qt` vì QT2025 không chấm
điểm.

Lưu ý phạm vi: trong QT2025 thật, nút "Lọc Name" còn gọi API metruyencv (MTC) và nút
"Lọc Gemini" gửi cả chương cho Gemini 1.5 Flash; phần local `LocNameQT` chỉ là một
nhánh. Hai nhánh online đó tương ứng với AI extract + AI fallback của engine này.

## Mode `hybrid`

Hybrid giữ seed Jieba và thêm overlapping n-gram dài 2–8 trong từng run ký tự Hán.
Trước khi chấm điểm, các hard filter (rút từ hành vi QT2025 + tín hiệu lexicon) loại
thẳng những candidate không thể là tên:

- chứa stopword QT2025 (`的`, `了`, `在`, …);
- chuỗi số Hán hoặc chứa ≥2 chữ số Hán liên tiếp (`新历一百`);
- exact VietPhrase thông thường, trừ khi có trigger giới thiệu tên hoặc bắt đầu bằng
  `HoNguoi` kèm bằng chứng mạnh (không nằm trong lexicon Jieba, hoặc xuất hiện ≥5
  lần — che case nhân vật chính đã có sẵn trong VietPhrase như `李顺`);
- n-gram thuần (không phải token Jieba) không có trigger/hậu tố phải có họ kép
  (`公叔`), hoặc họ + phần còn lại là danh từ riêng đã biết (`姜`+`太阿=Thái A`),
  hoặc xuất hiện ≥3 lần;
- value gợi ý chỉ có một âm tiết.

Mỗi candidate còn lại được chấm từ các tín hiệu:

- tần suất trong chương;
- Jieba coi là một token; token nằm ngoài lexicon Jieba (OOV) được cộng điểm,
  token thuộc lexicon bị trừ — OOV là tín hiệu danh từ riêng mạnh;
- đứng sau trigger như `名为`, `叫做`, `姓`, `自称`;
- bắt đầu bằng `HoNguoi`; họ + danh từ riêng đã biết được cộng thêm;
- hậu tố người/địa danh/tổ chức hoặc entry `DanhTu`/`HauTu`;
- accepted book memory có score `1.0`; rejected memory bị loại ngay.

Tựa sách `《…》` luôn được thêm với score `0.90`. Value gợi ý dùng template `DanhTu`
khi khớp hậu tố (`冷山县` → `huyện Lãnh Sơn`), nếu không thì như trước.

Sau scoring, candidate dài bị bỏ nếu chỉ là phần mở rộng yếu hơn rõ rệt của một candidate
ngắn. Default hybrid confidence là `0.60`. Trên 8 chương `phuong-thon-dao-chu` với
glossary đã duyệt làm gold: recall 53% với ~46 candidate/chương (trước cải tiến:
49% với ~167/chương; thuật toán local QT2025 nguyên bản: 14%). Phần entity còn thiếu
(kỹ năng/pháp bảo xuất hiện một lần, biệt danh như `老冯`) là việc của tầng AI extract.

Vị trí occurrence được thu ngay lúc tạo token/n-gram, rồi đổi sang UTF-16 qua một prefix
map của chương. Vì vậy pipeline không scan lại toàn chương cho từng candidate.

## Book memory

`NameFilterMemory` gồm:

- `known_names`: map chữ Hán → value đã duyệt;
- `rejected_names`: set candidate đã loại.

API stateless: client phải gửi memory đúng truyện trong request. `qt-web` persist nhiều
profile theo mã truyện, tự đưa accepted entry vào draft Names2 và gỡ entry đó khi reject
hoặc xóa profile. CLI nhận `--known-names-file`/`--rejected-names-file`.

## AI provider (DeepSeek / Gemini)

ONNX NER đã bị gỡ. Vai trò phát hiện entity ngoài rules giờ do một AI provider đảm nhận,
cấu hình qua biến môi trường; DeepSeek được ưu tiên khi cả hai cùng được cấu hình:

| Biến | Mặc định | Ý nghĩa |
| --- | --- | --- |
| `QT_DEEPSEEK_API_KEY` | không có | Bật provider DeepSeek |
| `QT_DEEPSEEK_MODEL` | `deepseek-chat` | Model chat completions |
| `QT_DEEPSEEK_BASE_URL` | `https://api.deepseek.com` | Endpoint OpenAI-compatible |
| `QT_GEMINI_API_KEY` | không có | Bật provider Gemini (giữ tương thích) |
| `QT_GEMINI_MODEL` | bắt buộc khi có key | Model Gemini |
| `QT_GEMINI_BASE_URL` | `https://generativelanguage.googleapis.com` | Endpoint Gemini |

`GET /capabilities` trả `aiConfigured` và `aiProvider` (`deepseek`/`gemini`).

Provider phục vụ hai vai trò độc lập trong `POST /names/filter`:

### `aiExtract` — trích xuất toàn chương

Gửi cả chương (chia chunk ≤15k ký tự theo dòng, giống flow Gemini của QT2025) và yêu cầu
trích mọi entity danh từ riêng: nhân vật kể cả biệt danh, địa danh, tổ chức, công pháp/
pháp bảo, tên sách. Đây là thay thế cho ONNX NER và là nguồn duy nhất bắt được:

- tên phiên âm phương Tây (`艾德里安`, `贝尔纳` — rules bó tay vì stopword `尔`, không họ
  Trung, dấu `·` cắt run ký tự Hán) với suggested là dạng Latin gốc (`Adrian`,
  `Dolores Jane Umbridge`);
- biệt danh/gọi tắt (`老冯`) và entity chỉ xuất hiện một lần.

Server chỉ nhận entity có mặt nguyên văn trong scan text (định vị occurrence và map range
về input gốc), loại entity đã có trong Names/Names2 hoặc rejected memory, rồi merge với
candidate rules — trùng thì cộng dồn confidence, mới thì thêm với source `ai-fallback`.

### `aiFallback` — duyệt candidate mơ hồ

AI không tự sinh danh sách ở vai trò này. API chỉ gửi tối đa 50 candidate có score trong
vùng mơ hồ, kèm một context ngắn. Structured output chỉ được phép quyết định keep/drop,
entity type, confidence và sửa suggested value. Server kiểm tra mọi decision phải thuộc
candidate đầu vào; lỗi/timeout trả warning và giữ kết quả rules.

## Benchmark

Gold file dùng format `name=value`. Lệnh sau in thời gian trung bình cùng
precision/recall/F1:

```bash
cargo run --release -p qt-core --example name_filter_bench -- \
  QT2025 chapter.txt gold-names.txt 20 hybrid
```

Nên benchmark tách theo thể loại và theo truyện. Name trong web novel có độ lệch domain
lớn; một threshold tốt cho tiên hiệp chưa chắc phù hợp lịch sử hoặc đô thị.
