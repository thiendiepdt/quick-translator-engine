# Lọc name

Module `qt-core::name_filter` tách việc **đề xuất candidate** khỏi thao tác ghi Names2.
Core không gọi network và không giữ state theo người dùng; caller truyền book memory vào
mỗi lần lọc. API mới ghép thêm ONNX NER và AI fallback.

## Chuẩn hóa input

Trước khi chạy rules, ONNX NER hoặc AI fallback, engine match
`IgnoredChinesePhrases` bằng cùng logic chuẩn hóa dùng cho dịch, rồi mask các span tương
ứng trên raw text bằng separator. Hai phía không bị nối lại nên không sinh candidate giả
bắc cầu qua phần bị ignore. Override `ignoredChinesePhrases` trong request thay toàn bộ
default, kể cả chuỗi rỗng.

Scan document giữ mapping về UTF-16 của raw input. Vì vậy `ranges` và context trả về vẫn
trỏ đúng nguyên văn, kể cả khi phía trước candidate có emoji, HTML entity, chữ phồn thể
hoặc phrase đã bị ignore. Span NER đi qua separator bị loại trước khi merge.

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

### Runtime chuyên dụng

Ba package dùng chung name-only router và contract của `POST /names/filter`:

- `qt-ner-cli`: đọc chương từ file/stdin, output Names2 hoặc JSON; ONNX bật mặc định;
- `qt-ner-api`: server local/container trên port mặc định `3001`;
- `qt-ner-lambda`: Lambda ARM64 với defaults QT2025 nhúng trong binary.

Không cấu hình `QT_NER_MODEL` thì provider ONNX không được khởi tạo nhưng rules/hybrid vẫn
hoạt động. `GET /capabilities` của API/Lambda cho biết provider thực tế có sẵn.

### Chọn model tương thích

Model phải dùng BIO labels mà adapter hiểu:

- người: `B-PER`/`I-PER`, `B-PERSON`/`I-PERSON` hoặc `B-NAME`/`I-NAME`;
- địa điểm: `B-LOC`/`I-LOC` hoặc `B-PLACE`/`I-PLACE`;
- tổ chức: `B-ORG`/`I-ORG`;
- token ngoài entity: `O`.

Checkpoint khởi đầu phù hợp là
[`shibing624/bert4ner-base-chinese`](https://huggingface.co/shibing624/bert4ner-base-chinese).
Model này là BERT token-classification tiếng Trung, dùng BIO labels PER/LOC/ORG và có
license Apache-2.0. Đây là baseline NER tổng quát, chưa được fine-tune riêng cho tên hư
cấu trong web novel; phải benchmark trên corpus truyện thật trước khi chọn threshold.

Không dùng trực tiếp model có BIOES/BILOU labels như `E-PER`, `S-PER`, `L-PER` hoặc
`U-PER`: decoder hiện tại chỉ ghép span từ `B-*` và `I-*`. Muốn dùng model như vậy phải
mở rộng `parse_bio_label` và logic decode trước.

### Cài công cụ và export ONNX

Chạy từ repository root. `_tools/` và `dist/` đã được gitignore; không commit virtual
environment, checkpoint hay runtime binary:

```bash
python3 -m venv _tools/onnx-venv
source _tools/onnx-venv/bin/activate

python -m pip install --upgrade pip
python -m pip install "optimum[onnx]" transformers onnxruntime

optimum-cli export onnx \
  -m shibing624/bert4ner-base-chinese \
  --task token-classification \
  dist/ner-fp32
```

Thư mục kết quả tối thiểu phải có:

```text
dist/ner-fp32/
├── model.onnx
├── tokenizer.json
└── config.json
```

Một số checkpoint cũ chỉ lưu `vocab.txt`, nên exporter có thể không sinh
`tokenizer.json`. Tạo fast-tokenizer tương ứng vào cùng thư mục:

```bash
python - <<'PY'
from transformers import AutoTokenizer

tokenizer = AutoTokenizer.from_pretrained(
    "shibing624/bert4ner-base-chinese",
    use_fast=True,
)
tokenizer.save_pretrained("dist/ner-fp32")
PY
```

Không lấy `tokenizer.json` hoặc `config.json` từ checkpoint khác: token ID và thứ tự
`id2label` phải khớp chính xác với weights trong `model.onnx`.

### Kiểm tra artifact

Kiểm tra file, input graph và labels trước khi khởi động Rust server:

```bash
test -s dist/ner-fp32/model.onnx
test -s dist/ner-fp32/tokenizer.json
test -s dist/ner-fp32/config.json

python - <<'PY'
import json

import onnxruntime as ort
from tokenizers import Tokenizer

model_dir = "dist/ner-fp32"
Tokenizer.from_file(f"{model_dir}/tokenizer.json")

with open(f"{model_dir}/config.json", encoding="utf-8") as handle:
    labels = json.load(handle)["id2label"]

session = ort.InferenceSession(
    f"{model_dir}/model.onnx",
    providers=["CPUExecutionProvider"],
)
inputs = {item.name for item in session.get_inputs()}
unsupported = inputs - {"input_ids", "attention_mask", "token_type_ids"}

assert "input_ids" in inputs, inputs
assert not unsupported, unsupported
assert session.get_outputs(), "model has no output"
assert labels, "config.id2label is empty"

print("inputs:", sorted(inputs))
print("output:", session.get_outputs()[0].name)
print("labels:", labels)
PY
```

Adapter dùng output đầu tiên làm logits `f32` với shape `[batch, sequence, labels]`. Nếu
script trên qua nhưng server báo output shape hoặc type không hợp lệ, model export không
đúng task `token-classification`.

### Quantize INT8 tùy chọn

Dynamic quantization giảm đáng kể kích thước weights và thường phù hợp CPU inference,
nhưng có thể làm thay đổi confidence hoặc entity output. Luôn giữ bản FP32 để benchmark:

```bash
mkdir -p dist/ner-int8
cp dist/ner-fp32/tokenizer.json dist/ner-int8/
cp dist/ner-fp32/config.json dist/ner-int8/

python - <<'PY'
from onnxruntime.quantization import QuantType, quantize_dynamic

quantize_dynamic(
    "dist/ner-fp32/model.onnx",
    "dist/ner-int8/model.onnx",
    weight_type=QuantType.QInt8,
)
PY
```

Chạy lại bước kiểm tra artifact với `model_dir = "dist/ner-int8"` rồi so precision,
recall, F1 và latency với FP32 trên cùng gold corpus.

### Tải ONNX Runtime

Crate `ort` hiện build với ONNX Runtime API 24 và `load-dynamic`, vì vậy process phải tìm
thấy shared library ONNX Runtime 1.24.x qua `ORT_DYLIB_PATH`. Ví dụ dùng bản 1.24.2:

```bash
ORT_VERSION=1.24.2
ORT_ARCH=x64
ORT_OUTPUT="dist/ort-${ORT_ARCH}"

mkdir -p "${ORT_OUTPUT}"
curl -fL \
  "https://github.com/microsoft/onnxruntime/releases/download/v${ORT_VERSION}/onnxruntime-linux-${ORT_ARCH}-${ORT_VERSION}.tgz" \
  -o "dist/onnxruntime-${ORT_ARCH}.tgz"
tar -xzf "dist/onnxruntime-${ORT_ARCH}.tgz" -C "${ORT_OUTPUT}"
```

Dùng `ORT_ARCH=x64` cho Linux x86_64 local. Dùng `ORT_ARCH=aarch64` cho Lambda ARM64.
Model ONNX và JSON độc lập kiến trúc; chỉ binary Rust và `libonnxruntime.so` phải khớp
kiến trúc máy chạy.

### Chạy local với NER

Ví dụ Linux x86_64:

```bash
export QT_NER_MODEL="$PWD/dist/ner-fp32/model.onnx"
export QT_NER_TOKENIZER="$PWD/dist/ner-fp32/tokenizer.json"
export QT_NER_CONFIG="$PWD/dist/ner-fp32/config.json"
export ORT_DYLIB_PATH="$PWD/dist/ort-x64/onnxruntime-linux-x64-1.24.2/lib/libonnxruntime.so.1.24.2"

QT_DATA_DIR=QT2025 QT_PORT=3000 \
  cargo run --release -p qt-api --features onnx --bin qt-server
```

NER chỉ chạy khi server được build với feature `onnx`, đã load đủ artifact và request bật
`ner.enabled`:

```bash
curl -X POST http://localhost:3000/names/filter \
  -H 'content-type: application/json' \
  -d '{
    "text": "来人名为萧炎。萧炎看向云韵。",
    "mode": "hybrid",
    "minOccurrences": 1,
    "ner": {
      "enabled": true,
      "minConfidence": 0.65
    }
  }'
```

Response cần có `capabilities.nerConfigured=true`. `stats.nerCandidates` có thể bằng `0`
nếu model không nhận ra entity trong sample; đó không phải lỗi cấu hình.

### Xử lý lỗi khởi động

`name NER provider was not initialized: failed to load NER tokenizer: No such file or
directory` nghĩa là `QT_NER_TOKENIZER` trỏ tới file không tồn tại trong filesystem của
process. Kiểm tra cả bốn path:

```bash
ls -l \
  "$QT_NER_MODEL" \
  "$QT_NER_TOKENIZER" \
  "$QT_NER_CONFIG" \
  "$ORT_DYLIB_PATH"
```

Server đọc các biến này một lần lúc khởi động, nên phải dừng process cũ và restart sau khi
đổi path. Các lỗi thường gặp:

- chỉ export biến môi trường nhưng chưa tạo artifact dưới `dist/`;
- chạy lệnh từ thư mục khác nhưng dùng relative path;
- export model không sinh `tokenizer.json`;
- build server thiếu `--features onnx`;
- dùng ONNX Runtime x64 cho Lambda ARM64 hoặc ngược lại;
- dùng runtime cũ hơn API mà crate `ort` yêu cầu.

Muốn chạy rules/hybrid mà không bật NER, bỏ cấu hình rồi restart:

```bash
unset QT_NER_MODEL QT_NER_TOKENIZER QT_NER_CONFIG ORT_DYLIB_PATH
```

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
