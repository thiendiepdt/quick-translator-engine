/**
 * Danh sách nút trang dạng cửa sổ: trang đầu, trang cuối và lân cận trang
 * hiện tại; khe hở dài thành "ellipsis", khe đúng 1 trang thì hiện luôn số
 * trang đó (không ai muốn "…" che đúng một trang). Neo mỗi đầu chỉ một trang
 * để cửa sổ đứng gần mép không nở thành dãy sáu bảy số liền nhau.
 *
 * Trang đánh số từ 0. Ví dụ 772 trang, đang ở trang 767 (hiện là "768"):
 * 1 … 767 768 769 … 772.
 */
export function paginationItems(
  page: number,
  pageCount: number,
): Array<number | "ellipsis"> {
  const count = Math.max(1, pageCount);
  const current = Math.min(Math.max(0, page), count - 1);
  if (count <= 7) {
    return Array.from({ length: count }, (_, index) => index);
  }
  const pages = new Set<number>();
  for (const candidate of [0, current - 1, current, current + 1, count - 1]) {
    if (candidate >= 0 && candidate < count) pages.add(candidate);
  }
  const sorted = [...pages].sort((left, right) => left - right);
  const items: Array<number | "ellipsis"> = [];
  let previous: number | undefined;
  for (const pageIndex of sorted) {
    if (previous !== undefined) {
      if (pageIndex - previous === 2) items.push(previous + 1);
      else if (pageIndex - previous > 2) items.push("ellipsis");
    }
    items.push(pageIndex);
    previous = pageIndex;
  }
  return items;
}
