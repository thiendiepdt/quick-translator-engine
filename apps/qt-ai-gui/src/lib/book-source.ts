/**
 * Tác giả + ảnh bìa từ link nguồn của truyện, cho export EPUB. Dò link và bóc dữ liệu chép từ novelkit
 * (`src/features/ttc-uploader/sources`), rút gọn còn các nguồn thư viện đang dùng. Rust tải trang
 * (`source_fetch_text`, chỉ host trong danh sách) và ảnh (`source_fetch_image`, có thử lại); ở đây chỉ bóc.
 */

export type BookSourceId = "fanqie" | "qq" | "qidian" | "qimao" | "sfacg" | "jjwxc";

export const BOOK_SOURCE_LABELS: Record<BookSourceId, string> = {
  fanqie: "Fanqie",
  qq: "QQ đọc sách",
  qidian: "Qidian",
  qimao: "Qimao",
  sfacg: "SFACG",
  jjwxc: "Tấn Giang (JJWXC)",
};

export interface BookSourceRef {
  source: BookSourceId;
  bookId: string;
  fetchUrl: string;
}

export interface SourceBook {
  source: BookSourceId;
  title: string;
  author: string;
  /** Link ảnh bìa, tốt nhất trước. */
  coverUrls: string[];
}

// Web QQ dùng bid hoặc 1_000_000_000 + bid trong đường dẫn.
const QQ_OFFSET = 1_000_000_000;

const FETCH_URL: Record<BookSourceId, (id: string) => string> = {
  // www.qidian.com trả HTTP 202 kèm trang dò bot; bản mobile có dữ liệu trong HTML.
  qidian: (id) => `https://m.qidian.com/book/${id}/`,
  fanqie: (id) => `https://fanqienovel.com/page/${id}`,
  qq: (id) => `https://detailadr.reader.qq.com/book/queryDetailPage?bid=${id}&sex=1&qmk=1,3,7,9,10`,
  qimao: (id) => `https://www.qimao.com/shuku/${id}/`,
  sfacg: (id) => `https://m.sfacg.com/b/${id}/`,
  // Trang web là GB18030; API app trả cùng dữ liệu bằng JSON UTF-8.
  jjwxc: (id) => `https://app.jjwxc.net/androidapi/novelbasicinfo?novelId=${id}`,
};

const ref = (source: BookSourceId, bookId: string): BookSourceRef => ({ source, bookId, fetchUrl: FETCH_URL[source](bookId) });

const hostIs = (host: string, domain: string) => host === domain || host.endsWith(`.${domain}`);

/** Link JJWXC lưu hành dưới nhiều tên miền (www/m/my.jjwxc.net, mirror đổi tên): chỉ lấy id. */
const JJWXC_HOST = /(?:^|\.)jjwxc[a-z0-9-]*\.(?:net|com)$/;

function parseUrl(input: string): URL | null {
  try {
    const url = new URL(input.trim());
    return url.protocol === "http:" || url.protocol === "https:" ? url : null;
  } catch {
    return null;
  }
}

/** Link trang sách (hoặc trang chương có id sách) của nguồn hỗ trợ → nguồn + id; không hỗ trợ thì `null`. */
export function detectBookSource(input: string): BookSourceRef | null {
  const url = parseUrl(input);
  if (!url) return null;
  const host = url.hostname.toLowerCase();

  if (hostIs(host, "qidian.com")) {
    const m = url.pathname.match(/\/(?:book|info|chapter)\/(\d+)/);
    return m ? ref("qidian", m[1]) : null;
  }
  if (hostIs(host, "fanqienovel.com") || hostIs(host, "changdunovel.com") || hostIs(host, "fqnovel.com")) {
    const id = url.searchParams.get("book_id") ?? url.pathname.match(/\/page\/(\d+)/)?.[1];
    return id && /^\d+$/.test(id) ? ref("fanqie", id) : null;
  }
  if (hostIs(host, "qq.com")) {
    const id = url.pathname.match(/\/(\d{5,})(?=[/.]|$)/)?.[1] ?? url.searchParams.get("bid");
    if (!id || !/^\d{5,}$/.test(id)) return null;
    const n = Number(id);
    return ref("qq", String(n >= QQ_OFFSET ? n - QQ_OFFSET : n));
  }
  if (hostIs(host, "qimao.com")) {
    const m = url.pathname.match(/\/shuku\/(\d+)(?:-\d+)?(?:\/|$)/);
    return m ? ref("qimao", m[1]) : null;
  }
  if (hostIs(host, "sfacg.com")) {
    const m = url.pathname.match(/\/(?:Novel|b|i)\/(\d+)(?:\/|$)/i);
    return m ? ref("sfacg", m[1]) : null;
  }
  if (JJWXC_HOST.test(host)) {
    const param = [...url.searchParams].find(([key]) => key.toLowerCase() === "novelid")?.[1];
    const id = param ?? url.pathname.match(/\/book2\/(\d+)(?:\/|$)/)?.[1];
    return id && /^\d+$/.test(id) ? ref("jjwxc", id) : null;
  }
  // Mirror sangtacviet: /truyen/{nguồn}/{n}/{id}/ trên đủ loại tên miền — đường dẫn đã nêu sách gốc.
  const mirror = url.pathname.match(/\/truyen\/(qidian|fanqie|jjwxc|sfacg|qimao)\/\d+\/(\d+)(?:\/|$)/i);
  if (mirror) return ref(mirror[1].toLowerCase() as BookSourceId, mirror[2]);
  return null;
}

/** Chuỗi hoặc số trong dữ liệu nguồn thành chuỗi đã trim; kiểu khác (object, null…) là rỗng. */
const str = (v: unknown) => (typeof v === "string" || typeof v === "number" ? String(v).trim() : "");

function uniq(values: unknown[]): string[] {
  const out: string[] = [];
  for (const v of values) {
    const s = str(v);
    if (s && !out.includes(s)) out.push(s);
  }
  return out;
}

/** Link ảnh `//…` và `http://…` thành https (Rust chỉ tải ảnh https). */
function httpsUrl(url: string): string {
  if (url.startsWith("//")) return `https:${url}`;
  return url.replace(/^http:\/\//i, "https://");
}

/**
 * Ảnh bìa Fanqie: link trong trang có chữ ký (hết hạn) và nhỏ. Cùng đường dẫn trên p6-novel.byteimg.com
 * tải được không chữ ký, không cần referer: bản thu 640px trước, bản gốc dự phòng.
 */
export function fanqieCovers(url: string): string[] {
  const m = url.match(/^https?:\/\/[^/]+\/(.+)$/);
  const path = m?.[1].split("?")[0].split("~")[0];
  if (!path) return url ? [url] : [];
  const host = "https://p6-novel.byteimg.com";
  return [`${host}/${path}~tplv-shrink:640:0.image`, `${host}/origin/${path}`];
}

/** Ảnh bìa JJWXC dán từ Baidu (`*.bdstatic.com`) thường nặng vài MB và chậm: thử bản 600px trước. */
export function baiduCovers(url: string): string[] {
  let host: string;
  try {
    host = new URL(url).hostname.toLowerCase();
  } catch {
    return url ? [url] : [];
  }
  const onBaidu = host === "bdstatic.com" || host.endsWith(".bdstatic.com");
  if (!onBaidu || /[@?]/.test(url.slice(url.indexOf(host) + host.length))) return [url];
  return [`${url}@w_600,q_80`, url];
}

/** Đi theo đường khoá trong JSON không có schema; thiếu nhánh nào thì `undefined`. */
function at(value: unknown, ...keys: string[]): unknown {
  let current = value;
  for (const key of keys) {
    if (current === null || typeof current !== "object") return undefined;
    current = (current as Record<string, unknown>)[key];
  }
  return current;
}

function parseJson(text: string, error: string): unknown {
  try {
    return JSON.parse(text) as unknown;
  } catch {
    throw new Error(error);
  }
}

/** fanqienovel.com/page/{id}: mọi thứ nằm trong `window.__INITIAL_STATE__` (literal JS, không hẳn JSON). */
function parseFanqie(html: string): Omit<SourceBook, "source"> {
  const m = html.match(/window\.__INITIAL_STATE__\s*=\s*(\{[\s\S]*?\})\s*;/);
  if (!m) throw new Error("Không đọc được dữ liệu trang Fanqie (trang đổi cấu trúc hoặc bị chặn)");
  const json = m[1].replace(/\bundefined\b/g, "null").replace(/\bNaN\b/g, "null").replace(/,\s*([}\]])/g, "$1");
  const page = at(parseJson(json, "Dữ liệu trang Fanqie không hợp lệ"), "page");
  if (!str(at(page, "bookName"))) throw new Error("Fanqie không trả về thông tin truyện (sai link hoặc truyện đã gỡ)");
  return {
    title: str(at(page, "bookName")),
    author: str(at(page, "authorName")) || str(at(page, "author")),
    coverUrls: fanqieCovers(str(at(page, "thumbUri")) || str(at(page, "thumbUrl"))),
  };
}

/** m.qidian.com/book/{id}/: page context SSR nhúng trong thẻ script JSON. */
function parseQidian(html: string, bookId: string): Omit<SourceBook, "source"> {
  const m = html.match(/<script id="vite-plugin-ssr_pageContext" type="application\/json">([\s\S]*?)<\/script>/);
  if (!m) throw new Error("Không đọc được dữ liệu trang Qidian (trang đổi cấu trúc hoặc bị chặn)");
  const info = at(parseJson(m[1], "Dữ liệu trang Qidian không hợp lệ"), "pageContext", "pageProps", "pageData", "bookInfo");
  if (!str(at(info, "bookName"))) throw new Error("Qidian không trả về thông tin truyện (sai link hoặc truyện đã gỡ)");
  return {
    title: str(at(info, "bookName")),
    author: str(at(info, "authorName")),
    coverUrls: [`https://bookcover.yuewen.com/qdbimg/349573/${bookId}/600`],
  };
}

/** detailadr.reader.qq.com/book/queryDetailPage: JSON công khai, không chữ ký. */
function parseQq(text: string, bookId: string): Omit<SourceBook, "source"> {
  const j = parseJson(text, "QQ đọc sách trả về dữ liệu không hợp lệ");
  const book = at(j, "introinfo", "book");
  if (!str(at(book, "title"))) {
    throw new Error(str(at(j, "msg")) || "QQ đọc sách không trả về thông tin truyện (sai link hoặc truyện đã gỡ)");
  }
  const n = Number(bookId);
  return {
    title: str(at(book, "title")),
    author: str(at(book, "author")),
    // Ảnh trong dữ liệu trước, rồi CDN bookcover (luôn dựng được từ bid).
    coverUrls: uniq([at(book, "coverUrl"), `https://bookcover.reader.qq.com/cover/${n % 1000}/${n}/t9_${n}.webp`]),
  };
}

const parseHtml = (html: string): Document => new DOMParser().parseFromString(html, "text/html");
const text = (el: Element | null | undefined) => str(el?.textContent).replace(/\s+/g, " ");

/** Qimao render sẵn HTML (`__NUXT__` là chương trình JS chứ không phải dữ liệu) → đọc markup. */
function parseQimao(html: string): Omit<SourceBook, "source"> {
  const info = parseHtml(html).querySelector(".book-information");
  const title = text(info?.querySelector(".title .txt"));
  if (!info || !title) throw new Error("Không đọc được dữ liệu trang Qimao (sai link, truyện đã gỡ hoặc trang đổi cấu trúc)");
  const authorRow = Array.from(info.querySelectorAll(".sub-title .txt")).find((el) => el.textContent?.includes("作者"));
  // Trang hiện bản 360x480; bỏ đuôi kích thước là bản gốc.
  const cover = httpsUrl(str(info.querySelector(".wrap-pic img")?.getAttribute("src")));
  return { title, author: text(authorRow?.querySelector("em")), coverUrls: uniq([cover, cover.replace(/_\d+x\d+(\.\w+)$/, "$1")]) };
}

/** m.sfacg.com/b/{id}/ — tác giả ở `.book_info3` dạng "{tác giả} / {số chữ}字…". */
function parseSfacg(html: string): Omit<SourceBook, "source"> {
  const doc = parseHtml(html);
  const title = text(doc.querySelector(".book_newtitle"));
  if (!title) throw new Error("Không đọc được dữ liệu trang SFACG (sai link, truyện đã gỡ hoặc trang đổi cấu trúc)");
  const cover = httpsUrl(str(doc.querySelector(".book_info img")?.getAttribute("src")));
  return { title, author: text(doc.querySelector(".book_info3")).split(" / ")[0].trim(), coverUrls: cover ? [cover] : [] };
}

/** app.jjwxc.net/androidapi/novelbasicinfo: JSON UTF-8 công khai; sách không còn thì trả `{ code, message }`. */
function parseJjwxc(body: string): Omit<SourceBook, "source"> {
  const j = parseJson(body, "JJWXC trả về dữ liệu không hợp lệ");
  if (!str(at(j, "novelName"))) {
    throw new Error(str(at(j, "message")) || "JJWXC không trả về thông tin truyện (sai link hoặc truyện đã gỡ)");
  }
  return {
    title: str(at(j, "novelName")),
    author: str(at(j, "authorName")),
    coverUrls: uniq([at(j, "novelCover"), at(j, "originalCover")].map((url) => httpsUrl(str(url)))).flatMap(baiduCovers),
  };
}

/** Bóc tên, tác giả, link ảnh bìa từ nội dung đã tải của `ref.fetchUrl`. */
export function parseSourceBook(ref: BookSourceRef, body: string): SourceBook {
  const parsed = (() => {
    switch (ref.source) {
      case "fanqie":
        return parseFanqie(body);
      case "qidian":
        return parseQidian(body, ref.bookId);
      case "qq":
        return parseQq(body, ref.bookId);
      case "qimao":
        return parseQimao(body);
      case "sfacg":
        return parseSfacg(body);
      case "jjwxc":
        return parseJjwxc(body);
    }
  })();
  return { source: ref.source, ...parsed };
}
