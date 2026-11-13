# Tải raw qua extension

Ngày: 2026-11-07. Trạng thái: chờ duyệt. Đây là phần 1 trong 3 phần:

1. **Phần 1 (tài liệu này):** link tải raw của truyện, định dạng extension, sandbox chạy extension, auth,
   cài đặt, cache, session data, webview, tải lẻ, tải loạt, mẫu tên file, cài extension từ file hoặc link,
   chế độ developer, tài liệu cho developer.
2. **Phần 2:** kho lưu trữ (một link chứa nhiều extension, cài và cập nhật từ kho). Spec riêng.
3. **Phần 3:** tích hợp marketplace do user dựng. Spec riêng, làm khi marketplace có API.

## 1. Mục tiêu

Người dùng tải raw thẳng trong app thay vì dùng công cụ ngoài rồi chép file vào `raw/`. Phần biết cách
lấy chương từ từng trang nguồn nằm ở extension. Bất kỳ ai cũng viết và phát hành được extension, nên app
coi mọi extension là code lạ: chạy trong sandbox, chỉ gọi mạng tới domain đã khai, không đụng vào ổ đĩa.

Không làm ở phần 1: kho lưu trữ, marketplace, lệnh CLI `qt-ai raw`, chạy extension theo định dạng của app đọc truyện khác, chèn script vào
trang trong webview, hàm lấy thông tin sách (tên, tác giả, bìa). Xem mục 13.

## 2. Hiện trạng

- Truyện có `sourceUrl` ("Link nguồn") trong `story.json`, dùng cho AI điền hồ sơ và export EPUB. App
  chưa tự tải raw.
- Chương raw là các file `raw/<id>.txt`; id là tên file, sắp theo thứ tự tự nhiên (`natural_chapter_compare`
  trong `story_fs.rs`). `init` đồng bộ `state.json` theo danh sách file.
- File raw: dòng đầu là tiêu đề, một dòng trống, rồi các đoạn. Chương xin nghỉ có thể chỉ một câu
  (`long-tang/raw/chuong-1619.txt`: 请假条 / 今晚事务繁忙，先请个假。).
- Kiểu tên file trong thư viện `bookshelf/books/common`: 48 truyện `0001_<mã chương>`, 18 truyện
  `chuong-0001`, 10 truyện `<mã sách>_0001_<tiêu đề>`, vài truyện `0001 第1章 <tiêu đề>`.
- Số trong tiêu đề không khớp được với vị trí: file `1283_…` của `ac-nu-ta-muon-nguoi-giup-ta-tu-hanh`
  có tiêu đề 第1284章.
- `story.json` được đọc theo danh sách trường cố định rồi ghi lại đúng danh sách đó, nên bản app cũ lưu hồ
  sơ sẽ xoá trường lạ. Vì vậy `cast.json`, `titles.json` nằm file riêng, và phần tải raw cũng vậy.

## 3. Dữ liệu của truyện: `download.json`

File mới ở thư mục truyện, cạnh `story.json`:

```json
{
  "rawUrl": "https://fanqienovel.com/page/7445623259162542654",
  "extension": "io.github.thiendiepdt.fanqie",
  "fileTemplate": "{chuong:4}_{chapter-id}",
  "settings": { "io.github.thiendiepdt.fanqie": { "proxy": "" } },
  "chapters": { "7445626961335960126": "0002_7445626961335960126" }
}
```

| Trường | Ý nghĩa |
|---|---|
| `rawUrl` | Link tải raw. Thiếu hoặc rỗng thì app dùng `sourceUrl` của `story.json`. Chỉ ghi khi người dùng sửa. |
| `extension` | Extension đã chọn cho truyện. Rỗng thì tự chọn theo `match` (mục 4). |
| `fileTemplate` | Mẫu tên file. Rỗng thì dùng mẫu chung trong Cài đặt (mặc định `{chuong:4}_{chapter-id}`). |
| `settings` | Giá trị các trường cài đặt `scope: "story"`, theo id extension. Không bao giờ chứa trường `secret`. |
| `chapters` | Sổ chương: mã chương trong mục lục → id file raw. |

Thư mục truyện có thể là repo git (thư viện `bookshelf` là vậy), nên `download.json` không chứa khoá,
mật khẩu, cookie hay session.

Giao diện:
- Hộp tạo truyện thêm ô "Link tải raw". Ô này tự chép theo "Link nguồn" cho tới khi người dùng sửa tay,
  giống ô slug.
- Hồ sơ truyện thêm ô "Link tải raw" ngay dưới "Link nguồn". Truyện cũ hiện sẵn link nguồn.

## 4. Gói extension

Một zip (tối đa 5 MB, tối đa 200 file) hoặc một thư mục (chế độ developer):

```
manifest.json
main.js          điểm vào, ES module
lib/…            tuỳ chọn, main.js import được
icon.png         tuỳ chọn
```

`manifest.json`:

```json
{
  "id": "io.github.thiendiepdt.fanqie",
  "name": "Fanqie",
  "version": "1.2.0",
  "apiVersion": 1,
  "description": "Tải raw từ fanqienovel.com",
  "developer": { "name": "thiendiepdt", "url": "https://github.com/thiendiepdt", "email": "" },
  "license": "MIT",
  "homepage": "https://github.com/thiendiepdt/qt-ext-fanqie",
  "match": ["^https://(www\\.)?fanqienovel\\.com/page/\\d+"],
  "hosts": ["fanqienovel.com", "*.fqnovel.com"],
  "permissions": ["webview"],
  "auth": {
    "required": false,
    "schemes": [
      { "id": "apiKey", "label": "API key", "fields": [{ "key": "apiKey", "label": "API key", "type": "secret" }] },
      { "id": "password", "label": "Tài khoản", "fields": [
        { "key": "username", "label": "Tên đăng nhập", "type": "text" },
        { "key": "password", "label": "Mật khẩu", "type": "secret" }
      ] }
    ]
  },
  "settings": [
    { "key": "server", "label": "Máy chủ", "type": "select", "choices": ["cn", "global"], "default": "cn" },
    { "key": "fullWidth", "label": "Giữ dấu câu toàn khổ", "type": "boolean", "default": true },
    { "key": "proxy", "label": "Proxy riêng", "type": "text", "scope": "story" }
  ],
  "limits": { "concurrency": 3, "delayMs": 300 }
}
```

| Trường | Bắt buộc | Kiểm khi cài |
|---|---|---|
| `id` | có | dạng tên miền đảo ngược, `^[a-z0-9]+(\.[a-z0-9-]+)+$` |
| `name`, `description` | có | chuỗi không rỗng |
| `version` | có | semver |
| `apiVersion` | có | app hỗ trợ; cao hơn thì báo "cần app mới hơn" |
| `developer.name` | có | chuỗi không rỗng; `url`, `email` tuỳ chọn |
| `license`, `homepage` | không | |
| `match` | có | mỗi mục là regex biên dịch được |
| `hosts` | có | tên miền, cho phép `*.` ở đầu; không IP, không cổng. `localhost` chỉ ở chế độ developer |
| `permissions` | không | hiện chỉ có `webview` |
| `auth` | không | kiểu xếp theo thứ tự ưu tiên; trường có `type` `text` hoặc `secret` |
| `settings` | không | `type`: `text`, `number`, `boolean`, `select`, `secret`; `secret` không được khai `scope: "story"` |
| `limits` | không | `concurrency` mặc định 1, app chặn trần 8; `delayMs` mặc định 0 |

Chọn extension cho truyện: lấy các extension đang bật có `match` khớp link tải raw. Một cái thì dùng luôn;
nhiều cái thì người dùng chọn và lựa chọn được ghi vào `download.json`; không cái nào thì trang Tải raw báo
chưa có extension cho link này.

## 5. Hàm của extension

```js
export async function toc(ctx) { … }            // bắt buộc
export async function chapter(ctx, ch) { … }    // bắt buộc
export async function chapters(ctx, list) { … } // tuỳ chọn
export async function login(ctx) { … }          // tuỳ chọn
```

| Hàm | Trả về |
|---|---|
| `toc(ctx)` | `[{ id, title, locked?, volume?, data? }]` theo đúng thứ tự đọc. `id` là chuỗi, không trùng. `data` là dữ liệu riêng của extension, app trả lại nguyên vẹn khi gọi `chapter`. |
| `chapter(ctx, ch)` | `{ content, title?, format? }`. `ch` là mục của mục lục cộng `index` (vị trí, bắt đầu từ 1). `format` là `"text"` (mặc định) hoặc `"html"`. |
| `chapters(ctx, list)` | `[{ id, content, title?, format?, error? }]` cho API tải gộp. Không có hàm này thì app gọi `chapter` lần lượt. |
| `login(ctx)` | `{ token, expiresAt?, data? }`. App lưu vào session data và đưa lại qua `ctx.session`. |

Lỗi có sẵn để extension ném:

| Lỗi | App xử lý |
|---|---|
| `AuthExpired` | xoá phiên, gọi `login`, chạy lại đúng lệnh đó một lần |
| `Locked` | đánh dấu chương Khoá, không ghi file |
| `NeedsInteraction` | báo người dùng cần mở app để đăng nhập (dùng khi chạy nơi không có webview) |
| `ExtensionError(message, { retry })` | `retry: true` thì thử lại như lỗi mạng, `false` thì đánh dấu Lỗi ngay |

`ctx` (chỉ đọc):

```js
ctx = {
  sourceUrl, rawUrl,
  story: { name, chapterCount, current: { index, id, title } | null },
  request: { mode: "new" | "selected" | "single", ids: [ … ] },
  auth: { scheme: "apiKey", values: { apiKey: "…" } } | null,
  session: { token, expiresAt, data } | null,
  settings: { server: "cn", fullWidth: true, proxy: "" }   // cài đặt chung đè bởi cài đặt của truyện
}
```

`story.current` là chương mới nhất đã có trong `raw/`. `id` là mã trong mục lục nếu sổ chương biết, không
thì `null`. `title` là dòng đầu file raw.

## 6. API app cấp cho extension

| API | Nội dung |
|---|---|
| `http.request({ method, url, headers, query, body, timeout, encoding, cookies })`, `http.get`, `http.post` | Trả `{ status, url, headers, text(), json(), bytes() }`. `encoding` ép mã hoá trang (`gbk`, `big5`…); bỏ trống thì đoán theo header rồi nội dung. `cookies` (mặc định `true`) dùng hũ cookie riêng của extension, lưu trong session data. |
| `html.parse(text)` | Tài liệu có `select(css)`, `selectFirst(css)`; phần tử có `text()`, `html()`, `outerHtml()`, `attr(name)`, `children()`. |
| `crypto` | `md5`, `sha1`, `sha256`, `hmac(alg, key, data)`, `aes.encrypt` / `aes.decrypt` (cbc, ecb, gcm), `randomBytes`. |
| `encoding` | `decode(bytes, label)`, `encode(text, label)`, `base64`, `hex`. |
| `cache` | `get(key)`, `set(key, value, { ttl })`, `delete(key)`. Mục 7. |
| `session` | `get(key)`, `set(key, value, { ttl })`, `delete(key)`, `clear()`. Mục 7. |
| `webview.open(options)` | Mục 8. Cần quyền `webview`. |
| `log.info` / `log.warn` / `log.error` | Ghi vào log của trang Tải raw. |
| `progress(done, total, message?)` | Báo tiến độ trong `toc` dài (mục lục nhiều trang). |
| `sleep(ms)` | Chờ. |

`ttl` tính bằng giây; bỏ trống thì giữ tới khi bị xoá. Giá trị là chuỗi hoặc dữ liệu JSON. Mọi API có I/O
trả Promise.

## 7. Lưu trữ phía app

Mọi thứ của extension nằm trong thư mục cấu hình của app (`<config>`), không nằm trong thư mục truyện.

| Dữ liệu | Chỗ lưu | Ghi chú |
|---|---|---|
| Danh sách đã cài | `<config>/extensions/installed.json` | id, version, nguồn cài (file / link / thư mục dev), sha256, quyền đã cấp, bật/tắt, ngày cài |
| Gói | `<config>/extensions/<id>/pkg/` | giải nén, chặn đường dẫn thoát thư mục (zip slip), chỉ file thường |
| Cài đặt chung | `<config>/extensions/<id>/settings.json` | trừ trường `secret` |
| Auth và cài đặt `secret` | Windows Credential Manager (thư viện `keyring`), khoá `qt-ext/<id>/<field>` | máy không có kho khoá thì ghi file trong `<config>/extension-data/<id>/` và hiện cảnh báo |
| Session data | `<config>/extension-data/<id>/session.json` | gắn với tài khoản: mất khi hết `ttl`, khi đổi hoặc xoá auth, khi gọi `session.clear()`, khi bấm "Đăng xuất". Tối đa 1 MB. Phiên `login` và hũ cookie nằm ở đây. |
| Cache | `<config>/extension-data/<id>/cache.sqlite` | tối đa 50 MB; đầy thì xoá mục ít dùng nhất. Xoá bằng nút "Xoá cache" hoặc khi gỡ extension. |
| Hồ sơ webview | `<config>/extension-data/<id>/webview/` | cookie giữ giữa các lần mở; "Đăng xuất" xoá |

Phiên đăng nhập: trước mỗi lượt, nếu có `login` mà phiên thiếu hoặc quá `expiresAt` thì app gọi `login`
trước. Lỗi `AuthExpired` xử lý như mục 5.

## 8. Webview

```js
const result = await webview.open({
  url: "https://fanqienovel.com/login",
  title: "Đăng nhập Fanqie",
  visible: true,
  until: { url: "^https://fanqienovel\\.com/main", cookie: "sessionid", selector: ".user-avatar" },
  timeout: 300,
  collect: { cookies: true, localStorage: ["token"], sessionStorage: false, html: true }
});
// { url, cookies: [{ name, value, domain, path, expires }], localStorage: { token: "…" },
//   sessionStorage: {}, html: "…", closedBy: "until" | "user" | "timeout" }
```

- Cửa sổ hiện: người dùng đăng nhập, giải captcha… Cửa sổ đóng khi khớp một điều kiện của `until`, khi
  người dùng bấm "Xong" hoặc đóng, hoặc khi hết `timeout` (tối đa 900 giây). Mỗi extension một cửa sổ hiện
  cùng lúc.
- Cửa sổ ẩn (`visible: false`): cho trang cần chạy JS mới ra nội dung. Đóng theo `until` hoặc `timeout`.
  Mỗi extension tối đa 2 cửa sổ ẩn cùng lúc.
- `url` và mọi thứ `collect` trả về chỉ thuộc `hosts`. Cookie của domain khác không trả cho extension.
- Cửa sổ không có quyền gọi lệnh của app: nhãn `ext-<id>-<n>`, không thuộc capability nào.
- Thanh tiêu đề luôn ghi tên extension và domain đang mở.
- Mỗi extension một hồ sơ trình duyệt riêng (mục 7).
- Extension không chèn được script vào trang. `collect` do app tự lấy bằng script cố định của app. Lý do:
  script chèn vào chạy bằng quyền của trang, gửi được cookie đi bất kỳ đâu, vượt giới hạn `hosts`. Nếu sau
  này gặp nguồn bắt buộc phải chạy script trong trang thì mở một quyền riêng có cảnh báo lúc cài.
- Nơi không có webview (CLI về sau) thì `webview.open` ném `NeedsInteraction`.

## 9. Sandbox và bảo mật

Crate mới `crates/qt-ext`: QuickJS qua `rquickjs` (MIT) cộng các API ở mục 6. App gọi crate này; CLI dùng
được về sau. Core không phụ thuộc QuickJS.

| Giới hạn | Mức |
|---|---|
| RAM mỗi lượt | 256 MB |
| Stack | 1 MB |
| JS chạy liên tục không chờ I/O | 30 giây thì ngắt |
| `toc` / `chapter` / `login` | 5 phút / 2 phút / 2 phút |
| Phản hồi `http` | 20 MB |
| Huỷ | ngắt ngay qua interrupt handler |

- Mỗi lượt tải một môi trường JS mới. Trong lượt, app gọi tối đa `concurrency` lệnh `chapter` song song
  trên cùng môi trường, nên biến JS dùng chung trong cả lượt.
- Chỉ `import` file trong gói. Không `require`, không mô-đun hệ thống, không đọc hay ghi file, không chạy
  chương trình, không đọc dữ liệu của extension khác, không gọi lệnh của app.
- Mạng (cả `http` lẫn webview): chỉ domain trong `hosts`, kiểm lại ở từng bước redirect. Chặn IP nội bộ,
  loopback, link-local và các đuôi tên miền nội bộ (dùng lại danh sách trong `source_http.rs`). `http://`
  được phép với domain đã khai.
- Mỗi domain: `concurrency` và `delayMs` theo manifest, app chặn trần 8 luồng. Phản hồi 429 thì chờ theo
  `Retry-After` (tối đa 120 giây) rồi thử lại.
- Khoá: extension chỉ đọc được auth của chính nó. Log và thông báo lỗi che mọi giá trị `secret` (auth và
  cài đặt) thành `***` trước khi hiện.
- Quyền lúc cài: hộp thoại hiện `hosts`, quyền `webview`, auth bắt buộc hay không; người dùng đồng ý mới
  cài. Bản cập nhật thêm domain hoặc quyền thì hỏi lại; chưa đồng ý thì giữ bản cũ.
- sha256 của gói ghi lại lúc cài; phần 2 so với mã kho công bố.

## 10. Luồng tải

### 10.1 Trang Tải raw

Trang mới trên thanh bên, giữa "Hồ sơ truyện" và "Kiểm tra VN".

1. Đầu trang: link tải raw (sửa tại chỗ), extension (tự chọn, hoặc danh sách khi nhiều cái khớp), trạng
   thái auth kèm nút "Đăng nhập", các trường cài đặt `scope: "story"`, mẫu tên file.
2. Nút "Tải mục lục" gọi `toc`. Mỗi dòng: vị trí, tiêu đề, trạng thái (Đã có / Chưa có / Khoá / Lỗi /
   Ngắn), ô tick, nút tải lẻ (chương đã có thì là "Tải lại").
3. "Tải chương mới" (`mode: "new"`, từ sau chương hiện tại tới hết), "Tải đã chọn" (`mode: "selected"`),
   tải lẻ (`mode: "single"`). Thanh tiến độ, log, nút Huỷ.

### 10.2 Khớp chương đã có

- Có trong sổ `chapters`: khớp theo mã.
- Truyện chưa có sổ: file raw thứ N (thứ tự tự nhiên) ứng với mục N của mục lục.
- Kiểm tiêu đề: so dòng đầu file raw với tiêu đề mục lục ở chương cuối đã có và 5 vị trí rải đều. Hai bên
  đều bỏ khoảng trắng, dấu câu và chuyển phồn sang giản (`zhconv`) trước khi so. Lệch thì hiện các cặp lệch
  và ô "chương cuối đã có là mục số …"; người dùng chỉnh rồi mới tải được.
- Sau lượt tải đầu, sổ ghi cả các chương đã có theo cách khớp đã chốt, nên lần sau khớp theo mã.

### 10.3 Mẫu tên file

- Token: `{chuong}` (vị trí trong mục lục, từ 1), `{chuong:N}` (đệm số 0 cho đủ N chữ số, dài hơn thì giữ
  nguyên), `{chapter-id}`, `{chapterTitle}`, `{volume}` (rỗng nếu mục lục không có). Đuôi luôn `.txt`.
- Mặc định `{chuong:4}_{chapter-id}`.
- Bắt buộc có `{chuong}` hoặc `{chuong:N}`; thiếu thì không lưu được mẫu.
- Bỏ ký tự cấm trên Windows (`\/:*?"<>|` và ký tự điều khiển), cắt khoảng trắng và dấu chấm cuối, tên tối
  đa 120 ký tự, trùng tên thì thêm `-2`, `-3`…
- Truyện cũ: đoán mẫu từ tên file đang có.
  - Toàn `chuong-\d+` → `chuong-{chuong:W}`.
  - Toàn `\d+_\d+` → `{chuong:W}_{chapter-id}`.
  - Toàn `<cùng tiền tố>_\d+_…` → `<tiền tố>_{chuong:W}_{chapterTitle}`.
  - `W` là số chữ số đang dùng. Không đoán được thì dùng mẫu chung.
- Trước khi tải, app sinh tên cho chương kế tiếp và báo nếu nó bị sắp lên trước file cuối đã có.

### 10.4 Ghi nội dung

- `format: "html"`: `<p>`, `<br>`, `<div>` thành dòng, bỏ thẻ, giải mã thực thể HTML.
- Chuẩn hoá: bỏ khoảng trắng đầu/cuối dòng, bỏ dòng trống thừa. File ghi theo dạng tiêu đề, một dòng
  trống, rồi các đoạn. Nội dung chưa mở đầu bằng tiêu đề thì app chèn tiêu đề (`title` trả về, không có thì
  tiêu đề mục lục).
- **Chỉ nội dung trống mới là lỗi:** không còn chữ nào sau khi bỏ dòng tiêu đề. Chương ngắn vẫn ghi, chỉ
  gắn nhãn "Ngắn" (dưới 50 ký tự) trong mục lục.
- Mỗi chương ghi ngay khi tải xong bằng ghi an toàn (file tạm rồi đổi tên), rồi cập nhật sổ.
- Tải lại chương đã có: chép bản cũ vào `work/raw-backup/<id>.<thời điểm>.txt` trước khi ghi đè. Chương đó
  đã dịch thì báo bản dịch có thể lệch raw mới; trạng thái dịch giữ nguyên.
- Cuối lượt (cả khi huỷ) app đồng bộ `state.json` như `init`, nên chương mới hiện ở trang Dịch ở trạng thái
  chờ dịch. Việc này phải đi qua cơ chế khoá truyện đang dùng cho phiên dịch, để không ghi đè `state.json`
  khi phiên dịch đang chạy (xác minh lúc lập kế hoạch).

### 10.5 Lỗi và tải tiếp

| Tình huống | Xử lý |
|---|---|
| Lỗi mạng, 5xx, `ExtensionError` có `retry` | thử lại 2 lần, cách 2 rồi 6 giây; vẫn lỗi thì đánh dấu Lỗi, tải tiếp chương khác |
| 429 | chờ theo `Retry-After`, không tính vào số lần thử |
| `Locked` hoặc mục lục `locked: true` | đánh dấu Khoá, không ghi file; "Tải chương mới" bỏ qua và đi tiếp |
| Nội dung trống | đánh dấu Lỗi, không ghi file |
| `AuthExpired` | mục 5 |
| Huỷ, tắt app | chương đã ghi giữ nguyên; lần sau chỉ tải phần còn thiếu |
| Cuối lượt | tổng kết: số chương đã tải, Khoá, Lỗi (kèm lý do), Ngắn |

## 11. Cài và quản lý extension

- Cài từ file zip (chọn file hoặc kéo thả) hoặc từ link zip https. Nguồn cài từ link được nhớ để "Kiểm tra
  cập nhật": tải lại zip, bản có `version` lớn hơn thì cài theo luật quyền ở mục 9.
- Kiểm manifest theo bảng mục 4, rồi hộp xin quyền, rồi giải nén.
- Cài đặt → Extension: mỗi extension một dòng gồm icon, tên, phiên bản, nhà phát triển (tên, link, email),
  giấy phép, `hosts`, quyền; nút Bật/Tắt, Kiểm tra cập nhật, Auth, Cài đặt, Xoá cache, Gỡ.
- Form Auth: chọn kiểu theo thứ tự ưu tiên của manifest, nhập trường, nút "Đăng nhập" (gọi `login` nếu có)
  và "Đăng xuất". Form Cài đặt: các trường chung; trường `scope: "story"` sửa ở trang Tải raw.
- Gỡ: xoá gói, cài đặt, auth, session data, cache, hồ sơ webview. `download.json` của truyện giữ nguyên.

### Chế độ developer

- "Nạp thư mục": chạy extension từ thư mục đang viết, tự nạp lại khi file đổi, nhãn "Dev", log chi tiết.
- "Chạy thử": nhập link, chạy `toc`, chọn một chương chạy `chapter`; hiện kết quả thô, thời gian, từng
  request (method, url, status, ms). Không ghi gì vào truyện.
- Chỉ extension nạp ở chế độ này mới được khai `localhost` / `127.0.0.1` trong `hosts`.

## 12. Tài liệu cho developer và kiểm thử

Tài liệu ở `docs/extensions/`:
- Hướng dẫn viết extension và tham chiếu API, tiếng Việt.
- `qt-ext.d.ts` cho gợi ý code.
- `manifest.schema.json` để kiểm manifest trong VS Code.
- 2 extension mẫu ở `docs/extensions/examples/`: trang HTML thường (mục lục nhiều trang, GBK) và nguồn có
  API key, `login`, session data, cache.

Kiểm thử:
- `qt-ext`: vòng lặp vô hạn bị ngắt, vượt RAM bị dừng, không có API file. Domain ngoài `hosts`, redirect
  sang domain ngoài, IP nội bộ đều bị chặn. Che khoá trong log và lỗi. Cache theo `ttl` và quota, xoá mục ít
  dùng nhất. Session mất khi đổi auth. Manifest sai bị từ chối. Zip slip bị chặn.
- Chạy trọn vòng với máy chủ giả trên máy: trang GBK, mục lục nhiều trang, 429, redirect, chương khoá,
  chương trống, chương ngắn, `AuthExpired`. Kiểm: tên file đúng mẫu, sổ `download.json` đúng, huỷ rồi tải
  tiếp chỉ lấy phần thiếu, khớp truyện cũ theo thứ tự và báo lệch tiêu đề, backup khi tải lại, `state.json`
  có chương mới.
- GUI (vitest): trang Tải raw, kiểm mẫu tên file, đoán mẫu từ tên cũ, hộp xin quyền, form auth và cài đặt
  vẽ từ manifest.
- Webview: kiểm tay theo danh sách (đăng nhập thật, `until`, `collect`, cửa sổ không gọi được lệnh app,
  cookie domain ngoài bị lọc).

## 13. Ngoài phạm vi phần 1

- Kho lưu trữ (phần 2) và marketplace (phần 3).
- Lệnh CLI `qt-ai raw`.
- Chạy extension theo định dạng của app đọc truyện khác (`plugin.json`, `toc.js`, `chap.js`).
- Chèn script vào trang trong webview.
- Hàm `info` lấy tên, tác giả, bìa để điền hồ sơ truyện.
- Tải raw tự động theo lịch.

## 14. Thứ tự làm

Mỗi mốc test xong mới sang mốc sau.

1. `qt-ext`: sandbox, giới hạn, `http`, `html`, `crypto`, `encoding`, `log`, `sleep`; chạy `toc` và
   `chapter` của extension mẫu với máy chủ giả.
2. `download.json`, ô Link tải raw, mẫu tên file, khớp chương, ghi file, đồng bộ `state.json`, trang Tải raw.
3. Auth (kho khoá), cài đặt, cache, session data, `login` và `AuthExpired`.
4. Webview.
5. Cài từ file và link, trang Cài đặt → Extension, chế độ developer, tài liệu và 2 extension mẫu.

Rủi ro cần xác minh sớm:
- `rquickjs` với Promise và hàm host chạy song song trên Windows (mốc 1).
- Ghi `state.json` khi phiên dịch đang chạy (mốc 2).
- Hồ sơ WebView2 riêng cho từng cửa sổ trong Tauri 2 (`data_directory`) (mốc 4).
