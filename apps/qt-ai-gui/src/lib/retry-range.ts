import type { ChapterRow } from "@/lib/types";

export interface RetryRangePreview {
  valid: boolean;
  /** Chương sẽ về hàng đợi (mọi trạng thái trừ queued). */
  targets: ChapterRow[];
  /** Trong đó chương done — bản dịch hiện có sẽ thành out/<id>.txt.bak. */
  done: number;
  /** Chương đã queued sẵn, bỏ qua. */
  queued: number;
}

/** Xem trước dịch lại theo thứ tự chương trong snapshot. Rỗng = từ đầu / tới cuối. */
export function previewRetryRange(chapters: ChapterRow[], from: string, to: string): RetryRangePreview {
  const ids = chapters.map((c) => c.id);
  const start = from ? ids.indexOf(from) : 0;
  const end = to ? ids.indexOf(to) : ids.length - 1;
  if (start < 0 || end < 0 || start > end) return { valid: false, targets: [], done: 0, queued: 0 };
  const slice = chapters.slice(start, end + 1);
  const targets = slice.filter((c) => c.status !== "queued");
  return {
    valid: true,
    targets,
    done: targets.filter((c) => c.status === "done").length,
    queued: slice.length - targets.length,
  };
}
