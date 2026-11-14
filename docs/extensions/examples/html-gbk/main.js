// Extension mẫu: trang truyện HTML thường, mã hoá GBK, mục lục chia nhiều trang.
//
// Trang nguồn (giả định):
//   /book/<mã>/list?page=N   mục lục: <div id="list"><a href="/c/123">第1章 …</a>…</div>, trang sau <a class="next">
//                            chương VIP có class "vip"
//   /c/<mã chương>           <h1>tiêu đề</h1><div id="content"><p>…</p></div>
import { nextPage, origin, parseToc } from "./lib/parse.js";

export async function toc(ctx) {
  const maxPages = Number(ctx.settings.maxPages) || 50;
  const items = [];
  let url = ctx.rawUrl.replace(/\/$/, "") + "/list?page=1";
  for (let page = 1; url && page <= maxPages; page++) {
    // Trang luôn là GBK: ép mã hoá thay vì để app đoán.
    const res = await http.get(url);
    if (!res.ok) throw new ExtensionError("mục lục trả HTTP " + res.status, { retry: res.status >= 500 });
    const doc = html.parse(res.text("gbk"));
    items.push(...parseToc(doc));
    url = nextPage(doc, url);
    progress(page, 0, "trang mục lục " + page);
  }
  if (url) log.warn("dừng ở " + maxPages + " trang mục lục — tăng \"Số trang mục lục tối đa\" nếu truyện dài hơn");
  return items;
}

export async function chapter(ctx, ch) {
  const res = await http.get(origin(ctx.rawUrl) + ch.data.href);
  if (res.status === 404) throw new ExtensionError("chương không còn trên trang nguồn");
  if (!res.ok) throw new ExtensionError("chương trả HTTP " + res.status, { retry: true });
  const doc = html.parse(res.text("gbk"));
  const body = doc.selectFirst("#content");
  if (!body) throw new Locked("trang không có nội dung — có thể là chương VIP");
  // Trả HTML: app tự đổi <p>, <br> thành dòng và bỏ thẻ.
  return { content: body.html(), title: doc.selectFirst("h1")?.text() || ch.title, format: "html" };
}
