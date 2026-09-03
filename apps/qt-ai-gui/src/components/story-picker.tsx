import { FolderOpen, Sparkles } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ApiError, initStory, openStory, pickFolder, recentSummaries } from "@/lib/api";
import type { RecentSummary } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

export function StoryPicker() {
  const recent = useStoryStore((s) => s.config?.recent ?? []);
  const open = useStoryStore((s) => s.openStory);
  const [summaries, setSummaries] = useState<RecentSummary[]>([]);
  const [pendingInit, setPendingInit] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let cancelled = false;
    if (recent.length === 0) return;
    recentSummaries()
      .then((list) => {
        if (!cancelled) setSummaries(list);
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [recent]);

  async function tryOpen(root: string) {
    setBusy(true);
    try {
      open(await openStory(root));
    } catch (error) {
      if (error instanceof ApiError && error.kind === "story_not_found") setPendingInit(root);
      else toast.error(error instanceof Error ? error.message : "Không mở được truyện");
    } finally {
      setBusy(false);
    }
  }

  async function confirmInit() {
    if (!pendingInit) return;
    setBusy(true);
    try {
      open(await initStory(pendingInit));
      toast.success("Đã khởi tạo folder truyện");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không khởi tạo được");
    } finally {
      setBusy(false);
      setPendingInit(undefined);
    }
  }

  async function pickAndOpen() {
    const root = await pickFolder("Chọn folder truyện");
    if (root) await tryOpen(root);
  }

  const rows: RecentSummary[] = recent.map(
    (root) => summaries.find((s) => s.root === root) ?? { root, name: null, done: null, total: null },
  );

  return (
    <main className="fine-scrollbar flex h-full items-start justify-center overflow-auto p-8">
      <div className="w-full max-w-2xl">
        <header className="mb-8">
          <p className="text-xs font-medium tracking-widest text-primary uppercase">QT AI Translator</p>
          <h1 className="mt-1 text-3xl font-semibold tracking-tight">Chọn truyện để dịch</h1>
          <p className="mt-2 text-sm text-muted-foreground">
            Folder truyện có thư mục <code className="font-mono">raw/</code> chứa các chương{" "}
            <code className="font-mono">.txt</code>. Folder mới sẽ được khởi tạo.
          </p>
        </header>
        <Button size="lg" disabled={busy} onClick={() => void pickAndOpen()}>
          <FolderOpen /> Mở folder truyện
        </Button>
        {rows.length > 0 && (
          <section className="mt-8">
            <h2 className="mb-2 text-xs font-medium tracking-widest text-muted-foreground uppercase">Mở gần đây</h2>
            <ul className="grid grid-cols-[minmax(0,1fr)] gap-2">
              {rows.map((item) => {
                const percent = item.total ? Math.round(((item.done ?? 0) / item.total) * 100) : 0;
                const broken = item.total === null;
                return (
                  <li key={item.root} className="min-w-0">
                    <button
                      type="button"
                      disabled={busy}
                      onClick={() => void tryOpen(item.root)}
                      className="flex w-full items-center gap-4 overflow-hidden rounded-lg border bg-card p-4 text-left transition-colors hover:bg-accent disabled:opacity-50"
                    >
                      <div className="min-w-0 flex-1">
                        {item.name ? (
                          <>
                            <p className="truncate font-medium">{item.name}</p>
                            <p className="truncate font-mono text-xs text-muted-foreground">{item.root}</p>
                          </>
                        ) : (
                          <p
                            className={cn(
                              "truncate font-mono text-sm",
                              broken ? "text-muted-foreground" : "text-foreground",
                            )}
                            title={broken ? "Không đọc được folder này" : undefined}
                          >
                            {item.root}
                          </p>
                        )}
                      </div>
                      {!broken && (
                        <div className="w-32 shrink-0 text-right">
                          <p className="text-sm tabular-nums">
                            {item.done}/{item.total}
                          </p>
                          <div className="mt-1 h-1.5 overflow-hidden rounded-full bg-muted">
                            <div className="h-full bg-status-done" style={{ width: `${percent}%` }} />
                          </div>
                        </div>
                      )}
                    </button>
                  </li>
                );
              })}
            </ul>
          </section>
        )}
      </div>
      <Dialog
        open={pendingInit !== undefined}
        onOpenChange={(value) => {
          if (!value) setPendingInit(undefined);
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Khởi tạo folder truyện?</DialogTitle>
            <DialogDescription>
              <code className="font-mono">{pendingInit}</code> chưa có <code>state.json</code>. Khởi tạo sẽ tạo{" "}
              <code>story.json</code>, <code>state.json</code>, <code>AGENTS.md</code> và đưa mọi chương trong{" "}
              <code>raw/</code> vào hàng đợi. Không đụng file gốc.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setPendingInit(undefined)}>
              Bỏ
            </Button>
            <Button disabled={busy} onClick={() => void confirmInit()}>
              <Sparkles /> Khởi tạo
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </main>
  );
}
