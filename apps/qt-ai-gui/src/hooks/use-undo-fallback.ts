import { useEffect } from "react";

/**
 * WebKitGTK (Tauri trên Linux) không nối Ctrl+Z / Ctrl+Shift+Z / Ctrl+Y với undo/redo của input và
 * textarea: phím tới DOM nhưng không có gì xảy ra, trong khi `document.execCommand("undo")` vẫn chạy
 * đúng (đo thực tế trên WebKitGTK 2.52). Hook này bắt phím ở window và gọi execCommand thay trình
 * duyệt. Chỉ đụng input/textarea có thể sửa; editor contenteditable (Plate) có history riêng, để yên.
 */
export type UndoCommand = "undo" | "redo";

export function undoCommandFor(event: KeyboardEvent): UndoCommand | undefined {
  if (!(event.ctrlKey || event.metaKey) || event.altKey) return undefined;
  const key = event.key.toLowerCase();
  if (key === "z") return event.shiftKey ? "redo" : "undo";
  if (key === "y" && !event.shiftKey) return "redo";
  return undefined;
}

export function isEditableField(target: EventTarget | null): target is HTMLInputElement | HTMLTextAreaElement {
  if (!(target instanceof HTMLTextAreaElement || target instanceof HTMLInputElement)) return false;
  return !target.readOnly && !target.disabled;
}

/** Bắt undo/redo chỉ khi trình duyệt là WebKit trên Linux; nơi khác phím tắt vốn đã chạy. */
export function needsUndoFallback(userAgent = navigator.userAgent): boolean {
  return /Linux/.test(userAgent) && /AppleWebKit/.test(userAgent) && !/Chrome|Chromium/.test(userAgent);
}

export function createUndoFallbackHandler(exec: (command: UndoCommand) => boolean = (c) => document.execCommand(c)) {
  return (event: KeyboardEvent) => {
    if (event.defaultPrevented) return;
    const command = undoCommandFor(event);
    if (!command || !isEditableField(event.target)) return;
    event.preventDefault();
    exec(command);
  };
}

export function useUndoFallback(enabled = needsUndoFallback()) {
  useEffect(() => {
    if (!enabled) return;
    const onKey = createUndoFallbackHandler();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [enabled]);
}
