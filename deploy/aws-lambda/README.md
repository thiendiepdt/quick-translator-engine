# Deploy lên AWS Lambda

Lambda dùng native Rust runtime và trực tiếp chạy Axum router của `qt-api`. Artifact nhúng
hai dictionary cố định:

- `QT2025/VietPhrase/VietPhrase.txt`
- `QT2025/Resources/ChinesePhienAmWords.txt`

Các dictionary còn lại mặc định rỗng và được truyền theo field `dictionaries` của từng
request. Vì dữ liệu cố định dùng `include_str!`, thay VietPhrase/Hán Việt cần build và
deploy một version Lambda mới.

## Cấu hình mặc định

- Region đề xuất cho người dùng Việt Nam: `ap-southeast-1` (Singapore).
- Kiến trúc: ARM64.
- Memory: 1.769 MB, tương đương khoảng một vCPU Lambda.
- Timeout: 30 giây cho một chương/request.
- Reserved concurrency: 20 để giới hạn burst và chi phí.
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
    ReservedConcurrency=20
```

`AWS_IAM` là mặc định an toàn. Client gọi Function URL phải ký SigV4 và có cả
`lambda:InvokeFunctionUrl` lẫn `lambda:InvokeFunction`. Nếu đặt Cloudflare Worker phía
trước, dùng [Cloudflare gateway](../cloudflare-worker/README.md) để ký request gửi về
origin và giữ Function URL ở `AWS_IAM`.

Chỉ dùng `FunctionUrlAuthType=NONE` để test nhanh hoặc khi đã có một lớp gateway bảo vệ
đúng cách. `NONE` biến URL thành public endpoint; engine chưa tự cung cấp authentication
hay rate limiting.

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

Artifact Lambda chứa nguyên VietPhrase và ChinesePhienAmWords từ `QT2025/`. Quyền phân
phối các dữ liệu tham chiếu này tách biệt với license GPL-3.0-only của code Rust. Trước
khi cung cấp artifact hoặc service công khai, cần kiểm tra quyền sử dụng và phân phối;
xem [THIRD_PARTY_NOTICES.md](../../THIRD_PARTY_NOTICES.md).
