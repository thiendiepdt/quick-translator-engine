import { Search } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { countByFilter, FILTER_LABELS, FILTER_ORDER, filterChapters, type ChapterFilter } from "@/lib/chapters";
import { rangeBounds } from "@/lib/export-range";
import type { ChapterRow, ChapterStatus } from "@/lib/types";
import { cn } from "@/lib/utils";

const DOT: Record<ChapterStatus, string> = {
  queued: "bg-status-queued",
  translating: "bg-status-translating animate-pulse",
  done: "bg-status-done",
  error: "bg-status-error",
  skipped: "bg-status-queued ring-1 ring-foreground/30",
};

interface Props {
  rows: ChapterRow[];
  /** Vị trí (0-based) đã resolve của ô Từ/Đến; -1 = trống hoặc không khớp. */
  fromIndex: number;
  toIndex: number;
  /** Trả số thứ tự 1-based như cột #. */
  onPickFrom: (ordinal: number) => void;
  onPickTo: (ordinal: number) => void;
}

/** Danh sách chương cho trang Export: lọc/tìm cục bộ, mỗi dòng có nút Từ/Đến, dòng trong khoảng tô nền. */
export function ExportChapterList({ rows, fromIndex, toIndex, onPickFrom, onPickTo }: Props) {
  const [filter, setFilter] = useState<ChapterFilter>("all");
  const [query, setQuery] = useState("");
  const visible = filterChapters(rows, filter, query);
  const counts = countByFilter(rows);
  const ordinal = new Map(rows.map((row, index) => [row.id, index + 1]));
  const width = String(rows.length).length;
  const bounds = rangeBounds(rows.length, fromIndex, toIndex);
  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-col gap-2 border-b p-3">
        <div className="relative">
          <Search className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            type="search"
            role="searchbox"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Tìm mã hoặc số thứ tự (#12)…"
            className="h-8 pl-8 font-mono text-xs"
          />
        </div>
        <div className="flex flex-wrap gap-1">
          {FILTER_ORDER.filter((f) => f === "all" || counts[f] > 0).map((f) => (
            <button
              key={f}
              type="button"
              onClick={() => setFilter(f)}
              aria-pressed={filter === f}
              className={cn(
                "rounded-full border px-2 py-0.5 text-xs transition-colors hover:bg-accent",
                filter === f
                  ? "border-primary bg-primary text-primary-foreground hover:bg-primary"
                  : "text-muted-foreground",
              )}
            >
              {FILTER_LABELS[f]} {counts[f]}
            </button>
          ))}
        </div>
      </div>
      <ul aria-label="Danh sách chương" className="fine-scrollbar flex-1 overflow-y-auto">
        {visible.length === 0 && <li className="p-4 text-sm text-muted-foreground">Không có chương nào khớp.</li>}
        {visible.map((row) => {
          const n = ordinal.get(row.id) ?? 0;
          const inRange = bounds !== null && n - 1 >= bounds[0] && n - 1 <= bounds[1];
          const gap = inRange && row.status !== "done";
          return (
            <li
              key={row.id}
              data-in-range={inRange ? "true" : "false"}
              className={cn(
                "group flex items-center gap-2.5 border-l-2 border-transparent px-3 py-1.5 text-sm hover:bg-accent/60",
                inRange && "border-l-primary bg-accent",
              )}
            >
              <span className={cn("size-2 shrink-0 rounded-full", DOT[row.status])} aria-hidden />
              <span
                className="shrink-0 font-mono text-[11px] text-muted-foreground tabular-nums"
                style={{ minWidth: `${width + 1}ch` }}
              >
                #{n}
              </span>
              <span className="min-w-0 flex-1 truncate font-mono text-xs">{row.id}</span>
              {gap && (
                <span className="rounded-full bg-status-warning/15 px-1.5 text-[11px] font-medium text-status-warning">
                  hổng
                </span>
              )}
              <span className="flex shrink-0 gap-1 opacity-40 group-hover:opacity-100 has-[:focus-visible]:opacity-100">
                <Button
                  type="button"
                  size="xs"
                  variant="outline"
                  aria-label={`Từ chương #${n}`}
                  title="Đặt làm chương bắt đầu"
                  onClick={() => onPickFrom(n)}
                >
                  Từ
                </Button>
                <Button
                  type="button"
                  size="xs"
                  variant="outline"
                  aria-label={`Đến chương #${n}`}
                  title="Đặt làm chương kết thúc"
                  onClick={() => onPickTo(n)}
                >
                  Đến
                </Button>
              </span>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
