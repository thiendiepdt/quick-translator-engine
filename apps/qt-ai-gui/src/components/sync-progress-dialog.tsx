import { RefreshCw } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { progressDrift, syncProgress } from "@/lib/api";
import type { ProgressDrift } from "@/lib/types";
import { isRunning, useStoryStore } from "@/store/story";

interface Props {
  root: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/**
 * Folder truyện đồng bộ qua Google Drive có lúc về đủ out/ nhưng state.json là bản cũ: chương đã dịch vẫn ghi chưa
 * dịch, bấm Dịch là dịch lại. Xem trước số chương lệch rồi đánh dấu đã dịch theo out/.
 */
export function SyncProgressDialog({ root, open, onOpenChange }: Props) {
  // Chỉ dựng khi mở (gỡ khi đóng): mỗi lần mở là state sạch và đối chiếu lại từ đầu.
  return open ? <SyncProgressContent key={root} root={root} onOpenChange={onOpenChange} /> : null;
}

function SyncProgressContent({ root, onOpenChange }: Omit<Props, "open">) {
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const running = useStoryStore((s) => isRunning(s, root));
  const [drift, setDrift] = useState<ProgressDrift>();
  const [error, setError] = useState<string>();
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let alive = true;
    progressDrift(root)
      .then((value) => alive && setDrift(value))
      .catch((e: unknown) => alive && setError(e instanceof Error ? e.message : "Không đọc được tiến độ"));
    return () => {
      alive = false;
    };
  }, [root]);

  async function run() {
    setBusy(true);
    try {
      const result = await syncProgress(root);
      setSnapshot(result.snapshot);
      toast.success(`Đã đồng bộ ${result.markedDone.length} chương`);
      onOpenChange(false);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : "Không đồng bộ được tiến độ");
    } finally {
      setBusy(false);
    }
  }

  const untracked = drift?.untracked.length ?? 0;
  const missing = drift?.missing.length ?? 0;
  return (
    <Dialog open onOpenChange={(value) => !busy && onOpenChange(value)}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Đồng bộ tiến độ</DialogTitle>
          <DialogDescription>Đánh dấu đã dịch theo bản dịch có trong out/, khi state.json về chậm hơn out/ sau lúc đồng bộ.</DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-2 rounded-md bg-muted p-3 text-sm">
          {error ? (
            <p className="text-destructive">{error}</p>
          ) : !drift ? (
            <p className="text-muted-foreground">Đang đối chiếu tiến độ với out/…</p>
          ) : (
            <>
              <p className="tabular-nums">{`${untracked} chương đã có bản dịch trong out/ nhưng tiến độ ghi chưa dịch → sẽ đánh dấu đã dịch.`}</p>
              {missing > 0 && (
                <p className="tabular-nums text-status-warning">
                  {`${missing} chương ghi đã dịch nhưng chưa thấy bản dịch — giữ nguyên (có thể Google Drive chưa đồng bộ xong).`}
                </p>
              )}
            </>
          )}
          {running && <p className="text-status-warning">Truyện đang có phiên dịch chạy — bấm Dừng trước khi đồng bộ.</p>}
        </div>
        <p className="text-xs text-muted-foreground">
          Cảnh báo từng chương, cặp xưng hô và glossary học ở máy kia nằm trong story.json / cast.json — đồng bộ được tới đâu có tới đó.
        </p>
        <DialogFooter>
          <Button variant="outline" disabled={busy} onClick={() => onOpenChange(false)}>
            Huỷ
          </Button>
          <Button disabled={busy || running || untracked === 0} onClick={() => void run()}>
            <RefreshCw /> Đồng bộ
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
