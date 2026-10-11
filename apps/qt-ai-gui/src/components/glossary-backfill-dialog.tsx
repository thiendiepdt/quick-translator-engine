import { Replace, Search } from "lucide-react";
import { useEffect, useState } from "react";
import { z } from "zod";

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
import { glossaryBackfill, glossaryScan } from "@/lib/api";
import { glossaryUsageSchema } from "@/lib/schema";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  root: string;
  /** Khoá Hán của mục glossary đang rà. */
  source: string;
  /** Cách dịch hiện tại của mục (giá trị trên form, kể cả chưa lưu). */
  target: string;
  /** Phiên dịch đang chạy thì không cho ghi chương cũ. */
  running: boolean;
}

type Usage = z.infer<typeof glossaryUsageSchema>;

/**
 * Rà ngược chương đã dịch theo một mục glossary: đoạn raw có khoá mà bản dịch thiếu cách dịch hiện tại (mục học
 * muộn, tên tác giả gõ lệch, vừa đổi bản dịch). Người dùng gõ cụm sai, xem trước số chỗ, rồi thay — chỉ trong các
 * đoạn có khoá ở raw.
 */
export function GlossaryBackfillDialog({ open, onOpenChange, root, source, target, running }: Props) {
  const [usage, setUsage] = useState<Usage>();
  const [from, setFrom] = useState("");
  const [to, setTo] = useState(target);
  const [preview, setPreview] = useState<string>();
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);

  // Hộp thoại chỉ được dựng khi mở (gỡ khi đóng), nên state khởi tạo đã sạch — effect chỉ rà.
  useEffect(() => {
    let alive = true;
    void glossaryScan(root, source, target)
      .then((result) => alive && setUsage(result))
      .catch((error: unknown) => alive && setStatus(error instanceof Error ? error.message : "Không rà được"));
    return () => {
      alive = false;
    };
  }, [root, source, target]);

  async function run(apply: boolean) {
    setBusy(true);
    try {
      const result = await glossaryBackfill(root, source, from, to, apply);
      const summary = `${result.occurrences} chỗ trong ${result.paragraphs} đoạn, ${result.chapters} chương`;
      if (apply) {
        setPreview(undefined);
        setStatus(`Đã thay ${summary}.`);
        setUsage(await glossaryScan(root, source, target));
      } else {
        setPreview(result.occurrences > 0 ? `Sẽ thay ${summary}.` : "Không có chỗ nào khớp cụm này trong các đoạn có khoá.");
      }
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "Lệnh thất bại");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle>Rà chương cũ: {source}</DialogTitle>
          <DialogDescription>
            Đoạn đã dịch mà raw có <code>{source}</code> nhưng bản dịch không có “{target}”.
          </DialogDescription>
        </DialogHeader>
        {!usage && !status && <p className="text-sm text-muted-foreground">Đang rà…</p>}
        {usage && (
          <div className="flex flex-col gap-2">
            <p className="text-sm" role="status" aria-label="Kết quả rà">
              {usage.total === 0
                ? "Mọi đoạn có khoá đều đã dùng đúng cách dịch."
                : `${usage.total} đoạn ở ${usage.chapters} chương${usage.total > usage.hits.length ? ` (hiện ${usage.hits.length} đoạn đầu)` : ""}.`}
              {usage.skipped > 0 && ` Bỏ qua ${usage.skipped} chương số đoạn lệch raw.`}
            </p>
            {usage.hits.length > 0 && (
              <ul className="max-h-60 overflow-y-auto rounded-md border bg-muted/30 p-2 text-xs" aria-label="Đoạn thiếu cách dịch">
                {usage.hits.map((hit) => (
                  <li key={`${hit.chapter}-${hit.paragraph}`} className="py-0.5">
                    <span className="font-mono text-muted-foreground">
                      {hit.chapter} [[{hit.paragraph + 1}]]
                    </span>{" "}
                    {hit.text}
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
        <div className="grid grid-cols-[auto_1fr] items-center gap-2 text-sm">
          <label htmlFor="backfill-from">Cụm sai</label>
          <Input
            id="backfill-from"
            value={from}
            onChange={(e) => {
              setFrom(e.target.value);
              setPreview(undefined);
            }}
            placeholder="vd. Lục Đình Vân"
          />
          <label htmlFor="backfill-to">Thay bằng</label>
          <Input
            id="backfill-to"
            value={to}
            onChange={(e) => {
              setTo(e.target.value);
              setPreview(undefined);
            }}
          />
        </div>
        {preview && <p className="text-sm">{preview}</p>}
        {status && <p className="rounded-md border bg-muted/40 px-3 py-2 text-xs">{status}</p>}
        <DialogFooter>
          <Button type="button" variant="outline" disabled={busy || !from || from === to} onClick={() => void run(false)}>
            <Search /> Xem trước
          </Button>
          <Button type="button" disabled={busy || running || !preview || !preview.startsWith("Sẽ thay")} onClick={() => void run(true)}>
            <Replace /> Thay trong chương cũ
          </Button>
        </DialogFooter>
        {running && <p className="text-xs text-muted-foreground">Truyện đang dịch — bấm Dừng trước khi thay.</p>}
      </DialogContent>
    </Dialog>
  );
}
