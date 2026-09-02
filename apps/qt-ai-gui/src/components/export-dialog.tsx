import { Download, FolderOpen } from "lucide-react";
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
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { exportChapters, pickSaveFile, revealFolder } from "@/lib/api";
import type { ChapterRow, ExportOutcome } from "@/lib/types";
import { useStoryStore } from "@/store/story";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function ExportDialog({ open, onOpenChange }: Props) {
  const root = useStoryStore((s) => s.root);
  const chapters = useStoryStore((s) => s.snapshot?.chapters ?? []);
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Export chương đã dịch</DialogTitle>
          <DialogDescription>
            Gộp các chương <em>done</em> trong khoảng thành một file .txt, mỗi chương cách một dòng trống. Chương
            chưa xong trong khoảng sẽ được báo hổng.
          </DialogDescription>
        </DialogHeader>
        {/* DialogContent unmount khi đóng → state form tự reset mỗi lần mở, không cần effect */}
        {root && <ExportForm root={root} chapters={chapters} onClose={() => onOpenChange(false)} />}
      </DialogContent>
    </Dialog>
  );
}

interface FormProps {
  root: string;
  chapters: ChapterRow[];
  onClose: () => void;
}

function ExportForm({ root, chapters, onClose }: FormProps) {
  const done = chapters.filter((c) => c.status === "done");
  const [from, setFrom] = useState<string | undefined>(done[0]?.id);
  const [to, setTo] = useState<string | undefined>(done[done.length - 1]?.id);
  const [result, setResult] = useState<ExportOutcome | undefined>();
  const [busy, setBusy] = useState(false);

  async function run(pickPath: boolean) {
    setBusy(true);
    try {
      const out = pickPath ? await pickSaveFile(`${from ?? "dau"}-${to ?? "cuoi"}.txt`) : undefined;
      if (pickPath && !out) return;
      const outcome = await exportChapters(root, { from, to, out });
      setResult(outcome);
      toast.success(`Đã gộp ${outcome.ids.length} chương`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Export thất bại");
    } finally {
      setBusy(false);
    }
  }

  const picker = (label: string, value: string | undefined, onChange: (v: string) => void) => (
    <div className="flex-1">
      <Label>{label}</Label>
      <Select value={value ?? ""} onValueChange={onChange}>
        <SelectTrigger>
          <SelectValue placeholder="—" />
        </SelectTrigger>
        <SelectContent className="max-h-72">
          {chapters.map((c) => (
            <SelectItem key={c.id} value={c.id}>
              {c.id}
              {c.status !== "done" ? ` (${c.status})` : ""}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );

  return (
    <>
      <div className="flex gap-3">
        {picker("Từ chương", from, setFrom)}
        {picker("Đến chương", to, setTo)}
      </div>
      {result && (
        <div className="rounded border p-3 text-sm">
          <p>
            Đã ghi <code className="break-all">{result.outPath}</code> ({result.ids.length} chương).
          </p>
          {result.gaps.length > 0 && (
            <p className="mt-1 text-amber-700">
              Hổng {result.gaps.length} chương chưa done: {result.gaps.join(", ")}
            </p>
          )}
          <Button
            size="xs"
            variant="ghost"
            className="mt-2"
            onClick={() => void revealFolder(result.outPath.replace(/[\\/][^\\/]+$/, ""))}
          >
            <FolderOpen /> Mở folder
          </Button>
        </div>
      )}
      <DialogFooter>
        <Button variant="outline" onClick={onClose}>
          Đóng
        </Button>
        <Button variant="secondary" disabled={busy || done.length === 0} onClick={() => void run(true)}>
          Chọn nơi lưu…
        </Button>
        <Button disabled={busy || done.length === 0} onClick={() => void run(false)}>
          <Download /> Export vào export/
        </Button>
      </DialogFooter>
    </>
  );
}
