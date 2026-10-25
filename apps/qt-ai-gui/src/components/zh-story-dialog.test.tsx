import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ZhStoryBanner, ZhStoryDialog } from "@/components/zh-story-dialog";
import { storySnapshot, zhStoryConvert, zhStoryStatus, zhStoryUndo } from "@/lib/api";

vi.mock("@/lib/api", () => ({
  storySnapshot: vi.fn(),
  zhStoryStatus: vi.fn(),
  zhStoryConvert: vi.fn(),
  zhStoryUndo: vi.fn(),
}));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const status = (traditional: number, undoChapters: number | null = null) => ({ scanned: 20, traditional, notUtf8: 1, undoChapters });

describe("ZhStoryBanner", () => {
  beforeEach(() => vi.mocked(zhStoryStatus).mockReset());

  it("raw giản thể thì không hiện", async () => {
    vi.mocked(zhStoryStatus).mockResolvedValue(status(0));
    const { container } = render(<ZhStoryBanner root="D:/t" version="1" onOpen={() => undefined} />);
    await vi.waitFor(() => expect(zhStoryStatus).toHaveBeenCalledWith("D:/t", false));
    expect(container).toBeEmptyDOMElement();
  });

  it("có chương phồn thể thì nhắc và mở hộp chuyển", async () => {
    vi.mocked(zhStoryStatus).mockResolvedValue(status(18));
    const onOpen = vi.fn();
    render(<ZhStoryBanner root="D:/t" version="1" onOpen={onOpen} />);
    expect(await screen.findByText(/Raw truyện này là phồn thể \(18\/20 chương đầu\)/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Chuyển sang giản thể…" }));
    expect(onOpen).toHaveBeenCalled();
  });
});

describe("ZhStoryDialog", () => {
  beforeEach(() => {
    vi.mocked(zhStoryStatus).mockReset();
    vi.mocked(zhStoryConvert).mockReset().mockResolvedValue({ chapters: 19, keys: 4, merged: 1 });
    vi.mocked(zhStoryUndo).mockReset().mockResolvedValue({ restored: 18, conflicts: ["0004"], profileRestored: 1, profileConflicts: [] });
    vi.mocked(storySnapshot).mockReset().mockResolvedValue({ root: "D:/t", chapters: [], counts: { skipped: 0, error: 0 } } as never);
  });

  it("đọc cả truyện, chuyển rồi làm mới hồ sơ và thanh nhắc", async () => {
    vi.mocked(zhStoryStatus).mockResolvedValue(status(18));
    const onChanged = vi.fn();
    const onOpenChange = vi.fn();
    render(<ZhStoryDialog root="D:/t" running={false} open onOpenChange={onOpenChange} onChanged={onChanged} />);
    expect(await screen.findByText("18 chương phồn thể, 1 chương mã hoá khác UTF-8 trên 20 chương.")).toBeInTheDocument();
    expect(zhStoryStatus).toHaveBeenCalledWith("D:/t", true);
    expect(screen.queryByRole("button", { name: /Hoàn tác/ })).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Chuyển 19 chương" }));
    expect(zhStoryConvert).toHaveBeenCalledWith("D:/t");
    await vi.waitFor(() => expect(onChanged).toHaveBeenCalled());
    expect(storySnapshot).toHaveBeenCalledWith("D:/t");
    expect(onOpenChange).toHaveBeenCalledWith(false);
  });

  it("đang dịch thì khoá nút; có lần chuyển trước thì hoàn tác được", async () => {
    vi.mocked(zhStoryStatus).mockResolvedValue(status(0, 18));
    const { rerender } = render(<ZhStoryDialog root="D:/t" running open onOpenChange={() => undefined} onChanged={() => undefined} />);
    expect(await screen.findByText("Lần chuyển gần nhất: 18 chương.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Hoàn tác/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Chuyển 1 chương" })).toBeDisabled();
    rerender(<ZhStoryDialog root="D:/t" running={false} open onOpenChange={() => undefined} onChanged={() => undefined} />);
    await userEvent.click(screen.getByRole("button", { name: /Hoàn tác/ }));
    expect(zhStoryUndo).toHaveBeenCalledWith("D:/t");
  });
});
