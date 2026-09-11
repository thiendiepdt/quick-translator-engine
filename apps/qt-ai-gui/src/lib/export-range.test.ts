import { describe, expect, it } from "vitest";

import { previewRange } from "@/lib/export-range";
import type { ChapterRow } from "@/lib/types";

const rows: ChapterRow[] = ["0001", "0002", "0003", "0004"].map((id, i) => ({
  id,
  status: i === 2 ? "skipped" : "done",
  reviewRound: 0,
  reason: null,
  warnings: [],
}));

describe("previewRange", () => {
  it("đếm done và hổng trong khoảng; rỗng = toàn bộ", () => {
    const p = previewRange(rows, "0002", "0004");
    expect(p.valid).toBe(true);
    expect(p.included.map((c) => c.id)).toEqual(["0002", "0004"]);
    expect(p.gaps).toEqual(["0003"]);
    expect(previewRange(rows, "", "").included).toHaveLength(3);
  });

  it("khoảng sai thì invalid", () => {
    expect(previewRange(rows, "0004", "0001").valid).toBe(false);
    expect(previewRange(rows, "9999", "").valid).toBe(false);
  });
});
