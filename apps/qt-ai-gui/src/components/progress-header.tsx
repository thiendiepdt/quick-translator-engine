import { BookOpen, Download, FolderOpen, Play, Settings2, Square } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { sessionStart, sessionStop, storySnapshot } from "@/lib/api";
import { useStoryStore } from "@/store/story";

interface Props {
  onOpenStory: () => void;
  onOpenSettings: () => void;
  onOpenExport: () => void;
}

export function ProgressHeader({ onOpenStory, onOpenSettings, onOpenExport }: Props) {
  const root = useStoryStore((s) => s.root);
  const snapshot = useStoryStore((s) => s.snapshot);
  const session = useStoryStore((s) => s.session);
  const progress = useStoryStore((s) => s.progress);
  const agy = useStoryStore((s) => s.agy);
  const config = useStoryStore((s) => s.config);
  const closeStory = useStoryStore((s) => s.closeStory);
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
    <header className="flex flex-col gap-2 border-b bg-card px-4 py-3">
      <div className="flex items-center gap-2">
        <Button variant="ghost" size="sm" disabled={running} onClick={closeStory} title="Về màn chọn truyện">
          <FolderOpen /> {snapshot?.story.name || root}
        </Button>
        <div className="flex-1" />
        <Select value={model ?? ""} onValueChange={(value) => setModel(value || undefined)} disabled={running}>
          <SelectTrigger className="h-8 w-56" aria-label="Model">
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
        <Button size="sm" variant={running ? "destructive" : "default"} disabled={busy} onClick={() => void toggle()}>
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
        <Button size="sm" variant="outline" onClick={onOpenStory}>
          <BookOpen /> Hồ sơ truyện
        </Button>
        <Button size="sm" variant="outline" onClick={onOpenExport} disabled={(counts?.done ?? 0) === 0}>
          <Download /> Export
        </Button>
        <Button size="icon-sm" variant="ghost" onClick={onOpenSettings} aria-label="Cài đặt">
          <Settings2 />
        </Button>
      </div>
      <div className="flex items-center gap-3 text-xs text-muted-foreground">
        <div
          className="h-2 flex-1 overflow-hidden rounded bg-muted"
          role="progressbar"
          aria-valuenow={percent}
          aria-valuemin={0}
          aria-valuemax={100}
        >
          <div className="h-full bg-primary transition-all" style={{ width: `${percent}%` }} />
        </div>
        <span className="tabular-nums">
          {done}/{total} chương ({percent}%)
        </span>
        {counts && (
          <span>
            · chờ {progress?.queued ?? counts.queued} · lỗi {progress?.error ?? counts.error} · bỏ qua{" "}
            {progress?.skipped ?? counts.skipped}
          </span>
        )}
        {session.status === "running" && (
          <span className="text-primary">
            Phiên {session.sessionNo}
            {progress?.current ? ` — đang dịch ${progress.current}` : ""}
          </span>
        )}
      </div>
    </header>
  );
}
