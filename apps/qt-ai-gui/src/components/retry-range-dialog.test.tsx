import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { RetryRangeDialog } from "@/components/retry-range-dialog";
import { chaptersRetry, storySnapshot } from "@/lib/api";
import type { ChapterRow } from "@/lib/types";

vi.mock("@/lib/api", () => ({ chaptersRetry: vi.fn(), storySnapshot: vi.fn() }));

const row = (id: string, status: ChapterRow["status"]): ChapterRow => ({ id, status, reviewRound: 0, reason: null, warnings: [] });
const chapters = [row("0001", "done"), row("0002", "error"), row("0003", "queued"), row("0004", "done")];

describe("RetryRangeDialog", () => {
  beforeEach(() => {
    vi.mocked(chaptersRetry).mockReset().mockResolvedValue({ retried: ["0001", "0002"], backedUp: ["0001"], alreadyQueued: ["0003"] });
    vi.mocked(storySnapshot).mockReset().mockResolvedValue({ root: "D:\\t", chapters, counts: {}, settings: {}, story: { name: "" }, sessionRunning: false } as never);
  });

  it("trống = toàn bộ; nhập khoảng thì xem trước đổi; xác nhận gọi chapters_retry đúng from/to rồi đóng", async () => {
    const user = userEvent.setup();
    const onOpenChange = vi.fn();
    render(<RetryRangeDialog root={"D:\\t"} chapters={chapters} open onOpenChange={onOpenChange} />);
    expect(screen.getByText(/Toàn bộ truyện/)).toHaveTextContent("3 chương sẽ về hàng đợi");
    expect(screen.getByRole("button", { name: /Dịch lại 3 chương/ })).toBeEnabled();

    await user.type(screen.getByLabelText("Từ chương"), "1"); // số thứ tự
    await user.type(screen.getByLabelText("Đến chương"), "0003"); // hoặc nguyên mã
    expect(screen.getByText("→ 0001")).toBeInTheDocument();
    expect(screen.getByText(/Trong khoảng/)).toHaveTextContent("2 chương sẽ về hàng đợi");
    expect(screen.getByText(/Trong khoảng/)).toHaveTextContent("1 chương đã dịch xong");
    await user.click(screen.getByRole("button", { name: /Dịch lại 2 chương/ }));
    await waitFor(() => expect(chaptersRetry).toHaveBeenCalledWith("D:\\t", { from: "0001", to: "0003" }));
    await waitFor(() => expect(onOpenChange).toHaveBeenCalledWith(false));
  });

  it("khoảng ngược thì báo không hợp lệ và khoá nút", async () => {
    const user = userEvent.setup();
    render(<RetryRangeDialog root={"D:\\t"} chapters={chapters} open onOpenChange={vi.fn()} />);
    await user.type(screen.getByLabelText("Từ chương"), "4");
    await user.type(screen.getByLabelText("Đến chương"), "1");
    expect(screen.getByText(/không hợp lệ/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Dịch lại 0 chương/ })).toBeDisabled();
    await user.clear(screen.getByLabelText("Đến chương"));
    await user.type(screen.getByLabelText("Đến chương"), "9");
    expect(screen.getByText("không có chương này")).toBeInTheDocument();
  });
});
