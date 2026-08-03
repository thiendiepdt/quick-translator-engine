import { describe, expect, it } from "vitest";

import { paginationItems } from "@/lib/pagination";

describe("paginationItems", () => {
  it("lists every page when there are few", () => {
    expect(paginationItems(0, 1)).toEqual([0]);
    expect(paginationItems(2, 7)).toEqual([0, 1, 2, 3, 4, 5, 6]);
  });

  it("windows around the current page with single head and tail anchors", () => {
    expect(paginationItems(10, 21)).toEqual([0, "ellipsis", 9, 10, 11, "ellipsis", 20]);
    // Sát mép: cửa sổ nhập vào neo, không nở thành dãy dài (bug 767…772 cũ).
    expect(paginationItems(767, 772)).toEqual([0, "ellipsis", 766, 767, 768, "ellipsis", 771]);
    expect(paginationItems(770, 772)).toEqual([0, "ellipsis", 769, 770, 771]);
    expect(paginationItems(1, 21)).toEqual([0, 1, 2, "ellipsis", 20]);
  });

  it("shows the page number instead of an ellipsis for one-page gaps", () => {
    // Khe giữa 0 và 2 chỉ là trang 1 → hiện số 1 thay vì "…".
    expect(paginationItems(3, 21)).toEqual([0, 1, 2, 3, 4, "ellipsis", 20]);
    expect(paginationItems(2, 21)).toEqual([0, 1, 2, 3, "ellipsis", 20]);
    expect(paginationItems(18, 21)).toEqual([0, "ellipsis", 17, 18, 19, 20]);
  });

  it("clamps out-of-range pages", () => {
    expect(paginationItems(99, 8)).toEqual([0, "ellipsis", 6, 7]);
    expect(paginationItems(-5, 8)).toEqual([0, 1, "ellipsis", 7]);
  });
});
