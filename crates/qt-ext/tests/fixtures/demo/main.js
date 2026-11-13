import { nextPage, origin, parseToc } from "./lib/parse.js";

export async function toc(ctx) {
  const items = [];
  let url = ctx.rawUrl + "/list?page=1";
  for (let page = 1; url && page <= 20; page++) {
    const doc = html.parse((await http.get(url)).text());
    items.push(...parseToc(doc));
    url = nextPage(doc, url);
    progress(page, 0, "trang mục lục " + page);
  }
  return items;
}

export async function chapter(ctx, ch) {
  if (ch.data.href.endsWith("/err")) throw new ExtensionError("hỏng", { retry: true });
  const doc = html.parse((await http.get(origin(ctx.rawUrl) + ch.data.href)).text());
  return { content: doc.selectFirst("#content").html(), title: doc.selectFirst("h1").text(), format: "html" };
}
