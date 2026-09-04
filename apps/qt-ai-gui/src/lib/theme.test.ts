import { describe, expect, it } from "vitest";

import { applyPalette, bootPalette, isPalette, isThemeMode, PALETTES, rememberPalette } from "@/lib/theme";

function storage(initial: Record<string, string> = {}) {
  const map = new Map(Object.entries(initial));
  return {
    getItem: (k: string) => map.get(k) ?? null,
    setItem: (k: string, v: string) => void map.set(k, v),
    map,
  };
}

describe("theme", () => {
  it("guard kiểu", () => {
    expect(isPalette("studio")).toBe(true);
    expect(isPalette("neon")).toBe(false);
    expect(isThemeMode("system")).toBe(true);
    expect(isThemeMode("auto")).toBe(false);
    expect(PALETTES.map((p) => p.id)).toEqual(["editorial", "studio", "soft"]);
  });

  it("applyPalette gắn data-palette", () => {
    const root = document.createElement("html");
    applyPalette(root, "soft");
    expect(root.dataset.palette).toBe("soft");
  });

  it("bootPalette đọc cache, sai thì về editorial; rememberPalette ghi cache", () => {
    const root = document.createElement("html");
    expect(bootPalette(root, storage({ "qt-ai-palette": "studio" }))).toBe("studio");
    expect(root.dataset.palette).toBe("studio");
    expect(bootPalette(root, storage({ "qt-ai-palette": "rác" }))).toBe("editorial");
    const s = storage();
    rememberPalette(s, "soft");
    expect(s.map.get("qt-ai-palette")).toBe("soft");
  });
});
