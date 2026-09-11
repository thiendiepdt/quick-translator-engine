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
