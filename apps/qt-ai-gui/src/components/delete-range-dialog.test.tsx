import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { DeleteRangeDialog } from "@/components/delete-range-dialog";
import { chaptersDelete, storySnapshot } from "@/lib/api";
import { describeDelete } from "@/lib/chapters";
import type { ChapterRow } from "@/lib/types";

vi.mock("@/lib/api", () => ({ chaptersDelete: vi.fn(), storySnapshot: vi.fn() }));

const row = (id: string, status: ChapterRow["status"]): ChapterRow => ({ id, status, reviewRound: 0, reason: null, warnings: [] });
const chapters = [row("0001", "done"), row("0002", "error"), row("0003", "queued"), row("0004", "done")];

describe("DeleteRangeDialog", () => {
  beforeEach(() => {
    vi.mocked(chaptersDelete).mockReset().mockResolvedValue({ removed: ["0001", "0002"], keptOutputs: ["0001"] });
    vi.mocked(storySnapshot).mockReset().mockResolvedValue({ root: "D:\\t", chapters: chapters.slice(2), counts: {}, settings: {}, story: { name: "" }, sessionRunning: false } as never);
  });

  it("trống cả hai thì khoá; nhập khoảng thì xem trước đếm done; xác nhận gọi chapters_delete đúng ids rồi đóng", async () => {
    const user = userEvent.setup();
    const onOpenChange = vi.fn();
    render(<DeleteRangeDialog root={"D:\\t"} chapters={chapters} open onOpenChange={onOpenChange} />);
    expect(screen.getByText(/Điền ít nhất một ô/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Xoá 0 chương/ })).toBeDisabled();

    await user.type(screen.getByLabelText("Từ chương"), "1");
    await user.type(screen.getByLabelText("Đến chương"), "0002");
    expect(screen.getByText("→ 0001")).toBeInTheDocument();
    expect(screen.getByText(/Sẽ xoá/)).toHaveTextContent("2 chương, trong đó 1 chương đã dịch xong");
    await user.click(screen.getByRole("button", { name: /Xoá 2 chương/ }));
    await waitFor(() => expect(chaptersDelete).toHaveBeenCalledWith("D:\\t", ["0001", "0002"]));
    await waitFor(() => expect(onOpenChange).toHaveBeenCalledWith(false));
  });

  it("một ô trống thì lấy tới cuối; khoảng ngược hoặc số lạ thì báo và khoá", async () => {
    const user = userEvent.setup();
    render(<DeleteRangeDialog root={"D:\\t"} chapters={chapters} open onOpenChange={vi.fn()} />);
    await user.type(screen.getByLabelText("Từ chương"), "3");
    expect(screen.getByText(/Sẽ xoá/)).toHaveTextContent("2 chương");
    await user.type(screen.getByLabelText("Đến chương"), "1");
    expect(screen.getByText(/không hợp lệ/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Xoá 0 chương/ })).toBeDisabled();
    await user.clear(screen.getByLabelText("Đến chương"));
    await user.type(screen.getByLabelText("Đến chương"), "9");
    expect(screen.getByText("không có chương này")).toBeInTheDocument();
    expect(chaptersDelete).not.toHaveBeenCalled();
  });

  it("describeDelete nêu số bản dịch giữ lại khi có", () => {
    expect(describeDelete(3, 0)).toBe("Đã xoá 3 chương");
    expect(describeDelete(3, 2)).toBe("Đã xoá 3 chương (2 bản dịch trong out/ giữ nguyên)");
  });
});
