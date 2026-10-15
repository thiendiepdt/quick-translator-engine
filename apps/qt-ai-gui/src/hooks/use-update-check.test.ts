import { describe, expect, it, vi } from "vitest";

import { runUpdateCheck, updatePrompt, type UpdateCheckDeps, type UpdateHandle } from "./use-update-check";

function deps(overrides: Partial<UpdateCheckDeps> = {}, body?: string): UpdateCheckDeps & { calls: string[] } {
  const calls: string[] = [];
  const update: UpdateHandle = {
    version: "0.2.0",
    body,
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

  it("có release notes: hộp thoại hỏi kèm danh sách Có gì mới", async () => {
    const d = deps({}, "- Thêm **EPUB**.\n- Sửa `out/` lệch.");
    await runUpdateCheck(d);
    expect(d.ask).toHaveBeenCalledWith(
      "Phiên bản mới 0.2.0 đã có!\n\nCó gì mới:\n• Thêm EPUB.\n• Sửa out/ lệch.\n\nBạn có muốn cập nhật ngay không?",
      expect.anything(),
    );
  });
});

describe("updatePrompt", () => {
  it("không có notes (hoặc không có bullet) thì giữ câu hỏi ngắn", () => {
    expect(updatePrompt("1.0.22")).toBe("Phiên bản mới 1.0.22 đã có! Bạn có muốn cập nhật ngay không?");
    expect(updatePrompt("1.0.22", "")).toBe("Phiên bản mới 1.0.22 đã có! Bạn có muốn cập nhật ngay không?");
    expect(updatePrompt("1.0.22", "Xem changelog bên dưới.")).toBe("Phiên bản mới 1.0.22 đã có! Bạn có muốn cập nhật ngay không?");
  });

  it("CRLF, dòng thừa được bỏ; quá 12 bullet thì cắt và thêm …", () => {
    const body = Array.from({ length: 14 }, (_, i) => `- Mục ${i + 1}.`).join("\r\n");
    const text = updatePrompt("1.0.22", `${body}\r\n`);
    const bullets = text.split("\n").filter((line) => line.startsWith("• "));
    expect(bullets).toHaveLength(12);
    expect(bullets[11]).toBe("• Mục 12.");
    expect(text).toContain("• Mục 12.\n…\n\nBạn có muốn cập nhật ngay không?");
    expect(text).not.toContain("\r");
  });

  it("bullet quá dài bị cắt ~160 ký tự kèm …; tổng phần ghi chú không quá ~1200 ký tự, cắt thì thêm …", () => {
    const long = "x".repeat(400);
    const text = updatePrompt("1.0.22", Array.from({ length: 12 }, (_, i) => `- ${i} ${long}`).join("\n"));
    const notes = text.slice(text.indexOf("Có gì mới:\n") + "Có gì mới:\n".length, text.indexOf("\n\nBạn có muốn"));
    const lines = notes.split("\n");
    expect(lines[0]).toBe(`• 0 ${"x".repeat(157)}…`);
    for (const line of lines) expect(line.length).toBeLessThanOrEqual(162);
    expect(notes.length).toBeLessThanOrEqual(1200);
    expect(lines[lines.length - 1]).toBe("…");
    expect(lines.length).toBeLessThan(12);
  });

  it("ghi chú ngắn vừa ngân sách thì giữ nguyên, không thêm …", () => {
    const text = updatePrompt("1.0.22", "- Một.\n- Hai.");
    expect(text).not.toContain("…");
  });
});
