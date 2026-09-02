import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { STATUS_LABELS, type ChapterRow, type ChapterStatus } from "@/lib/types";
import { cn } from "@/lib/utils";

export type StatusFilter = ChapterStatus | "all";

export function filterRows(rows: ChapterRow[], filter: StatusFilter): ChapterRow[] {
  return filter === "all" ? rows : rows.filter((row) => row.status === filter);
}

const STATUS_VARIANT: Record<ChapterStatus, "default" | "secondary" | "destructive" | "outline"> = {
  queued: "outline",
  translating: "default",
  done: "secondary",
  error: "destructive",
  skipped: "outline",
};

interface Props {
  rows: ChapterRow[];
  filter: StatusFilter;
  selectedId?: string;
  onSelect: (id: string) => void;
  onFilter: (filter: StatusFilter) => void;
}

export function ChapterTable({ rows, filter, selectedId, onSelect, onFilter }: Props) {
  const visible = filterRows(rows, filter);
  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between gap-2 border-b p-2">
        <span className="text-xs text-muted-foreground">
          {visible.length}/{rows.length} chương
        </span>
        <Select value={filter} onValueChange={(value) => onFilter(value as StatusFilter)}>
          <SelectTrigger className="h-8 w-40" aria-label="Lọc trạng thái">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">Tất cả</SelectItem>
            {(Object.keys(STATUS_LABELS) as ChapterStatus[]).map((status) => (
              <SelectItem key={status} value={status}>
                {STATUS_LABELS[status]}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
      <ScrollArea className="flex-1">
        <ul>
          {visible.map((row) => (
            <li key={row.id}>
              <button
                type="button"
                onClick={() => onSelect(row.id)}
                className={cn(
                  "flex w-full items-center gap-2 border-b px-3 py-2 text-left text-sm hover:bg-accent",
                  selectedId === row.id && "bg-accent",
                )}
              >
                <span className="w-28 shrink-0 truncate font-mono text-xs">{row.id}</span>
                <Badge variant={STATUS_VARIANT[row.status]}>{STATUS_LABELS[row.status]}</Badge>
                {row.warnings.length > 0 && (
                  <Badge variant="outline" className="text-amber-700">
                    {row.warnings.length} cảnh báo
                  </Badge>
                )}
                {row.reviewRound > 0 && <span className="text-xs text-muted-foreground">soát {row.reviewRound}</span>}
              </button>
            </li>
          ))}
        </ul>
      </ScrollArea>
    </div>
  );
}
