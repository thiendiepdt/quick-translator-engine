import { KeyRound, LoaderCircle, Play, RefreshCw, RotateCcw, Square, Trash2, TriangleAlert } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";

import { DeleteRangeDialog } from "@/components/delete-range-dialog";
import { RetryRangeDialog } from "@/components/retry-range-dialog";
import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { chaptersRetryIds, rescanStory, sessionStart, sessionStop, storySnapshot } from "@/lib/api";
import { gapsBeforeFrontier } from "@/lib/chapters";
import { engineLabel, STATUS_LABELS } from "@/lib/types";
import { cn } from "@/lib/utils";
import { selectCurrentProgress, selectCurrentSession, useStoryStore } from "@/store/story";

/** Số chip chương hổng hiện tối đa; truyện vài nghìn chương skip nhiều thì xem tiếp ở bộ lọc Bỏ qua/Lỗi. */
const GAP_CHIPS = 12;

function Stat({ label, value, tone }: { label: string; value: number; tone: string }) {
  return (
    <span className={cn("inline-flex items-center gap-1.5 rounded-full border px-2 py-0.5 text-xs", tone)}>
      <span className="size-1.5 rounded-full bg-current" aria-hidden />
      {label} <span className="font-medium tabular-nums">{value}</span>
    </span>
  );
}

export function TranslateToolbar() {
  const root = useStoryStore((s) => s.root);
  const snapshot = useStoryStore((s) => s.snapshot);
  const session = useStoryStore(selectCurrentSession);
  const progress = useStoryStore(selectCurrentProgress);
  const agy = useStoryStore((s) => s.agy);
  const config = useStoryStore((s) => s.config);
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const setPage = useStoryStore((s) => s.setPage);
  const select = useStoryStore((s) => s.select);
  // Chương chưa dịch đứng trước chương done cuối (skip vì model từ chối, lỗi…) — dễ bị bỏ quên khi phiên chạy tiếp.
  const gapInfo = useMemo(() => (snapshot ? gapsBeforeFrontier(snapshot.chapters) : null), [snapshot]);
  // Chương hổng còn phải đưa về hàng đợi (queued sẵn thì thôi).
  const gapRetryIds = useMemo(
    () => (gapInfo?.gaps ?? []).filter(({ row }) => row.status !== "queued").map(({ row }) => row.id),
    [gapInfo],
  );
  const [model, setModel] = useState<string | undefined>(config?.model ?? undefined);
  const [busy, setBusy] = useState(false);
  const [scanning, setScanning] = useState(false);
  const [retryingGaps, setRetryingGaps] = useState(false);
  const [retryOpen, setRetryOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);

  /** Đưa mọi chương hổng (skip/lỗi) về hàng đợi; không tự chạy phiên. */
  async function retryGaps() {
    if (!root || gapRetryIds.length === 0) return;
    setRetryingGaps(true);
    try {
      const outcome = await chaptersRetryIds(root, gapRetryIds);
      setSnapshot(await storySnapshot(root));
      toast.success(`Đã đưa ${outcome.retried.length} chương về hàng đợi — bấm Bắt đầu để dịch`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Dịch lại thất bại");
    } finally {
      setRetryingGaps(false);
    }
  }

  /** Quét raw/ lấy chương mới vào hàng đợi (copy tay vào raw/ xong bấm đây). */
  async function rescan() {
    if (!root) return;
    setScanning(true);
    try {
      const before = new Set((snapshot?.chapters ?? []).map((c) => c.id));
      const next = await rescanStory(root);
      setSnapshot(next);
      const after = new Set(next.chapters.map((c) => c.id));
      const added = next.chapters.filter((c) => !before.has(c.id)).length;
      const removed = [...before].filter((id) => !after.has(id)).length;
      const parts = [
        added > 0 ? `thêm ${added} chương mới vào hàng đợi` : "",
        removed > 0 ? `gỡ ${removed} chương raw đã mất` : "",
      ].filter(Boolean);
      toast.message(parts.length > 0 ? `Quét lại: ${parts.join(", ")}` : "Không có thay đổi trong raw/");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không quét được raw/");
    } finally {
      setScanning(false);
    }
  }

  const counts = snapshot?.counts;
  const done = progress?.done ?? counts?.done ?? 0;
  const total = counts?.total ?? 0;
  const percent = total > 0 ? Math.round((done / total) * 100) : 0;
  const running = session.status === "running";

  async function toggle() {
    if (!root) return;
    setBusy(true);
    try {
      if (running) {
        await sessionStop(root);
        setSnapshot(await storySnapshot(root));
      } else {
        await sessionStart(root, model);
        toast.message("Đã bắt đầu phiên dịch");
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không đổi được trạng thái phiên");
    } finally {
      setBusy(false);
    }
  }

  return (
    <header className={cn("border-b bg-card px-5 py-3 transition-colors", running && "border-b-primary/50")}>
      <div className="flex items-center gap-3">
        <div className="min-w-0 flex-1">
          {snapshot?.story.name ? (
            <>
              <h1 className="truncate text-base font-semibold tracking-tight">{snapshot.story.name}</h1>
              <p className="truncate font-mono text-xs text-muted-foreground">{root}</p>
            </>
          ) : (
            <>
              <h1 className="truncate font-mono text-sm font-medium">{root}</h1>
              <p className="text-xs text-muted-foreground">Chưa đặt tên truyện — vào Hồ sơ truyện để điền.</p>
            </>
          )}
        </div>
        {config?.engine === "api" ? (
          <Button
            type="button"
            variant="outline"
            className="h-9 max-w-72 font-mono text-xs"
            title="Động cơ API — đổi model/key trong Cài đặt"
            disabled={running}
            onClick={() => setPage("settings")}
          >
            <KeyRound /> <span className="truncate">{engineLabel(config)}</span>
          </Button>
        ) : (
          <Select value={model ?? ""} onValueChange={(value) => setModel(value || undefined)} disabled={running}>
            <SelectTrigger className="h-9 w-56" aria-label="Model">
              <SelectValue placeholder="Model mặc định của agy" />
            </SelectTrigger>
            <SelectContent>
              {(agy?.models ?? []).map((item) => (
                <SelectItem key={item} value={item}>
                  {item}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
        <Button
          type="button"
          variant="outline"
          className="h-9"
          title="Quét raw/ lấy chương mới vào hàng đợi"
          disabled={running || scanning}
          onClick={() => void rescan()}
        >
          <RefreshCw className={cn(scanning && "animate-spin")} /> Quét lại
        </Button>
        <Button
          type="button"
          variant="outline"
          className="h-9"
          title="Dịch lại toàn bộ hoặc một khoảng chương"
          disabled={running || !snapshot}
          onClick={() => setRetryOpen(true)}
        >
          <RotateCcw /> Dịch lại…
        </Button>
        <Button
          type="button"
          variant="outline"
          className="h-9"
          title="Xoá hẳn một khoảng chương (raw/ + work/); bản dịch trong out/ giữ nguyên"
          disabled={running || !snapshot}
          onClick={() => setDeleteOpen(true)}
        >
          <Trash2 /> Xoá…
        </Button>
        {root && (
          <>
            <RetryRangeDialog root={root} chapters={snapshot?.chapters ?? []} open={retryOpen} onOpenChange={setRetryOpen} />
            <DeleteRangeDialog root={root} chapters={snapshot?.chapters ?? []} open={deleteOpen} onOpenChange={setDeleteOpen} />
          </>
        )}
        <Button
          size="lg"
          variant={running ? "destructive" : "default"}
          disabled={busy}
          onClick={() => void toggle()}
          className="min-w-36"
        >
          {running && busy ? (
            <>
              <LoaderCircle className="animate-spin" /> Đang dừng…
            </>
          ) : running ? (
            <>
              <Square /> Dừng
            </>
          ) : (
            <>
              <Play /> Bắt đầu dịch
            </>
          )}
        </Button>
      </div>
      <div className="mt-3 flex items-center gap-3">
        <div
          className="h-1.5 flex-1 overflow-hidden rounded-full bg-muted"
          role="progressbar"
          aria-valuenow={percent}
          aria-valuemin={0}
          aria-valuemax={100}
        >
          <div className="h-full rounded-full bg-status-done transition-[width]" style={{ width: `${percent}%` }} />
        </div>
        <span className="text-sm tabular-nums">
          {done}/{total} <span className="text-muted-foreground">({percent}%)</span>
        </span>
      </div>
      <div className="mt-2 flex flex-wrap items-center gap-1.5">
        {counts && (
          <>
            <Stat label="Chờ" value={progress?.queued ?? counts.queued} tone="text-muted-foreground" />
            <Stat
              label="Lỗi"
              value={progress?.error ?? counts.error}
              tone="text-status-error border-status-error/30"
            />
            <Stat label="Bỏ qua" value={progress?.skipped ?? counts.skipped} tone="text-muted-foreground" />
            <Stat
              label="Cảnh báo"
              value={progress?.warnings_count ?? counts.withWarnings}
              tone="text-status-warning border-status-warning/30"
            />
          </>
        )}
        {running && (
          <span className="ml-auto inline-flex items-center gap-2 text-xs text-primary">
            <span className="size-2 animate-pulse rounded-full bg-status-translating" aria-hidden />
            Phiên {session.sessionNo}
            {progress?.current ? ` · đang dịch ${progress.current}` : ""}
          </span>
        )}
      </div>
      {gapInfo && (
        <p
          role="status"
          aria-label="Hổng chương chưa dịch"
          className="mt-2 flex flex-wrap items-center gap-1.5 text-xs text-status-warning"
        >
          <TriangleAlert className="size-3.5" aria-hidden />
          <span>
            {gapInfo.gaps.length} chương trước #{gapInfo.frontier.ordinal} chưa dịch:
          </span>
          {gapInfo.gaps.slice(0, GAP_CHIPS).map(({ ordinal, row }) => (
            <button
              key={row.id}
              type="button"
              aria-label={`#${ordinal} ${row.id}`}
              title={`${row.id} — ${STATUS_LABELS[row.status]}${row.reason ? `: ${row.reason}` : ""}`}
              onClick={() => select(row.id)}
              className="rounded border border-status-warning/40 px-1.5 py-0.5 font-mono tabular-nums hover:bg-status-warning/10"
            >
              #{ordinal} <span className="opacity-70">{STATUS_LABELS[row.status]}</span>
            </button>
          ))}
          {gapInfo.gaps.length > GAP_CHIPS && <span>… và {gapInfo.gaps.length - GAP_CHIPS} chương nữa</span>}
          {gapRetryIds.length > 0 && (
            <Button
              type="button"
              variant="outline"
              className="h-6 px-2 text-xs"
              title="Đưa mọi chương hổng (bỏ qua/lỗi) về hàng đợi; chương đã queued không tính"
              disabled={running || retryingGaps || !root}
              onClick={() => void retryGaps()}
            >
              {retryingGaps ? <LoaderCircle className="animate-spin" /> : <RotateCcw />} Dịch lại cả {gapRetryIds.length}
            </Button>
          )}
        </p>
      )}
    </header>
  );
}
