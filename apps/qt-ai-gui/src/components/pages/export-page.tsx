import { Download, FolderOpen } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { ChapterRefHint } from "@/components/chapter-ref-hint";
import { ExportChapterList } from "@/components/export-chapter-list";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { exportChapters, pickSaveFile, revealFolder } from "@/lib/api";
import { resolveChapterRef } from "@/lib/chapters";
import { previewRange } from "@/lib/export-range";
import type { ChapterRow, ExportOutcome } from "@/lib/types";
import { useStoryStore } from "@/store/story";

/** Số thứ tự (1-based, dạng chuỗi để điền vào ô) của chương done đầu/cuối; rỗng khi chưa có chương done. */
function doneBounds(chapters: ChapterRow[]): { from: string; to: string } {
  let first = -1;
  let last = -1;
  chapters.forEach((c, index) => {
    if (c.status !== "done") return;
    if (first < 0) first = index;
    last = index;
  });
  return first < 0 ? { from: "", to: "" } : { from: String(first + 1), to: String(last + 1) };
}

export function ExportPage() {
  const root = useStoryStore((s) => s.root);
  const chapters = useStoryStore((s) => s.snapshot?.chapters ?? []);
  const [from, setFrom] = useState(() => doneBounds(chapters).from);
  const [to, setTo] = useState(() => doneBounds(chapters).to);
  const [result, setResult] = useState<ExportOutcome | undefined>();
  const [busy, setBusy] = useState(false);
  // Ô nhập nhận số thứ tự (cột # trong danh sách) hoặc nguyên mã chương; rỗng = đầu/cuối.
  const fromIndex = resolveChapterRef(chapters, from);
  const toIndex = resolveChapterRef(chapters, to);
  // Có chữ mà không khớp → "\0" để previewRange báo khoảng không hợp lệ.
  const fromId = from.trim() ? (chapters[fromIndex]?.id ?? "\0") : "";
  const toId = to.trim() ? (chapters[toIndex]?.id ?? "\0") : "";
  const preview = previewRange(chapters, fromId, toId);
  const canRun = !busy && preview.valid && preview.included.length > 0;

  async function run(pickPath: boolean) {
    if (!root) return;
    setBusy(true);
    try {
      const out = pickPath ? await pickSaveFile(`${fromId || "dau"}-${toId || "cuoi"}.txt`) : undefined;
      if (pickPath && !out) return;
      const outcome = await exportChapters(root, { from: fromId || undefined, to: toId || undefined, out });
      setResult(outcome);
      toast.success(`Đã gộp ${outcome.ids.length} chương`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Export thất bại");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="grid h-full min-h-0 grid-cols-[300px_1fr]">
      <aside className="min-h-0 border-r bg-card/50">
        <ExportChapterList
          rows={chapters}
          fromIndex={fromIndex}
          toIndex={toIndex}
          onPickFrom={(n) => setFrom(String(n))}
          onPickTo={(n) => setTo(String(n))}
        />
      </aside>
      <div className="fine-scrollbar min-h-0 overflow-y-auto">
        <div className="mx-auto flex max-w-2xl flex-col gap-6 px-8 py-8">
          <header>
            <h1 className="text-2xl font-semibold tracking-tight">Export chương đã dịch</h1>
            <p className="text-sm text-muted-foreground">
              Gộp các chương <em>done</em> trong khoảng thành một file .txt, mỗi chương cách một dòng trống. Gõ số thứ
              tự như cột # bên trái (1–{chapters.length}) hoặc bấm <strong>Từ</strong>/<strong>Đến</strong> trên dòng
              chương; để trống là từ đầu / tới cuối. Chương chưa xong trong khoảng được báo hổng.
            </p>
          </header>
          <section className="rounded-lg border bg-card p-5">
            <div className="grid grid-cols-2 gap-4">
              <div className="flex flex-col gap-1.5">
                <Label htmlFor="from">Từ chương</Label>
                <Input
                  id="from"
                  inputMode="numeric"
                  value={from}
                  onChange={(e) => setFrom(e.target.value)}
                  className="font-mono"
                  placeholder="Đầu"
                />
                <ChapterRefHint text={from} id={chapters[fromIndex]?.id ?? ""} />
              </div>
              <div className="flex flex-col gap-1.5">
                <Label htmlFor="to">Đến chương</Label>
                <Input
                  id="to"
                  inputMode="numeric"
                  value={to}
                  onChange={(e) => setTo(e.target.value)}
                  className="font-mono"
                  placeholder="Cuối"
                />
                <ChapterRefHint text={to} id={chapters[toIndex]?.id ?? ""} />
              </div>
            </div>
            <div className="mt-4 rounded-md bg-muted p-3 text-sm">
              {!preview.valid ? (
                <p className="text-destructive">
                  Khoảng không hợp lệ: số thứ tự/mã không tồn tại hoặc "từ" đứng sau "đến".
                </p>
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
                Đã ghi <code className="font-mono text-xs break-all">{result.outPath}</code> ({result.ids.length}{" "}
                chương).
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
    </div>
  );
}
