import { LoaderCircle, LockOpen, Skull, Square } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { toast } from "sonner";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import {
  agyKill,
  lockRemove,
  processOverview,
  sessionStop,
  storyKill,
  storySnapshot,
  type AgyProcess,
  type ProcessOverview,
  type StoryProcesses,
} from "@/lib/api";
import { pathKey, samePath } from "@/lib/paths";
import { cn } from "@/lib/utils";
import { isRunning, useStoryStore } from "@/store/story";

/** Bảng tự làm mới trong lúc mở; một lần quét ở Rust ~10–20 ms. */
const POLL_MS = 2000;

const KIND_LABELS: Record<AgyProcess["kind"], string> = {
  translate: "dịch truyện",
  setup: "AI điền hồ sơ",
  other: "lệnh khác",
};

function lockLabel(lock: NonNullable<StoryProcesses["lock"]>): string {
  switch (lock.holder) {
    case "session":
      return "phiên đang chạy của app này giữ";
    case "app":
      return "app này giữ nhưng phiên đã dừng — chờ phiên tự thả, kẹt lâu thì Gỡ khoá";
    case "other":
      return /vncvt/i.test(lock.holderName ?? "")
        ? "một cửa sổ VNCVT khác đang dịch truyện này"
        : `tiến trình khác còn sống giữ (${lock.holderName ?? "không rõ tên"})`;
    case "stale":
      return "khoá mồ côi (app trước bị tắt ngang) — bấm Dịch là tự gỡ";
  }
}

function since(startedAt: number, now: number): string {
  const minutes = Math.max(0, Math.floor((now - startedAt) / 60_000));
  return minutes < 60 ? `${minutes} phút` : `${Math.floor(minutes / 60)} giờ ${minutes % 60} phút`;
}

function AgyLine({ agy, now, onKill, busy }: { agy: AgyProcess; now: number; onKill?: () => void; busy?: boolean }) {
  return (
    <li className="flex items-center gap-2 text-xs">
      <span className="font-mono tabular-nums">agy PID {agy.pid}</span>
      <span className="text-muted-foreground">
        · {KIND_LABELS[agy.kind]} · {Math.round(agy.memory / 1048576)} MB · CPU {agy.cpu.toFixed(1)}% · chạy {since(agy.startedAt, now)}
      </span>
      {!agy.owned && (
        <Badge variant="outline" className="border-status-warning/50 text-status-warning">
          không thuộc app đang mở
        </Badge>
      )}
      {onKill && (
        <Button size="sm" variant="ghost" className="ml-auto h-7 text-destructive" disabled={busy} onClick={onKill}>
          <Skull /> Kill
        </Button>
      )}
    </li>
  );
}

/** Mở từ thanh bên trái, màn chọn truyện, hoặc toolbar truyện (cuộn tới truyện đó). */
export function ProcessesDialog() {
  const processes = useStoryStore((s) => s.processes);
  const closeProcesses = useStoryStore((s) => s.closeProcesses);
  return processes ? <ProcessesContent focus={processes.focus} onClose={closeProcesses} /> : null;
}

function ProcessesContent({ focus, onClose }: { focus: string | null; onClose: () => void }) {
  const names = useStoryStore((s) => s.names);
  const [overview, setOverview] = useState<ProcessOverview>();
  const [error, setError] = useState<string>();
  const [busy, setBusy] = useState<Record<string, boolean>>({});
  const [now, setNow] = useState(() => Date.now());
  const inFlight = useRef(false);
  const focusRef = useRef<HTMLLIElement>(null);
  const scrolled = useRef(false);

  useEffect(() => {
    let alive = true;
    const tick = () => {
      if (inFlight.current) return;
      inFlight.current = true;
      processOverview()
        .then((next) => {
          if (!alive) return;
          setOverview(next);
          setNow(Date.now());
          setError(undefined);
        })
        .catch((e: unknown) => alive && setError(e instanceof Error ? e.message : "Không đọc được tiến trình"))
        .finally(() => {
          inFlight.current = false;
        });
    };
    tick();
    const timer = window.setInterval(tick, POLL_MS);
    return () => {
      alive = false;
      window.clearInterval(timer);
    };
  }, []);

  useEffect(() => {
    if (overview && !scrolled.current && focusRef.current) {
      scrolled.current = true;
      focusRef.current.scrollIntoView?.({ block: "nearest" });
    }
  }, [overview]);

  async function act(key: string, action: () => Promise<ProcessOverview>, done: string) {
    setBusy((b) => ({ ...b, [key]: true }));
    try {
      setOverview(await action());
      toast.success(done);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Không làm được");
    } finally {
      setBusy((b) => ({ ...b, [key]: false }));
    }
  }

  /** Phiên vừa bị dừng/kill: store về "đã dừng" ngay (event Stopped có thể tới muộn khi agy treo), nạp lại snapshot. */
  async function settle(root: string) {
    const state = useStoryStore.getState();
    if (isRunning(state, root)) state.applySessionEvent(root, { type: "stopped", kind: "user_cancelled" });
    if (state.root && samePath(state.root, root)) {
      try {
        state.setSnapshot(await storySnapshot(root));
      } catch {
        // Nạp lại hỏng không sao: lần sau mở truyện là đúng.
      }
    }
  }

  const stop = (story: StoryProcesses) =>
    act(
      story.root,
      async () => {
        await sessionStop(story.root);
        await settle(story.root);
        return processOverview();
      },
      "Đã dừng phiên",
    );

  const kill = (story: StoryProcesses) => {
    if (
      !window.confirm(
        `Kill "${title(story)}"? agy bị giết ngay, không chờ hết lượt. Chương đang dịch dở giữ trạng thái đang dịch — bấm Dịch lại là dịch tiếp chương đó.`,
      )
    )
      return;
    void act(
      story.root,
      async () => {
        const next = await storyKill(story.root);
        await settle(story.root);
        return next;
      },
      "Đã kill",
    );
  };

  const unlock = (story: StoryProcesses) => {
    const lock = story.lock;
    if (!lock) return;
    const warn =
      lock.holder === "other"
        ? " Tiến trình giữ khoá vẫn còn sống — nếu đó là một cửa sổ app khác đang dịch, hai phiên sẽ ghi đè nhau."
        : "";
    if (!window.confirm(`Gỡ khoá phiên của "${title(story)}"?${warn}`)) return;
    void act(story.root, () => lockRemove(story.root), "Đã gỡ khoá");
  };

  /** Khoá không còn ai dùng: mồ côi hoặc app này giữ sau khi phiên đã dừng. Khoá do tiến trình khác còn sống giữ
   * phải gỡ từng cái (có cảnh báo riêng). */
  const freeable = (overview?.stories ?? []).filter(
    (story) => !story.running && (story.lock?.holder === "stale" || story.lock?.holder === "app"),
  );

  const unlockAll = () => {
    if (!window.confirm(`Gỡ khoá phiên của ${freeable.length} truyện không còn phiên nào chạy?`)) return;
    const roots = freeable.map((story) => story.root);
    void act(
      "all",
      async () => {
        let next: ProcessOverview | undefined;
        for (const root of roots) next = await lockRemove(root);
        return next ?? processOverview();
      },
      `Đã gỡ ${roots.length} khoá`,
    );
  };

  function title(story: StoryProcesses): string {
    return names[pathKey(story.root)] || story.name;
  }

  const stories = overview?.stories ?? [];
  const focusMissing = focus !== null && overview !== undefined && !stories.some((s) => samePath(s.root, focus));

  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="flex max-h-[85vh] flex-col sm:max-w-3xl">
        <DialogHeader>
          <DialogTitle>Tiến trình</DialogTitle>
          <DialogDescription>
            Phiên dịch, agy và khoá phiên của từng truyện, tự làm mới mỗi 2 giây. Dùng Kill khi bấm Dừng mà treo, Gỡ khoá
            khi truyện báo “đang có phiên khác chạy” mà không có phiên nào.
          </DialogDescription>
        </DialogHeader>
        {freeable.length > 1 && (
          <div className="flex justify-end">
            <Button size="sm" variant="outline" disabled={busy.all} onClick={unlockAll}>
              {busy.all ? <LoaderCircle className="animate-spin" /> : <LockOpen />} Gỡ {freeable.length} khoá không dùng
            </Button>
          </div>
        )}
        <div className="-mx-6 min-h-0 flex-1 overflow-y-auto px-6 fine-scrollbar">
          {error && <p className="mb-2 text-sm text-destructive">{error}</p>}
          {!overview && !error && (
            <p className="flex items-center gap-2 text-sm text-muted-foreground">
              <LoaderCircle className="size-4 animate-spin" /> Đang quét…
            </p>
          )}
          {focusMissing && (
            <p className="mb-2 text-sm text-muted-foreground">
              Truyện này không có phiên, agy hay khoá phiên nào — không có gì kẹt.
            </p>
          )}
          {overview && stories.length === 0 && overview.stray.length === 0 && (
            <p className="text-sm text-muted-foreground">Không có phiên, agy hay khoá phiên nào.</p>
          )}
          <ul className="flex flex-col gap-2">
            {stories.map((story) => {
              const focused = focus !== null && samePath(story.root, focus);
              const working = busy[story.root] ?? false;
              return (
                <li
                  key={story.root}
                  ref={focused ? focusRef : undefined}
                  aria-label={title(story)}
                  className={cn("rounded-lg border p-3", focused && "border-primary ring-1 ring-primary/40")}
                >
                  <div className="flex items-center gap-2">
                    <span className="min-w-0 truncate font-medium">{title(story)}</span>
                    {story.running && <Badge>Đang dịch</Badge>}
                    <div className="ml-auto flex shrink-0 items-center gap-1">
                      {story.running && (
                        <Button size="sm" variant="outline" className="h-7" disabled={working} onClick={() => void stop(story)}>
                          {working ? <LoaderCircle className="animate-spin" /> : <Square />} Dừng
                        </Button>
                      )}
                      {(story.running || story.agy.length > 0) && (
                        <Button size="sm" variant="destructive" className="h-7" onClick={() => kill(story)}>
                          <Skull /> Kill
                        </Button>
                      )}
                      {story.lock && !story.running && (
                        <Button size="sm" variant="outline" className="h-7" disabled={working} onClick={() => unlock(story)}>
                          <LockOpen /> Gỡ khoá
                        </Button>
                      )}
                    </div>
                  </div>
                  <p className="truncate font-mono text-[11px] text-muted-foreground" title={story.root}>
                    {story.root}
                  </p>
                  {story.agy.length > 0 && (
                    <ul className="mt-2 flex flex-col gap-1">
                      {story.agy.map((agy) => (
                        <AgyLine key={agy.pid} agy={agy} now={now} />
                      ))}
                    </ul>
                  )}
                  {story.running && story.agy.length === 0 && (
                    <p className="mt-2 text-xs text-muted-foreground">Không có agy đang chạy (dịch bằng API key, hoặc giữa hai lượt).</p>
                  )}
                  {story.lock && (
                    <p className={cn("mt-1 text-xs", story.lock.holder === "session" ? "text-muted-foreground" : "text-status-warning")}>
                      Khoá phiên PID {story.lock.pid}: {lockLabel(story.lock)}
                    </p>
                  )}
                </li>
              );
            })}
          </ul>
          {overview && overview.stray.length > 0 && (
            <section className="mt-3 rounded-lg border p-3">
              <p className="text-sm font-medium">agy của app không rõ truyện</p>
              <ul className="mt-2 flex flex-col gap-1">
                {overview.stray.map((agy) => (
                  <AgyLine
                    key={agy.pid}
                    agy={agy}
                    now={now}
                    busy={busy[`pid:${agy.pid}`]}
                    onKill={() => {
                      if (window.confirm(`Kill agy PID ${agy.pid}?`)) void act(`pid:${agy.pid}`, () => agyKill(agy.pid), "Đã kill");
                    }}
                  />
                ))}
              </ul>
            </section>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
