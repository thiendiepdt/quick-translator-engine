// Bóc mục lục và link trang sau của trang nguồn giả.
export function origin(url) {
  return url.match(/^https?:\/\/[^/]+/)[0];
}

export function parseToc(doc) {
  return doc.select("#list a").map((a) => ({
    id: a.attr("href").split("/").pop(),
    title: a.text(),
    locked: a.attr("class") === "vip",
    data: { href: a.attr("href") },
  }));
}

export function nextPage(doc, current) {
  const next = doc.selectFirst("a.next");
  return next ? origin(current) + next.attr("href") : null;
}
