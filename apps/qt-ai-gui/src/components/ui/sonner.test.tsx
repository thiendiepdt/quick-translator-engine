import { act, render, screen } from "@testing-library/react";
import { toast } from "sonner";
import { beforeAll, describe, expect, it, vi } from "vitest";

import { Toaster } from "./sonner";

// jsdom không có matchMedia; sonner dùng để tính sẵn theme/prefers-reduced-motion.
beforeAll(() => {
  window.matchMedia = vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: () => undefined,
    removeEventListener: () => undefined,
    addListener: () => undefined,
    removeListener: () => undefined,
    dispatchEvent: () => false,
  }));
});

describe("Toaster", () => {
  it("toast nào cũng có nút đóng để tắt ngay, không phải chờ hết giờ", async () => {
    render(<Toaster />);
    act(() => {
      toast.error("tien-tu-lai-xu-long: Đã dừng theo yêu cầu.");
    });
    expect(await screen.findByText(/Đã dừng theo yêu cầu/)).toBeInTheDocument();
    expect(await screen.findByRole("button", { name: "Đóng thông báo" })).toBeInTheDocument();
  });

  it("thông báo hiện ở giữa cạnh dưới, không đè lên các nút thao tác phía trên", async () => {
    render(<Toaster />);
    act(() => {
      toast.success("Đã bắt đầu phiên dịch");
    });
    const list = (await screen.findByText("Đã bắt đầu phiên dịch")).closest("ol");
    expect(list).toHaveAttribute("data-y-position", "bottom");
    expect(list).toHaveAttribute("data-x-position", "center");
  });
});
