import { RotateCcw } from "lucide-react";
import { useState } from "react";
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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { chaptersRetry, storySnapshot } from "@/lib/api";
import { resolveChapterRef } from "@/lib/chapters";
import { previewRetryRange } from "@/lib/retry-range";
import { cn } from "@/lib/utils";
import type { ChapterRow } from "@/lib/types";
import { useStoryStore } from "@/store/story";

interface Props {
  root: string;
  chapters: ChapterRow[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/** Mã chương ứng với số vừa gõ, để người dùng chắc mình chọn đúng. */
function RefHint({ text, id }: { text: string; id: string }) {
  if (!text.trim()) return null;
  return (
    <p className={cn("truncate font-mono text-[11px]", id ? "text-muted-foreground" : "text-destructive")} title={id}>
      {id ? `→ ${id}` : "không có chương này"}
    </p>
  );
}

/** Dịch lại toàn bộ hoặc một khoảng chương: xem trước số chương, số bản dịch cũ sẽ thành .bak, rồi xác nhận. */
export function RetryRangeDialog({ root, chapters, open, onOpenChange }: Props) {
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [busy, setBusy] = useState(false);
  // Ô nhập nhận số thứ tự (1-based, như cột # trong danh sách) hoặc nguyên mã chương.
  const fromIndex = resolveChapterRef(chapters, from);
  const toIndex = resolveChapterRef(chapters, to);
  const fromId = from.trim() ? (chapters[fromIndex]?.id ?? "") : "";
  const toId = to.trim() ? (chapters[toIndex]?.id ?? "") : "";
  const unresolved = (from.trim() !== "" && fromIndex < 0) || (to.trim() !== "" && toIndex < 0);
  const preview = unresolved
    ? { valid: false, targets: [], done: 0, queued: 0 }
    : previewRetryRange(chapters, fromId, toId);
  const whole = !from.trim() && !to.trim();

  async function run() {
    setBusy(true);
    try {
      const outcome = await chaptersRetry(root, { from: fromId || undefined, to: toId || undefined });
      setSnapshot(await storySnapshot(root));
      toast.success(
        `Đã đưa ${outcome.retried.length} chương về hàng đợi` +
          (outcome.backedUp.length > 0 ? `, ${outcome.backedUp.length} bản dịch cũ giữ .bak` : ""),
      );
      onOpenChange(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không dịch lại được");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={(value) => !busy && onOpenChange(value)}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Dịch lại nhiều chương</DialogTitle>
          <DialogDescription>
            Gõ số thứ tự như cột # trong danh sách (1–{chapters.length}) hoặc nguyên mã chương. Để trống cả hai ô là
            dịch lại toàn bộ truyện. Chương đã dịch xong sẽ giữ bản cũ thành out/&lt;id&gt;.txt.bak (đè bản .bak cũ),
            chương đang chờ sẵn bỏ qua. Xong thì bấm Bắt đầu dịch.
          </DialogDescription>
        </DialogHeader>
        <div className="grid grid-cols-2 gap-3">
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="retry-from">Từ chương</Label>
            <Input
              id="retry-from"
              inputMode="numeric"
              value={from}
              onChange={(e) => setFrom(e.target.value)}
              className="font-mono"
              placeholder="Đầu"
              disabled={busy}
            />
            <RefHint text={from} id={fromId} />
          </div>
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="retry-to">Đến chương</Label>
            <Input
              id="retry-to"
              inputMode="numeric"
              value={to}
              onChange={(e) => setTo(e.target.value)}
              className="font-mono"
              placeholder="Cuối"
              disabled={busy}
            />
            <RefHint text={to} id={toId} />
          </div>
        </div>
        <div className="rounded-md bg-muted p-3 text-sm">
          {!preview.valid ? (
            <p className="text-destructive">Khoảng không hợp lệ: số thứ tự/mã không tồn tại hoặc "từ" đứng sau "đến".</p>
          ) : (
            <p>
              {whole ? "Toàn bộ truyện: " : "Trong khoảng: "}
              <strong className="tabular-nums">{preview.targets.length}</strong> chương sẽ về hàng đợi
              {preview.done > 0 && (
                <>
                  , trong đó <strong className="tabular-nums">{preview.done}</strong> chương đã dịch xong mất bản hiện
                  có (giữ .bak)
                </>
              )}
              {preview.queued > 0 && <>; {preview.queued} chương đang chờ sẵn bỏ qua</>}.
            </p>
          )}
        </div>
        <DialogFooter>
          <Button variant="outline" disabled={busy} onClick={() => onOpenChange(false)}>
            Huỷ
          </Button>
          <Button
            variant={preview.done > 0 ? "destructive" : "default"}
            disabled={busy || !preview.valid || preview.targets.length === 0}
            onClick={() => void run()}
          >
            <RotateCcw /> Dịch lại {preview.valid ? preview.targets.length : 0} chương
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
