import { describe, expect, it } from "vitest";

import { countByFilter, filterChapters, gapsBeforeFrontier, resolveChapterRef } from "@/lib/chapters";
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

  it("tìm theo số thứ tự: '3' khớp chương thứ 3 lẫn mã chứa 3, '#3' chỉ chương thứ 3", () => {
    expect(filterChapters(rows, "all", "#3").map((r) => r.id)).toEqual(["chuong-0003"]);
    expect(filterChapters(rows, "all", "4").map((r) => r.id)).toEqual(["chuong-0010"]); // thứ 4, mã không chứa 4
    expect(filterChapters(rows, "all", "#9")).toHaveLength(0);
  });

  it("resolveChapterRef: số thứ tự 1-based hoặc nguyên mã; ngoài khoảng/lạ → -1", () => {
    expect(resolveChapterRef(rows, "1")).toBe(0);
    expect(resolveChapterRef(rows, "#4")).toBe(3);
    expect(resolveChapterRef(rows, "chuong-0003")).toBe(2);
    expect(resolveChapterRef(rows, "5")).toBe(-1);
    expect(resolveChapterRef(rows, "0")).toBe(-1);
    expect(resolveChapterRef(rows, "")).toBe(-1);
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

describe("gapsBeforeFrontier", () => {
  it("liệt kê chương chưa done đứng trước chương done cuối, kèm số thứ tự", () => {
    const list: ChapterRow[] = [
      { id: "c1", status: "done", reviewRound: 0, reason: null, warnings: [] },
      { id: "c2", status: "skipped", reviewRound: 0, reason: "model từ chối", warnings: [] },
      { id: "c3", status: "error", reviewRound: 3, reason: null, warnings: [] },
      { id: "c4", status: "done", reviewRound: 0, reason: null, warnings: [] },
      { id: "c5", status: "translating", reviewRound: 0, reason: null, warnings: [] },
      { id: "c6", status: "queued", reviewRound: 0, reason: null, warnings: [] },
    ];
    expect(gapsBeforeFrontier(list)).toEqual({
      frontier: { ordinal: 4, id: "c4" },
      gaps: [
        { ordinal: 2, row: list[1] },
        { ordinal: 3, row: list[2] },
      ],
    });
  });

  it("không có chương done hoặc không hổng thì trả null", () => {
    expect(gapsBeforeFrontier([{ id: "c1", status: "queued", reviewRound: 0, reason: null, warnings: [] }])).toBeNull();
    expect(gapsBeforeFrontier(rows.slice(0, 2))).toBeNull();
  });
});
