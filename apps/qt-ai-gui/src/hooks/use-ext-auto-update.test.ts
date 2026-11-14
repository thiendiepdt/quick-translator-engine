import { renderHook, waitFor } from "@testing-library/react";
import { toast } from "sonner";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { resetExtAutoUpdate, useExtAutoUpdate } from "@/hooks/use-ext-auto-update";
import { extAutoUpdate } from "@/lib/api";
import { invalidateRawChecks } from "@/store/raw-check";

vi.mock("@/lib/api", () => ({ extAutoUpdate: vi.fn() }));
vi.mock("@/store/raw-check", () => ({ invalidateRawChecks: vi.fn() }));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), message: vi.fn() } }));

const item = (name: string, to: string) => ({ id: `org.kho.${name}`, name, from: "1.0.0", to });

describe("useExtAutoUpdate", () => {
  beforeEach(() => {
    resetExtAutoUpdate();
    vi.mocked(extAutoUpdate).mockReset();
    vi.mocked(toast.success).mockReset();
    vi.mocked(toast.message).mockReset();
    vi.mocked(invalidateRawChecks).mockReset();
  });

  it("mở app chạy đúng một lần; báo bản đã cập nhật và bản cần đồng ý quyền", async () => {
    vi.mocked(extAutoUpdate).mockResolvedValue({ updated: [item("Fanqie", "1.0.1")], needsConsent: [item("Qimao", "1.1.0")], failed: ["org.kho: mạng lỗi"] });
    const { rerender } = renderHook(({ on }) => useExtAutoUpdate(on), { initialProps: { on: true } });
    rerender({ on: true });
    await waitFor(() => expect(toast.success).toHaveBeenCalledWith("Đã cập nhật extension: Fanqie 1.0.1"));
    const [title, options] = vi.mocked(toast.message).mock.calls[0] ?? [];
    expect(title).toBe("Có bản mới cần bạn đồng ý quyền: Qimao 1.1.0");
    expect(options?.description).toContain("Cài đặt → Extension tải raw");
    expect(invalidateRawChecks).toHaveBeenCalledTimes(1);
    expect(extAutoUpdate).toHaveBeenCalledTimes(1);
  });

  it("tắt ở Cài đặt hoặc chưa đọc xong cấu hình thì không chạy; không có gì mới thì im lặng", async () => {
    renderHook(() => useExtAutoUpdate(false));
    renderHook(() => useExtAutoUpdate(undefined));
    expect(extAutoUpdate).not.toHaveBeenCalled();

    vi.mocked(extAutoUpdate).mockResolvedValue({ updated: [], needsConsent: [], failed: [] });
    renderHook(() => useExtAutoUpdate(true));
    await waitFor(() => expect(extAutoUpdate).toHaveBeenCalledTimes(1));
    expect(toast.success).not.toHaveBeenCalled();
    expect(toast.message).not.toHaveBeenCalled();
  });
});
