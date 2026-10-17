import { ShieldAlert, Square } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { onVnLibraryProgress, vnScanLibrary, vnScanLibraryCancel } from "@/lib/api";
import type { RecentSummary } from "@/lib/types";

/** Câu báo sau khi quét xong: bao nhiêu truyện còn chỗ nhạy cảm VN chưa xác minh. */
function describeVnScan(scanned: number, total: number, cancelled: boolean, stories: RecentSummary[]): string {
  const flagged = stories.filter((story) => (story.vnFlags ?? 0) > 0);
  const spots = flagged.reduce((sum, story) => sum + (story.vnFlags ?? 0), 0);
  const head = cancelled ? `Đã dừng sau ${scanned}/${total} truyện` : `Quét nhạy cảm VN xong ${total} truyện`;
  const tail = flagged.length > 0 ? `${flagged.length} truyện còn ${spots} chỗ chưa xác minh (⚠ VN)` : "không truyện nào còn chỗ chưa xác minh";
  return `${head}: ${tail}.`;
}

/**
 * Nút "Quét nhạy cảm VN" của thư viện: quét nền mọi truyện (Rust, từng truyện, ít luồng — máy không giật), hiện
 * tiến độ trên nút; bấm lần nữa khi đang quét thì dừng sau truyện đang quét. Xong thì trả danh sách thư viện mới.
 */
export function VnLibraryScanButton({ disabled, onDone }: { disabled?: boolean; onDone: (stories: RecentSummary[]) => void }) {
  const [running, setRunning] = useState(false);
  const [progress, setProgress] = useState<{ done: number; total: number } | undefined>();
  const [stopping, setStopping] = useState(false);

  useEffect(() => {
    if (!running) return;
    let unlisten: (() => void) | undefined;
    let active = true;
    void onVnLibraryProgress(({ done, total }) => setProgress({ done, total })).then((stop) => {
      if (active) unlisten = stop;
      else stop();
    });
    return () => {
      active = false;
      unlisten?.();
    };
  }, [running]);

  async function start() {
    setRunning(true);
    setStopping(false);
    setProgress(undefined);
    try {
      const result = await vnScanLibrary();
      onDone(result.stories);
      toast.message(describeVnScan(result.scanned, result.total, result.cancelled, result.stories));
      if (result.failed.length > 0) {
        toast.error(`Không quét được ${result.failed.length} truyện (giữ kết quả cũ): ${result.failed.slice(0, 3).join(", ")}`);
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không quét được thư viện");
    } finally {
      setRunning(false);
      setProgress(undefined);
    }
  }

  async function stop() {
    setStopping(true);
    try {
      await vnScanLibraryCancel();
    } catch {
      setStopping(false);
    }
  }

  if (running) {
    return (
      <Button
        variant="ghost"
        size="sm"
        disabled={stopping}
        title="Dừng sau truyện đang quét"
        aria-label="Dừng quét nhạy cảm VN"
        onClick={() => void stop()}
      >
        <Square className="animate-pulse" />
        {stopping ? "Đang dừng…" : progress ? `Đang quét ${progress.done}/${progress.total}…` : "Đang quét…"}
      </Button>
    );
  }
  return (
    <Button
      variant="ghost"
      size="sm"
      disabled={disabled}
      title="Quét chỗ nhạy cảm về Việt Nam trong mọi truyện của thư viện (chạy nền, chỉ đọc chương mới/đổi)"
      aria-label="Quét nhạy cảm VN cả thư viện"
      onClick={() => void start()}
    >
      <ShieldAlert /> Quét nhạy cảm VN
    </Button>
  );
}
