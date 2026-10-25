import { FileType, TriangleAlert, Undo2 } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { storySnapshot, zhStoryConvert, zhStoryStatus, zhStoryUndo } from "@/lib/api";
import { zhStoryStatusSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

type ZhStoryStatus = z.infer<typeof zhStoryStatusSchema>;

const FILE_LABELS: Record<string, string> = { "story.json": "Hồ sơ truyện", "cast.json": "bảng nhân vật", "titles.json": "sổ tiêu đề" };

interface DialogProps {
  root: string;
  running: boolean;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Raw / hồ sơ vừa đổi — thanh nhắc đọc lại. */
  onChanged: () => void;
}

/** Chuyển raw phồn thể của truyện sang giản thể: đọc cả truyện để báo số chương, chuyển, hoặc hoàn tác lần gần nhất. */
export function ZhStoryDialog({ open, onOpenChange, ...props }: DialogProps) {
  const [busy, setBusy] = useState(false);
  return (
    <Dialog open={open} onOpenChange={(value) => !busy && onOpenChange(value)}>
      <DialogContent>
        {/* Nội dung chỉ gắn khi hộp mở, nên mỗi lần mở đọc lại raw từ đầu. */}
        <ZhStoryBody {...props} onOpenChange={onOpenChange} busy={busy} setBusy={setBusy} />
      </DialogContent>
    </Dialog>
  );
}

function ZhStoryBody({
  root,
  running,
  onOpenChange,
  onChanged,
  busy,
  setBusy,
}: Omit<DialogProps, "open"> & { busy: boolean; setBusy: (busy: boolean) => void }) {
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const [status, setStatus] = useState<ZhStoryStatus>();

  useEffect(() => {
    let alive = true;
    zhStoryStatus(root, true)
      .then((next) => alive && setStatus(next))
      .catch((error: unknown) => alive && toast.error(error instanceof Error ? error.message : "Không đọc được raw"));
    return () => {
      alive = false;
    };
  }, [root]);

  const pending = status ? status.traditional + status.notUtf8 : 0;
  const canUndo = typeof status?.undoChapters === "number";

  async function finish(message: string) {
    toast.success(message);
    setSnapshot(await storySnapshot(root));
    onChanged();
    onOpenChange(false);
  }

  async function convert() {
    setBusy(true);
    try {
      const done = await zhStoryConvert(root);
      const merged = done.merged ? ` (${done.merged} khoá trùng giữ mục có sẵn)` : "";
      const keys = done.keys ? `, đổi ${done.keys} khoá Hán trong hồ sơ${merged}` : "";
      await finish(`Đã chuyển ${done.chapters} chương sang giản thể${keys}. Bản gốc ở raw-phon-the/.`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không chuyển được");
    } finally {
      setBusy(false);
    }
  }

  async function undo() {
    setBusy(true);
    try {
      const done = await zhStoryUndo(root);
      const files = done.profileRestored ? ` và ${done.profileRestored} file hồ sơ` : "";
      const kept = done.conflicts.length ? ` ${done.conflicts.length} chương đã sửa sau đó nên giữ nguyên.` : "";
      const keptFiles = done.profileConflicts.length
        ? ` ${done.profileConflicts.map((name) => FILE_LABELS[name] ?? name).join(", ")} đã sửa sau đó nên giữ nguyên.`
        : "";
      await finish(`Đã trả ${done.restored} chương về bản phồn thể gốc${files}.${kept}${keptFiles}`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không hoàn tác được");
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <DialogHeader>
        <DialogTitle>Chuyển raw phồn thể sang giản thể</DialogTitle>
        <DialogDescription>
          Chương phồn thể và chương mã hoá GBK / Big5 được ghi lại thành giản thể UTF-8, bản gốc chép vào
          raw-phon-the/. Khoá Hán trong glossary, bảng nhân vật, sổ tiêu đề đổi theo để khớp tên. Chương vốn là giản
          thể và tên file chương giữ nguyên.
        </DialogDescription>
      </DialogHeader>
      <p role="status" className="rounded-md bg-muted p-3 text-sm">
        {!status
          ? "Đang đọc raw…"
          : pending === 0
            ? `Raw ${status.scanned} chương đã là giản thể UTF-8 — không có gì để chuyển.`
            : `${status.traditional} chương phồn thể${status.notUtf8 ? `, ${status.notUtf8} chương mã hoá khác UTF-8` : ""} trên ${status.scanned} chương.`}
      </p>
      {canUndo && (
        <div className="flex items-center gap-2 rounded-md border px-3 py-2 text-xs">
          <span className="flex-1">Lần chuyển gần nhất: {status?.undoChapters} chương.</span>
          <Button type="button" size="xs" variant="outline" disabled={busy || running} onClick={() => void undo()}>
            <Undo2 /> Hoàn tác
          </Button>
        </div>
      )}
      {running && <p className="text-xs text-muted-foreground">Truyện đang dịch — bấm Dừng trước khi chuyển.</p>}
      <DialogFooter>
        <Button variant="outline" disabled={busy} onClick={() => onOpenChange(false)}>
          Huỷ
        </Button>
        <Button disabled={busy || running || pending === 0} onClick={() => void convert()}>
          <FileType /> Chuyển {pending} chương
        </Button>
      </DialogFooter>
    </>
  );
}

/**
 * Thanh nhắc ở trang Dịch: mấy chương đầu có chương phồn thể thì mời chuyển (khoá glossary là giản thể nên tên riêng
 * không khớp). `version` đổi (vừa quét chương mới, vừa chuyển) thì đọc lại.
 */
export function ZhStoryBanner({ root, version, onOpen }: { root: string; version: string; onOpen: () => void }) {
  const [status, setStatus] = useState<ZhStoryStatus>();

  useEffect(() => {
    let alive = true;
    zhStoryStatus(root, false)
      .then((next) => alive && setStatus(next))
      .catch(() => undefined);
    return () => {
      alive = false;
    };
  }, [root, version]);

  if (!status || status.traditional === 0) return null;
  return (
    <div role="status" aria-label="Raw phồn thể" className="mt-2 flex flex-wrap items-center gap-1.5 text-xs text-status-warning">
      <TriangleAlert className="size-3.5" aria-hidden />
      <span>
        Raw truyện này là phồn thể ({status.traditional}/{status.scanned} chương đầu) — glossary và tên riêng (khoá giản
        thể) sẽ không khớp.
      </span>
      <Button type="button" size="xs" variant="outline" onClick={onOpen}>
        <FileType /> Chuyển sang giản thể…
      </Button>
    </div>
  );
}
