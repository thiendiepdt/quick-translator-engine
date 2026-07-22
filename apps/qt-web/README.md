# qt-web

Web client cho Quick Translator Engine, chỉ sử dụng mode `vietphrase-one`. Giao diện kết
hợp split workspace của direction B với typography/output reader của direction C.

## Stack

- Vite + React + TypeScript.
- Tailwind CSS v4 qua `@tailwindcss/vite`.
- shadcn/ui dạng open-code trên Radix primitives.
- React Hook Form + Zod cho endpoint và engine options.
- TanStack Query cho health check và translate mutation.
- Zustand cho source, output, range selection và dictionary draft trong session.
- Vitest cho UTF-16 range, API client và dictionary semantics.

Nội dung chương và dictionary **không** được persist vào `localStorage`.

## Development với Rust API local

Khởi động API ở repository root:

```bash
QT_DATA_DIR=QT2025 QT_PORT=3000 cargo run -q -p qt-api --bin qt-server
```

Sau đó chạy web:

```bash
cd apps/qt-web
npm ci
npm run dev
```

Endpoint mặc định là `/api`. Vite proxy đường dẫn này tới `http://localhost:3000` và bỏ
prefix `/api` trước khi forward.

## Gọi Cloudflare gateway

Copy file environment mẫu:

```bash
cp .env.example .env.local
```

Sửa thành URL của Worker:

```dotenv
VITE_QT_API_URL=https://translate.example.com
```

Trong `deploy/cloudflare-worker/wrangler.jsonc`, thêm origin chính xác của web app vào
`CORS_ALLOWED_ORIGINS`, ví dụ:

```jsonc
"CORS_ALLOWED_ORIGINS": "https://qt.example.com,http://localhost:5173"
```

Web app không cần và không được nhận AWS credentials. Worker giữ credentials trong
Workers Secrets và ký SigV4 tới Lambda.

## Cloudflare Pages

Cấu hình project Pages:

- Root directory: `apps/qt-web`
- Build command: `npm run build`
- Build output directory: `dist`
- Environment variable: `VITE_QT_API_URL=https://<worker-domain>`

Sau lần deploy Pages đầu tiên, thêm origin Pages hoặc custom domain vào
`CORS_ALLOWED_ORIGINS` rồi deploy lại gateway.

## Commands

```bash
npm run typecheck
npm run lint
npm test
npm run build
npm run check
```

## Request behavior

- `mode` luôn là `vietphrase-one`.
- `ranges` luôn là `true` để click đối chiếu hai chiều.
- Dictionary chưa chỉnh không xuất hiện trong payload.
- Dictionary được đặt rỗng sẽ gửi chuỗi rỗng, đúng semantics “thay bằng tập rỗng”.
- VietPhrase và ChinesePhienAmWords không xuất hiện trong form vì Lambda cố định hai bộ
  này.
- Request timeout ở browser là 45 giây; Worker/Lambda vẫn giữ giới hạn riêng của hạ tầng.

Các bản thiết kế đã dùng để duyệt nằm trong `design-demos/`.
