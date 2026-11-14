// Bóc dữ liệu từ HTML của trang nguồn. Tách riêng để main.js gọn và dễ sửa khi trang đổi giao diện.

/** `https://www.truyen-mau.example/book/1` → `https://www.truyen-mau.example` */
export function origin(url) {
  const found = url.match(/^https?:\/\/[^/]+/);
  if (!found) throw new ExtensionError("link không hợp lệ: " + url);
  return found[0];
}

/** Các chương trong một trang mục lục. `data` giữ link chương để `chapter` dùng lại. */
export function parseToc(doc) {
  return doc.select("#list a").map((a) => ({
    id: a.attr("href").split("/").pop(),
    title: a.text(),
    locked: (a.attr("class") || "").split(" ").includes("vip"),
    data: { href: a.attr("href") },
  }));
}

/** Link trang mục lục kế tiếp, hoặc null ở trang cuối. */
export function nextPage(doc, current) {
  const next = doc.selectFirst("a.next");
  return next ? origin(current) + next.attr("href") : null;
}
