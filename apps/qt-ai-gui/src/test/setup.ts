import "@testing-library/jest-dom/vitest";

import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

// Không bật globals nên RTL không tự cleanup — dọn DOM sau mỗi test để render không rò sang test khác.
afterEach(cleanup);

class ResizeObserverMock implements ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

globalThis.ResizeObserver = ResizeObserverMock;

if (!globalThis.PointerEvent) {
  globalThis.PointerEvent = MouseEvent as typeof PointerEvent;
}

Object.defineProperty(Element.prototype, "hasPointerCapture", {
  configurable: true,
  value: () => false,
});

Object.defineProperty(Element.prototype, "setPointerCapture", {
  configurable: true,
  value: () => undefined,
});

Object.defineProperty(Element.prototype, "releasePointerCapture", {
  configurable: true,
  value: () => undefined,
});

// Radix Select trong jsdom cần các API này khi mở/đóng danh sách (như setup của qt-web).
Object.defineProperty(Element.prototype, "hasPointerCapture", { configurable: true, value: () => false });
Object.defineProperty(Element.prototype, "setPointerCapture", { configurable: true, value: () => undefined });
Object.defineProperty(Element.prototype, "releasePointerCapture", { configurable: true, value: () => undefined });
Object.defineProperty(Element.prototype, "scrollIntoView", { configurable: true, value: () => undefined });
