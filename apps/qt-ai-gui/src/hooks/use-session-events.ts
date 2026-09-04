import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { toast } from "sonner";

import { storySnapshot } from "@/lib/api";
import { sessionEventSchema } from "@/lib/schema";
import { stopReasonLabel } from "@/lib/types";
import { useStoryStore } from "@/store/story";

/** Nghe `session-event` từ Rust; khi phiên dừng thì nạp lại snapshot để bảng chương khớp state.json. */
export function useSessionEvents() {
  useEffect(() => {
    let cancelled = false;
    const unlisten = listen("session-event", (raw) => {
      const parsed = sessionEventSchema.safeParse(raw.payload);
      if (!parsed.success) return;
      const event = parsed.data;
      const store = useStoryStore.getState();
      store.applySessionEvent(event);
      if (event.type === "stopped") {
        const { type: _type, ...reason } = event;
        const message = stopReasonLabel(reason);
        if (reason.kind === "finished") toast.success(message);
        else if (reason.kind === "user_cancelled") toast.message(message);
        else toast.error(message);
      }
      if ((event.type === "stopped" || event.type === "progress") && store.root) {
        void storySnapshot(store.root)
          .then((snapshot) => {
            if (!cancelled) useStoryStore.getState().setSnapshot(snapshot);
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
