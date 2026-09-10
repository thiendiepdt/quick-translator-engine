import { ChevronDown, LayoutGrid } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { StorySwitcherDialog } from "@/components/story-switcher-dialog";
import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { openStory as openStoryApi } from "@/lib/api";
import { dockEntries, initials } from "@/lib/dock";
import { samePath } from "@/lib/paths";
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
  const switchStory = useStoryStore((s) => s.switchStory);
  const [switcherOpen, setSwitcherOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [limit, setLimit] = useState(SIDEBAR_PAGE);
  const entries = useMemo(
    () => dockEntries({ root, opened, sessions, roots, names, progress }),
    [root, opened, sessions, roots, names, progress],
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
    <aside className="flex h-full w-14 shrink-0 flex-col items-center border-l bg-card py-3">
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Chuyển truyện (Ctrl+K)"
            onClick={() => setSwitcherOpen(true)}
            className="mb-2 size-10 rounded-lg text-muted-foreground hover:text-foreground"
          >
            <LayoutGrid className="size-5" />
          </Button>
        </TooltipTrigger>
        <TooltipContent side="left">Chuyển truyện (Ctrl+K)</TooltipContent>
      </Tooltip>
      <div
        className="fine-scrollbar flex min-h-0 flex-1 flex-col items-center gap-1.5 overflow-y-auto"
        role="list"
        aria-label="Truyện đã mở trong phiên này"
      >
        {visible.map((item) => {
          const label = item.name || item.root;
          const detail = item.running
            ? `Đang dịch${item.currentChapter ? ` · ${item.currentChapter}` : ""}${item.percent !== undefined ? ` · ${item.percent}%` : ""}`
            : item.current
              ? "Đang mở"
              : "Đã mở trong phiên này";
          return (
            <Tooltip key={item.root}>
              <TooltipTrigger asChild>
                <button
                  type="button"
                  role="listitem"
                  aria-label={`${label} — ${detail}`}
                  aria-current={item.current ? "true" : undefined}
                  disabled={busy}
                  onClick={() => void switchTo(item.root)}
                  className={cn(
                    "relative flex size-10 shrink-0 items-center justify-center rounded-lg border text-xs font-semibold transition-colors",
                    item.current
                      ? "border-primary bg-primary/10 text-primary"
                      : "border-transparent bg-muted text-muted-foreground hover:bg-accent hover:text-foreground",
                  )}
                  style={
                    item.running && item.percent !== undefined
                      ? { backgroundImage: `conic-gradient(var(--status-done) ${item.percent}%, transparent 0)` }
                      : undefined
                  }
                >
                  <span className={cn("flex size-8 items-center justify-center rounded-md", item.running && "bg-card")}>
                    {initials(item.name, item.root)}
                  </span>
                  {item.running && (
                    <span
                      className="absolute -top-0.5 -right-0.5 size-2 rounded-full bg-status-translating ring-2 ring-card animate-pulse"
                      aria-hidden
                    />
                  )}
                </button>
              </TooltipTrigger>
              <TooltipContent side="left">
                <p className="font-medium">{label}</p>
                <p className="text-xs opacity-80">{detail}</p>
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
            className="h-8 w-10 shrink-0 px-0 text-xs text-muted-foreground"
          >
            +{hidden}
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
