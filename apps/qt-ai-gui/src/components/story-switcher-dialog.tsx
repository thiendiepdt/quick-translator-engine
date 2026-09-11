import { Search } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { libraryList, recentSummaries } from "@/lib/api";
import { filterCards, mergeCandidates, PAGE_SIZE } from "@/lib/switcher";
import type { RecentSummary } from "@/lib/types";
import { cn } from "@/lib/utils";
import { runningRoots, useStoryStore } from "@/store/story";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onPick: (root: string) => void;
}

export function StorySwitcherDialog({ open, onOpenChange, onPick }: Props) {
  const currentRoot = useStoryStore((s) => s.root);
  const sessions = useStoryStore((s) => s.sessions);
  const roots = useStoryStore((s) => s.roots);
  const names = useStoryStore((s) => s.names);
  const progress = useStoryStore((s) => s.progress);
  const [library, setLibrary] = useState<RecentSummary[]>([]);
  const [recent, setRecent] = useState<RecentSummary[]>([]);
  const [query, setQuery] = useState("");
  const [limit, setLimit] = useState(PAGE_SIZE);

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    Promise.all([libraryList().catch(() => []), recentSummaries().catch(() => [])])
      .then(([lib, rec]) => {
        if (cancelled) return;
        setLibrary(lib);
        setRecent(rec);
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [open]);

  const cards = useMemo(
    () =>
      mergeCandidates({
        running: runningRoots({ sessions, roots }),
        names,
        progress,
        currentRoot,
        library,
        recent,
      }),
    [sessions, roots, names, progress, currentRoot, library, recent],
  );
  const filtered = useMemo(() => filterCards(cards, query), [cards, query]);
  const visible = filtered.slice(0, limit);

  return (
    <Dialog
      open={open}
      onOpenChange={(value) => {
        if (!value) {
          setQuery("");
          setLimit(PAGE_SIZE);
        }
        onOpenChange(value);
      }}
    >
      <DialogContent className="w-[min(96vw,64rem)]">
        <DialogHeader>
          <DialogTitle>Chuyển truyện</DialogTitle>
          <DialogDescription>Đang dịch và đang mở lên đầu, rồi thư viện và gần đây. Giữ nguyên trang đang xem.</DialogDescription>
        </DialogHeader>
        <div className="relative">
          <Search className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            autoFocus
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setLimit(PAGE_SIZE);
            }}
            placeholder="Tìm theo tên hoặc folder…"
            aria-label="Tìm truyện"
            className="pl-8"
          />
        </div>
        <div className="fine-scrollbar max-h-[60vh] overflow-y-auto">
          {visible.length === 0 ? (
            <p className="p-6 text-center text-sm text-muted-foreground">Không có truyện nào khớp.</p>
          ) : (
            <ul className="grid grid-cols-1 gap-2 sm:grid-cols-2 lg:grid-cols-3" aria-label="Danh sách truyện">
              {visible.map((card) => {
                const percent = card.total ? Math.round(((card.done ?? 0) / card.total) * 100) : 0;
                return (
                  <li key={card.root}>
                    <button
                      type="button"
                      disabled={!card.openable}
                      aria-current={card.current ? "true" : undefined}
                      onClick={() => onPick(card.root)}
                      className={cn(
                        "flex w-full flex-col gap-1.5 rounded-lg border bg-card p-3 text-left transition-colors hover:bg-accent disabled:cursor-not-allowed disabled:opacity-50",
                        card.current && "border-primary",
                      )}
                    >
                      <div className="flex items-center gap-2">
                        <span className="min-w-0 flex-1 truncate font-medium">
                          {card.name ?? card.root.replace(/[\\/]+$/, "").split(/[\\/]/).pop()}
                        </span>
                        {card.running && (
                          <span className="shrink-0 rounded-full bg-status-translating/15 px-1.5 text-[11px] font-medium text-status-translating">
                            Đang dịch
                          </span>
                        )}
                        {card.current && (
                          <span className="shrink-0 rounded-full bg-primary/10 px-1.5 text-[11px] font-medium text-primary">Đang mở</span>
                        )}
                      </div>
                      <span className="truncate font-mono text-[11px] text-muted-foreground">{card.root}</span>
                      {card.total !== null ? (
                        <div className="flex items-center gap-2">
                          <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-muted">
                            <div className="h-full bg-status-done" style={{ width: `${percent}%` }} />
                          </div>
                          <span className="text-xs tabular-nums">
                            {card.done ?? 0}/{card.total}
                          </span>
                        </div>
                      ) : (
                        <span className="text-xs text-muted-foreground">Chưa khởi tạo — mở từ màn chọn truyện</span>
                      )}
                    </button>
                  </li>
                );
              })}
            </ul>
          )}
          {filtered.length > visible.length && (
            <div className="flex justify-center pt-3">
              <Button variant="outline" size="sm" onClick={() => setLimit((v) => v + PAGE_SIZE)}>
                Xem thêm ({filtered.length - visible.length})
              </Button>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
