import { describe, expect, it } from "vitest";

import { buildTextSegments, rangeText } from "@/lib/ranges";

describe("UTF-16 range rendering", () => {
  it("keeps JavaScript UTF-16 offsets including emoji", () => {
    const segments = buildTextSegments("甲😀乙", [
      { start: 1, length: 2 },
      { start: 3, length: 1 },
    ]);

    expect(segments).toEqual([
      { kind: "plain", text: "甲", key: "plain-0-1" },
      { kind: "mapped", text: "😀", rangeIndex: 0, key: "mapped-0-1" },
      { kind: "mapped", text: "乙", rangeIndex: 1, key: "mapped-1-3" },
    ]);
    expect(rangeText("甲😀乙", { start: 1, length: 2 })).toBe("😀");
  });

  it("ignores invalid and overlapping ranges without corrupting text", () => {
    const segments = buildTextSegments("abcdef", [
      { start: 1, length: 3 },
      { start: 2, length: 2 },
      { start: 20, length: 1 },
    ]);
    expect(segments.map(({ text }) => text).join("")).toBe("abcdef");
    expect(segments.filter(({ kind }) => kind === "mapped")).toHaveLength(1);
  });
});
