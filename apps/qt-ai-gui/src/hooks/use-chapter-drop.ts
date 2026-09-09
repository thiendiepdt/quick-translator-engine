import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useEffect, useState } from "react";

import { importChapters } from "@/lib/api";
import type { ImportOutcome } from "@/lib/types";

/** Câu toast sau khi thả: chỉ nêu phần khác 0. */
export function describeImport(outcome: Pick<ImportOutcome, "added" | "skippedExisting" | "ignored">): string {
  const parts = [`Đã thêm ${outcome.added.length} chương`];
  if (outcome.skippedExisting.length > 0) parts.push(`bỏ qua ${outcome.skippedExisting.length} trùng tên`);
  if (outcome.ignored.length > 0) parts.push(`${outcome.ignored.length} file không phải .txt`);
  return parts.join(", ");
}

/**
 * Kéo thả file vào cửa sổ khi đang mở truyện → copy vào raw/ (Rust `import_chapters`).
 * Trả `dragging` để trang vẽ lớp phủ "Thả để thêm".
 */
export function useChapterDrop(
  root: string | undefined,
  onImported: (outcome: ImportOutcome) => void,
  onError: (message: string) => void,
): boolean {
  const [dragging, setDragging] = useState(false);

  useEffect(() => {
    if (!root) return;
    let unlisten: (() => void) | undefined;
    let disposed = false;
    getCurrentWebview()
      .onDragDropEvent((event) => {
        const payload = event.payload;
        if (payload.type === "enter" || payload.type === "over") {
          setDragging(true);
        } else if (payload.type === "leave") {
          setDragging(false);
        } else if (payload.type === "drop") {
          setDragging(false);
          importChapters(root, payload.paths)
            .then(onImported)
            .catch((error: unknown) => onError(error instanceof Error ? error.message : "Không nhập được chương"));
        }
      })
      .then((stop) => {
        if (disposed) stop();
        else unlisten = stop;
      })
      .catch(() => undefined);
    return () => {
      disposed = true;
      unlisten?.();
      setDragging(false);
    };
  }, [root, onImported, onError]);

  return dragging;
}
