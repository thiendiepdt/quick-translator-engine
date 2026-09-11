import { Trash2 } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { ChapterRefHint } from "@/components/chapter-ref-hint";
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
import { chaptersDelete, storySnapshot } from "@/lib/api";
import { describeDelete, resolveChapterRef } from "@/lib/chapters";
import { idsInRange } from "@/lib/export-range";
import type { ChapterRow } from "@/lib/types";
import { useStoryStore } from "@/store/story";

interface Props {
  root: string;
  chapters: ChapterRow[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/** Xoá hẳn một khoảng chương: xoá raw/ + work/, gỡ khỏi danh sách; bản dịch trong out/ giữ nguyên. */
export function DeleteRangeDialog({ root, chapters, open, onOpenChange }: Props) {
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [busy, setBusy] = useState(false);
  const fromIndex = resolveChapterRef(chapters, from);
  const toIndex = resolveChapterRef(chapters, to);
  const fromId = from.trim() ? (chapters[fromIndex]?.id ?? "") : "";
  const toId = to.trim() ? (chapters[toIndex]?.id ?? "") : "";
  const unresolved = (from.trim() !== "" && fromIndex < 0) || (to.trim() !== "" && toIndex < 0);
  // Không cho để trống cả hai: xoá cả truyện phải gõ rõ 1–N, tránh lỡ tay.
  const bothEmpty = !from.trim() && !to.trim();
  const ids = unresolved || bothEmpty ? null : idsInRange(chapters, fromId, toId);
  const doneCount = ids ? ids.filter((id) => chapters.find((c) => c.id === id)?.status === "done").length : 0;

  async function run() {
    if (!ids) return;
    setBusy(true);
    try {
      const outcome = await chaptersDelete(root, ids);
      setSnapshot(await storySnapshot(root));
      toast.success(describeDelete(outcome.removed.length, outcome.keptOutputs.length));
      setFrom("");
      setTo("");
      onOpenChange(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không xoá được chương");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={(value) => !busy && onOpenChange(value)}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Xoá nhiều chương</DialogTitle>
          <DialogDescription>
            Gõ số thứ tự như cột # trong danh sách (1–{chapters.length}) hoặc nguyên mã chương. File gốc trong raw/
            và nháp trong work/ bị xoá, không hoàn tác được; bản dịch đã xong trong out/ giữ nguyên trên đĩa. Phải
            điền ít nhất một ô.
          </DialogDescription>
        </DialogHeader>
        <div className="grid grid-cols-2 gap-3">
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="delete-from">Từ chương</Label>
            <Input
              id="delete-from"
              inputMode="numeric"
              value={from}
              onChange={(e) => setFrom(e.target.value)}
              className="font-mono"
              placeholder="Đầu"
              disabled={busy}
            />
            <ChapterRefHint text={from} id={fromId} />
          </div>
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="delete-to">Đến chương</Label>
            <Input
              id="delete-to"
              inputMode="numeric"
              value={to}
              onChange={(e) => setTo(e.target.value)}
              className="font-mono"
              placeholder="Cuối"
              disabled={busy}
            />
            <ChapterRefHint text={to} id={toId} />
          </div>
        </div>
        <div className="rounded-md bg-muted p-3 text-sm">
          {bothEmpty ? (
            <p className="text-muted-foreground">Điền ít nhất một ô để chọn khoảng cần xoá.</p>
          ) : !ids ? (
            <p className="text-destructive">Khoảng không hợp lệ: số thứ tự/mã không tồn tại hoặc "từ" đứng sau "đến".</p>
          ) : (
            <p>
              Sẽ xoá <strong className="tabular-nums">{ids.length}</strong> chương
              {doneCount > 0 && (
                <>
                  , trong đó <strong className="tabular-nums">{doneCount}</strong> chương đã dịch xong (bản dịch trong
                  out/ giữ nguyên)
                </>
              )}
              .
            </p>
          )}
        </div>
        <DialogFooter>
          <Button variant="outline" disabled={busy} onClick={() => onOpenChange(false)}>
            Huỷ
          </Button>
          <Button variant="destructive" disabled={busy || !ids || ids.length === 0} onClick={() => void run()}>
            <Trash2 /> Xoá {ids?.length ?? 0} chương
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
