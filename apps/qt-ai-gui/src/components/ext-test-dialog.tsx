import { LoaderCircle, Play } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { extTestChapter, extTestToc, type ExtTestReport, type ExtTestTocItem } from "@/lib/api";
import { extTestTocItemSchema } from "@/lib/schema";

const MAX_ROWS = 50;
const message = (error: unknown) => (error instanceof Error ? error.message : String(error));

interface Props {
  ext: { id: string; name: string };
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/** Chạy thử extension (chế độ developer): `toc` với một link, rồi `chapter` một mục. Không ghi gì vào truyện. */
export function ExtTestDialog({ ext, open, onOpenChange }: Props) {
  const [url, setUrl] = useState("");
  const [toc, setToc] = useState<ExtTestReport>();
  const [chapter, setChapter] = useState<ExtTestReport>();
  const [busy, setBusy] = useState(false);
  const parsed = toc?.ok ? z.array(extTestTocItemSchema).safeParse(toc.result) : undefined;
  const items: ExtTestTocItem[] = parsed?.success ? parsed.data : [];

  async function runToc() {
    setBusy(true);
    setChapter(undefined);
    try {
      setToc(await extTestToc(ext.id, url.trim()));
    } catch (error) {
      toast.error(message(error));
    } finally {
      setBusy(false);
    }
  }

  async function runChapter(item: ExtTestTocItem, index: number) {
    setBusy(true);
    try {
      setChapter(await extTestChapter(ext.id, url.trim(), item, index));
    } catch (error) {
      toast.error(message(error));
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-3xl">
        <DialogHeader>
          <DialogTitle>Chạy thử — {ext.name}</DialogTitle>
          <DialogDescription>Gọi toc với link bên dưới, rồi chọn một mục để gọi chapter. Không ghi gì vào truyện.</DialogDescription>
        </DialogHeader>
        <div className="flex items-end gap-2">
          <div className="flex flex-1 flex-col gap-1.5">
            <Label htmlFor="ext-test-url">Link tải raw</Label>
            <Input id="ext-test-url" value={url} onChange={(e) => setUrl(e.target.value)} placeholder="https://…" />
          </div>
          <Button onClick={() => void runToc()} disabled={busy || !url.trim()}>
            {busy ? <LoaderCircle className="animate-spin" /> : <Play />} Chạy toc
          </Button>
        </div>
        {toc && (
          <ReportView label="toc" report={toc}>
            {toc.ok && (
              <div className="flex flex-col gap-1">
                <p className="text-sm">
                  {items.length} mục{items.length > MAX_ROWS ? ` (hiện ${MAX_ROWS} mục đầu)` : ""}
                </p>
                <ul aria-label="Mục lục chạy thử" className="fine-scrollbar max-h-56 divide-y overflow-y-auto rounded border text-sm">
                  {items.slice(0, MAX_ROWS).map((item, i) => (
                    <li key={item.id} className="flex items-center gap-2 px-3 py-1">
                      <span className="w-10 shrink-0 text-right font-mono text-xs text-muted-foreground">{i + 1}</span>
                      <span className="min-w-0 flex-1 truncate">{item.title}</span>
                      {item.locked && <span className="text-xs text-muted-foreground">khoá</span>}
                      <span className="font-mono text-xs text-muted-foreground">{item.id}</span>
                      <Button variant="ghost" size="sm" disabled={busy} aria-label={`Chạy chapter ${i + 1}`} onClick={() => void runChapter(item, i + 1)}>
                        chapter
                      </Button>
                    </li>
                  ))}
                </ul>
              </div>
            )}
          </ReportView>
        )}
        {chapter && (
          <ReportView label="chapter" report={chapter}>
            {chapter.ok && <ChapterResult result={chapter.result} />}
          </ReportView>
        )}
      </DialogContent>
    </Dialog>
  );
}

const chapterSchema = z.object({ saved: z.string().nullable(), chars: z.number().nullable(), short: z.boolean(), format: z.string() });

function ChapterResult({ result }: { result: unknown }) {
  const parsed = chapterSchema.safeParse(result);
  if (!parsed.success) return <pre className="text-xs">{JSON.stringify(result, null, 2)}</pre>;
  const { saved, chars, short, format } = parsed.data;
  return (
    <div className="flex flex-col gap-1 text-sm">
      <p>
        {saved === null ? "Nội dung trống — sẽ báo lỗi, không ghi file." : `${chars ?? 0} ký tự${short ? " · Ngắn" : ""} · định dạng ${format}`}
      </p>
      {saved !== null && <pre aria-label="Nội dung sẽ ghi" className="fine-scrollbar max-h-64 overflow-y-auto whitespace-pre-wrap rounded bg-muted p-3 text-xs">{saved}</pre>}
    </div>
  );
}

function ReportView({ label, report, children }: { label: string; report: ExtTestReport; children?: React.ReactNode }) {
  return (
    <section aria-label={`Kết quả ${label}`} className="flex flex-col gap-2 rounded-lg border p-3">
      <p className={report.ok ? "text-sm font-medium" : "text-sm font-medium text-destructive"}>
        {label}: {report.ok ? "xong" : `lỗi — ${report.error ?? ""}`} · {report.ms} ms · {report.requests.length} request
      </p>
      {children}
      {report.requests.length > 0 && (
        <ul aria-label={`Request ${label}`} className="fine-scrollbar max-h-40 overflow-y-auto font-mono text-xs">
          {report.requests.map((r, i) => (
            <li key={i} className={r.status !== null && r.status < 400 ? "" : "text-destructive"}>
              {r.method} {r.url} → {r.status ?? r.error ?? "?"} ({r.ms} ms)
            </li>
          ))}
        </ul>
      )}
      {report.logs.length > 0 && (
        <pre aria-label={`Log ${label}`} className="fine-scrollbar max-h-40 overflow-y-auto rounded bg-muted p-2 text-xs">
          {report.logs.map((l) => `[${l.level}] ${l.message}`).join("\n")}
        </pre>
      )}
    </section>
  );
}
