# qt-web

Web client cho Quick Translator Engine, chỉ sử dụng mode `vietphrase-one`. Giao diện kết
hợp split workspace của direction B với typography/output reader của direction C.

## Stack

- Vite + React + TypeScript.
- Tailwind CSS v4 qua `@tailwindcss/vite`.
- shadcn/ui dạng open-code trên Radix primitives.
- React Hook Form + Zod cho endpoint và engine options.
- TanStack Query cho health check, default dictionaries và translate mutation.
- Zustand cho source, output, range selection, dictionary draft, memory lọc name và
  workspace truyện Dịch AI.
- Vitest cho UTF-16 range, API client và dictionary semantics.

Nội dung chương và toàn bộ raw dictionary **không** được persist vào `localStorage`.
Các workspace dùng IndexedDB (`qt-web` / `key-value`): Dịch AI lưu metadata truyện,
prompt, glossary, style, check rules, hàng đợi chương và kết quả để phục hồi sau reload.
Raw dictionary mặc định từ server vẫn không bị nhân bản; chỉ override và entry đã sửa
được lưu. API key/model/theme tiếp tục là preference nhỏ trong `localStorage`.

## Dịch AI theo truyện

- Mỗi workspace tương ứng một truyện và có cấu hình riêng: thông tin truyện, từ điển,
  style, prompt và regex kiểm tra sau dịch.
- `AI fill` ưu tiên Gemini và bật Google Search để điền metadata/glossary/style; nếu
  chưa có key Gemini thì dùng provider Dịch AI hiện tại mà không có search grounding.
- Có thể chọn hoặc kéo-thả nhiều file `.txt`/`.md`; chương được xếp tự nhiên theo tên
  file và dịch tuần tự để không tạo nhiều request provider đồng thời.
- Reload giữa chừng giữ raw/output đã có và đưa chương đang chạy về `Chờ dịch`.
  Người dùng bấm `Dịch hàng đợi` để tiếp tục; app không tự gọi API có tính phí sau reload.

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
"CORS_ALLOWED_ORIGINS": "https://dich.vn-converter.org,http://localhost:5173"
```

Web app không cần và không được nhận AWS credentials. Worker giữ credentials trong
Workers Secrets và ký SigV4 tới Lambda.

## Cloudflare Workers Static Assets

Web app được cấu hình trong `wrangler.jsonc` dưới dạng SPA, nên các đường dẫn không khớp
asset sẽ trả về `index.html`. Trước khi build, đặt gateway public trong `.env.local`:

```dotenv
VITE_QT_API_URL=https://qt-api.vn-converter.org
```

Kiểm tra và deploy từ thư mục này:

```bash
npm ci
npm run check
npm run deploy:dry-run
npm run deploy
```

Production được phục vụ tại `https://dich.vn-converter.org`. URL `workers.dev` bị tắt
bằng `workers_dev: false`. Origin production phải có trong `CORS_ALLOWED_ORIGINS` của
gateway trước khi web app gọi API từ browser.

## Commands

```bash
npm run typecheck
npm run lint
npm test
npm run build
npm run check
```

## Request behavior

- Dịch chương dùng `vietphrase-one`; tiện ích Hán Việt trong dialog dùng `hanviet`.
- `ranges` luôn là `true` để click đối chiếu hai chiều.
- Click phải một range output (long-press trên touch) mở menu cập nhật nhanh VietPhrase,
  Name chính/phụ, Phiên Âm, Danh Từ, Hậu Từ, Họ Người và Luật Nhân.
- Menu lấy key tiếng Trung bằng positional range mapping; text bôi đen trong cùng range
  được dùng làm value tiếng Việt ban đầu.
- Entry cập nhật nhanh được persist local. Các dictionary tùy biến được ghép vào draft;
  VietPhrase/Phiên Âm được gửi dưới dạng `dictionaryPatches` nhỏ ở mỗi request.
- Dialog cập nhật có thể điền âm Hán Việt, nhờ AI dịch theo cửa sổ ngữ cảnh raw, tra
  Lạc Việt qua `POST /meanings`, tra nghĩa bằng AI và đổi nhanh bốn kiểu chữ tiếng Việt.
- Web tải tám raw dictionary mặc định từ `GET /dictionaries/defaults`.
- Sidebar chỉ chọn và hiển thị tóm tắt dictionary để không render file lớn khi đổi tab.
- Dialog editor hiển thị records theo trang 100 dòng, hỗ trợ search key/value, sửa inline,
  thêm và xóa record.
- Editor bảo toàn BOM, kiểu xuống dòng, dòng raw/comment và trailing newline khi lưu.
- Dictionary chưa chỉnh không xuất hiện trong payload.
- Khi người dùng sửa hoặc thêm entry, web gửi toàn bộ nội dung file đã thay đổi.
- Dictionary được đặt rỗng sẽ gửi chuỗi rỗng, đúng semantics “thay bằng tập rỗng”.
- Khôi phục đúng bản QT2025 sẽ bỏ dictionary đó khỏi payload.
- VietPhrase và ChinesePhienAmWords không xuất hiện dưới dạng raw file trong form vì
  Lambda giữ base cố định; chỉ entry patch local được gửi lên.
- Request timeout ở browser là 45 giây cho API dịch/lọc; mỗi lượt gọi AI provider có
  timeout riêng 60 giây. Worker/Lambda vẫn giữ giới hạn riêng của hạ tầng.
- Tab **Tên** gọi `POST /names/filter`, hỗ trợ QT/hybrid, bật AI trích/duyệt tùy chọn, search,
  sửa tên Việt inline, duyệt/loại từng record và duyệt nhanh candidate ≥85%.
- Tính năng AI chạy ngay trong trình duyệt (`src/lib/ai-client.ts`): trình duyệt gọi
  thẳng DeepSeek/Gemini bằng key của chính người dùng — key không bao giờ đi qua server
  của app. Lọc tên chỉ gửi entity đã trích qua field `aiEntities`; dịch/tra nghĩa từ điển
  chỉ cập nhật UI local. Chọn provider, nhập
  key/model và Base URL proxy (tùy chọn) trong dialog Cài đặt. Base URL cho phép dùng
  endpoint trung gian OpenAI-compatible/relay Gemini, kể cả proxy chạy local
  (`http://localhost` được miễn trừ mixed-content); endpoint phải cho phép CORS. Cấu
  hình lưu ở localStorage tách riêng theo từng provider — đổi provider là đổi sang bộ
  key của provider đó, không bao giờ gửi key của công ty này sang endpoint công ty kia.
- Name được duyệt tự append/update vào draft `Names2`, nên request dịch tiếp theo sử dụng
  ngay mà không phải copy thủ công.

Các bản thiết kế đã dùng để duyệt nằm trong `design-demos/`.
