import { describe, expect, it, vi } from "vitest";

import { runUpdateCheck, type UpdateCheckDeps } from "./use-update-check";

function deps(overrides: Partial<UpdateCheckDeps> = {}): UpdateCheckDeps & { calls: string[] } {
  const calls: string[] = [];
  const update = {
    version: "0.2.0",
    downloadAndInstall: vi.fn(() => {
      calls.push("install");
      return Promise.resolve();
    }),
  };
  return {
    calls,
    check: vi.fn(() => Promise.resolve(update)),
    ask: vi.fn(() => {
      calls.push("ask");
      return Promise.resolve(true);
    }),
    message: vi.fn(() => {
      calls.push("message");
      return Promise.resolve();
    }),
    relaunch: vi.fn(() => {
      calls.push("relaunch");
      return Promise.resolve();
    }),
    warn: vi.fn(),
    ...overrides,
  };
}

describe("runUpdateCheck", () => {
  it("có bản mới và đồng ý: hỏi → tải → báo → khởi động lại", async () => {
    const d = deps();
    await expect(runUpdateCheck(d)).resolves.toBe("installed");
    expect(d.calls).toEqual(["ask", "install", "message", "relaunch"]);
    expect(d.ask).toHaveBeenCalledWith(
      expect.stringContaining("0.2.0"),
      expect.objectContaining({ title: "Cập nhật VNCVT AI Translator" }),
    );
  });

  it("từ chối thì không tải", async () => {
    const d = deps({ ask: vi.fn(() => Promise.resolve(false)) });
    await expect(runUpdateCheck(d)).resolves.toBe("declined");
    expect(d.calls).toEqual([]);
    expect(d.relaunch).not.toHaveBeenCalled();
  });

  it("không có bản mới thì không hỏi", async () => {
    const d = deps({ check: vi.fn(() => Promise.resolve(null)) });
    await expect(runUpdateCheck(d)).resolves.toBe("none");
    expect(d.ask).not.toHaveBeenCalled();
  });

  it("check ném lỗi: chỉ warn, không ném ra ngoài", async () => {
    const error = new Error("offline");
    const d = deps({
      check: vi.fn(() => Promise.reject(error)),
    });
    await expect(runUpdateCheck(d)).resolves.toBe("failed");
    expect(d.warn).toHaveBeenCalledWith("Kiểm tra cập nhật thất bại:", error);
    expect(d.ask).not.toHaveBeenCalled();
  });
});
