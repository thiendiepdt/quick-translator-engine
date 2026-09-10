import { FolderOpen, FolderPlus, LibraryBig, Sparkles, Square, X } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { CreateStoryDialog } from "@/components/create-story-dialog";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  ApiError,
  appConfigSet,
  initStory,
  libraryList,
  openStory,
  pickFolder,
  recentSummaries,
  sessionStop,
} from "@/lib/api";
import { pathKey, samePath } from "@/lib/paths";
import type { Progress, RecentSummary } from "@/lib/types";
import { cn } from "@/lib/utils";
import { runningRoots, useStoryStore } from "@/store/story";

function StoryRow({
  item,
  busy,
  live,
  onOpen,
  onForget,
  onStop,
}: {
  item: RecentSummary;
  busy: boolean;
  /** Tiến độ live từ phiên đang chạy (đè lên số đọc từ đĩa). */
  live?: Progress;
  onOpen: () => void;
  onForget?: () => void;
  /** Có = truyện đang dịch, hiện nhãn + nút Dừng. */
  onStop?: () => void;
}) {
  const done = live?.done ?? item.done ?? 0;
  const percent = item.total ? Math.round((done / item.total) * 100) : 0;
  const uninitialized = item.total === null;
  return (
    <li className="flex min-w-0 items-stretch gap-1">
      <button
        type="button"
        disabled={busy}
        onClick={onOpen}
        className="flex min-w-0 flex-1 items-center gap-4 overflow-hidden rounded-lg border bg-card p-4 text-left transition-colors hover:bg-accent disabled:opacity-50"
      >
        <div className="min-w-0 flex-1">
          {item.name ? (
            <>
              <p className="flex items-center gap-2 truncate font-medium">
                <span className="truncate">{item.name}</span>
                {onStop && (
                  <span className="shrink-0 rounded-full bg-status-translating/15 px-2 py-0.5 text-[11px] font-medium text-status-translating">
                    Đang dịch{live?.current ? ` · ${live.current}` : ""}
                  </span>
                )}
              </p>
              <p className="truncate font-mono text-xs text-muted-foreground">{item.root}</p>
            </>
          ) : (
            <p
              className={cn("truncate font-mono text-sm", uninitialized ? "text-muted-foreground" : "text-foreground")}
              title={uninitialized ? "Chưa khởi tạo hoặc không đọc được — bấm để khởi tạo" : undefined}
            >
              {item.root}
            </p>
          )}
        </div>
        {!uninitialized && (
          <div className="w-32 shrink-0 text-right">
            <p className="text-sm tabular-nums">
              {done}/{item.total}
            </p>
            <div className="mt-1 h-1.5 overflow-hidden rounded-full bg-muted">
              <div className="h-full bg-status-done" style={{ width: `${percent}%` }} />
            </div>
          </div>
        )}
      </button>
      {onStop && (
        <Button
          variant="ghost"
          size="icon"
          disabled={busy}
          aria-label={`Dừng dịch ${item.root}`}
          title="Dừng phiên dịch của truyện này"
          onClick={onStop}
          className="h-auto shrink-0 self-stretch text-status-error hover:text-status-error"
        >
          <Square />
        </Button>
      )}
      {onForget && (
        <Button
          variant="ghost"
          size="icon"
          disabled={busy}
          aria-label={`Bỏ ${item.root} khỏi danh sách`}
          title="Bỏ khỏi danh sách (không xoá file)"
          onClick={onForget}
          className="h-auto shrink-0 self-stretch text-muted-foreground hover:text-foreground"
        >
          <X />
        </Button>
      )}
    </li>
  );
}

export function StoryPicker() {
  const recent = useStoryStore((s) => s.config?.recent ?? []);
  const libraryRoot = useStoryStore((s) => s.config?.libraryRoot ?? null);
  const open = useStoryStore((s) => s.openStory);
  const setConfig = useStoryStore((s) => s.setConfig);
  const sessions = useStoryStore((s) => s.sessions);
  const sessionRoots = useStoryStore((s) => s.roots);
  const progress = useStoryStore((s) => s.progress);
  const maxParallel = useStoryStore((s) => s.config?.maxParallel ?? 2);
  const applySessionEvent = useStoryStore((s) => s.applySessionEvent);
  const running = useMemo(() => runningRoots({ sessions, roots: sessionRoots }), [sessions, sessionRoots]);
  const isRunning = (root: string) => running.some((r) => samePath(r, root));
  const [summaries, setSummaries] = useState<RecentSummary[]>([]);
  const [library, setLibrary] = useState<RecentSummary[]>([]);
  const [libraryVersion, setLibraryVersion] = useState(0);
  const [pendingInit, setPendingInit] = useState<string | undefined>();
  /** Folder mở thủ công không phải truyện nhưng chứa truyện con → hỏi đặt làm thư viện thay vì init nhầm. */
  const [pendingLibrary, setPendingLibrary] = useState<{ root: string; count: number } | undefined>();
  const [creating, setCreating] = useState(false);
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

  useEffect(() => {
    let cancelled = false;
    if (!libraryRoot) return;
    libraryList()
      .then((list) => {
        if (!cancelled) setLibrary(list);
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [libraryRoot, libraryVersion]);

  async function tryOpen(root: string) {
    setBusy(true);
    try {
      open(await openStory(root));
    } catch (error) {
      if (error instanceof ApiError && error.kind === "story_not_found") {
        const inside = await libraryList(root).catch(() => []);
        const count = inside.filter((item) => item.total !== null).length;
        if (count > 0) setPendingLibrary({ root, count });
        else setPendingInit(root);
      } else toast.error(error instanceof Error ? error.message : "Không mở được truyện");
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

  /** Chỉ bỏ khỏi danh sách gần đây; không đụng file nào trong folder truyện. */
  async function forget(root: string) {
    const config = useStoryStore.getState().config;
    if (!config) return;
    try {
      setConfig(await appConfigSet({ ...config, recent: config.recent.filter((item) => item !== root) }));
      toast.success("Đã bỏ khỏi danh sách, không xoá file");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được cấu hình");
    }
  }

  async function pickAndOpen() {
    const root = await pickFolder("Chọn folder truyện");
    if (root) await tryOpen(root);
  }

  /** Dừng phiên của một truyện ngay tại dòng; store sẽ nhận event stopped từ Rust. */
  async function stop(root: string) {
    setBusy(true);
    try {
      await sessionStop(root);
      applySessionEvent(root, { type: "stopped", kind: "user_cancelled" });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không dừng được phiên");
    } finally {
      setBusy(false);
    }
  }

  const rowProps = (item: RecentSummary) =>
    isRunning(item.root)
      ? { live: progress[pathKey(item.root)], onStop: () => void stop(item.root) }
      : {};

  const saveLibrary = useCallback(
    async (root: string) => {
      const config = useStoryStore.getState().config;
      if (!config) return;
      try {
        setConfig(await appConfigSet({ ...config, libraryRoot: root }));
      } catch (error) {
        toast.error(error instanceof Error ? error.message : "Không lưu được cấu hình");
      }
    },
    [setConfig],
  );

  const chooseLibrary = useCallback(async () => {
    const root = await pickFolder("Chọn thư viện (folder cha chứa truyện)");
    if (root) await saveLibrary(root);
  }, [saveLibrary]);

  const libraryRows = libraryRoot ? library : [];
  const recentRows: RecentSummary[] = recent
    .filter((root) => !libraryRows.some((item) => samePath(item.root, root)))
    .map((root) => summaries.find((s) => s.root === root) ?? { root, name: null, done: null, total: null });

  return (
    <main className="fine-scrollbar flex h-full items-start justify-center overflow-auto p-8">
      <div className="w-full max-w-2xl">
        <header className="mb-6">
          <p className="text-xs font-medium tracking-widest text-primary uppercase">VNCVT AI Translator</p>
          <h1 className="mt-1 flex items-center gap-3 text-3xl font-semibold tracking-tight">
            Chọn truyện để dịch
            {running.length > 0 && (
              <span className="rounded-full bg-status-translating/15 px-2.5 py-1 text-xs font-medium text-status-translating">
                Đang dịch {running.length}/{maxParallel} truyện
              </span>
            )}
          </h1>
          <p className="mt-2 text-sm text-muted-foreground">
            Tạo truyện mới trong thư viện rồi thả file chương <code className="font-mono">.txt</code> vào app, hoặc mở
            một folder truyện có sẵn <code className="font-mono">raw/</code>. Folder mới sẽ được khởi tạo.
          </p>
        </header>
        <div className="flex flex-wrap gap-2">
          <Button
            size="lg"
            disabled={busy || !libraryRoot}
            title={libraryRoot ? undefined : "Chọn thư viện trước"}
            onClick={() => setCreating(true)}
          >
            <FolderPlus /> Tạo truyện mới
          </Button>
          <Button size="lg" variant="outline" disabled={busy} onClick={() => void pickAndOpen()}>
            <FolderOpen /> Mở folder truyện
          </Button>
        </div>

        <section className="mt-8">
          <div className="mb-2 flex items-center gap-2">
            <h2 className="text-xs font-medium tracking-widest text-muted-foreground uppercase">Thư viện</h2>
            {libraryRoot && (
              <>
                <span className="min-w-0 flex-1 truncate font-mono text-xs text-muted-foreground" title={libraryRoot}>
                  {libraryRoot}
                </span>
                <Button variant="ghost" size="sm" disabled={busy} onClick={() => void chooseLibrary()}>
                  Đổi
                </Button>
              </>
            )}
          </div>
          {libraryRoot ? (
            libraryRows.length === 0 ? (
              <p className="rounded-lg border border-dashed p-4 text-sm text-muted-foreground">
                Thư viện chưa có truyện nào — bấm Tạo truyện mới.
              </p>
            ) : (
              <ul className="grid grid-cols-[minmax(0,1fr)] gap-2">
                {libraryRows.map((item) => (
                  <StoryRow
                    key={item.root}
                    item={item}
                    busy={busy}
                    onOpen={() => void tryOpen(item.root)}
                    {...rowProps(item)}
                  />
                ))}
              </ul>
            )
          ) : (
            <button
              type="button"
              disabled={busy}
              onClick={() => void chooseLibrary()}
              className="flex w-full items-center gap-3 rounded-lg border border-dashed p-4 text-left text-sm transition-colors hover:bg-accent disabled:opacity-50"
            >
              <LibraryBig className="size-5 shrink-0 text-muted-foreground" />
              <span>
                <span className="font-medium">Chọn thư viện</span>
                <span className="block text-xs text-muted-foreground">
                  Một folder cha chứa mọi truyện. Truyện mới sẽ tạo vào đây và danh sách hiện ở chỗ này.
                </span>
              </span>
            </button>
          )}
        </section>

        {recentRows.length > 0 && (
          <section className="mt-8">
            <h2 className="mb-2 text-xs font-medium tracking-widest text-muted-foreground uppercase">Mở gần đây</h2>
            <ul className="grid grid-cols-[minmax(0,1fr)] gap-2">
              {recentRows.map((item) => (
                <StoryRow
                  key={item.root}
                  item={item}
                  busy={busy}
                  onOpen={() => void tryOpen(item.root)}
                  onForget={() => void forget(item.root)}
                  {...rowProps(item)}
                />
              ))}
            </ul>
          </section>
        )}
      </div>
      {libraryRoot && (
        <CreateStoryDialog
          libraryRoot={libraryRoot}
          open={creating}
          onOpenChange={setCreating}
          onCreated={(snapshot) => {
            setCreating(false);
            setLibraryVersion((v) => v + 1);
            open(snapshot);
          }}
        />
      )}
      <Dialog
        open={pendingLibrary !== undefined}
        onOpenChange={(value) => {
          if (!value) setPendingLibrary(undefined);
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Đây là thư viện?</DialogTitle>
            <DialogDescription>
              <code className="font-mono">{pendingLibrary?.root}</code> không phải folder truyện nhưng chứa{" "}
              {pendingLibrary?.count} truyện đã khởi tạo. Đặt làm thư viện để liệt kê và tạo truyện mới vào đây; hay
              vẫn khởi tạo chính folder này thành một truyện?
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setPendingLibrary(undefined)}>
              Bỏ
            </Button>
            <Button
              variant="outline"
              onClick={() => {
                setPendingInit(pendingLibrary?.root);
                setPendingLibrary(undefined);
              }}
            >
              Vẫn khởi tạo
            </Button>
            <Button
              onClick={() => {
                const root = pendingLibrary?.root;
                setPendingLibrary(undefined);
                if (root) void saveLibrary(root);
              }}
            >
              <LibraryBig /> Đặt làm thư viện
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
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
