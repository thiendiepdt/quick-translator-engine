import { AlertTriangle, FolderOpen, RotateCcw, ShieldCheck, SkipForward } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  chapterForceAccept,
  chapterRetry,
  chapterSkip,
  readChapter,
  revealFolder,
  storySnapshot,
} from "@/lib/api";
import { STATUS_LABELS, type ChapterRow, type ChapterView } from "@/lib/types";
import { useStoryStore } from "@/store/story";

interface Props {
  root: string;
  row: ChapterRow;
}

export function ChapterPanel({ root, row }: Props) {
  const running = useStoryStore((s) => s.session.status === "running");
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  // View gắn key theo (chương, trạng thái, vòng soát): đổi chương → key lệch → hiện "Đang đọc…" mà
  // không cần reset state đồng bộ trong effect.
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

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-wrap items-center gap-2 border-b p-2">
        <span className="font-mono text-sm">{row.id}</span>
        <span className="text-xs text-muted-foreground">
          {STATUS_LABELS[row.status]}
          {row.reason ? ` — ${row.reason}` : ""}
        </span>
        <div className="flex-1" />
        <Button
          size="xs"
          variant="outline"
          disabled={!canRetry || running}
          onClick={() => void act("Đã đưa về hàng đợi", () => chapterRetry(root, row.id))}
        >
          <RotateCcw /> Dịch lại
        </Button>
        <Button size="xs" variant="outline" disabled={!canSkip || running} onClick={() => setSkipOpen(true)}>
          <SkipForward /> Bỏ qua
        </Button>
        <Button
          size="xs"
          variant="outline"
          disabled={!canForce || running}
          title="Chốt bản nháp hiện có dù check chưa đạt"
          onClick={() => void act("Đã chốt (force)", () => chapterForceAccept(root, row.id))}
        >
          <ShieldCheck /> Chốt --force
        </Button>
        <Button size="xs" variant="ghost" onClick={() => void revealFolder(root)}>
          <FolderOpen /> Mở folder
        </Button>
      </div>
      {row.warnings.length > 0 && (
        <div className="border-b bg-amber-50 p-2 text-xs text-amber-900">
          <div className="mb-1 flex items-center gap-1 font-medium">
            <AlertTriangle className="size-3" /> {row.warnings.length} cảnh báo còn lại
          </div>
          <ul className="list-disc pl-5">
            {row.warnings.map((w) => (
              <li key={w}>{w}</li>
            ))}
          </ul>
        </div>
      )}
      <Tabs defaultValue={view?.output ? "output" : "raw"} className="flex min-h-0 flex-1 flex-col">
        <TabsList className="mx-2 mt-2">
          <TabsTrigger value="output" disabled={!view?.output}>
            Bản dịch
          </TabsTrigger>
          <TabsTrigger value="draft" disabled={!view?.draft}>
            Bản nháp
          </TabsTrigger>
          <TabsTrigger value="review" disabled={!view?.review}>
            Yêu cầu sửa
          </TabsTrigger>
          <TabsTrigger value="raw">Gốc</TabsTrigger>
        </TabsList>
        {(["output", "draft", "review", "raw"] as const).map((key) => (
          <TabsContent key={key} value={key} className="min-h-0 flex-1">
            <ScrollArea className="h-full">
              <pre className="whitespace-pre-wrap p-3 font-serif text-sm leading-relaxed">
                {view ? (key === "raw" ? view.raw : (view[key] ?? "")) : "Đang đọc…"}
              </pre>
            </ScrollArea>
          </TabsContent>
        ))}
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
