import { STATUS_LABELS, type ChapterRow, type ChapterStatus } from "@/lib/types";

export type ChapterFilter = ChapterStatus | "all" | "warning";

export const FILTER_ORDER: ChapterFilter[] = ["all", "queued", "translating", "done", "warning", "error", "skipped"];

export const FILTER_LABELS: Record<ChapterFilter, string> = { ...STATUS_LABELS, all: "Tất cả", warning: "Cảnh báo" };

function matches(row: ChapterRow, filter: ChapterFilter): boolean {
  if (filter === "all") return true;
  if (filter === "warning") return row.status === "done" && row.warnings.length > 0;
  return row.status === filter;
}

/**
 * Tìm theo mã (chứa, không phân biệt hoa thường) hoặc theo số thứ tự trong danh sách đầy đủ:
 * "12" khớp mã chứa "12" lẫn chương thứ 12; "#12" chỉ khớp chương thứ 12.
 */
export function filterChapters(rows: ChapterRow[], filter: ChapterFilter, query: string): ChapterRow[] {
  const needle = query.trim().toLowerCase();
  const ordinalOnly = needle.startsWith("#");
  const ordinal = /^#?\d+$/.test(needle) ? Number(needle.replace("#", "")) : undefined;
  return rows.filter((row, index) => {
    if (!matches(row, filter)) return false;
    if (needle === "") return true;
    if (ordinal !== undefined && index + 1 === ordinal) return true;
    return !ordinalOnly && row.id.toLowerCase().includes(needle);
  });
}

/**
 * Người dùng gõ "số thứ tự" (1-based, theo danh sách đầy đủ) hoặc nguyên mã chương → vị trí trong `rows`,
 * -1 nếu không khớp. Truyện vài ngàn chương không thể chọn mã dài từ datalist, gõ số là đủ.
 */
export function resolveChapterRef(rows: ChapterRow[], text: string): number {
  const ref = text.trim();
  if (ref === "") return -1;
  if (/^#?\d+$/.test(ref)) {
    const ordinal = Number(ref.replace("#", ""));
    if (ordinal >= 1 && ordinal <= rows.length) return ordinal - 1;
  }
  return rows.findIndex((row) => row.id === ref);
}

export function countByFilter(rows: ChapterRow[]): Record<ChapterFilter, number> {
  const counts = Object.fromEntries(FILTER_ORDER.map((f) => [f, 0])) as Record<ChapterFilter, number>;
  for (const row of rows) {
    for (const filter of FILTER_ORDER) if (matches(row, filter)) counts[filter] += 1;
  }
  return counts;
}
