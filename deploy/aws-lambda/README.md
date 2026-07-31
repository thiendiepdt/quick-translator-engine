# Deploy lên AWS Lambda

Lambda dùng native Rust runtime và trực tiếp chạy Axum router của `qt-api`. Artifact nhúng
toàn bộ dictionary mặc định từ `QT2025`, gồm hai dictionary cố định:

- `QT2025/VietPhrase/VietPhrase.txt`
- `QT2025/Resources/ChinesePhienAmWords.txt`

Names, Names2, Luật Nhân, Pronouns, Danh Từ, Họ Người, Hậu Từ và Ignored Chinese Phrases
cũng được nhúng làm default. Client có thể tải raw content qua `GET
/dictionaries/defaults` và thay từng file bằng field `dictionaries` của request. Vì dữ
liệu dùng `include_str!`, thay bất kỳ default nào cũng cần build và deploy version Lambda
mới.

## Cấu hình mặc định

- Region đề xuất cho người dùng Việt Nam: `ap-southeast-1` (Singapore).
- Kiến trúc: ARM64.
- Memory: 1.769 MB, tương đương khoảng một vCPU Lambda.
- Timeout: 30 giây cho một chương/request.
- Reserved concurrency: 50 để giới hạn burst và chi phí.
- Function URL auth: `AWS_IAM`.
- CloudWatch log retention: 14 ngày.

## Yêu cầu

- Rust stable.
- [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html).
- AWS SAM CLI và AWS credentials đã cấu hình.

## Build và test local

Build trực tiếp bằng Cargo để kiểm tra code:

```bash
cargo test -p qt-lambda
cargo build --release -p qt-lambda
```

Build đúng artifact ARM64 của Lambda:

```bash
cargo lambda build --release --arm64 --bin qt-lambda
```

Build có ONNX NER (artifact mặc định không kèm ONNX để tránh tăng cold start):

```bash
cargo lambda build --release --arm64 --bin qt-lambda --features onnx
```

Build qua SAM. Build method Rust của SAM hiện cần bật beta feature và dùng Cargo Lambda:

```bash
sam build \
  --template-file deploy/aws-lambda/template.yaml \
  --beta-features

sam local invoke QuickTranslatorFunction \
  --template-file .aws-sam/build/template.yaml \
  --event deploy/aws-lambda/events/translate.json
```

Local response phải có status `200` và body chứa `{"translated":"Rất tốt"}`.

Nếu máy x86_64 không có ARM emulation, build một artifact riêng chỉ để test local:

```bash
sam build \
  --template-file deploy/aws-lambda/template.yaml \
  --build-dir .aws-sam/build-local \
  --parameter-overrides FunctionArchitecture=x86_64 \
  --beta-features

sam local invoke QuickTranslatorFunction \
  --template-file .aws-sam/build-local/template.yaml \
  --event deploy/aws-lambda/events/translate.json \
  --parameter-overrides FunctionArchitecture=x86_64
```

Build lại không có parameter override trước khi deploy để artifact production dùng ARM64.

## Deploy

Deploy lần đầu bằng CloudFormation/SAM để có state và rollback:

```bash
sam deploy \
  --template-file .aws-sam/build/template.yaml \
  --stack-name quick-translator-engine \
  --region ap-southeast-1 \
  --resolve-s3 \
  --capabilities CAPABILITY_IAM \
  --confirm-changeset \
  --parameter-overrides \
    FunctionArchitecture=arm64 \
    FunctionUrlAuthType=AWS_IAM \
    FunctionMemorySize=1769 \
    ReservedConcurrency=50
```

`AWS_IAM` là mặc định an toàn. Client gọi Function URL phải ký SigV4 và có cả
`lambda:InvokeFunctionUrl` lẫn `lambda:InvokeFunction`. Nếu đặt Cloudflare Worker phía
trước, dùng [Cloudflare gateway](../cloudflare-worker/README.md) để ký request gửi về
origin và giữ Function URL ở `AWS_IAM`.

Chỉ dùng `FunctionUrlAuthType=NONE` để test nhanh hoặc khi đã có một lớp gateway bảo vệ
đúng cách. `NONE` biến URL thành public endpoint; engine chưa tự cung cấp authentication
hay rate limiting.

## Lambda lọc name chuyên dụng

`qt-ner-lambda` chỉ expose:

- `GET /health`;
- `GET /capabilities`;
- `POST /names/filter`.

Binary bật ONNX mặc định, nhúng dictionaries QT2025 nhưng không nhúng model/runtime. Build
ARM64:

```bash
cargo lambda build --release --arm64 --bin qt-ner-lambda
```

Build và test event bằng SAM template riêng:

```bash
sam build \
  --template-file deploy/aws-lambda/ner-template.yaml \
  --beta-features

sam local invoke QuickTranslatorNerFunction \
  --template-file .aws-sam/build/template.yaml \
  --event deploy/aws-lambda/events/name-filter.json
```

Không attach Layer và không truyền path thì Lambda vẫn chạy QT/hybrid rules; request bật
NER sẽ nhận warning provider chưa được cấu hình.

Sau khi publish Layer có `ner/` và `lib/` ở root, deploy bằng ARN cùng bốn path `/opt`:

```bash
sam deploy \
  --template-file .aws-sam/build/template.yaml \
  --stack-name quick-translator-ner \
  --region ap-southeast-1 \
  --resolve-s3 \
  --capabilities CAPABILITY_IAM \
  --confirm-changeset \
  --parameter-overrides \
    FunctionArchitecture=arm64 \
    FunctionUrlAuthType=AWS_IAM \
    FunctionMemorySize=3008 \
    ReservedConcurrency=10 \
    NerLayerArn=arn:aws:lambda:REGION:ACCOUNT:layer:qt-ner:VERSION \
    NerModelPath=/opt/ner/model.onnx \
    NerTokenizerPath=/opt/ner/tokenizer.json \
    NerConfigPath=/opt/ner/config.json \
    OrtDylibPath=/opt/lib/libonnxruntime.so.1.24.2
```

Cloudflare Worker hiện đã cho phép route `/names/filter`, nên có thể đổi Lambda Function
URL origin sang function chuyên dụng mà không đổi request contract.

## Provider lọc name tùy chọn

`POST /names/filter` luôn có QT/hybrid rules chạy local. Hai provider sau là opt-in:

### ONNX NER

Chuẩn bị `model.onnx`, `tokenizer.json`, `config.json`, ONNX Runtime và kiểm tra model
theo hướng dẫn end-to-end tại
[`docs/engine/name-filter.md`](../../docs/engine/name-filter.md#onnx-ner). Binary Lambda
phải được build cho đúng kiến trúc và bật feature:

```bash
cargo lambda build --release --arm64 --bin qt-lambda --features onnx
```

Với Lambda ARM64, tải gói ONNX Runtime `linux-aarch64` cùng version API mà crate `ort`
yêu cầu. Không dùng shared library `linux-x64` từ máy development.

Artifact runtime phải xuất hiện trong execution environment theo layout:

```text
/opt/
├── lib/
│   └── libonnxruntime.so.1.24.2
└── ner/
    ├── model.onnx
    ├── tokenizer.json
    └── config.json
```

Cấu hình environment bằng path tuyệt đối, trỏ trực tiếp tới file thực:

```text
QT_NER_MODEL=/opt/ner/model.onnx
QT_NER_TOKENIZER=/opt/ner/tokenizer.json
QT_NER_CONFIG=/opt/ner/config.json
ORT_DYLIB_PATH=/opt/lib/libonnxruntime.so.1.24.2
```

`config.json` cần `id2label` theo BIO, ví dụ `B-PER`, `I-PER`, `B-LOC`, `B-ORG`. Adapter
chạy một inference tại một thời điểm trên mỗi Lambda instance và cắt chương thành block
400 ký tự.

Lambda Layer được giải nén vào `/opt`, nên file zip của layer phải có `lib/` và `ner/` ở
root. Zip deployment và toàn bộ layer dùng chung giới hạn kích thước giải nén của Lambda;
checkpoint FP32 BERT thường không vừa khi cộng binary và dictionaries. Xem
[Lambda quotas](https://docs.aws.amazon.com/lambda/latest/dg/gettingstarted-limits.html)
trước khi publish.

- Dùng container image cho FP32 hoặc khi tổng artifact vượt giới hạn zip/layer.
- Có thể dùng Layer cho INT8 nếu tổng kích thước giải nén của function và mọi layer vẫn
  nằm dưới giới hạn; upload layer lớn qua S3.
- Model, tokenizer và config phải lấy từ cùng checkpoint.
- Đo cold start, peak memory và latency một chương trước khi bật traffic production.

Template `template.yaml` của Lambda đầy đủ không tự đóng gói hoặc attach ONNX artifacts.
Template `ner-template.yaml` của `qt-ner-lambda` nhận `NerLayerArn` và bốn path qua
parameters, nhưng bước export model, tạo Layer và publish Layer vẫn là thao tác deploy
riêng. Khi dùng container cần một template `PackageType: Image` và image ARM64 chứa đúng
layout ở trên.

### Gemini fallback

Gemini chỉ duyệt nhóm candidate có score mơ hồ, không thay toàn bộ rules/NER:

```text
QT_GEMINI_API_KEY=<secret>
QT_GEMINI_MODEL=<model hỗ trợ structured output>
```

Không đưa key vào request từ web. Lưu key bằng cơ chế secret/KMS phù hợp của AWS và inject
vào environment lúc deploy. Request provider lỗi vẫn trả kết quả rules kèm `warnings`, do
đó một lỗi mạng ngoài không làm mất toàn bộ kết quả lọc name.

## Request

Một chương là một `POST /translate`:

```json
{
  "text": "萧炎看着她说道……",
  "mode": "vietphrase-one",
  "pretty": true,
  "dictionaries": {
    "names": "萧炎=Tiêu Viêm",
    "names2": "",
    "pronouns": "她=nàng",
    "luatNhan": "在{n}身后=sau lưng {n}",
    "hoNguoi": "萧=Tiêu",
    "hauTu": "先生=tiên sinh",
    "ignoredChinesePhrases": "本章完"
  }
}
```

Giới hạn JSON body của router là 5 MiB. Names riêng của một truyện nên giữ nhỏ và gửi
inline. Nếu một bộ dictionary cỡ nhiều MiB được tái sử dụng qua hàng nghìn chương, nên bổ
sung `dictionaryId` cùng storage/cache thay vì gửi và parse lại ở mọi request.

## Lưu ý về dữ liệu nhúng

Artifact Lambda chứa nguyên các dictionary mặc định từ `QT2025/`; endpoint defaults còn
phân phối raw content của tám file tùy biến tới web client. Quyền phân phối các dữ liệu
tham chiếu này tách biệt với license GPL-3.0-only của code Rust. Trước khi cung cấp
artifact hoặc service công khai, cần kiểm tra quyền sử dụng và phân phối; xem
[THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md).
