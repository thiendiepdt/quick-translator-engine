import "fake-indexeddb/auto";
import "@testing-library/jest-dom/vitest";

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

Object.defineProperty(Element.prototype, "scrollIntoView", {
  configurable: true,
  value: () => undefined,
});
