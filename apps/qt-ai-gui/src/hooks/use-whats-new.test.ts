import { getVersion } from "@tauri-apps/api/app";
import { act, renderHook, waitFor } from "@testing-library/react";
import { StrictMode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { decideWhatsNew, LAST_SEEN_KEY, useWhatsNew } from "@/hooks/use-whats-new";

vi.mock("@tauri-apps/api/app", () => ({ getVersion: vi.fn() }));

const settle = () => act(() => new Promise((r) => setTimeout(r, 20)));

describe("decideWhatsNew", () => {
  it("lên bản mới hơn: mở từ bản đã thấy, lưu bản mới", () => {
    expect(decideWhatsNew("1.0.19", "1.0.21", false)).toEqual({ since: "1.0.19", store: "1.0.21" });
  });

  it("cùng bản hoặc hạ bản: không mở, giữ bản cao nhất đã thấy", () => {
    expect(decideWhatsNew("1.0.21", "1.0.21", true)).toEqual({ since: undefined, store: "1.0.21" });
    expect(decideWhatsNew("1.0.22", "1.0.21", true)).toEqual({ since: undefined, store: "1.0.22" });
    expect(decideWhatsNew("1.0.9", "1.0.10", true)).toEqual({ since: "1.0.9", store: "1.0.10" });
  });

  it("chưa có khoá: có dấu hiệu đã dùng app → mở chỉ bản đang dùng (since null); cài mới → lưu im lặng", () => {
    expect(decideWhatsNew(null, "1.0.21", true)).toEqual({ since: null, store: "1.0.21" });
    expect(decideWhatsNew(null, "1.0.21", false)).toEqual({ since: undefined, store: "1.0.21" });
    expect(decideWhatsNew("rác", "1.0.21", false)).toEqual({ since: undefined, store: "1.0.21" });
  });
});

describe("useWhatsNew", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.mocked(getVersion).mockReset().mockResolvedValue("1.0.21");
  });
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("cài mới (không khoá, không dấu hiệu đã dùng): lưu version im lặng, không mở", async () => {
    const { result } = renderHook(() => useWhatsNew(false));
    await waitFor(() => expect(localStorage.getItem(LAST_SEEN_KEY)).toBe("1.0.21"));
    expect(result.current.version).toBeUndefined();
  });

  it("người dùng cũ chưa có khoá (bản đầu có tính năng này): mở Có gì mới của riêng bản đang dùng", async () => {
    const { result } = renderHook(() => useWhatsNew(true));
    await waitFor(() => expect(result.current.version).toBe("1.0.21"));
    expect(result.current.since).toBeNull();
    expect(localStorage.getItem(LAST_SEEN_KEY)).toBe("1.0.21");
  });

  it("chờ biết có dấu hiệu đã dùng (config chưa nạp) rồi mới quyết, và chỉ quyết một lần", async () => {
    const { result, rerender } = renderHook(({ prior }: { prior: boolean | undefined }) => useWhatsNew(prior), {
      initialProps: { prior: undefined as boolean | undefined },
    });
    await settle();
    expect(localStorage.getItem(LAST_SEEN_KEY)).toBeNull();
    rerender({ prior: false });
    await waitFor(() => expect(localStorage.getItem(LAST_SEEN_KEY)).toBe("1.0.21"));
    // Mở truyện đầu tiên làm recent không rỗng: không được coi là nâng cấp.
    localStorage.removeItem(LAST_SEEN_KEY);
    rerender({ prior: true });
    await settle();
    expect(result.current.version).toBeUndefined();
  });

  it("bỏ qua vài bản (1.0.19 → 1.0.21): mở với since 1.0.19, lưu 1.0.21; dismiss đóng", async () => {
    localStorage.setItem(LAST_SEEN_KEY, "1.0.19");
    const { result } = renderHook(() => useWhatsNew(true));
    await waitFor(() => expect(result.current.version).toBe("1.0.21"));
    expect(result.current.since).toBe("1.0.19");
    expect(localStorage.getItem(LAST_SEEN_KEY)).toBe("1.0.21");
    act(() => result.current.dismiss());
    expect(result.current.version).toBeUndefined();
  });

  it("hạ bản: không mở, giữ bản cao hơn đã thấy", async () => {
    localStorage.setItem(LAST_SEEN_KEY, "1.0.22");
    const { result } = renderHook(() => useWhatsNew(true));
    await settle();
    expect(result.current.version).toBeUndefined();
    expect(localStorage.getItem(LAST_SEEN_KEY)).toBe("1.0.22");
  });

  it("cùng version đã xem thì không mở", async () => {
    localStorage.setItem(LAST_SEEN_KEY, "1.0.21");
    const { result } = renderHook(() => useWhatsNew(true));
    await settle();
    expect(result.current.version).toBeUndefined();
  });

  it("StrictMode (effect chạy hai lần) vẫn mở hộp thoại", async () => {
    localStorage.setItem(LAST_SEEN_KEY, "1.0.20");
    const { result } = renderHook(() => useWhatsNew(true), { wrapper: StrictMode });
    await waitFor(() => expect(result.current.version).toBe("1.0.21"));
  });

  it("các bản mới chưa có mục nào trong CHANGELOG thì không mở hộp thoại rỗng, vẫn lưu version", async () => {
    localStorage.setItem(LAST_SEEN_KEY, "9.9.8");
    vi.mocked(getVersion).mockResolvedValue("9.9.9");
    const { result } = renderHook(() => useWhatsNew(true));
    await waitFor(() => expect(localStorage.getItem(LAST_SEEN_KEY)).toBe("9.9.9"));
    expect(result.current.version).toBeUndefined();
  });

  it("localStorage hỏng (ném lỗi) thì không vỡ", async () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("SecurityError");
    });
    const setItem = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("QuotaExceeded");
    });
    const { result } = renderHook(() => useWhatsNew(false));
    await waitFor(() => expect(setItem).toHaveBeenCalled());
    expect(result.current.version).toBeUndefined();
  });
});
