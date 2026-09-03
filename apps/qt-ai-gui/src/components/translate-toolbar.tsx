import { Play, Square } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { sessionStart, sessionStop, storySnapshot } from "@/lib/api";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

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
  const session = useStoryStore((s) => s.session);
  const progress = useStoryStore((s) => s.progress);
  const agy = useStoryStore((s) => s.agy);
  const config = useStoryStore((s) => s.config);
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const [model, setModel] = useState<string | undefined>(config?.model ?? undefined);
  const [busy, setBusy] = useState(false);

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
        await sessionStop();
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
          <h1 className="truncate text-base font-semibold tracking-tight">{snapshot?.story.name || root}</h1>
          <p className="truncate font-mono text-xs text-muted-foreground">{root}</p>
        </div>
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
        <Button
          size="lg"
          variant={running ? "destructive" : "default"}
          disabled={busy}
          onClick={() => void toggle()}
          className="min-w-36"
        >
          {running ? (
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
    </header>
  );
}
