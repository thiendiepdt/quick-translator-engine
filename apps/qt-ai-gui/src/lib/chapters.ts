import { STATUS_LABELS, type ChapterRow, type ChapterStatus } from "@/lib/types";

export type ChapterFilter = ChapterStatus | "all" | "warning" | "short";

export const FILTER_ORDER: ChapterFilter[] = ["all", "queued", "translating", "done", "warning", "short", "error", "skipped"];

export const FILTER_LABELS: Record<ChapterFilter, string> = { ...STATUS_LABELS, all: "Tất cả", warning: "Cảnh báo", short: "Ngắn" };

/** `short`: mã các chương đã dịch dưới ngưỡng chữ của nền tảng đăng truyện (chưa biết thì rỗng). */
function matches(row: ChapterRow, filter: ChapterFilter, short?: ReadonlySet<string>): boolean {
  if (filter === "all") return true;
  if (filter === "warning") return row.status === "done" && row.warnings.length > 0;
  if (filter === "short") return short?.has(row.id) ?? false;
  return row.status === filter;
}

/**
 * Tìm theo mã (chứa, không phân biệt hoa thường) hoặc theo số thứ tự trong danh sách đầy đủ:
 * "12" khớp mã chứa "12" lẫn chương thứ 12; "#12" chỉ khớp chương thứ 12.
 */
export function filterChapters(rows: ChapterRow[], filter: ChapterFilter, query: string, short?: ReadonlySet<string>): ChapterRow[] {
  const needle = query.trim().toLowerCase();
  const ordinalOnly = needle.startsWith("#");
  const ordinal = /^#?\d+$/.test(needle) ? Number(needle.replace("#", "")) : undefined;
  return rows.filter((row, index) => {
    if (!matches(row, filter, short)) return false;
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

export function countByFilter(rows: ChapterRow[], short?: ReadonlySet<string>): Record<ChapterFilter, number> {
  const counts = Object.fromEntries(FILTER_ORDER.map((f) => [f, 0])) as Record<ChapterFilter, number>;
  for (const row of rows) {
    for (const filter of FILTER_ORDER) if (matches(row, filter, short)) counts[filter] += 1;
  }
  return counts;
}

export interface ChapterGaps {
  /** Chương done cuối cùng theo thứ tự danh sách (số thứ tự 1-based); null khi chưa có chương done. */
  frontier: { ordinal: number; id: string } | null;
  /** Chương phiên sẽ không tự quay lại: trước frontier mà chưa done, không đang dịch (queued, error, skipped);
   * sau frontier thì chỉ error, skipped (queued phía sau phiên tự dịch tới). */
  gaps: { ordinal: number; row: ChapterRow }[];
}

/**
 * Chương "hổng": chưa dịch nhưng đứng trước chương done cuối — thường do skip (model từ chối) hay lỗi rồi phiên dịch
 * tiếp chương sau — cộng chương bỏ qua / lỗi nằm sau chương done cuối (vd. bỏ qua vì lỗi API rồi phiên dừng ngay):
 * phiên không tự quay lại các chương này. Trả null khi không có chương nào như vậy.
 */
export function chapterGaps(rows: ChapterRow[]): ChapterGaps | null {
  let last = -1;
  rows.forEach((row, index) => {
    if (row.status === "done") last = index;
  });
  const gaps = rows
    .map((row, index) => ({ ordinal: index + 1, row }))
    .filter(({ row, ordinal }) =>
      ordinal - 1 < last
        ? row.status !== "done" && row.status !== "translating"
        : row.status === "error" || row.status === "skipped",
    );
  if (gaps.length === 0) return null;
  return { frontier: last < 0 ? null : { ordinal: last + 1, id: rows[last].id }, gaps };
}

/** Thông điệp sau khi xoá chương, dùng chung cho xoá lẻ và xoá khoảng. */
export function describeDelete(removed: number, keptOutputs: number): string {
  return `Đã xoá ${removed} chương` + (keptOutputs > 0 ? ` (${keptOutputs} bản dịch trong out/ giữ nguyên)` : "");
}
