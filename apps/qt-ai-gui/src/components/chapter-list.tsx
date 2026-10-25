import { Search } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";

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

/** Mỗi dòng cao cố định (h-9) nên tính được dòng nào đang thấy mà không phải đo từng dòng. */
const ROW_HEIGHT = 36;
/** Vẽ thêm chừng này dòng trên / dưới vùng thấy để cuộn nhanh không lộ khoảng trắng. */
const OVERSCAN = 10;
/** Chưa đo được khung danh sách (lần vẽ đầu, môi trường test) thì coi khung cao chừng này. */
const FALLBACK_HEIGHT = 800;

interface Props {
  rows: ChapterRow[];
  filter: ChapterFilter;
  query: string;
  selectedId?: string;
  onSelect: (id: string) => void;
  onFilter: (filter: ChapterFilter) => void;
  onQuery: (query: string) => void;
}

/**
 * Danh sách chương chỉ vẽ các dòng đang thấy (cộng vài dòng đệm): truyện 2.500 chương vẽ đủ là hơn 10 nghìn phần tử,
 * chặn giao diện ~220 ms lúc mở truyện và mỗi lần snapshot đổi khi đang dịch.
 */
export function ChapterList({ rows, filter, query, selectedId, onSelect, onFilter, onQuery }: Props) {
  const visible = useMemo(() => filterChapters(rows, filter, query), [rows, filter, query]);
  const counts = useMemo(() => countByFilter(rows), [rows]);
  // Số thứ tự theo danh sách đầy đủ (không đổi khi lọc) — dùng để gõ khoảng "dịch lại 120–180".
  const ordinal = useMemo(() => new Map(rows.map((row, index) => [row.id, index + 1])), [rows]);
  const width = String(rows.length).length;
  const list = useRef<HTMLUListElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [height, setHeight] = useState(0);
  const selectedIndex = useMemo(() => visible.findIndex((row) => row.id === selectedId), [visible, selectedId]);

  // ResizeObserver báo kích thước ngay khi bắt đầu theo dõi, rồi mỗi lần khung đổi cỡ.
  useEffect(() => {
    const element = list.current;
    if (!element) return;
    const observer = new ResizeObserver(() => setHeight(element.clientHeight));
    observer.observe(element);
    return () => observer.disconnect();
  }, []);

  // Chọn chương từ nơi khác (chip hổng ở toolbar, Trước/Sau trong reader) thì cuộn tới hàng đó; hàng đang thấy thì
  // không nhúc nhích. Chạy lại khi lọc/tìm đổi vì hàng có thể vừa hiện ra.
  useEffect(() => {
    const element = list.current;
    if (selectedIndex < 0 || !element) return;
    const top = selectedIndex * ROW_HEIGHT;
    const view = element.clientHeight || FALLBACK_HEIGHT;
    if (top < element.scrollTop) element.scrollTop = top;
    else if (top + ROW_HEIGHT > element.scrollTop + view) element.scrollTop = top + ROW_HEIGHT - view;
  }, [selectedIndex, filter, query]);

  const viewport = height || FALLBACK_HEIGHT;
  const first = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
  const last = Math.min(visible.length, Math.ceil((scrollTop + viewport) / ROW_HEIGHT) + OVERSCAN);
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
            placeholder="Tìm mã hoặc số thứ tự (#12)…"
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
      <ul
        ref={list}
        role="listbox"
        aria-label="Danh sách chương"
        className="fine-scrollbar flex-1 overflow-y-auto"
        onScroll={(e) => setScrollTop(e.currentTarget.scrollTop)}
      >
        {visible.length === 0 && <li className="p-4 text-sm text-muted-foreground">Không có chương nào khớp.</li>}
        {first > 0 && <li role="presentation" aria-hidden style={{ height: first * ROW_HEIGHT }} />}
        {visible.slice(first, last).map((row) => {
          const selected = selectedId === row.id;
          return (
            <li
              key={row.id}
              data-id={row.id}
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
                "flex h-9 cursor-pointer items-center gap-2.5 border-l-2 border-transparent px-3 text-sm hover:bg-accent/60",
                selected && "border-l-primary bg-accent",
              )}
            >
              <span className={cn("size-2 shrink-0 rounded-full", DOT[row.status])} aria-hidden />
              <span className="shrink-0 font-mono text-[11px] text-muted-foreground tabular-nums" style={{ minWidth: `${width + 1}ch` }}>
                #{ordinal.get(row.id)}
              </span>
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
        {last < visible.length && <li role="presentation" aria-hidden style={{ height: (visible.length - last) * ROW_HEIGHT }} />}
      </ul>
    </div>
  );
}
