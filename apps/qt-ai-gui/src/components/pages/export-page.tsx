import { Download, FolderOpen } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { exportChapters, pickSaveFile, revealFolder } from "@/lib/api";
import { previewRange } from "@/lib/export-range";
import type { ExportOutcome } from "@/lib/types";
import { useStoryStore } from "@/store/story";

export function ExportPage() {
  const root = useStoryStore((s) => s.root);
  const chapters = useStoryStore((s) => s.snapshot?.chapters ?? []);
  const done = chapters.filter((c) => c.status === "done");
  const [from, setFrom] = useState(done[0]?.id ?? "");
  const [to, setTo] = useState(done[done.length - 1]?.id ?? "");
  const [result, setResult] = useState<ExportOutcome | undefined>();
  const [busy, setBusy] = useState(false);
  const preview = previewRange(chapters, from, to);
  const canRun = !busy && preview.valid && preview.included.length > 0;

  async function run(pickPath: boolean) {
    if (!root) return;
    setBusy(true);
    try {
      const out = pickPath ? await pickSaveFile(`${from || "dau"}-${to || "cuoi"}.txt`) : undefined;
      if (pickPath && !out) return;
      const outcome = await exportChapters(root, { from: from || undefined, to: to || undefined, out });
      setResult(outcome);
      toast.success(`Đã gộp ${outcome.ids.length} chương`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Export thất bại");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="fine-scrollbar h-full overflow-y-auto">
      <div className="mx-auto flex max-w-2xl flex-col gap-6 px-8 py-8">
        <header>
          <h1 className="text-2xl font-semibold tracking-tight">Export chương đã dịch</h1>
          <p className="text-sm text-muted-foreground">
            Gộp các chương <em>done</em> trong khoảng thành một file .txt, mỗi chương cách một dòng trống. Chương chưa
            xong trong khoảng được báo hổng.
          </p>
        </header>
        <section className="rounded-lg border bg-card p-5">
          <datalist id="chapter-ids">
            {chapters.map((c) => (
              <option key={c.id} value={c.id} />
            ))}
          </datalist>
          <div className="grid grid-cols-2 gap-4">
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="from">Từ chương</Label>
              <Input
                id="from"
                list="chapter-ids"
                value={from}
                onChange={(e) => setFrom(e.target.value)}
                className="font-mono"
                placeholder="Đầu"
              />
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="to">Đến chương</Label>
              <Input
                id="to"
                list="chapter-ids"
                value={to}
                onChange={(e) => setTo(e.target.value)}
                className="font-mono"
                placeholder="Cuối"
              />
            </div>
          </div>
          <div className="mt-4 rounded-md bg-muted p-3 text-sm">
            {!preview.valid ? (
              <p className="text-destructive">Khoảng không hợp lệ: mã chương không tồn tại hoặc "từ" đứng sau "đến".</p>
            ) : (
              <>
                <p>
                  Sẽ gộp <strong className="tabular-nums">{preview.included.length}</strong> chương done.
                </p>
                {preview.gaps.length > 0 && (
                  <p className="mt-1 text-status-warning">
                    Hổng {preview.gaps.length} chương chưa done:{" "}
                    <span className="font-mono text-xs">
                      {preview.gaps.slice(0, 20).join(", ")}
                      {preview.gaps.length > 20 ? "…" : ""}
                    </span>
                  </p>
                )}
              </>
            )}
          </div>
          <div className="mt-4 flex justify-end gap-2">
            <Button variant="outline" disabled={!canRun} onClick={() => void run(true)}>
              Chọn nơi lưu…
            </Button>
            <Button disabled={!canRun} onClick={() => void run(false)}>
              <Download /> Export vào export/
            </Button>
          </div>
        </section>
        {result && (
          <section className="rounded-lg border border-status-done/40 bg-status-done/10 p-5 text-sm">
            <p>
              Đã ghi <code className="font-mono text-xs break-all">{result.outPath}</code> ({result.ids.length} chương).
            </p>
            {result.gaps.length > 0 && (
              <p className="mt-1 text-status-warning">
                Hổng {result.gaps.length} chương: {result.gaps.join(", ")}
              </p>
            )}
            <Button
              size="sm"
              variant="outline"
              className="mt-3"
              onClick={() => void revealFolder(result.outPath.replace(/[\\/][^\\/]+$/, ""))}
            >
              <FolderOpen /> Mở folder
            </Button>
          </section>
        )}
      </div>
    </div>
  );
}
