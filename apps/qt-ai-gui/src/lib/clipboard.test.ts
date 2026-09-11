import { afterEach, describe, expect, it, vi } from "vitest";

import { copyText } from "@/lib/clipboard";

describe("copyText", () => {
  afterEach(() => vi.unstubAllGlobals());

  it("dùng navigator.clipboard khi có", async () => {
    const writeText = vi.fn(() => Promise.resolve());
    vi.stubGlobal("navigator", { clipboard: { writeText } });
    await copyText("xin chào");
    expect(writeText).toHaveBeenCalledWith("xin chào");
  });

  it("clipboard API hỏng hoặc thiếu thì rơi về execCommand copy, thất bại thì ném lỗi", async () => {
    vi.stubGlobal("navigator", { clipboard: { writeText: () => Promise.reject(new Error("blocked")) } });
    const exec = vi.fn(() => true);
    document.execCommand = exec;
    await copyText("abc");
    expect(exec).toHaveBeenCalledWith("copy");
    expect(document.querySelector("textarea")).toBeNull(); // dọn textarea tạm

    document.execCommand = vi.fn(() => false);
    await expect(copyText("abc")).rejects.toThrow(/clipboard/);
  });
});
