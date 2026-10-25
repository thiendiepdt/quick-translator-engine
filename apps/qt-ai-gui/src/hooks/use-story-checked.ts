import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

import { storySnapshot } from "@/lib/api";
import { samePath } from "@/lib/paths";
import { storyCheckedSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

/**
 * Mở truyện trả giao diện ngay; quét kiểm tra VN và độ lệch tiến độ chạy nền rồi Rust phát `story-checked`. Truyện đó
 * còn đang mở thì nạp lại snapshot (cảnh báo VN mới) kèm số chương lệch để hiện banner đồng bộ.
 */
export function useStoryChecked() {
  useEffect(() => {
    let cancelled = false;
    const unlisten = listen("story-checked", (raw) => {
      const parsed = storyCheckedSchema.safeParse(raw.payload);
      if (!parsed.success) return;
      const { root, progressDrift } = parsed.data;
      const current = useStoryStore.getState().root;
      if (!current || !samePath(current, root)) return;
      void storySnapshot(current)
        .then((snapshot) => {
          const latest = useStoryStore.getState();
          if (!cancelled && latest.root && samePath(latest.root, root)) latest.setSnapshot({ ...snapshot, progressDrift });
        })
        .catch(() => undefined);
    });
    return () => {
      cancelled = true;
      void unlisten.then((fn) => fn());
    };
  }, []);
}
