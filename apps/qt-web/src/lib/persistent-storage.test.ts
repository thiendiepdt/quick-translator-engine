import { afterEach, describe, expect, it, vi } from "vitest";

import { ensurePersistentStorage } from "@/lib/persistent-storage";

afterEach(() => {
  vi.unstubAllGlobals();
});

function stubStorage(value: unknown) {
  vi.stubGlobal("navigator", Object.create(navigator, {
    storage: { value, configurable: true },
  }));
}

describe("ensurePersistentStorage", () => {
  it("reports an already persisted origin without re-requesting", async () => {
    const persist = vi.fn();
    stubStorage({ persisted: () => Promise.resolve(true), persist });
    await expect(ensurePersistentStorage()).resolves.toBe("persisted");
    expect(persist).not.toHaveBeenCalled();
  });

  it("requests persistence and reports the grant", async () => {
    stubStorage({
      persisted: () => Promise.resolve(false),
      persist: () => Promise.resolve(true),
    });
    await expect(ensurePersistentStorage()).resolves.toBe("granted");
  });

  it("reports denial so the app can warn about eviction", async () => {
    stubStorage({
      persisted: () => Promise.resolve(false),
      persist: () => Promise.resolve(false),
    });
    await expect(ensurePersistentStorage()).resolves.toBe("denied");
  });

  it("treats missing or throwing APIs as unsupported", async () => {
    stubStorage(undefined);
    await expect(ensurePersistentStorage()).resolves.toBe("unsupported");
    stubStorage({
      persisted: () => Promise.reject(new Error("blocked")),
      persist: () => Promise.resolve(true),
    });
    await expect(ensurePersistentStorage()).resolves.toBe("unsupported");
  });
});
