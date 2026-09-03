import { STATUS_LABELS, type ChapterRow, type ChapterStatus } from "@/lib/types";

export type ChapterFilter = ChapterStatus | "all" | "warning";

export const FILTER_ORDER: ChapterFilter[] = ["all", "queued", "translating", "done", "warning", "error", "skipped"];

export const FILTER_LABELS: Record<ChapterFilter, string> = { ...STATUS_LABELS, all: "Tất cả", warning: "Cảnh báo" };

function matches(row: ChapterRow, filter: ChapterFilter): boolean {
  if (filter === "all") return true;
  if (filter === "warning") return row.status === "done" && row.warnings.length > 0;
  return row.status === filter;
}

export function filterChapters(rows: ChapterRow[], filter: ChapterFilter, query: string): ChapterRow[] {
  const needle = query.trim().toLowerCase();
  return rows.filter((row) => matches(row, filter) && (needle === "" || row.id.toLowerCase().includes(needle)));
}

export function countByFilter(rows: ChapterRow[]): Record<ChapterFilter, number> {
  const counts = Object.fromEntries(FILTER_ORDER.map((f) => [f, 0])) as Record<ChapterFilter, number>;
  for (const row of rows) {
    for (const filter of FILTER_ORDER) if (matches(row, filter)) counts[filter] += 1;
  }
  return counts;
}
