import { describe, expect, it } from "vitest";

import { pauseMotionWhenInactive } from "@/lib/motion";

describe("pauseMotionWhenInactive", () => {
  it("blur gắn data-inactive, focus gỡ; gỡ listener thì trả về trạng thái đang dùng", () => {
    const root = document.createElement("html");
    const stop = pauseMotionWhenInactive(window, root);
    expect(root.hasAttribute("data-inactive")).toBe(false);
    window.dispatchEvent(new Event("blur"));
    expect(root.hasAttribute("data-inactive")).toBe(true);
    window.dispatchEvent(new Event("focus"));
    expect(root.hasAttribute("data-inactive")).toBe(false);
    window.dispatchEvent(new Event("blur"));
    stop();
    expect(root.hasAttribute("data-inactive")).toBe(false);
    window.dispatchEvent(new Event("blur"));
    expect(root.hasAttribute("data-inactive")).toBe(false);
  });
});
