import { describe, expect, it } from "vitest";

import { chapterGaps, countByFilter, filterChapters, resolveChapterRef } from "@/lib/chapters";
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

describe("chapterGaps", () => {
  const row = (id: string, status: ChapterRow["status"], reason: string | null = null): ChapterRow => ({
    id,
    status,
    reviewRound: 0,
    reason,
    warnings: [],
  });

  it("chương chưa done trước chương done cuối, cộng chương bỏ qua/lỗi nằm sau; queued phía sau không tính", () => {
    const list: ChapterRow[] = [
      row("c1", "done"),
      row("c2", "skipped", "model từ chối"),
      row("c3", "error"),
      row("c4", "queued"),
      row("c5", "done"),
      row("c6", "skipped", "lỗi API: 503"),
      row("c7", "translating"),
      row("c8", "queued"),
      row("c9", "error"),
    ];
    expect(chapterGaps(list)).toEqual({
      frontier: { ordinal: 5, id: "c5" },
      gaps: [
        { ordinal: 2, row: list[1] },
        { ordinal: 3, row: list[2] },
        { ordinal: 4, row: list[3] },
        { ordinal: 6, row: list[5] },
        { ordinal: 9, row: list[8] },
      ],
    });
  });

  it("chương bỏ qua vì lỗi API nằm sau chương done cuối vẫn được gom (ta-mo-phong-truong-sinh-lo #1672)", () => {
    const list = [row("chuong-1671", "done"), row("chuong-1672", "skipped", "lỗi API: OpenAI trả 503"), row("chuong-1673", "queued")];
    expect(chapterGaps(list)).toEqual({ frontier: { ordinal: 1, id: "chuong-1671" }, gaps: [{ ordinal: 2, row: list[1] }] });
  });

  it("chưa có chương done: chỉ gom chương bỏ qua/lỗi, frontier null", () => {
    const list = [row("c1", "skipped", "lỗi API"), row("c2", "queued")];
    expect(chapterGaps(list)).toEqual({ frontier: null, gaps: [{ ordinal: 1, row: list[0] }] });
  });

  it("không hổng, không bỏ qua/lỗi thì trả null", () => {
    expect(chapterGaps([row("c1", "queued")])).toBeNull();
    expect(chapterGaps(rows.slice(0, 2))).toBeNull();
    expect(chapterGaps([row("c1", "done"), row("c2", "translating"), row("c3", "queued")])).toBeNull();
  });
});
