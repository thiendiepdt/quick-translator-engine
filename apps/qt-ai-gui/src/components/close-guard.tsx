import { getCurrentWindow } from "@tauri-apps/api/window";
import { LoaderCircle } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { storyLabel } from "@/hooks/use-session-events";
import { sessionsStopAll } from "@/lib/api";
import { pathKey } from "@/lib/paths";
import { runningRoots, useStoryStore } from "@/store/story";

/**
 * Bấm X khi đang dịch: hỏi trước rồi dừng mọi phiên (giết agy) mới đóng. Trước đây đóng app để agy chạy mồ côi —
 * dịch tiếp, ăn quota, giữ folder truyện không xoá được. App crash thì core child_guard lo (Job Object / PDEATHSIG).
 */
export function CloseGuard() {
  const [running, setRunning] = useState<string[]>([]);
  const [stopping, setStopping] = useState(false);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let active = true;
    let current: ReturnType<typeof getCurrentWindow>;
    try {
      current = getCurrentWindow();
    } catch {
      return; // Không chạy trong Tauri (test, trình duyệt): không có cửa sổ để chặn đóng.
    }
    void current
      .onCloseRequested((event) => {
        const roots = runningRoots(useStoryStore.getState());
        if (roots.length === 0) return;
        event.preventDefault();
        setRunning(roots);
      })
      .then((stop) => {
        if (active) unlisten = stop;
        else stop();
      });
    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  async function stopAndClose() {
    setStopping(true);
    try {
      await sessionsStopAll();
    } catch (error) {
      // Dừng lỗi vẫn đóng: core đã buộc agy chết theo app.
      toast.error(error instanceof Error ? error.message : "Không dừng được phiên");
    }
    await getCurrentWindow().destroy();
  }

  const names = useStoryStore((s) => s.names);
  const label = (root: string) => names[pathKey(root)] || storyLabel(root);

  return (
    <Dialog open={running.length > 0} onOpenChange={(open) => !open && !stopping && setRunning([])}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Đang dịch {running.length} truyện</DialogTitle>
          <DialogDescription>
            Đóng app sẽ dừng các phiên dịch này. Chương đang dịch dở được giữ lại — mở app bấm Dịch là dịch tiếp chương đó.
          </DialogDescription>
        </DialogHeader>
        <ul className="list-disc pl-5 text-sm">
          {running.map((root) => (
            <li key={root}>{label(root)}</li>
          ))}
        </ul>
        <DialogFooter>
          <Button variant="outline" disabled={stopping} onClick={() => setRunning([])}>
            Huỷ
          </Button>
          <Button variant="destructive" disabled={stopping} onClick={() => void stopAndClose()}>
            {stopping && <LoaderCircle className="animate-spin" />}
            {stopping ? "Đang dừng…" : "Dừng và đóng"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
