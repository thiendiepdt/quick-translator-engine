import { describe, expect, it } from "vitest";

import { READING_WIDTHS, readingWidthOf } from "@/lib/reading";

describe("reading width", () => {
  it("nhận đúng 4 mức, giá trị lạ về normal", () => {
    expect(READING_WIDTHS).toEqual(["narrow", "normal", "wide", "full"]);
    expect(readingWidthOf("wide")).toBe("wide");
    expect(readingWidthOf("huge")).toBe("normal");
    expect(readingWidthOf(undefined)).toBe("normal");
  });
});
