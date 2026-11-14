# Viết extension tải raw

Extension cho trang **Tải raw** của VNCVT AI Translator biết cách lấy mục lục và nội dung chương từ một trang
nguồn. App lo phần còn lại: chọn chương cần tải, đặt tên file, ghi `raw/`, thử lại khi lỗi mạng, đưa chương vào
hàng đợi dịch.

Extension là JavaScript (ES module) chạy trong sandbox QuickJS của app. Nó không đọc ghi được file, không chạy được
chương trình, và chỉ gọi mạng tới các domain đã khai trong `manifest.json`.

- [Gói extension](#gói-extension)
- [manifest.json](#manifestjson)
- [Hàm extension export](#hàm-extension-export)
- [`ctx`](#ctx)
- [API app cấp](#api-app-cấp)
- [Lỗi](#lỗi)
- [Giới hạn](#giới-hạn)
- [Chạy thử khi đang viết](#chạy-thử-khi-đang-viết)
- [Đóng gói và phát hành](#đóng-gói-và-phát-hành)
- [Kho extension](#kho-extension)

Gợi ý code trong VS Code: chép [`qt-ext.d.ts`](qt-ext.d.ts) và [`jsconfig.json`](jsconfig.json) vào thư mục
extension, sửa `"include"` của jsconfig thành `["qt-ext.d.ts", "**/*.js"]`. jsconfig bỏ thư viện kiểu của trình
duyệt vì `crypto` của extension trùng tên `crypto` của trình duyệt (extension không có `fetch`, `document`,
`window`). Manifest có `"$schema"` trỏ tới [`manifest.schema.json`](manifest.schema.json) thì VS Code kiểm từng
trường khi gõ.

Hai extension mẫu chạy được (app có test chạy chúng với máy chủ giả):
- [`examples/html-gbk`](examples/html-gbk): trang HTML thường, mục lục nhiều trang, mã hoá GBK, chương VIP.
- [`examples/api-key`](examples/api-key): API JSON, API key, `login` lấy token có hạn, `AuthExpired`, session data,
  cache.

## Gói extension

```
manifest.json
main.js          điểm vào, ES module
lib/…            tuỳ chọn; main.js import bằng đường dẫn tương đối: import { x } from "./lib/parse.js"
icon.png         tuỳ chọn, hiện trong danh sách extension
```

Tối đa 200 file, 5 MB. Chỉ `import` được file trong gói (`./`, `../` không ra ngoài gói); không có `require`, không
có module hệ thống, không có npm. Thư mục bắt đầu bằng dấu chấm và `node_modules` bị bỏ qua.

## manifest.json

```json
{
  "$schema": "https://…/manifest.schema.json",
  "id": "io.github.ten-ban.ten-nguon",
  "name": "Tên nguồn",
  "version": "1.2.0",
  "apiVersion": 1,
  "description": "Tải raw từ ten-nguon.com",
  "developer": { "name": "ten-ban", "url": "https://github.com/ten-ban", "email": "" },
  "license": "MIT",
  "homepage": "https://github.com/ten-ban/qt-ext-ten-nguon",
  "match": ["^https://(www\\.)?ten-nguon\\.com/book/\\d+"],
  "hosts": ["ten-nguon.com", "*.cdn-ten-nguon.com"],
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
    { "key": "proxy", "label": "Proxy riêng", "type": "text", "scope": "story" }
  ],
  "limits": { "concurrency": 3, "delayMs": 300 }
}
```

| Trường | Bắt buộc | Ghi chú |
|---|---|---|
| `id` | có | tên miền đảo ngược, chữ thường: `^[a-z0-9]+(\.[a-z0-9-]+)+$`. Không đổi giữa các bản. |
| `name`, `description` | có | |
| `version` | có | semver (`1.2.0`). Bản cập nhật phải có `version` lớn hơn. |
| `apiVersion` | có | hiện là `1`. App cũ hơn API extension cần thì báo người dùng cập nhật app. |
| `developer.name` | có | `url`, `email` tuỳ chọn; hiện ở danh sách extension và hộp xin quyền. |
| `license`, `homepage` | không | |
| `match` | có | regex (cú pháp Rust `regex`, gần như JS) trên link tải raw của truyện. Link khớp thì app chọn extension này. |
| `hosts` | có | domain extension được gọi tới (cả `http` lẫn webview, kiểm ở mọi bước redirect). `*.` ở đầu = mọi subdomain, không gồm chính tên gốc. Không IP, không cổng. `localhost` / `127.0.0.1` chỉ được khi nạp thư mục ở chế độ developer. |
| `permissions` | không | hiện chỉ có `"webview"`. |
| `auth` | không | các kiểu đăng nhập, xếp theo thứ tự ưu tiên (thường API key trước). Trường `type` là `text` hoặc `secret`. |
| `account` | không | tài khoản trang nguồn đăng nhập qua `login` (thường mở webview), khác `auth`: `{ "title": "Tài khoản QQ阅读", "required": false, "help": "…" }`. Xem [Tài khoản trang nguồn](#tài-khoản-trang-nguồn). |
| `settings` | không | trường cài đặt người dùng điền. `type`: `text`, `number`, `boolean`, `select` (cần `choices`), `secret`. `scope: "story"` = mỗi truyện một giá trị (sửa ở trang Tải raw); `secret` không được `scope: "story"`. `default` phải đúng kiểu. |
| `limits` | không | `concurrency` (mặc định 1, trần 8): số chương tải song song, cũng là số request song song tới mỗi domain. `delayMs` (mặc định 0): giãn cách giữa hai request tới cùng domain. |

Người dùng thấy `hosts`, quyền `webview` và auth bắt buộc trong hộp xin quyền trước khi cài. Bản cập nhật thêm domain
hoặc quyền thì app hỏi lại; người dùng chưa đồng ý thì giữ bản cũ.

## Hàm extension export

```js
export async function toc(ctx) { … }            // bắt buộc
export async function chapter(ctx, ch) { … }    // bắt buộc
export async function chapters(ctx, list) { … } // tuỳ chọn, tải gộp
export async function login(ctx) { … }          // tuỳ chọn
export async function account(ctx) { … }        // tuỳ chọn, kiểm tài khoản trang nguồn
```

**`toc(ctx)`** trả mục lục theo đúng thứ tự đọc:

```js
[{ id: "7445626961335960126", title: "第1章 …", locked: false, volume: "卷一", data: { href: "/c/1" } }]
```

`id` là chuỗi (số cũng được), không trùng; đây là mã app ghi vào sổ chương `download.json` của truyện, nên giữ ổn
định giữa các lần tải. `locked: true` = chương khoá (app không tải khi bấm "Tải chương mới"). `volume` tuỳ chọn.
`data` là dữ liệu riêng của extension, app trả lại nguyên vẹn khi gọi `chapter`. Mục lục dài nhiều trang thì gọi
`progress()` sau mỗi trang.

**`chapter(ctx, ch)`**: `ch` là một mục của mục lục cộng `index` (vị trí, từ 1). Trả:

```js
{ content: "…", title: "第1章 …", format: "text" }   // format: "text" (mặc định) hoặc "html"
```

`format: "html"`: app đổi `<p>`, `<br>`, `<div>` thành dòng, bỏ thẻ, giải mã thực thể. App tự chuẩn hoá khoảng
trắng, chèn tiêu đề nếu nội dung chưa mở đầu bằng tiêu đề, rồi ghi file: tiêu đề, một dòng trống, các đoạn. Chỉ nội
dung trống mới là lỗi; chương ngắn (xin nghỉ) vẫn ghi, gắn nhãn "Ngắn".

**`chapters(ctx, list)`** (tuỳ chọn): nguồn có API tải nhiều chương một lần. App gọi với tối đa 20 mục, trả
`[{ id, content, title?, format?, error? }]`; mục có `error` (chuỗi) coi như lỗi tạm, app thử lại. Có hàm này thì
app không gọi `chapter`.

**`login(ctx)`** (tuỳ chọn): trả `{ token?, expiresAt?, data? }`. `expiresAt` là mốc hết hạn, ví dụ
`Date.now() + 3600 * 1000` (số nhỏ hơn 1e11 coi là giây). App lưu kết quả và đưa lại qua `ctx.session`. App gọi
`login` trước lượt tải khi chưa có phiên hoặc phiên còn dưới 30 giây, và khi người dùng bấm **Đăng nhập**.
`login` ném lỗi thì lượt tải báo "đăng nhập không được: …". Đăng nhập chỉ cần cookie (server đặt cookie) thì trả
`{}`: cookie đã vào hũ cookie của `http`. Extension có khối `account` trả thêm `account: "tên tài khoản"` khi đăng
nhập được, hoặc `guest: true` cho phiên khách (chưa đăng nhập, vẫn tải được phần miễn phí).

**`account(ctx)`** (tuỳ chọn, cho extension có khối `account`): kiểm tài khoản trang nguồn còn đăng nhập không, trả
`{ loggedIn, name?, message? }`. App gọi khi người dùng mở trang Tải raw (nhớ kết quả 10 phút) và khi bấm
**Kiểm tra**; `ctx.request.mode` là `"account"`, `ctx.session` là phiên đã lưu (hoặc `null`). Hàm này **không được mở
webview** — chỉ hỏi máy chủ phiên còn sống không. Báo `loggedIn: false` (hoặc ném `AuthExpired`) khi phiên không
phải phiên khách thì app xoá phiên, lượt tải sau gọi lại `login`. Không có hàm này thì app suy từ phiên đã lưu.

Mọi hàm có thể là `async`. Biến toàn cục của module dùng chung trong cả lượt (một lượt = một lần bấm Tải mục lục
hoặc Tải chương), các chương chạy song song trên cùng môi trường JS.

### Tài khoản trang nguồn

Nguồn cần đăng nhập tài khoản của chính trang nguồn (QQ阅读, Qidian…) qua cửa sổ webview thì khai thêm khối `account`
— khác `auth` (API key, mật khẩu người dùng gõ vào app):

```json
"account": {
  "title": "Tài khoản QQ阅读",
  "required": false,
  "help": "Bấm Đăng nhập: cửa sổ book.qq.com mở ra, đăng nhập bằng QQ hoặc WeChat. Chưa đăng nhập vẫn tải được chương miễn phí."
}
```

Trang Tải raw hiện một dòng riêng: "Tài khoản QQ阅读: đã đăng nhập: <tên> / chưa đăng nhập — <message>", `help`, nút
**Đăng nhập Tài khoản QQ阅读** (gọi `login`, `ctx.request.mode = "login"`), **Kiểm tra** (gọi `account`), **Đăng xuất**.
`required: true` mà chưa đăng nhập (hoặc đang phiên khách) thì icon Tải raw có chấm cảnh báo và app không tự tải mục
lục nền khi mở truyện (tránh bật cửa sổ đăng nhập bất ngờ).

Mẫu: `login` ngoài nút Đăng nhập mà chưa từng đăng nhập thì trả phiên khách, không mở cửa sổ:

```js
export async function login(ctx) {
  if (ctx.request.mode !== "login" && !(await session.get("account"))) return { guest: true };
  const r = await webview.open({ url: "https://nguon.com/login", until: { cookie: "sid" }, collect: { cookies: true } });
  const sid = r.cookies.find((c) => c.name === "sid");
  if (!sid) throw new ExtensionError("chưa đăng nhập xong");
  const me = (await http.get("https://nguon.com/api/me")).json();
  await session.set("account", me.name);
  return { token: sid.value, account: me.name };
}

export async function account(ctx) {
  if (!ctx.session || ctx.session.guest) return { loggedIn: false, message: "đang tải kiểu khách — chỉ chương miễn phí" };
  const me = await http.get("https://nguon.com/api/me");
  return me.ok ? { loggedIn: true, name: me.json().name } : { loggedIn: false, message: "phiên đã hết hạn — bấm Đăng nhập" };
}
```

## `ctx`

`ctx` chỉ đọc (đã đóng băng).

```js
ctx = {
  sourceUrl: "https://…",      // link nguồn của truyện
  rawUrl: "https://…",         // link tải raw (mặc định bằng link nguồn)
  story: { name: "…", chapterCount: 12, current: { index: 12, id: "…" | null, title: "…" } | null },
  request: { mode: "toc" | "new" | "selected" | "single" | "login" | "account" | "test", ids: ["…"] },
  auth: { scheme: "apiKey", values: { apiKey: "…" } } | null,
  session: { token: "…", expiresAt: 1700000000000, data: …, account: "tên" | null, guest: false } | null,
  settings: { server: "cn", proxy: "" }   // mặc định → cài đặt chung → cài đặt của truyện
}
```

`story.current` là chương mới nhất đã có trong `raw/` của truyện. Khi người dùng Chạy thử (chế độ developer),
`story.name` rỗng và `request.mode` là `"test"`.

## API app cấp

Mọi API có I/O trả Promise. Các API là biến toàn cục, không ghi đè được.

### `http`

```js
const res = await http.request({
  method: "POST", url: "https://…", headers: { Referer: "…" }, query: { page: 2 },
  body: { a: 1 },          // object → JSON (tự thêm Content-Type), chuỗi, hoặc Uint8Array
  timeout: 30000,          // ms, mặc định 30 giây
  encoding: "gbk",         // ép mã hoá khi đọc text(); bỏ trống thì đoán theo header rồi nội dung
  cookies: true,           // false: không gửi, không nhận cookie của hũ
});
res.status; res.ok; res.url /* URL cuối sau redirect */; res.headers["content-type"];
res.text(); res.text("big5"); res.json(); res.bytes() /* Uint8Array */;
await http.get(url, { headers }); await http.post(url, body, { headers });
```

Redirect tự đi, mỗi bước kiểm `hosts`. Phản hồi tối đa 20 MB. HTTP 429 thì app chờ theo `Retry-After` (tối đa 120
giây) rồi gửi lại, tối đa 3 lần. Mỗi extension một hũ cookie riêng, giữ giữa các lượt tải (trong session data).

### `html`

```js
const doc = html.parse(text);
doc.select("#list a");            // mảng phần tử
doc.selectFirst("h1");            // phần tử hoặc null
el.text(); el.html(); el.outerHtml(); el.attr("href") /* hoặc null */; el.children(); el.select("…");
```

### `crypto`, `encoding`

```js
crypto.md5(data); crypto.sha1(data); crypto.sha256(data); crypto.sha512(data);   // hex
crypto.hmac("sha256", key, data);                                               // hex
crypto.aes.decrypt({ mode: "cbc" | "ecb" | "gcm", key, iv, data, padding: "pkcs7" | "none" }); // Uint8Array
crypto.aes.encrypt({ … });
crypto.randomBytes(16);                                                         // Uint8Array
encoding.decode(bytes, "gbk"); encoding.encode(text, "gbk");
encoding.utf8.decode(bytes); encoding.utf8.encode(text);
encoding.base64.encode(data, { urlSafe: false }); encoding.base64.decode(text);
encoding.hex.encode(data); encoding.hex.decode(text);
```

`data`, `key`, `iv` là chuỗi (UTF-8) hoặc `Uint8Array`.

### `cache`

```js
await cache.set("toc:123", items, { ttl: 600 });   // ttl giây; bỏ trống = giữ tới khi bị xoá
await cache.get("toc:123");                        // giá trị hoặc null
await cache.delete("toc:123");
```

Giá trị là chuỗi hoặc dữ liệu JSON. Mỗi extension tối đa 50 MB, đầy thì app xoá mục dùng lâu nhất; một mục tối đa
5 MB. Người dùng xoá bằng nút "Xoá cache". Dùng cho dữ liệu lấy lại được (mục lục, trang đã giải mã).

### `session`

```js
await session.set("deviceId", "…", { ttl: 86400 });
await session.get("deviceId"); await session.delete("deviceId"); await session.clear();
```

Session data gắn với tài khoản: mất khi hết `ttl`, khi người dùng đổi hoặc xoá auth, khi `session.clear()`, khi bấm
"Đăng xuất". Tối đa 1 MB. Dùng cho khoá phiên, mã thiết bị, dữ liệu theo tài khoản.

### `webview` (cần quyền `"webview"`)

```js
const r = await webview.open({
  url: "https://ten-nguon.com/login",
  title: "Đăng nhập",
  visible: true,                      // false: cửa sổ ẩn, cho trang cần chạy JS mới ra nội dung
  until: { url: "^https://ten-nguon\\.com/user", cookie: "sessionid", selector: ".avatar" },
  timeout: 300,                       // giây, tối đa 900
  collect: { cookies: true, localStorage: ["token"], sessionStorage: false, html: false },
});
// { url, cookies: [{ name, value, domain, path, expires }], localStorage: { token: "…" },
//   sessionStorage: {}, html: "…" | null, closedBy: "until" | "user" | "timeout" | "load" }
```

- Cửa sổ hiện: người dùng đăng nhập, giải captcha. Đóng khi khớp một điều kiện của `until`, khi người dùng đóng
  cửa sổ (coi như xong, app vẫn lấy dữ liệu), hoặc hết `timeout`. Thời gian chờ người dùng không tính vào giới hạn
  của `login` / `toc` / `chapter`.
- Cửa sổ ẩn: đóng theo `until` hoặc `timeout`; không có `until` thì đóng khi trang tải xong (`closedBy: "load"`).
- Mỗi extension 1 cửa sổ hiện và 2 cửa sổ ẩn cùng lúc; quá thì chờ.
- Cửa sổ chỉ mở được trang thuộc `hosts`; link ra domain khác không đi, popup bị chặn. `url` và mọi thứ `collect`
  trả về chỉ thuộc `hosts` (cookie domain khác bị lọc; trang cuối ngoài `hosts` thì `url` là `null`, không có
  storage, HTML).
- Cookie lấy được tự nạp vào hũ cookie của `http`, nên request sau đó gửi kèm luôn.
- Extension không chèn được script vào trang. `collect` do app lấy bằng script của app.
- Mỗi extension một hồ sơ trình duyệt riêng, cookie giữ giữa các lần mở; "Đăng xuất" xoá hồ sơ.
- Nơi không có cửa sổ (dòng lệnh về sau) thì `webview.open` ném `NeedsInteraction`.

### `log`, `progress`, `sleep`

```js
log.info("…"); log.warn("…"); log.error("…");   // hiện ở log trang Tải raw; console.log cũng được
progress(done, total, "trang 3");              // total = 0 khi chưa biết tổng
await sleep(500);
```

Log và thông báo lỗi tự che mọi giá trị bí mật (trường `secret` của auth và cài đặt, token của `login`) thành
`***`. Vẫn đừng log khoá.

## Lỗi

```js
throw new AuthExpired("token hết hạn");             // app xoá phiên, gọi login, chạy lại đúng lệnh đó một lần
throw new Locked("chương VIP");                     // đánh dấu Khoá, không ghi file
throw new NeedsInteraction("cần giải captcha");     // báo người dùng mở app để thao tác
throw new ExtensionError("trang lỗi", { retry: true });  // retry: thử lại như lỗi mạng (2 lần, cách 2 rồi 6 giây)
```

Lỗi khác (`Error`, `TypeError`…) và lỗi mạng: lỗi mạng, hết giờ được thử lại; còn lại đánh dấu Lỗi kèm thông báo.
`AuthExpired` lần hai liền, hoặc extension không có `login`, thì thành lỗi.

## Giới hạn

| | Mức |
|---|---|
| RAM mỗi lượt | 256 MB |
| JS chạy liên tục không chờ I/O | 30 giây thì bị ngắt (vòng lặp vô hạn, kể cả bọc `try/catch`) |
| `toc` / `chapter` / `login` | 5 phút / 2 phút / 2 phút |
| Phản hồi `http` | 20 MB |

Người dùng bấm Huỷ thì lượt dừng ngay; kết quả của lệnh đang chạy bị bỏ, kể cả khi extension bắt lỗi huỷ.

## Chạy thử khi đang viết

Cài đặt → **Extension tải raw** → **Nạp thư mục (developer)…**, chọn thư mục đang viết. Extension hiện nhãn "Dev":

- Chạy thẳng từ thư mục, mỗi lượt tải đọc lại file nên sửa xong là lượt sau dùng bản mới.
- Được khai `localhost` / `127.0.0.1` trong `hosts` để thử với máy chủ trên máy.
- Log trang Tải raw có thêm từng request (method, url, status, ms).
- **Chạy thử…**: nhập link, chạy `toc`, chọn một mục chạy `chapter`; xem kết quả thô, nội dung sẽ ghi, thời gian,
  từng request, log. Không ghi gì vào truyện.

Muốn nạp thư mục cho extension đã cài cùng `id` thì gỡ bản đã cài trước.

## Đóng gói và phát hành

1. Nén thư mục extension thành `.zip` (nén cả thư mục hay chỉ các file bên trong đều được).
2. Đăng file zip ở chỗ có link `https://` tải thẳng, ví dụ GitHub Releases:
   `https://github.com/ten-ban/qt-ext-ten-nguon/releases/download/v1.2.0/ten-nguon.zip`.
3. Người dùng dán link vào Cài đặt → Extension tải raw → **Cài**. Cài từ link thì có nút **Kiểm tra cập nhật**:
   app tải lại đúng link đó, bản có `version` lớn hơn thì cài. Muốn người dùng luôn nhận bản mới qua một link cố
   định, dùng link `…/releases/latest/download/ten-nguon.zip`.

Bản cập nhật giữ nguyên `id`. Thêm domain vào `hosts` hoặc thêm quyền thì người dùng được hỏi lại.

## Kho extension

Có nhiều extension thì đăng một **kho**: một file `index.json` công khai qua `https://` liệt kê mọi extension.
Người dùng thêm kho một lần (Cài đặt → Extension tải raw → **Thêm kho**), rồi cài, **Cập nhật** từng cái hoặc
**Cập nhật hết** từ danh sách "Có trong kho". Link kho là link `index.json` hoặc thư mục chứa nó.

```json
{
  "formatVersion": 1,
  "id": "io.github.ten-ban.kho",
  "name": "Kho của tên bạn",
  "description": "…",
  "maintainer": { "name": "ten-ban", "url": "https://github.com/ten-ban" },
  "auth": {
    "required": true,
    "schemes": [{ "id": "apiKey", "label": "API key", "fields": [{ "key": "apiKey", "label": "API key", "type": "secret" }] }]
  },
  "extensions": [
    {
      "id": "io.github.ten-ban.ten-nguon", "name": "Tên nguồn", "version": "1.2.0", "apiVersion": 1,
      "description": "…", "developer": { "name": "ten-ban" }, "hosts": ["api.ten-nguon.com"], "permissions": [],
      "authRequired": true, "sharedAuth": true,
      "url": "extensions/io.github.ten-ban.ten-nguon/1.2.0.zip", "sha256": "…64 ký tự hex…", "size": 2408,
      "icon": "extensions/io.github.ten-ban.ten-nguon/icon.png"
    }
  ]
}
```

- `url`, `icon` tương đối với link `index.json` hoặc tuyệt đối; phải là `https://`. App tải gói, so `sha256` với
  `index.json` (lệch thì từ chối), rồi kiểm `id`, `version` trong manifest của gói khớp kho. Các trường mô tả khác
  chỉ để hiện danh sách; lúc cài app dùng manifest trong gói.
- Mục sai (thiếu trường, version không phải semver, sha256 sai dạng, link không https) bị bỏ qua, app hiện cảnh
  báo; cả file sai thì app giữ bản cũ.
- **API key dùng chung**: kho khai `auth` (cùng cú pháp manifest) và đánh dấu `sharedAuth: true` cho extension
  dùng chung. Người dùng nhập key một lần ở **API key của kho…**; mọi extension đó nhận key qua `ctx.auth` như bình
  thường. Chỉ extension cài từ chính kho đó mới nhận key chung; cài cùng `id` từ file hay thư mục dev thì dùng auth
  riêng. Kiểu auth trong manifest của extension nên giống hệt `auth` của kho, để cài lẻ từ file vẫn chạy.
- Mỗi bản là một file zip mới đặt tên theo `version`; giữ file cũ để ai đang cài vẫn cài lại được. Ghi `index.json`
  sau cùng, sau khi zip đã lên, để kho không bao giờ trỏ tới gói chưa có.
- Gỡ kho trong app xoá API key chung; extension đã cài từ kho giữ nguyên.

