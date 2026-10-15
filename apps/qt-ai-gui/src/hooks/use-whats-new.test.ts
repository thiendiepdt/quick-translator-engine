import { getVersion } from "@tauri-apps/api/app";
import { act, renderHook, waitFor } from "@testing-library/react";
import { StrictMode } from "react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { LAST_SEEN_KEY, useWhatsNew } from "@/hooks/use-whats-new";

vi.mock("@tauri-apps/api/app", () => ({ getVersion: vi.fn() }));

describe("useWhatsNew", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.mocked(getVersion).mockReset().mockResolvedValue("1.0.21");
  });
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("lần đầu chạy: lưu version im lặng, không mở hộp thoại", async () => {
    const { result } = renderHook(() => useWhatsNew());
    await waitFor(() => expect(localStorage.getItem(LAST_SEEN_KEY)).toBe("1.0.21"));
    expect(result.current.version).toBeUndefined();
  });

  it("vừa cập nhật từ bản khác: mở Có gì mới của bản đang dùng, lưu version mới; dismiss đóng", async () => {
    localStorage.setItem(LAST_SEEN_KEY, "1.0.20");
    const { result } = renderHook(() => useWhatsNew());
    await waitFor(() => expect(result.current.version).toBe("1.0.21"));
    expect(localStorage.getItem(LAST_SEEN_KEY)).toBe("1.0.21");
    act(() => result.current.dismiss());
    expect(result.current.version).toBeUndefined();
  });

  it("StrictMode (effect chạy hai lần) vẫn mở hộp thoại", async () => {
    localStorage.setItem(LAST_SEEN_KEY, "1.0.20");
    const { result } = renderHook(() => useWhatsNew(), { wrapper: StrictMode });
    await waitFor(() => expect(result.current.version).toBe("1.0.21"));
  });

  it("cùng version đã xem thì không mở", async () => {
    localStorage.setItem(LAST_SEEN_KEY, "1.0.21");
    const { result } = renderHook(() => useWhatsNew());
    await act(() => new Promise((r) => setTimeout(r, 20)));
    expect(result.current.version).toBeUndefined();
  });

  it("bản đang dùng chưa có mục trong CHANGELOG thì không mở hộp thoại rỗng, vẫn lưu version", async () => {
    localStorage.setItem(LAST_SEEN_KEY, "1.0.20");
    vi.mocked(getVersion).mockResolvedValue("9.9.9");
    const { result } = renderHook(() => useWhatsNew());
    await waitFor(() => expect(localStorage.getItem(LAST_SEEN_KEY)).toBe("9.9.9"));
    expect(result.current.version).toBeUndefined();
  });

  it("localStorage hỏng (ném lỗi) thì không vỡ, không mở", async () => {
    vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
      throw new Error("SecurityError");
    });
    const setItem = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
      throw new Error("QuotaExceeded");
    });
    const { result } = renderHook(() => useWhatsNew());
    await waitFor(() => expect(setItem).toHaveBeenCalled());
    expect(result.current.version).toBeUndefined();
  });
});
