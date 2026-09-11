import { describe, expect, it, vi } from "vitest";

import { createUndoFallbackHandler, needsUndoFallback, undoCommandFor } from "@/hooks/use-undo-fallback";

function keydown(target: Element, init: KeyboardEventInit) {
  const event = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, ...init });
  target.dispatchEvent(event);
  return event;
}

describe("undo fallback cho WebKitGTK", () => {
  it("phân loại phím: Ctrl+Z undo, Ctrl+Shift+Z và Ctrl+Y redo, còn lại bỏ qua", () => {
    const k = (init: KeyboardEventInit) => undoCommandFor(new KeyboardEvent("keydown", init));
    expect(k({ key: "z", ctrlKey: true })).toBe("undo");
    expect(k({ key: "Z", ctrlKey: true, shiftKey: true })).toBe("redo");
    expect(k({ key: "y", ctrlKey: true })).toBe("redo");
    expect(k({ key: "z", metaKey: true })).toBe("undo");
    expect(k({ key: "z" })).toBeUndefined();
    expect(k({ key: "z", ctrlKey: true, altKey: true })).toBeUndefined();
    expect(k({ key: "s", ctrlKey: true })).toBeUndefined();
  });

  it("chỉ gọi execCommand cho input/textarea sửa được, chặn default; bỏ qua div, readOnly, đã preventDefault", () => {
    const exec = vi.fn(() => true);
    window.addEventListener("keydown", createUndoFallbackHandler(exec));
    const box = document.createElement("textarea");
    const input = document.createElement("input");
    const locked = document.createElement("textarea");
    locked.readOnly = true;
    const div = document.createElement("div");
    div.contentEditable = "true";
    document.body.append(box, input, locked, div);

    expect(keydown(box, { key: "z", ctrlKey: true }).defaultPrevented).toBe(true);
    expect(exec).toHaveBeenLastCalledWith("undo");
    expect(keydown(input, { key: "y", ctrlKey: true }).defaultPrevented).toBe(true);
    expect(exec).toHaveBeenLastCalledWith("redo");
    expect(keydown(locked, { key: "z", ctrlKey: true }).defaultPrevented).toBe(false);
    expect(keydown(div, { key: "z", ctrlKey: true }).defaultPrevented).toBe(false);
    const pre = new KeyboardEvent("keydown", { bubbles: true, cancelable: true, key: "z", ctrlKey: true });
    pre.preventDefault();
    box.dispatchEvent(pre);
    expect(exec).toHaveBeenCalledTimes(2);
  });

  it("chỉ bật trên WebKit Linux (WebKitGTK), không bật trên WebView2/Chromium hay macOS", () => {
    expect(needsUndoFallback("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15")).toBe(true);
    expect(needsUndoFallback("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36 Edg/124.0")).toBe(false);
    expect(needsUndoFallback("Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15")).toBe(false);
  });
});
