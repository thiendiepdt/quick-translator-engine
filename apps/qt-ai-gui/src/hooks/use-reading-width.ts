import { toast } from "sonner";

import { appConfigSet } from "@/lib/api";
import { readingWidthOf, type ReadingWidth } from "@/lib/reading";
import { useStoryStore } from "@/store/story";

/** Chiều ngang vùng đọc lưu trong AppConfig như palette — đổi ở trang đọc hay Cài đặt đều đồng bộ. */
export function useReadingWidth() {
  const config = useStoryStore((s) => s.config);
  const setConfig = useStoryStore((s) => s.setConfig);
  const width = readingWidthOf(config?.readingWidth);
  async function setWidth(next: ReadingWidth) {
    if (!config || next === width) return;
    try {
      setConfig(await appConfigSet({ ...config, readingWidth: next }));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được chiều ngang");
    }
  }
  return { width, setWidth };
}
