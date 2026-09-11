import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { toast } from "sonner";

import { storySnapshot } from "@/lib/api";
import { samePath } from "@/lib/paths";
import { rootedSessionEventSchema, sessionEventSchema } from "@/lib/schema";
import { stopReasonLabel } from "@/lib/types";
import { useStoryStore } from "@/store/story";

/** Tên folder cuối của root để toast phân biệt truyện khi nhiều phiên chạy. */
export function storyLabel(root: string): string {
  return root.replace(/[\\/]+$/, "").split(/[\\/]/).pop() || root;
}

/**
 * Nghe `session-event` từ Rust (`{root, type, ...}`), đưa vào store theo truyện; khi phiên của
 * truyện đang mở dừng/tiến triển thì nạp lại snapshot để bảng chương khớp state.json.
 */
export function useSessionEvents() {
  useEffect(() => {
    let cancelled = false;
    const unlisten = listen("session-event", (raw) => {
      const rooted = rootedSessionEventSchema.safeParse(raw.payload);
      const parsed = sessionEventSchema.safeParse(raw.payload);
      if (!rooted.success || !parsed.success) return;
      const { root } = rooted.data;
      const event = parsed.data;
      const store = useStoryStore.getState();
      store.applySessionEvent(root, event);
      if (event.type === "stopped") {
        const { type: _type, ...reason } = event;
        const message = `${storyLabel(root)}: ${stopReasonLabel(reason)}`;
        if (reason.kind === "finished") toast.success(message);
        else if (reason.kind === "user_cancelled") toast.message(message);
        else toast.error(message);
      }
      const current = store.root;
      if ((event.type === "stopped" || event.type === "progress") && current && samePath(current, root)) {
        void storySnapshot(current)
          .then((snapshot) => {
            const latest = useStoryStore.getState();
            if (!cancelled && latest.root && samePath(latest.root, root)) latest.setSnapshot(snapshot);
          })
          .catch(() => undefined);
      }
    });
    return () => {
      cancelled = true;
      void unlisten.then((fn) => fn());
    };
  }, []);
}
