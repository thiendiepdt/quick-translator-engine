// Kiểu cho extension tải raw của VNCVT AI Translator (apiVersion 1). Chép file này và jsconfig.json vào thư mục
// extension (sửa "include" thành ["qt-ext.d.ts", "**/*.js"]). jsconfig bỏ lib DOM vì `crypto` của extension trùng tên
// `crypto` của trình duyệt. Hướng dẫn: README.md cùng thư mục.

/** Giá trị JSON. */
type QtJson = string | number | boolean | null | QtJson[] | { [key: string]: QtJson };
/** Dữ liệu nhị phân hoặc chuỗi (UTF-8). */
type QtBytesLike = string | Uint8Array;

interface QtTocItem {
  /** Mã chương, không trùng, ổn định giữa các lần tải. */
  id: string | number;
  title: string;
  /** Chương khoá: "Tải chương mới" bỏ qua. */
  locked?: boolean;
  volume?: string;
  /** Dữ liệu riêng, app trả lại nguyên vẹn khi gọi `chapter`. */
  data?: QtJson;
}

interface QtChapterRef {
  id: string;
  title: string;
  locked: boolean;
  volume?: string;
  data?: QtJson;
  /** Vị trí trong mục lục, từ 1. */
  index: number;
}

interface QtChapterResult {
  content: string;
  title?: string;
  /** Mặc định "text". "html": app đổi <p>, <br>, <div> thành dòng và bỏ thẻ. */
  format?: "text" | "html";
}

interface QtBatchResult extends QtChapterResult {
  id: string | number;
  /** Có thì chương này lỗi (app thử lại). */
  error?: string;
}

interface QtLoginResult {
  token?: string;
  /** Mốc hết hạn, ms từ 1970 (Date.now() + …). Nhỏ hơn 1e11 thì coi là giây. */
  expiresAt?: number;
  data?: QtJson;
  /** Tên tài khoản hiện cho người dùng (extension có khối `account`). */
  account?: string;
  /** Phiên khách: chưa đăng nhập tài khoản trang nguồn, vẫn tải được phần không cần tài khoản. */
  guest?: boolean;
}

/** Kết quả hàm `account(ctx)`: tài khoản trang nguồn còn đăng nhập không. */
interface QtAccountStatus {
  loggedIn: boolean;
  /** Tên tài khoản để hiện. */
  name?: string;
  /** Nhắn người dùng, vd. "đang tải kiểu khách — chỉ chương miễn phí". */
  message?: string;
}

interface QtCtx {
  readonly sourceUrl: string;
  readonly rawUrl: string;
  readonly story: {
    readonly name: string;
    readonly chapterCount: number;
    readonly current: { readonly index: number; readonly id: string | null; readonly title: string } | null;
  };
  readonly request: { readonly mode: "toc" | "new" | "selected" | "single" | "login" | "account" | "test"; readonly ids: readonly string[] };
  readonly auth: { readonly scheme: string; readonly values: Readonly<Record<string, string>> } | null;
  readonly session: {
    readonly token: string | null;
    readonly expiresAt: number | null;
    readonly data: QtJson;
    readonly account: string | null;
    readonly guest: boolean;
  } | null;
  readonly settings: Readonly<Record<string, QtJson>>;
}

interface QtRequestOptions {
  method?: string;
  url: string;
  headers?: Record<string, string> | [string, string][];
  query?: Record<string, string | number> | [string, string][];
  /** object → JSON (tự thêm Content-Type), chuỗi, hoặc bytes. */
  body?: string | Uint8Array | QtJson;
  /** ms, mặc định 30000. */
  timeout?: number;
  /** Ép mã hoá khi đọc text(): "gbk", "big5"… */
  encoding?: string;
  /** false: không gửi, không nhận cookie của hũ. Mặc định true. */
  cookies?: boolean;
}

interface QtResponse {
  readonly status: number;
  readonly ok: boolean;
  /** URL cuối sau redirect. */
  readonly url: string;
  /** Tên header viết thường. */
  readonly headers: Readonly<Record<string, string>>;
  text(encoding?: string): string;
  json(): any;
  bytes(): Uint8Array;
}

interface QtHtmlNode {
  select(css: string): QtHtmlNode[];
  selectFirst(css: string): QtHtmlNode | null;
  /** Chữ, khoảng trắng gộp lại. */
  text(): string;
  html(): string;
  outerHtml(): string;
  attr(name: string): string | null;
  children(): QtHtmlNode[];
}

interface QtTtl {
  /** Giây; bỏ trống = giữ tới khi bị xoá. */
  ttl?: number;
}

interface QtWebviewOptions {
  url: string;
  title?: string;
  /** Mặc định true. false: cửa sổ ẩn. */
  visible?: boolean;
  until?: { url?: string | RegExp; cookie?: string; selector?: string };
  /** Giây, mặc định 300, tối đa 900. */
  timeout?: number;
  /** Mặc định { cookies: true }. */
  collect?: { cookies?: boolean; localStorage?: boolean | string[]; sessionStorage?: boolean | string[]; html?: boolean };
}

interface QtWebviewResult {
  /** null khi trang cuối ngoài `hosts`. */
  url: string | null;
  cookies: { name: string; value: string; domain: string; path: string; expires: number | null }[];
  localStorage: Record<string, string>;
  sessionStorage: Record<string, string>;
  html: string | null;
  closedBy: "until" | "user" | "timeout" | "load";
}

declare const http: {
  request(options: QtRequestOptions | string): Promise<QtResponse>;
  get(url: string, options?: Omit<QtRequestOptions, "url" | "method">): Promise<QtResponse>;
  post(url: string, body?: QtRequestOptions["body"], options?: Omit<QtRequestOptions, "url" | "method" | "body">): Promise<QtResponse>;
};

declare const html: { parse(text: string): QtHtmlNode };

declare const crypto: {
  md5(data: QtBytesLike): string;
  sha1(data: QtBytesLike): string;
  sha256(data: QtBytesLike): string;
  sha512(data: QtBytesLike): string;
  hmac(algorithm: "md5" | "sha1" | "sha256" | "sha512", key: QtBytesLike, data: QtBytesLike): string;
  aes: {
    encrypt(options: { mode?: "cbc" | "ecb" | "gcm"; key: QtBytesLike; iv?: QtBytesLike; data: QtBytesLike; padding?: "pkcs7" | "none" }): Uint8Array;
    decrypt(options: { mode?: "cbc" | "ecb" | "gcm"; key: QtBytesLike; iv?: QtBytesLike; data: QtBytesLike; padding?: "pkcs7" | "none" }): Uint8Array;
  };
  randomBytes(n: number): Uint8Array;
};

declare const encoding: {
  decode(bytes: Uint8Array, label?: string): string;
  encode(text: string, label?: string): Uint8Array;
  utf8: { decode(bytes: Uint8Array): string; encode(text: string): Uint8Array };
  base64: { encode(data: QtBytesLike, options?: { urlSafe?: boolean }): string; decode(text: string): Uint8Array };
  hex: { encode(data: QtBytesLike): string; decode(text: string): Uint8Array };
};

declare const cache: {
  /** Giá trị đã lưu (chuỗi hoặc JSON), hoặc null. */
  get(key: string): Promise<any>;
  set(key: string, value: QtJson, options?: QtTtl): Promise<void>;
  delete(key: string): Promise<void>;
};

declare const session: {
  /** Giá trị đã lưu (chuỗi hoặc JSON), hoặc null. */
  get(key: string): Promise<any>;
  set(key: string, value: QtJson, options?: QtTtl): Promise<void>;
  delete(key: string): Promise<void>;
  clear(): Promise<void>;
};

declare const webview: { open(options: QtWebviewOptions): Promise<QtWebviewResult> };

declare const log: { info(...args: unknown[]): void; warn(...args: unknown[]): void; error(...args: unknown[]): void };
declare function progress(done: number, total: number, message?: string): void;
declare function sleep(ms: number): Promise<void>;

/** Phiên đăng nhập hết hạn: app xoá phiên, gọi login, chạy lại lệnh một lần. */
declare class AuthExpired extends Error {}
/** Chương khoá: đánh dấu Khoá, không ghi file. */
declare class Locked extends Error {}
/** Cần người dùng mở app để thao tác. */
declare class NeedsInteraction extends Error {}
/** `retry: true` thì app thử lại như lỗi mạng. */
declare class ExtensionError extends Error {
  constructor(message: string, options?: { retry?: boolean });
  readonly retry: boolean;
}
