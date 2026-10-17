import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { toast } from "sonner";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { VnLibraryScanButton } from "@/components/vn-library-scan-button";
import { onVnLibraryProgress, vnScanLibrary, vnScanLibraryCancel } from "@/lib/api";
import type { RecentSummary } from "@/lib/types";

vi.mock("sonner", () => ({ toast: { message: vi.fn(), error: vi.fn() } }));
vi.mock("@/lib/api", () => ({ vnScanLibrary: vi.fn(), vnScanLibraryCancel: vi.fn(), onVnLibraryProgress: vi.fn() }));

const story = (root: string, vnFlags: number | null): RecentSummary => ({ root, name: root, done: 1, total: 2, pending: 1, vnFlags });

describe("VnLibraryScanButton", () => {
  let progress: (p: { done: number; total: number; root: string }) => void = () => {};
  let finish: (value: Awaited<ReturnType<typeof vnScanLibrary>>) => void = () => {};

  beforeEach(() => {
    vi.mocked(toast.message).mockReset();
    vi.mocked(toast.error).mockReset();
    vi.mocked(vnScanLibraryCancel).mockReset().mockResolvedValue(undefined);
    vi.mocked(onVnLibraryProgress).mockReset().mockImplementation((handler) => {
      progress = handler;
      return Promise.resolve(() => {});
    });
    vi.mocked(vnScanLibrary)
      .mockReset()
      .mockImplementation(() => new Promise((resolve) => (finish = resolve)));
  });

  it("quét nền: nút hiện tiến độ, xong thì trả danh sách mới và báo số truyện còn ⚠ VN", async () => {
    const onDone = vi.fn();
    const user = userEvent.setup();
    render(<VnLibraryScanButton onDone={onDone} />);
    await user.click(screen.getByRole("button", { name: "Quét nhạy cảm VN cả thư viện" }));
    expect(screen.getByRole("button", { name: "Dừng quét nhạy cảm VN" })).toHaveTextContent("Đang quét…");
    await waitFor(() => expect(onVnLibraryProgress).toHaveBeenCalled());
    act(() => progress({ done: 12, total: 62, root: "D:\\lib\\a" }));
    expect(screen.getByRole("button", { name: "Dừng quét nhạy cảm VN" })).toHaveTextContent("Đang quét 12/62…");

    const stories = [story("a", 4), story("b", 0), story("c", 19), story("d", null)];
    await act(() => Promise.resolve(finish({ scanned: 62, total: 62, failed: [], cancelled: false, stories })));
    expect(onDone).toHaveBeenCalledWith(stories);
    expect(toast.message).toHaveBeenCalledWith("Quét nhạy cảm VN xong 62 truyện: 2 truyện còn 23 chỗ chưa xác minh (⚠ VN).");
    expect(screen.getByRole("button", { name: "Quét nhạy cảm VN cả thư viện" })).toBeEnabled();
  });

  it("bấm khi đang quét thì dừng sau truyện đang quét; báo đã dừng và truyện lỗi", async () => {
    const user = userEvent.setup();
    render(<VnLibraryScanButton onDone={vi.fn()} />);
    await user.click(screen.getByRole("button", { name: "Quét nhạy cảm VN cả thư viện" }));
    await user.click(screen.getByRole("button", { name: "Dừng quét nhạy cảm VN" }));
    expect(vnScanLibraryCancel).toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Dừng quét nhạy cảm VN" })).toHaveTextContent("Đang dừng…");
    await act(() => Promise.resolve(finish({ scanned: 12, total: 62, failed: ["D:\\lib\\x"], cancelled: true, stories: [story("a", 0)] })));
    expect(toast.message).toHaveBeenCalledWith("Đã dừng sau 12/62 truyện: không truyện nào còn chỗ chưa xác minh.");
    expect(toast.error).toHaveBeenCalledWith("Không quét được 1 truyện (giữ kết quả cũ): D:\\lib\\x");
  });
});
