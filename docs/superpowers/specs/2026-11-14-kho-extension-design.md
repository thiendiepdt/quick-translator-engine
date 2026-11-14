# Kho extension (phần 2 của tải raw qua extension)

Phần 1: [2026-11-07-tai-raw-extension-design.md](2026-11-07-tai-raw-extension-design.md). Phần 3 (marketplace) làm
sau, spec riêng.

## 1. Mục tiêu

- Nhà phát triển đăng **một kho** gồm nhiều extension; người dùng thêm kho **một lần** bằng link, rồi cài, cập
  nhật từng extension hoặc cập nhật hết.
- Kho có thể khai **auth dùng chung**: người dùng nhập API key một lần, mọi extension của kho được kho cho phép
  dùng chung đều nhận key đó.
- Kho chính thức của VNCVT nằm ở repo riêng `vncvt-ai-translator-officical-extentions`, deploy lên Cloudflare R2
  gắn domain công khai (đề xuất một domain riêng). Extension chính thức gọi API của `book-downloader`
  (`<nguồn>-api.khotangtruyen.org`, header `x-api-key: userId:secretKey`), không gọi thẳng trang nguồn.

Đã chốt với người dùng (2026-11-14): kho công khai qua domain (không cần key để xem kho, khoá nằm ở worker); API
key nhập một lần cho cả kho; app không có kho cài sẵn, người dùng tự thêm link kho.

## 2. Định dạng kho: `index.json`

Link kho là link tới `index.json`, hoặc thư mục chứa nó (app thêm `index.json`). Chỉ `https://`, không mạng nội
bộ, tối đa 2 MB.

```json
{
  "formatVersion": 1,
  "id": "org.khotangtruyen.official",
  "name": "VNCVT — extension chính thức",
  "description": "Extension tải raw qua API Kho Tàng Truyện",
  "maintainer": { "name": "thiendiepdt", "url": "https://github.com/thiendiepdt", "email": "" },
  "homepage": "",
  "auth": {
    "required": true,
    "schemes": [
      { "id": "apiKey", "label": "API key", "fields": [{ "key": "apiKey", "label": "API key (userId:secretKey)", "type": "secret" }] }
    ]
  },
  "extensions": [
    {
      "id": "org.khotangtruyen.qimao",
      "name": "Qimao 七猫",
      "version": "1.0.0",
      "apiVersion": 1,
      "description": "…",
      "developer": { "name": "thiendiepdt", "url": "…", "email": "" },
      "license": "MIT",
      "match": ["^https?://(www\\.|m\\.)?qimao\\.com/shuku/\\d+"],
      "hosts": ["qimao-api.khotangtruyen.org"],
      "permissions": [],
      "authRequired": true,
      "sharedAuth": true,
      "url": "extensions/org.khotangtruyen.qimao/1.0.0.zip",
      "sha256": "…",
      "size": 2408,
      "icon": "extensions/org.khotangtruyen.qimao/icon.png",
      "updatedAt": "2026-11-14"
    }
  ]
}
```

| Trường | Kiểm khi thêm / làm mới kho |
|---|---|
| `formatVersion` | `1`; cao hơn thì báo "cần app mới hơn" |
| `id` | tên miền đảo ngược như id extension; không trùng kho đã thêm |
| `name`, `maintainer.name` | không rỗng |
| `auth` | tuỳ chọn; cùng cú pháp `auth` của manifest |
| `extensions[].id`, `version`, `url`, `sha256` | bắt buộc; `version` semver; `sha256` 64 ký tự hex; id không trùng trong kho |
| `extensions[].url`, `icon` | tương đối với link `index.json` hoặc tuyệt đối; sau khi ghép phải là `https://` |
| `extensions[].sharedAuth` | `true` = extension dùng auth chung của kho; chỉ hợp lệ khi kho có `auth` |
| Các trường mô tả khác | chỉ để hiện danh sách; lúc cài app dùng manifest trong gói |

Mục lỗi (thiếu trường, sai kiểu) bị bỏ qua và hiện cảnh báo; cả file sai định dạng thì không thêm / giữ bản cũ.

## 3. Cài và cập nhật từ kho

1. Tải zip từ `url`, kiểm `sha256` khớp `index.json`; lệch thì từ chối ("gói trên kho bị sửa hoặc kho chưa cập nhật").
2. Giải nén vào `.staging/` như cài từ file; manifest phải có `id` và `version` đúng như mục trong kho.
3. Hộp xin quyền như phần 1 (domain, webview, auth). Extension dùng auth chung thì ghi thêm "Dùng API key chung
   của kho <tên kho>".
4. Sổ `installed.json` ghi nguồn `{ kind: "repository", repository: <id kho>, url: <link zip> }`.

Cập nhật: "Làm mới" tải lại `index.json` của mọi kho; mục có `version` lớn hơn bản đang cài hiện nút **Cập nhật**,
cạnh đó có **Cập nhật hết (N)**. Luật quyền như phần 1: thêm domain hoặc quyền thì hỏi lại từng extension; không
thêm thì cập nhật luôn. "Kiểm tra cập nhật" ở dòng extension cài từ kho làm mới kho đó rồi so.

## 4. Auth dùng chung của kho

- Lưu như auth của extension nhưng theo kho: kho khoá `qt-ext/repo/<id kho>/auth/<kiểu>/<trường>`, file
  `<config>/extension-data/repo-<id kho>/auth.json` (kiểu đã chọn, trường text, mã phiên bản).
- Extension cài từ kho có `sharedAuth: true` (theo `index.json` đã lưu) đọc auth từ kho thay cho auth riêng;
  `ctx.auth` có dạng như cũ. Đổi hoặc xoá auth của kho thì mã phiên bản đổi, session data của mọi extension dùng
  chung thành trống.
- Chỉ extension cài **từ chính kho đó** và được kho đánh dấu `sharedAuth` mới nhận key chung. Extension cài từ file
  hay nạp thư mục dev dùng auth riêng, kể cả khi trùng id: extension lạ không tự xin được key của kho.
- Form auth của extension dùng chung thay bằng dòng "Dùng API key chung của kho <tên>" và nút mở form auth của kho.

## 5. Lưu trữ phía app

```
<config>/extensions/repositories.json         [{ id, url, addedAt, refreshedAt, hidden? }]
<config>/extensions/.repos/<id kho>.json       index.json đã tải lần gần nhất (để xem kho và quyết sharedAuth khi offline)
```

Gỡ kho: xoá khỏi danh sách, xoá index đã lưu và auth chung. Extension đã cài từ kho vẫn giữ, chuyển sang dùng auth
riêng (người dùng nhập lại nếu cần) và không còn nút cập nhật theo kho.

## 6. Giao diện (Cài đặt → Extension tải raw)

- **Kho extension**: ô link + "Thêm kho"; mỗi kho một dòng: tên, người duy trì, link, số extension, lúc làm mới;
  nút Làm mới, **API key của kho…** (khi kho có auth), Gỡ kho. Cảnh báo mục bị bỏ qua.
- **Có trong kho**: danh sách extension của mọi kho, ô lọc theo tên / domain. Mỗi dòng: tên, phiên bản, mô tả,
  nhà phát triển, domain, nhãn "Dùng API key của kho"; trạng thái Chưa cài / Đã cài / Có bản mới; nút Cài /
  Cập nhật.
- **Đã cài**: như phần 1; nguồn "Từ kho <tên>".

## 7. Repo kho chính thức

```
repository.json          id, name, description, maintainer, auth chung
extensions/<thư mục>/    manifest.json, main.js, lib/…, icon.png
scripts/build.mjs        nén từng extension (zip deflate, thứ tự file cố định), sha256, sinh dist/index.json
                         + dist/extensions/<id>/<version>.zip (+ icon); kiểm manifest cơ bản; đánh dấu sharedAuth
                         khi mọi kiểu auth của extension có trong auth chung của kho
scripts/deploy.mjs       đưa dist lên R2 bằng wrangler: zip trước (Cache-Control immutable, bỏ qua bản đã có),
                         index.json sau cùng (max-age=60)
test/                    chạy extension với máy chủ giả mô phỏng API book-downloader
```

Mỗi bản phát hành là một file zip mới theo `version`; zip cũ giữ trên R2 để ai đang cài vẫn cài lại được. Deploy
chạy tay, cần `wrangler login` vào tài khoản Cloudflare có bucket. Tạo bucket và gắn domain làm một lần trên
dashboard (hoặc `wrangler r2 bucket create` + `wrangler r2 bucket domain add`).

Extension chính thức đầu tiên: **Qimao** gọi `qimao-api.khotangtruyen.org`:
`GET /book/<mã>` (mục lục), `GET /content?id=<mã>&chapterIds=a,b,c` (tối đa 100 chương; app gửi 20),
401 = API key sai.

## 8. Kiểm thử

- Core: thêm kho từ máy chủ giả (index hợp lệ, mục lỗi bị bỏ, `formatVersion` cao, trùng id); cài từ kho kiểm
  sha256 (lệch bị từ chối), manifest khác id/version bị từ chối; cập nhật thêm host thì cần đồng ý; auth chung: hai
  extension cùng kho đọc chung key, extension cài từ file cùng id không đọc được; đổi auth kho làm trống session.
- GUI: thêm kho, danh sách có trong kho với trạng thái, cài / cập nhật / cập nhật hết, form API key của kho.
- Repo kho: build ra index.json đúng sha256, hai lần build ra cùng sha256; qimao chạy trong sandbox với máy chủ giả mô phỏng worker; với key thật thì Nạp thư mục + Chạy thử trong app; deploy kiểm tay.

## 9. Ngoài phạm vi

- Ký số kho / gói (dựa vào HTTPS + domain).
- Kho riêng tư cần key để xem.
- Tìm kiếm truyện, marketplace (phần 3).
