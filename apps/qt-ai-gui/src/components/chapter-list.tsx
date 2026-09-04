import { Search } from "lucide-react";

import { Input } from "@/components/ui/input";
import { countByFilter, FILTER_LABELS, FILTER_ORDER, filterChapters, type ChapterFilter } from "@/lib/chapters";
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
  filter: ChapterFilter;
  query: string;
  selectedId?: string;
  onSelect: (id: string) => void;
  onFilter: (filter: ChapterFilter) => void;
  onQuery: (query: string) => void;
}

export function ChapterList({ rows, filter, query, selectedId, onSelect, onFilter, onQuery }: Props) {
  const visible = filterChapters(rows, filter, query);
  const counts = countByFilter(rows);
  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-col gap-2 border-b p-3">
        <div className="relative">
          <Search className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            type="search"
            role="searchbox"
            value={query}
            onChange={(e) => onQuery(e.target.value)}
            placeholder="Tìm mã chương…"
            className="h-8 pl-8 font-mono text-xs"
          />
        </div>
        <div className="flex flex-wrap gap-1">
          {FILTER_ORDER.filter((f) => f === "all" || counts[f] > 0).map((f) => (
            <button
              key={f}
              type="button"
              onClick={() => onFilter(f)}
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
      <ul role="listbox" aria-label="Danh sách chương" className="fine-scrollbar flex-1 overflow-y-auto">
        {visible.length === 0 && <li className="p-4 text-sm text-muted-foreground">Không có chương nào khớp.</li>}
        {visible.map((row) => {
          const selected = selectedId === row.id;
          return (
            <li
              key={row.id}
              role="option"
              aria-selected={selected}
              tabIndex={0}
              onClick={() => onSelect(row.id)}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  onSelect(row.id);
                }
              }}
              className={cn(
                "flex cursor-pointer items-center gap-2.5 border-l-2 border-transparent px-3 py-2 text-sm hover:bg-accent/60",
                selected && "border-l-primary bg-accent",
              )}
            >
              <span className={cn("size-2 shrink-0 rounded-full", DOT[row.status])} aria-hidden />
              <span className="min-w-0 flex-1 truncate font-mono text-xs">{row.id}</span>
              {row.reviewRound > 0 && <span className="text-[11px] text-muted-foreground">soát {row.reviewRound}</span>}
              {row.warnings.length > 0 && (
                <span className="rounded-full bg-status-warning/15 px-1.5 text-[11px] font-medium text-status-warning">
                  {row.warnings.length}
                </span>
              )}
            </li>
          );
        })}
      </ul>
    </div>
  );
}
