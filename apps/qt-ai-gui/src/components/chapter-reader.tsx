import {
  AlertTriangle,
  ChevronLeft,
  ChevronRight,
  FolderOpen,
  RotateCcw,
  ShieldCheck,
  SkipForward,
} from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { LogPanel } from "@/components/log-panel";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useReadingWidth } from "@/hooks/use-reading-width";
import {
  chapterForceAccept,
  chapterRetry,
  chapterSkip,
  readChapter,
  revealFolder,
  storySnapshot,
} from "@/lib/api";
import { isReadingWidth, READING_WIDTH_LABELS, READING_WIDTHS } from "@/lib/reading";
import { STATUS_LABELS, type ChapterRow, type ChapterStatus, type ChapterView } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

const STATUS_TONE: Record<ChapterStatus, string> = {
  queued: "bg-status-queued/30 text-foreground",
  translating: "bg-status-translating/15 text-status-translating",
  done: "bg-status-done/15 text-status-done",
  error: "bg-status-error/15 text-status-error",
  skipped: "bg-muted text-muted-foreground",
};

interface Props {
  root: string;
  row: ChapterRow;
  hasPrev: boolean;
  hasNext: boolean;
  onPrev: () => void;
  onNext: () => void;
}

export function ChapterReader({ root, row, hasPrev, hasNext, onPrev, onNext }: Props) {
  const running = useStoryStore((s) => s.session.status === "running");
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const { width, setWidth } = useReadingWidth();
  // View gắn key theo (chương, trạng thái, vòng soát): đổi chương → key lệch → hiện "Đang đọc…"
  // mà không cần reset state đồng bộ trong effect.
  const viewKey = `${root}|${row.id}|${row.status}|${row.reviewRound}`;
  const [loaded, setLoaded] = useState<{ key: string; view: ChapterView } | undefined>();
  const view = loaded?.key === viewKey ? loaded.view : undefined;
  const [skipOpen, setSkipOpen] = useState(false);
  const [reason, setReason] = useState("");

  useEffect(() => {
    let cancelled = false;
    readChapter(root, row.id)
      .then((v) => {
        if (!cancelled) setLoaded({ key: viewKey, view: v });
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : "Không đọc được chương"));
    return () => {
      cancelled = true;
    };
  }, [root, row.id, viewKey]);

  async function act(label: string, action: () => Promise<unknown>) {
    try {
      await action();
      setSnapshot(await storySnapshot(root));
      toast.success(label);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : `${label} thất bại`);
    }
  }

  const canRetry = row.status === "error" || row.status === "skipped";
  const canSkip = row.status !== "done";
  const canForce = row.status === "translating" && Boolean(view?.draft);
  const defaultTab = view?.output ? "output" : view?.draft ? "draft" : "raw";

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-wrap items-center gap-2 border-b px-5 py-3">
        {/* Chuyển chương luôn nằm trên đầu, không phải cuộn xuống cuối bài. */}
        <div className="flex items-center gap-0.5">
          <Button size="icon-sm" variant="ghost" aria-label="Chương trước" title="Chương trước" disabled={!hasPrev} onClick={onPrev}>
            <ChevronLeft />
          </Button>
          <Button size="icon-sm" variant="ghost" aria-label="Chương sau" title="Chương sau" disabled={!hasNext} onClick={onNext}>
            <ChevronRight />
          </Button>
        </div>
        <span className="font-mono text-sm font-medium">{row.id}</span>
        <span className={cn("rounded-full px-2 py-0.5 text-xs font-medium", STATUS_TONE[row.status])}>
          {STATUS_LABELS[row.status]}
        </span>
        {row.reason && <span className="truncate text-xs text-muted-foreground">{row.reason}</span>}
        <div className="flex-1" />
        <Button
          size="sm"
          variant="outline"
          disabled={!canRetry || running}
          onClick={() => void act("Đã đưa về hàng đợi", () => chapterRetry(root, row.id))}
        >
          <RotateCcw /> Dịch lại
        </Button>
        <Button size="sm" variant="outline" disabled={!canSkip || running} onClick={() => setSkipOpen(true)}>
          <SkipForward /> Bỏ qua
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={!canForce || running}
          title="Chốt bản nháp hiện có dù check chưa đạt"
          onClick={() => void act("Đã chốt (force)", () => chapterForceAccept(root, row.id))}
        >
          <ShieldCheck /> Chốt --force
        </Button>
        <Button size="sm" variant="ghost" onClick={() => void revealFolder(root)}>
          <FolderOpen /> Mở folder
        </Button>
      </div>
      {row.warnings.length > 0 && (
        <div className="border-b bg-status-warning/10 px-5 py-2 text-xs">
          <div className="mb-1 flex items-center gap-1.5 font-medium text-status-warning">
            <AlertTriangle className="size-3.5" /> {row.warnings.length} cảnh báo còn lại
          </div>
          <ul className="list-disc space-y-0.5 pl-5 text-foreground/80">
            {row.warnings.map((w) => (
              <li key={w}>{w}</li>
            ))}
          </ul>
        </div>
      )}
      {/* key đổi khi view tải xong để defaultValue tính lại (Bản dịch nếu có, không thì Gốc) */}
      <Tabs
        key={`${viewKey}|${view ? "loaded" : "loading"}`}
        defaultValue={defaultTab}
        className="flex min-h-0 flex-1 flex-col gap-0"
      >
        <div className="mx-5 mt-2 flex items-end justify-between gap-2 border-b">
          <TabsList variant="line" className="border-b-0">
            <TabsTrigger value="output" disabled={!view?.output}>
              Bản dịch
            </TabsTrigger>
            <TabsTrigger value="draft" disabled={!view?.draft}>
              Nháp
            </TabsTrigger>
            <TabsTrigger value="review" disabled={!view?.review}>
              Yêu cầu sửa
            </TabsTrigger>
            <TabsTrigger value="raw">Gốc</TabsTrigger>
            <TabsTrigger value="log">Log</TabsTrigger>
          </TabsList>
          <Select value={width} onValueChange={(v) => isReadingWidth(v) && void setWidth(v)}>
            <SelectTrigger size="sm" className="mb-1 w-32" aria-label="Chiều ngang văn bản">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {READING_WIDTHS.map((item) => (
                <SelectItem key={item} value={item}>
                  {READING_WIDTH_LABELS[item]}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        {(["output", "draft", "review", "raw"] as const).map((key) => (
          <TabsContent key={key} value={key} className="fine-scrollbar min-h-0 flex-1 overflow-y-auto">
            <article className="reading mx-auto px-6 py-8" data-width={width}>
              <pre className="font-[inherit] whitespace-pre-wrap">
                {view ? (key === "raw" ? view.raw : (view[key] ?? "")) : "Đang đọc…"}
              </pre>
              <nav className="mt-10 flex items-center justify-between border-t pt-4 font-sans text-sm">
                <Button variant="ghost" size="sm" disabled={!hasPrev} onClick={onPrev}>
                  <ChevronLeft /> Chương trước
                </Button>
                <Button variant="ghost" size="sm" disabled={!hasNext} onClick={onNext}>
                  Chương sau <ChevronRight />
                </Button>
              </nav>
            </article>
          </TabsContent>
        ))}
        <TabsContent value="log" className="min-h-0 flex-1">
          <LogPanel />
        </TabsContent>
      </Tabs>
      <Dialog open={skipOpen} onOpenChange={setSkipOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Bỏ qua chương {row.id}</DialogTitle>
          </DialogHeader>
          <Input
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            placeholder="Lý do (vd. model từ chối nội dung)"
          />
          <DialogFooter>
            <Button variant="outline" onClick={() => setSkipOpen(false)}>
              Huỷ
            </Button>
            <Button
              disabled={!reason.trim()}
              onClick={() => {
                setSkipOpen(false);
                void act("Đã bỏ qua", () => chapterSkip(root, row.id, reason));
                setReason("");
              }}
            >
              Bỏ qua
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
