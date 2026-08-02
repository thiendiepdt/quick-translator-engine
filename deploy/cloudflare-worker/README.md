# Cloudflare gateway cho AWS Lambda

Worker này là public edge gateway của Quick Translator Engine:

```text
Client -> Cloudflare Worker -> SigV4 -> Lambda Function URL (AWS_IAM)
```

Worker giữ Lambda URL ở chế độ `AWS_IAM`, ký mỗi request bằng SigV4 và stream response
trở lại client. Client không nhận AWS credentials và không thể bypass Worker để gọi trực
tiếp Lambda. Worker chỉ proxy sáu route:

- `GET /health`
- `GET /modes`
- `GET /dictionaries/defaults`
- `POST /translate`
- `POST /translate/batch`
- `POST /names/filter`

Request body được đọc có giới hạn 5 MiB để tạo payload hash cho SigV4. Cookie và
`Authorization` của client không được forward tới AWS. Body đi nguyên vẹn tới Lambda và
Worker không log request body. API key AI của người dùng không bao giờ đi qua Worker:
trình duyệt gọi thẳng provider, request `/names/filter` chỉ chứa entity đã trích
(`aiEntities`). Response defaults được browser cache một giờ; các response khác luôn có
`Cache-Control: no-store`. Retry tới Lambda bị tắt để một chương không bị invoke lặp.

## Yêu cầu

- Node.js 22 trở lên.
- Lambda đã deploy với `FunctionUrlAuthType=AWS_IAM`.
- AWS IAM access key riêng, chỉ có quyền invoke đúng Lambda này.
- Tài khoản Cloudflare Workers và Wrangler đã login.

## 1. Tạo IAM principal cho Worker

Tạo một IAM user riêng, ví dụ `quick-translator-cloudflare`. Sửa account ID, region và
function name trong [iam-policy.example.json](iam-policy.example.json), sau đó gắn policy
đó vào user. Policy cấp đúng hai action AWS yêu cầu và chỉ cho invoke qua Function URL.

Ví dụ bằng AWS CLI:

```bash
aws iam create-user --user-name quick-translator-cloudflare

aws iam put-user-policy \
  --user-name quick-translator-cloudflare \
  --policy-name InvokeQuickTranslatorFunctionUrl \
  --policy-document file://iam-policy.example.json

aws iam create-access-key --user-name quick-translator-cloudflare
```

Lệnh cuối chỉ hiển thị secret access key một lần. Không ghi credentials vào
`wrangler.jsonc`, source code hay Git. Nên đặt lịch rotate access key.

## 2. Cấu hình Worker

Cài dependency:

```bash
cd deploy/cloudflare-worker
npm ci
```

Thay `LAMBDA_FUNCTION_URL` và `AWS_REGION` trong [wrangler.jsonc](wrangler.jsonc) bằng
output của SAM stack. Có thể lấy URL bằng:

```bash
aws cloudformation describe-stacks \
  --stack-name quick-translator-engine \
  --region ap-southeast-1 \
  --query "Stacks[0].Outputs[?OutputKey=='FunctionUrl'].OutputValue" \
  --output text
```

Lưu credentials bằng Workers Secrets. Wrangler sẽ hỏi value bằng prompt ẩn:

```bash
npx wrangler login
npx wrangler secret put AWS_ACCESS_KEY_ID
npx wrangler secret put AWS_SECRET_ACCESS_KEY
```

Hai secret được khai báo trong `secrets.required`; Wrangler sẽ chặn deploy nếu thiếu.

Đặt `CORS_ALLOWED_ORIGINS` thành danh sách origin chính xác được phép gọi API từ browser,
phân tách bằng dấu phẩy. Ví dụ khi deploy `qt-web` bằng Workers Static Assets:

```jsonc
"CORS_ALLOWED_ORIGINS": "https://dich.vn-converter.org,http://localhost:5173"
```

Worker chỉ chấp nhận preflight cho route/method hợp lệ và hai request header `accept`,
`content-type`. Origin ngoài allowlist trả `403`. Có thể dùng `*` cho API hoàn toàn public,
nhưng allowlist giúp hạn chế website bên thứ ba lợi dụng browser người dùng để đốt quota.

## 3. Kiểm tra và deploy

```bash
npm run check
npx wrangler deploy --minify
```

Worker mặc định có URL `workers.dev`. Để dùng custom domain, thêm route sau vào
`wrangler.jsonc` với domain thuộc Cloudflare zone tương ứng:

```jsonc
{
  "routes": [
    { "pattern": "translate.example.com", "custom_domain": true }
  ]
}
```

Sau khi deploy:

```bash
curl https://translate.example.com/health

curl -X POST https://translate.example.com/translate \
  -H "content-type: application/json" \
  -d '{"text":"很好","mode":"vietphrase-one","pretty":true}'
```

## Test local

Copy file mẫu và thay bằng credentials của IAM user nếu cần gọi Lambda thật từ
`wrangler dev`:

```bash
cp .dev.vars.example .dev.vars
npx wrangler dev
```

`.dev.vars` đã bị Git ignore. Unit test dùng credentials giả và mock outbound request,
không gọi AWS thật.

## Ranh giới bảo mật

SigV4 chỉ xác thực Worker với Lambda. Module này chưa xác thực end user; CORS chỉ chặn
browser, không chặn bot/cURL (request không có `Origin` vẫn được proxy). Vì vậy:

- Chi phí AI không phải rủi ro của operator: server không gọi AI — client tự gọi
  provider bằng key của mình và chỉ gửi kết quả dạng dữ liệu trơ.
- Trước khi public cho nhiều người dùng, cấu hình Cloudflare Access/API token, WAF và
  rate limiting trên custom domain (các route chưa có rate limit trong code, kẻ xấu vẫn
  đốt được compute Lambda).
- Không đổi Lambda sang auth `NONE`, nếu không caller vẫn có thể bypass toàn bộ lớp
  Cloudflare bằng Function URL gốc.
