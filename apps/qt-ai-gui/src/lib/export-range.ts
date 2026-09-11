import type { ChapterRow } from "@/lib/types";

export interface RangePreview {
  valid: boolean;
  included: ChapterRow[];
  gaps: string[];
}

/** Xem trước theo thứ tự chương trong snapshot (Rust đã sort natural). Rỗng = từ đầu / tới cuối. */
export function previewRange(chapters: ChapterRow[], from: string, to: string): RangePreview {
  const ids = chapters.map((c) => c.id);
  const start = from ? ids.indexOf(from) : 0;
  const end = to ? ids.indexOf(to) : ids.length - 1;
  if (start < 0 || end < 0 || start > end) return { valid: false, included: [], gaps: [] };
  const slice = chapters.slice(start, end + 1);
  return {
    valid: true,
    included: slice.filter((c) => c.status === "done"),
    gaps: slice.filter((c) => c.status !== "done").map((c) => c.id),
  };
}

/**
 * Khoảng [start, end] (0-based) theo cùng luật với `previewRange`: đầu trống (-1) = từ đầu, cuối trống = tới cuối.
 * Trả null khi hai đầu đều trống hay đảo ngược — danh sách không tô dòng nào.
 */
export function rangeBounds(length: number, fromIndex: number, toIndex: number): [number, number] | null {
  if (length === 0 || (fromIndex < 0 && toIndex < 0)) return null;
  const start = fromIndex < 0 ? 0 : fromIndex;
  const end = toIndex < 0 ? length - 1 : toIndex;
  return start <= end ? [start, end] : null;
}
