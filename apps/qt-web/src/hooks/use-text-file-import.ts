import { type DragEvent, useRef, useState } from "react";
import { toast } from "sonner";

import { looksLikeTextFile, readChapterFile } from "@/lib/text-file";

/** Nguồn văn bản vừa nhập — component dùng để đặt lời toast/hoàn tác. */
export type TextImportSource = { kind: "file"; name: string } | { kind: "clipboard" };

/**
 * Nhập văn bản từ file (nút chọn file hoặc kéo-thả) và từ clipboard (nút
 * Dán cho những khối không có ô nhập để Ctrl+V). Hook chỉ lo đọc/dò encoding
 * và trạng thái kéo-thả; đổ text vào đâu, toast gì do component quyết định.
 */
export function useTextFileImport(onLoaded: (text: string, source: TextImportSource) => void) {
  // Đếm dragenter/dragleave vì chúng bắn lại ở mỗi phần tử con bên trong.
  const dragDepthRef = useRef(0);
  const [dropActive, setDropActive] = useState(false);

  async function importFile(file: File) {
    if (!looksLikeTextFile(file)) {
      toast.error("Chỉ nhận file văn bản (.txt)");
      return;
    }
    try {
      onLoaded(await readChapterFile(file), { kind: "file", name: file.name });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không đọc được file");
    }
  }

  async function importClipboard() {
    let text: string;
    try {
      text = await navigator.clipboard.readText();
    } catch (error) {
      toast.error("Trình duyệt không cho phép đọc clipboard", {
        description: "Cấp quyền clipboard cho trang, hoặc dán bằng Ctrl+V vào ô nhập.",
      });
      if (import.meta.env.DEV) console.warn("clipboard read failed", error);
      return;
    }
    if (!text.trim()) {
      toast.error("Clipboard đang trống");
      return;
    }
    onLoaded(text, { kind: "clipboard" });
  }

  function dragHasFile(event: DragEvent) {
    return Array.from(event.dataTransfer.types).includes("Files");
  }

  const dropHandlers = {
    onDragEnter: (event: DragEvent) => {
      if (!dragHasFile(event)) return;
      event.preventDefault();
      dragDepthRef.current += 1;
      setDropActive(true);
    },
    onDragOver: (event: DragEvent) => {
      if (dragHasFile(event)) event.preventDefault();
    },
    onDragLeave: (event: DragEvent) => {
      if (!dragHasFile(event)) return;
      dragDepthRef.current = Math.max(0, dragDepthRef.current - 1);
      if (dragDepthRef.current === 0) setDropActive(false);
    },
    onDrop: (event: DragEvent) => {
      if (!dragHasFile(event)) return;
      event.preventDefault();
      dragDepthRef.current = 0;
      setDropActive(false);
      const file = event.dataTransfer.files[0];
      if (file) void importFile(file);
    },
  };

  return { dropActive, dropHandlers, importFile, importClipboard };
}
