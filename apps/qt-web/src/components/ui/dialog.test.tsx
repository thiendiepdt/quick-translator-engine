import { describe, expect, it } from "vitest";

import { isToastInteraction } from "@/components/ui/dialog";

// Radix không phát interact-outside trong jsdom (đã đo: spy = 0 cả với click
// ngoài thường), nên phần dây điện dialog↔Radix chỉ kiểm được trên trình
// duyệt thật. Ở đây khóa phần guard: selector toaster và kiểm tra Element.
describe("isToastInteraction", () => {
  it("matches clicks inside the sonner toaster, including nested elements", () => {
    const toaster = document.createElement("div");
    toaster.setAttribute("data-sonner-toaster", "true");
    const action = document.createElement("button");
    toaster.appendChild(action);
    document.body.appendChild(toaster);

    expect(isToastInteraction(action)).toBe(true);
    expect(isToastInteraction(toaster)).toBe(true);
    toaster.remove();
  });

  it("ignores everything else", () => {
    const outside = document.createElement("button");
    document.body.appendChild(outside);
    expect(isToastInteraction(outside)).toBe(false);
    expect(isToastInteraction(document)).toBe(false);
    expect(isToastInteraction(null)).toBe(false);
    outside.remove();
  });
});
