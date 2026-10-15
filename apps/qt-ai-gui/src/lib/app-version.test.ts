import { getVersion } from "@tauri-apps/api/app";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { appVersion, PACKAGE_VERSION } from "@/lib/app-version";

import packageJson from "../../package.json";

vi.mock("@tauri-apps/api/app", () => ({ getVersion: vi.fn() }));

describe("appVersion", () => {
  beforeEach(() => {
    vi.mocked(getVersion).mockReset();
  });

  it("lấy version của bản cài qua Tauri getVersion", async () => {
    vi.mocked(getVersion).mockResolvedValue("1.0.30");
    await expect(appVersion()).resolves.toBe("1.0.30");
  });

  it("ngoài Tauri (dev server, test) getVersion lỗi thì dùng version trong package.json", async () => {
    vi.mocked(getVersion).mockRejectedValue(new TypeError("window.__TAURI_INTERNALS__ is undefined"));
    expect(PACKAGE_VERSION).toBe(packageJson.version);
    await expect(appVersion()).resolves.toBe(packageJson.version);
  });

  it("getVersion ném lỗi đồng bộ cũng không lọt ra ngoài", async () => {
    vi.mocked(getVersion).mockImplementation(() => {
      throw new Error("boom");
    });
    await expect(appVersion()).resolves.toBe(PACKAGE_VERSION);
  });
});
