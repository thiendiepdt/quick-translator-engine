import { describe, expect, it } from "vitest";

import { countByFilter, filterChapters } from "@/lib/chapters";
import type { ChapterRow } from "@/lib/types";

const rows: ChapterRow[] = [
  { id: "chuong-0001", status: "done", reviewRound: 0, reason: null, warnings: [] },
  { id: "chuong-0002", status: "done", reviewRound: 3, reason: null, warnings: ["[[1]] CJK"] },
  { id: "chuong-0003", status: "error", reviewRound: 3, reason: "Quá 3 vòng", warnings: [] },
  { id: "chuong-0010", status: "queued", reviewRound: 0, reason: null, warnings: [] },
];

describe("chapters", () => {
  it("lọc theo trạng thái; warning = done có cảnh báo; all giữ hết", () => {
    expect(filterChapters(rows, "all", "")).toHaveLength(4);
    expect(filterChapters(rows, "error", "").map((r) => r.id)).toEqual(["chuong-0003"]);
    expect(filterChapters(rows, "warning", "").map((r) => r.id)).toEqual(["chuong-0002"]);
    expect(filterChapters(rows, "done", "")).toHaveLength(2);
  });

  it("tìm theo mã không phân biệt hoa thường, kết hợp với lọc", () => {
    expect(filterChapters(rows, "all", "001").map((r) => r.id)).toEqual(["chuong-0001", "chuong-0010"]);
    expect(filterChapters(rows, "done", "CHUONG-0002")).toHaveLength(1);
  });

  it("countByFilter đếm từng chip", () => {
    const counts = countByFilter(rows);
    expect(counts.all).toBe(4);
    expect(counts.done).toBe(2);
    expect(counts.warning).toBe(1);
    expect(counts.skipped).toBe(0);
  });
});
