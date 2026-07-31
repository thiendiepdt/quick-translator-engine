import { beforeEach, describe, expect, it } from "vitest";

import { createIndexedDbStateStorage } from "@/lib/indexed-db-storage";

const storage = createIndexedDbStateStorage();

beforeEach(async () => {
  localStorage.clear();
  await storage.removeItem("test-workspace");
  await storage.removeItem("migrated-workspace");
});

describe("IndexedDB state storage", () => {
  it("writes, reads, and removes values", async () => {
    await storage.setItem("test-workspace", "large workspace data");
    expect(await storage.getItem("test-workspace")).toBe("large workspace data");

    await storage.removeItem("test-workspace");
    expect(await storage.getItem("test-workspace")).toBeNull();
  });

  it("migrates legacy localStorage only after writing IndexedDB", async () => {
    localStorage.setItem("legacy-workspace", "legacy data");
    const migratingStorage = createIndexedDbStateStorage({
      legacyLocalStorageKeys: ["legacy-workspace"],
    });

    expect(await migratingStorage.getItem("migrated-workspace")).toBe(
      "legacy data",
    );
    expect(localStorage.getItem("legacy-workspace")).toBeNull();
    expect(await storage.getItem("migrated-workspace")).toBe("legacy data");
  });
});
