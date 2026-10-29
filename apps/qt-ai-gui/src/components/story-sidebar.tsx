import { ChevronDown, LayoutGrid, TriangleAlert } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { StorySwitcherDialog } from "@/components/story-switcher-dialog";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { openStory as openStoryApi } from "@/lib/api";
import { dockEntries, initials, issuesLabel } from "@/lib/dock";
import { samePath } from "@/lib/paths";
import { clockLabel, stopReasonBrief, stopReasonLabel } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

/** Số ô hiện ban đầu; dài hơn thì nút "+N" nạp thêm từng đợt để cột không lag. */
export const SIDEBAR_PAGE = 12;

/**
 * Cột ngoài cùng bên phải: mọi truyện đã mở trong phiên app, đang dịch xếp trên. Bấm ô để chuyển
 * truyện, giữ nguyên trang đang xem. Nút lưới (Ctrl+K) mở dialog Chuyển truyện.
 */
export function StorySidebar() {
  const root = useStoryStore((s) => s.root);
  const opened = useStoryStore((s) => s.opened);
  const sessions = useStoryStore((s) => s.sessions);
  const roots = useStoryStore((s) => s.roots);
  const names = useStoryStore((s) => s.names);
  const progress = useStoryStore((s) => s.progress);
  const issues = useStoryStore((s) => s.issues);
  const switchStory = useStoryStore((s) => s.switchStory);
  const [switcherOpen, setSwitcherOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [limit, setLimit] = useState(SIDEBAR_PAGE);
  const entries = useMemo(
    () => dockEntries({ root, opened, sessions, roots, names, progress, issues }),
    [root, opened, sessions, roots, names, progress, issues],
  );
  const visible = entries.slice(0, limit);
  const hidden = entries.length - visible.length;

  const switchTo = useCallback(
    async (target: string) => {
      if (root && samePath(root, target)) return;
      setBusy(true);
      try {
        switchStory(await openStoryApi(target));
      } catch (error) {
        toast.error(error instanceof Error ? error.message : "Không mở được truyện");
      } finally {
        setBusy(false);
      }
    },
    [root, switchStory],
  );

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setSwitcherOpen((value) => !value);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  return (
    <aside className="flex h-full w-48 shrink-0 flex-col border-l bg-card">
      <div className="flex items-center justify-between border-b px-3 py-2">
        <span className="text-xs font-medium tracking-widest text-muted-foreground uppercase">Phiên này</span>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              aria-label="Chuyển truyện (Ctrl+K)"
              onClick={() => setSwitcherOpen(true)}
              className="size-8 rounded-lg text-muted-foreground hover:text-foreground"
            >
              <LayoutGrid className="size-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="left">Chuyển truyện (Ctrl+K)</TooltipContent>
        </Tooltip>
      </div>
      <div
        className="fine-scrollbar flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto p-2"
        role="list"
        aria-label="Truyện đã mở trong phiên này"
      >
        {visible.length === 0 && (
          <p className="px-1 py-2 text-xs text-muted-foreground">Truyện mở trong phiên này sẽ hiện ở đây để chuyển nhanh.</p>
        )}
        {visible.map((item) => {
          const label = item.name || item.root;
          const folder = item.root.replace(/[\\/]+$/, "").split(/[\\/]/).pop() ?? item.root;
          const detail = item.retry && item.waitUntil
            ? `Lỗi tạm (${stopReasonBrief(item.retry.reason)}) · thử lại lúc ${clockLabel(item.waitUntil)} (lần ${item.retry.attempt}/${item.retry.max})`
            : item.waitUntil
            ? `Chờ quota agy · tự dịch tiếp lúc ${clockLabel(item.waitUntil)}`
            : item.stop
              ? `Đã dừng${item.stop.at ? ` lúc ${clockLabel(item.stop.at)}` : ""}: ${stopReasonLabel(item.stop.reason)}`
            : item.running
              ? `Đang dịch${item.currentChapter ? ` · ${item.currentChapter}` : ""}${item.percent !== undefined ? ` · ${item.percent}%` : ""}`
              : item.current
                ? "Đang mở"
                : "Đã mở trong phiên này";
          // Chương bỏ qua/lỗi không tự được dịch lại — nhắc ngay trên thẻ để khỏi phải mở từng truyện ra xem.
          const flagged = issuesLabel(item);
          const attention = flagged ? `Cần xem: ${flagged}` : "";
          return (
            <Tooltip key={item.root}>
              <TooltipTrigger asChild>
                <button
                  type="button"
                  role="listitem"
                  aria-label={`${label} — ${detail}${attention ? ` — ${attention}` : ""}`}
                  aria-current={item.current ? "true" : undefined}
                  disabled={busy}
                  onClick={() => void switchTo(item.root)}
                  className={cn(
                    "flex w-full shrink-0 items-start gap-2 rounded-lg border p-2 text-left transition-colors disabled:opacity-50",
                    item.current
                      ? "border-primary bg-primary/10"
                      : "border-transparent bg-muted/60 hover:bg-accent",
                  )}
                >
                  <span
                    className={cn(
                      "relative flex size-8 shrink-0 items-center justify-center rounded-md text-[11px] font-semibold",
                      item.current ? "bg-primary text-primary-foreground" : "bg-card text-muted-foreground",
                    )}
                  >
                    {initials(item.name, item.root)}
                    {item.running && (
                      <span
                        className="absolute -top-0.5 -right-0.5 size-2 rounded-full bg-status-translating ring-2 ring-card animate-pulse"
                        aria-hidden
                      />
                    )}
                  </span>
                  <span className="flex min-w-0 flex-1 flex-col gap-1">
                    <span className={cn("line-clamp-2 text-xs leading-snug font-medium", item.current && "text-primary")}>
                      {item.name || folder}
                    </span>
                    {item.running ? (
                      <span className="flex items-center gap-1.5">
                        <span className="h-1 flex-1 overflow-hidden rounded-full bg-muted">
                          <span className="block h-full bg-status-done" style={{ width: `${item.percent ?? 0}%` }} />
                        </span>
                        {item.waitUntil ? (
                          <span className="shrink-0 text-[10px] tabular-nums text-status-warning">
                            {item.retry ? "thử lại" : "chờ"} {clockLabel(item.waitUntil)}
                          </span>
                        ) : (
                          <span className="shrink-0 text-[10px] tabular-nums text-status-translating">
                            {item.percent !== undefined ? `${item.percent}%` : "…"}
                          </span>
                        )}
                      </span>
                    ) : (
                      <span className="truncate font-mono text-[10px] text-muted-foreground">{folder}</span>
                    )}
                    {item.stop && (
                      <span className="flex items-center gap-1 text-[10px] leading-none font-medium text-status-error">
                        <TriangleAlert className="size-3 shrink-0" aria-hidden />
                        <span className="truncate">
                          Dừng{item.stop.at ? ` ${clockLabel(item.stop.at)}` : ""}: {stopReasonBrief(item.stop.reason)}
                        </span>
                      </span>
                    )}
                    {flagged && (
                      <span
                        className={cn(
                          "flex items-center gap-1 text-[10px] leading-none font-medium",
                          item.error > 0 ? "text-status-error" : "text-status-warning",
                        )}
                      >
                        <TriangleAlert className="size-3 shrink-0" aria-hidden />
                        <span className="truncate">{flagged}</span>
                      </span>
                    )}
                  </span>
                </button>
              </TooltipTrigger>
              <TooltipContent side="left">
                <p className="font-medium">{label}</p>
                <p className="text-xs opacity-80">{detail}</p>
                {attention && (
                  <p className="text-xs">{attention} — mở truyện, lọc Bỏ qua/Lỗi để dịch lại hoặc xử lý.</p>
                )}
                <p className="font-mono text-[10px] opacity-60">{item.root}</p>
              </TooltipContent>
            </Tooltip>
          );
        })}
        {hidden > 0 && (
          <Button
            variant="ghost"
            size="sm"
            aria-label={`Xem thêm ${hidden} truyện`}
            onClick={() => setLimit((value) => value + SIDEBAR_PAGE)}
            className="h-8 shrink-0 text-xs text-muted-foreground"
          >
            Xem thêm {hidden}
            <ChevronDown className="size-3" />
          </Button>
        )}
      </div>
      <StorySwitcherDialog
        open={switcherOpen}
        onOpenChange={setSwitcherOpen}
        onPick={(target) => {
          setSwitcherOpen(false);
          void switchTo(target);
        }}
      />
    </aside>
  );
}
