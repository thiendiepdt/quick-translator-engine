import { describe, expect, it } from "vitest";

import css from "@/index.css?raw";
import { PALETTE_IDS } from "@/lib/theme";
import {
  contrastRatio,
  parseOklch,
  parsePaletteBlocks,
  relativeLuminance,
  REQUIRED_TOKENS,
} from "@/lib/theme-tokens";

const blocks = parsePaletteBlocks(css);

describe("theme tokens", () => {
  it("parseOklch + luminance: trắng ≈ 1, đen ≈ 0", () => {
    expect(parseOklch("oklch(1 0 0)")).toEqual({ l: 1, c: 0, h: 0 });
    expect(relativeLuminance({ l: 1, c: 0, h: 0 })).toBeCloseTo(1, 2);
    expect(relativeLuminance({ l: 0, c: 0, h: 0 })).toBeCloseTo(0, 3);
    expect(contrastRatio({ l: 1, c: 0, h: 0 }, { l: 0, c: 0, h: 0 })).toBeCloseTo(21, 0);
    expect(parseOklch("red")).toBeNull();
  });

  it.each(PALETTE_IDS.flatMap((p) => [`${p}/light`, `${p}/dark`]))("%s có đủ token bắt buộc", (key) => {
    const tokens = blocks.get(key);
    expect(tokens, `thiếu block ${key}`).toBeDefined();
    const missing = REQUIRED_TOKENS.filter((t) => !(t in (tokens ?? {})));
    expect(missing).toEqual([]);
  });

  const PAIRS: Array<[string, string]> = [
    ["--foreground", "--background"],
    ["--muted-foreground", "--background"],
    ["--card-foreground", "--card"],
    ["--primary-foreground", "--primary"],
    ["--log-fg", "--log-bg"],
  ];

  it.each([...blocks.keys()])("%s tương phản ≥ 4.5", (key) => {
    const tokens = blocks.get(key) ?? {};
    for (const [fg, bg] of PAIRS) {
      const a = parseOklch(tokens[fg] ?? "");
      const b = parseOklch(tokens[bg] ?? "");
      if (!a || !b) throw new Error(`${key}: ${fg} hoặc ${bg} không phải oklch`);
      const ratio = contrastRatio(a, b);
      expect(ratio, `${key}: ${fg} trên ${bg} = ${ratio.toFixed(2)}`).toBeGreaterThanOrEqual(4.5);
    }
  });
});
