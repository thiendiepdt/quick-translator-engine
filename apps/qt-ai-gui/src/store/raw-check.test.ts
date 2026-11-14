import { beforeEach, describe, expect, it, vi } from "vitest";

import { rawInfo, rawToc, type RawInfo, type RawToc } from "@/lib/api";
import { invalidateRawChecks, loadRawToc, rawConfigured, rawProblem, rememberRawToc, resetRawChecks, useRawNew } from "@/store/raw-check";

vi.mock("@/lib/api", () => ({ rawInfo: vi.fn(), rawToc: vi.fn() }));

const info = {
  rawUrl: "https://a.test/book/1",
  extension: { id: "x", name: "Demo" },
  candidates: [{ id: "x", name: "Demo" }],
  installed: 1,
  extConfig: { auth: { required: true, scheme: "apiKey" } },
} as unknown as RawInfo;

const toc = {
  rows: [
    { index: 1, id: "1", title: "a", locked: false, file: "a", status: "have" },
    { index: 2, id: "2", title: "b", locked: false, file: null, status: "missing" },
    { index: 3, id: "3", title: "c", locked: true, file: null, status: "locked" },
  ],
  lastIndex: 1,
} as unknown as RawToc;

describe("raw-check", () => {
  beforeEach(() => {
    resetRawChecks();
    vi.mocked(rawInfo).mockReset().mockResolvedValue(info);
    vi.mocked(rawToc).mockReset().mockResolvedValue(toc);
  });

  it("đủ cài đặt mới tải; mở truyện và vào trang dùng chung một lần tải", async () => {
    const [a, b] = await Promise.all([loadRawToc("D:\\s"), loadRawToc("D:\\s")]);
    expect(a).toBe(b);
    expect(rawToc).toHaveBeenCalledTimes(1);
    expect(useRawNew.getState().counts["D:\\s"]).toBe(1);
    await loadRawToc("D:\\s", { force: true });
    expect(rawToc).toHaveBeenCalledTimes(2);
    rememberRawToc("D:\\s", { ...toc, rows: toc.rows.map((r) => ({ ...r, status: "have" as const })) });
    expect(useRawNew.getState().counts["D:\\s"]).toBe(0);
  });

  it("chưa đủ cài đặt thì không gọi extension", async () => {
    vi.mocked(rawInfo).mockResolvedValue({ ...info, extConfig: { auth: { required: true, scheme: null } } } as unknown as RawInfo);
    expect(await loadRawToc("D:\\t")).toBeNull();
    expect(rawToc).not.toHaveBeenCalled();
    expect(useRawNew.getState().counts["D:\\t"]).toBeNull();
    expect(rawConfigured({ ...info, extension: null })).toBe(false);
    expect(useRawNew.getState().problems["D:\\t"]).toBe("Demo chưa nhập API key / đăng nhập");
  });

  it("lý do chưa tải raw được: link, extension, chọn extension, auth", () => {
    const with_ = (patch: Partial<RawInfo>) => rawProblem({ ...info, ...patch });
    expect(with_({ rawUrl: "  " })).toBe("chưa có link tải raw");
    expect(with_({ rawUrl: "https://www.book.qq.com/x", candidates: [], extension: null, installed: 0 })).toBe("chưa cài extension tải raw nào");
    expect(with_({ rawUrl: "https://www.book.qq.com/x", candidates: [], extension: null, installed: 3 })).toBe("chưa có extension tải raw cho book.qq.com");
    expect(with_({ candidates: [info.extension!, info.extension!], extension: null })).toBe("có 2 extension khớp link, chưa chọn cái nào");
    expect(with_({ extConfig: null })).toBeNull();
  });

  it("đổi extension / kho / auth thì bỏ kết quả đã nhớ, tải lại", async () => {
    await loadRawToc("D:\\u");
    invalidateRawChecks();
    expect(useRawNew.getState().epoch).toBe(1);
    await loadRawToc("D:\\u");
    expect(rawToc).toHaveBeenCalledTimes(2);
  });
});
