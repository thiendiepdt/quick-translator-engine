# Cloudflare reverse proxy cho hub AI OpenAI-compatible

Worker độc lập, không dính gì tới gateway Lambda. Nó đứng trước một hub
OpenAI-compatible (ví dụ `http://192.0.2.10/v1`) để qt-web deploy trên https
gọi được — trình duyệt chặn mixed-content, không cho trang https gọi thẳng `http://IP`.

```text
Browser (qt-web, https) -> Worker (https://…workers.dev) -> UPSTREAM_BASE_URL (http/https)
```

- Worker **không giữ API key**: header `Authorization` của trình duyệt được chuyển
  nguyên sang hub; key vẫn chỉ nằm trong localStorage của người dùng.
- Chỉ mở hai route qt-web cần: `POST /chat/completions` và `GET /models`. Route khác
  trả 404, sai method trả 405, thiếu `Authorization` trả 401 trước khi chạm hub.
- Body request đọc có trần 5 MiB; response (kể cả stream SSE) chảy thẳng về client,
  không đọc vào bộ nhớ, không log nội dung. Cookie và header khác không được forward.
- CORS theo allowlist `CORS_ALLOWED_ORIGINS` (origin chính xác, hoặc `*`).
- Lỗi của worker trả về dạng `{"error":{"message":…}}` như OpenAI để qt-web hiện
  được thông báo trong log dịch.

## Cấu hình

Tất cả nằm trong [wrangler.jsonc](wrangler.jsonc), không có secret:

| Biến                   | Ý nghĩa                                                                 |
| ---------------------- | ----------------------------------------------------------------------- |
| `UPSTREAM_BASE_URL`    | Base URL của hub, gồm cả prefix `/v1`. Worker chỉ nối tới đúng URL này. |
| `CORS_ALLOWED_ORIGINS` | Danh sách origin trình duyệt được phép, cách nhau bằng dấu phẩy.        |

Mặc định `workers_dev: true` để có URL https ngay sau deploy. Muốn domain riêng thì
thêm `routes` với `custom_domain: true` như gateway và tắt `workers_dev`.

## Chạy

```bash
cd deploy/cloudflare-ai-proxy
npm install
npm run check        # types, typecheck, lint, test, deploy --dry-run
npm run dev          # http://localhost:8787
npm run deploy       # cần `npx wrangler login` trước
```

Sau khi deploy, wrangler in ra URL dạng `https://quick-translator-ai-proxy.<account>.workers.dev`.

## Dùng với qt-web

Cài đặt → provider **OpenAI** → Base URL điền URL worker (không thêm `/v1`, vì
`UPSTREAM_BASE_URL` đã có sẵn prefix đó) → key của hub → model của hub, ví dụ
`gemini-3.7-flash`.

Thử nhanh bằng curl:

```bash
curl https://quick-translator-ai-proxy.<account>.workers.dev/chat/completions \
  -H "Authorization: Bearer <key hub>" \
  -H "Content-Type: application/json" \
  -d '{"model":"gemini-3.7-flash","messages":[{"role":"user","content":"Xin chào"}]}'
```

## Lưu ý bảo mật

Worker là relay công khai tới **một** upstream cố định. Ai biết URL worker vẫn phải
có key hợp lệ của hub mới dùng được, và allowlist CORS chỉ chặn trình duyệt, không chặn
curl. Nếu hub tính phí theo key thì bảo vệ key như bình thường; muốn khoá chặt hơn thì
đặt Cloudflare Access hoặc thêm secret riêng cho worker.
