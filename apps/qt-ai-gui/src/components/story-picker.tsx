import { FolderOpen, FolderPlus, FolderSync, LibraryBig, Play, RefreshCw, Search, Sparkles, Square, X } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { CreateStoryDialog } from "@/components/create-story-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
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
  libraryRescan,
  openStory,
  pickFolder,
  recentSummaries,
  sessionStart,
  sessionStop,
} from "@/lib/api";
import { storyLabel } from "@/hooks/use-session-events";
import { pathKey, samePath } from "@/lib/paths";
import { filterByQuery, PAGE_SIZE } from "@/lib/switcher";
import { clockLabel, engineLabel } from "@/lib/types";
import type { Progress, RecentSummary } from "@/lib/types";
import { cn } from "@/lib/utils";
import { runningRoots, useStoryStore } from "@/store/story";

/** Lỗi bắt đầu phiên mà truyện kế tiếp cũng sẽ gặp y hệt (hết chỗ, thiếu key, thiếu agy) — dừng cả lượt. */
const BATCH_STOP_KINDS = ["session_limit", "api_key_missing", "agy_missing"];
/** Số tên truyện liệt kê trong hộp xác nhận "Dịch tất cả". */
const BATCH_NAMES = 8;

const displayName = (item: { root: string; name: string | null }) => item.name?.trim() || storyLabel(item.root);

function StoryRow({
  item,
  busy,
  live,
  onOpen,
  onForget,
  onStop,
  waitUntil,
}: {
  item: RecentSummary;
  busy: boolean;
  /** Tiến độ live từ phiên đang chạy (đè lên số đọc từ đĩa). */
  live?: Progress;
  onOpen: () => void;
  onForget?: () => void;
  /** Có = truyện đang dịch, hiện nhãn + nút Dừng. */
  onStop?: () => void;
  /** Hết quota agy: phiên đang chờ tới giờ này (ms epoch) rồi tự chạy tiếp. */
  waitUntil?: number;
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
                  <span
                    className={cn(
                      "shrink-0 rounded-full px-2 py-0.5 text-[11px] font-medium",
                      waitUntil ? "bg-status-warning/15 text-status-warning" : "bg-status-translating/15 text-status-translating",
                    )}
                  >
                    {waitUntil
                      ? `Chờ quota · tự dịch tiếp lúc ${clockLabel(waitUntil)}`
                      : `Đang dịch${live?.current ? ` · ${live.current}` : ""}`}
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
  const maxParallel = useStoryStore((s) => s.config?.maxParallel ?? 20);
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
  const [refreshing, setRefreshing] = useState(false);
  const [scanning, setScanning] = useState(false);
  const [batchOpen, setBatchOpen] = useState(false);
  // Thư viện dài: lọc theo tên/folder và vẽ theo khúc PAGE_SIZE dòng để không lag khi hàng trăm truyện.
  const [query, setQuery] = useState("");
  const [limit, setLimit] = useState(PAGE_SIZE);

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

  /** Quét lại thư viện (folder copy tay vào sau khi mở app) và báo có truyện mới hay không. */
  async function refreshLibrary() {
    setRefreshing(true);
    try {
      const before = new Set(library.map((item) => pathKey(item.root)));
      const next = await libraryList();
      setLibrary(next);
      const fresh = next.filter((item) => !before.has(pathKey(item.root)));
      const gone = library.filter((item) => !next.some((row) => samePath(row.root, item.root))).length;
      const names = fresh.map((item) => item.name?.trim() || storyLabel(item.root));
      const parts = [
        fresh.length > 0 ? `${fresh.length} truyện mới: ${names.slice(0, 3).join(", ")}${names.length > 3 ? "…" : ""}` : "",
        gone > 0 ? `${gone} folder đã biến mất` : "",
      ].filter(Boolean);
      toast.message(parts.length > 0 ? `Thư viện: ${parts.join("; ")}` : "Thư viện không có truyện mới");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không quét được thư viện");
    } finally {
      setRefreshing(false);
    }
  }

  /** Quét raw/ của mọi truyện trong thư viện (raw import từ ngoài app) — như bấm Quét lại trong từng truyện. */
  async function rescanLibrary() {
    setScanning(true);
    try {
      const result = await libraryRescan();
      setLibrary(result.stories);
      const grown = result.changed.filter((item) => item.added > 0);
      const added = grown.reduce((sum, item) => sum + item.added, 0);
      const removed = result.changed.reduce((sum, item) => sum + item.removed, 0);
      const names = grown.map(displayName);
      const changes = [
        grown.length > 0
          ? `${grown.length} truyện có chương mới (+${added} chương): ${names.slice(0, 3).join(", ")}${names.length > 3 ? "…" : ""}`
          : "",
        removed > 0 ? `gỡ ${removed} chương raw đã mất` : "",
      ].filter(Boolean);
      const parts = [
        changes.length > 0 ? changes.join("; ") : "không truyện nào có chương mới",
        result.skippedRunning.length > 0 ? `bỏ qua ${result.skippedRunning.length} truyện đang dịch` : "",
      ].filter(Boolean);
      toast.message(`Quét thư viện: ${parts.join("; ")}`);
      if (result.failed.length > 0) {
        toast.error(
          `Không quét được ${result.failed.length} truyện: ${result.failed.slice(0, 3).join("; ")}${result.failed.length > 3 ? "…" : ""}`,
        );
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không quét được thư viện");
    } finally {
      setScanning(false);
    }
  }

  /**
   * Bắt đầu phiên cho các truyện trong `batch` rồi vào truyện đầu tiên. Truyện đầu được mở TRƯỚC khi phiên
   * của nó chạy: open_story quét raw/ và ghi state.json, không được chen vào lúc phiên đang ghi.
   */
  async function startBatch() {
    const targets = batch;
    setBatchOpen(false);
    if (targets.length === 0) return;
    setBusy(true);
    try {
      const first = await openStory(targets[0].root);
      const model = useStoryStore.getState().config?.model ?? undefined;
      const started: string[] = [];
      const failed: string[] = [];
      for (const item of targets) {
        try {
          await sessionStart(item.root, model);
          started.push(item.root);
        } catch (error) {
          failed.push(`${displayName(item)}: ${error instanceof Error ? error.message : "không bắt đầu được"}`);
          if (error instanceof ApiError && BATCH_STOP_KINDS.includes(error.kind)) break;
        }
      }
      if (failed.length > 0) {
        toast.error(failed.length === 1 ? failed[0] : `${failed.length} truyện không bắt đầu được — ${failed[0]}`);
      }
      if (started.length === 0) return;
      toast.success(`Đã bắt đầu dịch ${started.length} truyện`);
      open({ ...first, sessionRunning: started.some((root) => samePath(root, first.root)) });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không bắt đầu được");
    } finally {
      setBusy(false);
    }
  }

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

  const rowProps = (item: RecentSummary) => {
    if (!isRunning(item.root)) return {};
    const session = sessions[pathKey(item.root)];
    return {
      live: progress[pathKey(item.root)],
      onStop: () => void stop(item.root),
      waitUntil: session?.status === "running" ? session.waitUntil : undefined,
    };
  };

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

  const libraryRows = useMemo(() => (libraryRoot ? library : []), [libraryRoot, library]);
  const libraryFiltered = useMemo(() => filterByQuery(libraryRows, query), [libraryRows, query]);
  const libraryVisible = libraryFiltered.slice(0, limit);
  const showSearch = libraryRows.length > 8;
  // "Dịch tất cả": truyện thư viện còn chương chờ mà chưa có phiên, theo thứ tự danh sách; số bắt đầu được
  // lần này bị chặn bởi chỗ trống dưới giới hạn song song.
  const unfinished = useMemo(
    () => libraryRows.filter((item) => (item.pending ?? 0) > 0 && !running.some((root) => samePath(root, item.root))),
    [libraryRows, running],
  );
  const slots = Math.max(0, maxParallel - running.length);
  const batch = unfinished.slice(0, slots);
  const batchChapters = batch.reduce((sum, item) => sum + (item.pending ?? 0), 0);
  const engine = useStoryStore((s) => engineLabel(s.config));
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
          {libraryRoot && (
            <Button
              size="lg"
              variant="outline"
              disabled={busy || batch.length === 0}
              title={
                unfinished.length === 0
                  ? "Thư viện không có truyện nào còn chương chờ dịch mà chưa chạy"
                  : slots === 0
                    ? `Đang chạy đủ ${maxParallel} truyện — dừng bớt hoặc nâng "Số truyện dịch song song" trong Cài đặt`
                    : "Bắt đầu phiên dịch cho mọi truyện còn chương chờ, rồi mở truyện đầu tiên"
              }
              onClick={() => setBatchOpen(true)}
            >
              <Play /> Dịch tất cả truyện chưa xong{unfinished.length > 0 ? ` (${unfinished.length})` : ""}
            </Button>
          )}
        </div>

        <section className="mt-8">
          <div className="mb-2 flex items-center gap-2">
            <h2 className="text-xs font-medium tracking-widest text-muted-foreground uppercase">Thư viện</h2>
            {libraryRoot && (
              <>
                <span className="min-w-0 flex-1 truncate font-mono text-xs text-muted-foreground" title={libraryRoot}>
                  {libraryRoot}
                </span>
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={busy || refreshing}
                  title="Quét lại thư viện xem có truyện mới"
                  aria-label="Làm mới thư viện"
                  onClick={() => void refreshLibrary()}
                >
                  <RefreshCw className={cn(refreshing && "animate-spin")} /> Làm mới
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  disabled={busy || scanning}
                  title="Quét raw/ của mọi truyện trong thư viện, đưa chương mới vào hàng đợi (truyện đang dịch bỏ qua)"
                  aria-label="Quét chương mới cho cả thư viện"
                  onClick={() => void rescanLibrary()}
                >
                  <FolderSync className={cn(scanning && "animate-pulse")} /> Quét chương mới
                </Button>
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
              <>
                {showSearch && (
                  <div className="relative mb-2">
                    <Search className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
                    <Input
                      value={query}
                      onChange={(e) => {
                        setQuery(e.target.value);
                        setLimit(PAGE_SIZE);
                      }}
                      placeholder={`Tìm trong ${libraryRows.length} truyện…`}
                      aria-label="Tìm truyện trong thư viện"
                      className="pl-8"
                    />
                  </div>
                )}
                {libraryFiltered.length === 0 ? (
                  <p className="rounded-lg border border-dashed p-4 text-sm text-muted-foreground">Không có truyện khớp.</p>
                ) : (
                  <ul className="grid grid-cols-[minmax(0,1fr)] gap-2">
                    {libraryVisible.map((item) => (
                      <StoryRow
                        key={item.root}
                        item={item}
                        busy={busy}
                        onOpen={() => void tryOpen(item.root)}
                        {...rowProps(item)}
                      />
                    ))}
                  </ul>
                )}
                {libraryFiltered.length > libraryVisible.length && (
                  <div className="mt-2 flex justify-center">
                    <Button variant="outline" size="sm" onClick={() => setLimit((v) => v + PAGE_SIZE)}>
                      Xem thêm ({libraryFiltered.length - libraryVisible.length})
                    </Button>
                  </div>
                )}
              </>
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
      <Dialog open={batchOpen} onOpenChange={setBatchOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Dịch tất cả truyện chưa xong?</DialogTitle>
            <DialogDescription>
              Bắt đầu phiên dịch cho {batch.length} truyện ({batchChapters} chương còn chờ) bằng {engine}, rồi mở truyện
              đầu tiên.
            </DialogDescription>
          </DialogHeader>
          <ul className="fine-scrollbar max-h-48 list-disc overflow-auto pl-5 text-sm">
            {batch.slice(0, BATCH_NAMES).map((item) => (
              <li key={item.root}>
                {displayName(item)} <span className="text-muted-foreground tabular-nums">· {item.pending} chương</span>
              </li>
            ))}
            {batch.length > BATCH_NAMES && (
              <li className="text-muted-foreground">… và {batch.length - BATCH_NAMES} truyện nữa</li>
            )}
          </ul>
          {unfinished.length > batch.length && (
            <p role="status" className="text-sm text-status-warning">
              Còn {unfinished.length - batch.length} truyện chưa xong không được bắt đầu lần này: giới hạn {maxParallel}{" "}
              truyện dịch song song, đang chạy {running.length}. Bấm lại khi có truyện xong, hoặc nâng giới hạn trong Cài
              đặt.
            </p>
          )}
          <DialogFooter>
            <Button variant="outline" onClick={() => setBatchOpen(false)}>
              Bỏ
            </Button>
            <Button disabled={busy || batch.length === 0} onClick={() => void startBatch()}>
              <Play /> Bắt đầu dịch {batch.length} truyện
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
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
