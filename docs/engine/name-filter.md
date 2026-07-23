# Lọc name

Module `qt-core::name_filter` tách việc **đề xuất candidate** khỏi thao tác ghi Names2.
Core không gọi network và không giữ state theo người dùng; caller truyền book memory vào
mỗi lần lọc. API mới ghép thêm ONNX NER và AI fallback.

## Mode `qt`

Mode này tái hiện shape của `LocNameQT` trong QT2025:

1. Jieba cắt từ với HMM.
2. Chỉ giữ token Hán dài 2–5 ký tự.
3. Đếm số lần xuất hiện và áp dụng `minOccurrences`.
4. Loại entry đã có trong Names/Names2, VietPhrase thông thường và danh sách reject.
5. Với các candidate có cùng hai ký tự đầu, ưu tiên candidate ngắn hơn.
6. Gợi ý value từ VietPhrase một nghĩa hoặc âm Hán Việt title-case.

Đây là compatibility mode để so sánh/benchmark, không phải cam kết byte-for-byte với UI
Windows. `jieba-rs` dùng dictionary mặc định tương thích Jieba; QT2025 có thể chứa resource
Jieba khác phiên bản.

## Mode `hybrid`

Hybrid giữ seed Jieba và thêm overlapping n-gram dài 2–8 trong từng run ký tự Hán. Mỗi
candidate được chấm từ các tín hiệu:

- tần suất trong chương;
- Jieba coi là một token;
- đứng sau trigger như `名为`, `叫做`, `姓`, `自称`;
- bắt đầu bằng `HoNguoi`;
- hậu tố người/địa danh/tổ chức hoặc entry `DanhTu`/`HauTu`;
- exact VietPhrase thông thường là tín hiệu âm;
- accepted book memory có score `1.0`; rejected memory bị loại ngay.

Sau scoring, candidate dài bị bỏ nếu chỉ là phần mở rộng yếu hơn rõ rệt của một candidate
ngắn. Default hybrid confidence là `0.60`; QT dùng `0.55`. Đây là điểm vận hành cần tune
trên corpus truyện thật, không phải xác suất đã calibration tuyệt đối.

Vị trí occurrence được thu ngay lúc tạo token/n-gram, rồi đổi sang UTF-16 qua một prefix
map của chương. Vì vậy pipeline không scan lại toàn chương cho từng candidate.

## Book memory

`NameFilterMemory` gồm:

- `known_names`: map chữ Hán → value đã duyệt;
- `rejected_names`: set candidate đã loại.

API stateless: client phải gửi memory đúng truyện trong request. `qt-web` persist nhiều
profile theo mã truyện, tự đưa accepted entry vào draft Names2 và gỡ entry đó khi reject
hoặc xóa profile. CLI nhận `--known-names-file`/`--rejected-names-file`.

## ONNX NER

Feature `qt-api/onnx` nhận model Hugging Face token-classification đã export ONNX:

- tokenizer từ `tokenizer.json`;
- labels BIO từ `config.json::id2label`;
- input `input_ids`, `attention_mask`, optional `token_type_ids`;
- output logits `[batch, sequence, labels]`;
- entity hỗ trợ: PER/name, LOC/place, ORG.

Chương được cắt block 400 ký tự, overlap 32 để giảm mất entity ở biên. Span trùng được
deduplicate; confidence NER được kết hợp với score rules. Session nằm sau mutex vì API có
thể phục vụ nhiều request trên cùng warm Lambda instance.

## AI fallback

AI không đọc toàn bộ chương để tự sinh danh sách. API chỉ gửi tối đa 50 candidate có score
trong vùng mơ hồ, kèm một context ngắn. Structured output chỉ được phép quyết định
keep/drop, entity type, confidence và sửa suggested value. Server kiểm tra mọi decision
phải thuộc candidate đầu vào; lỗi/timeout trả warning và giữ kết quả rules/NER.

## Benchmark

Gold file dùng format `name=value`. Lệnh sau in thời gian trung bình cùng
precision/recall/F1:

```bash
cargo run --release -p qt-core --example name_filter_bench -- \
  QT2025 chapter.txt gold-names.txt 20 hybrid
```

Nên benchmark tách theo thể loại và theo truyện. Name trong web novel có độ lệch domain
lớn; một threshold tốt cho tiên hiệp chưa chắc phù hợp lịch sử hoặc đô thị.
