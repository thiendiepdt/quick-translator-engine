import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ExportChapterList } from "@/components/export-chapter-list";
import type { ChapterRow } from "@/lib/types";

const row = (id: string, status: ChapterRow["status"]): ChapterRow => ({ id, status, reviewRound: 0, reason: null, warnings: [] });
const rows = [row("0001", "done"), row("0002", "done"), row("0003", "skipped"), row("0004", "done"), row("0005", "queued")];

describe("ExportChapterList", () => {
  it("tô dòng trong khoảng, gắn nhãn hổng cho dòng chưa done, dòng ngoài khoảng không tô", () => {
    render(<ExportChapterList rows={rows} fromIndex={1} toIndex={3} onPickFrom={vi.fn()} onPickTo={vi.fn()} />);
    const items = screen.getAllByRole("listitem");
    expect(items).toHaveLength(5);
    expect(items[0]).toHaveAttribute("data-in-range", "false");
    expect(items[1]).toHaveAttribute("data-in-range", "true");
    expect(items[3]).toHaveAttribute("data-in-range", "true");
    expect(items[4]).toHaveAttribute("data-in-range", "false");
    expect(within(items[2]).getByText("hổng")).toBeInTheDocument();
    expect(within(items[1]).queryByText("hổng")).not.toBeInTheDocument();
    expect(within(items[4]).queryByText("hổng")).not.toBeInTheDocument(); // ngoài khoảng thì không phải hổng
  });

  it("một đầu trống thì khoảng chạy từ đầu / tới cuối; cả hai -1 thì không tô gì", () => {
    const { rerender } = render(
      <ExportChapterList rows={rows} fromIndex={-1} toIndex={1} onPickFrom={vi.fn()} onPickTo={vi.fn()} />,
    );
    expect(screen.getAllByRole("listitem").map((li) => li.getAttribute("data-in-range"))).toEqual([
      "true",
      "true",
      "false",
      "false",
      "false",
    ]);
    rerender(<ExportChapterList rows={rows} fromIndex={-1} toIndex={-1} onPickFrom={vi.fn()} onPickTo={vi.fn()} />);
    expect(screen.getAllByRole("listitem").every((li) => li.getAttribute("data-in-range") === "false")).toBe(true);
    rerender(<ExportChapterList rows={rows} fromIndex={3} toIndex={1} onPickFrom={vi.fn()} onPickTo={vi.fn()} />);
    expect(screen.getAllByRole("listitem").every((li) => li.getAttribute("data-in-range") === "false")).toBe(true);
  });

  it("nút Từ/Đến trả số thứ tự 1-based theo danh sách đầy đủ, kể cả khi đang lọc", async () => {
    const user = userEvent.setup();
    const onPickFrom = vi.fn();
    const onPickTo = vi.fn();
    render(<ExportChapterList rows={rows} fromIndex={-1} toIndex={-1} onPickFrom={onPickFrom} onPickTo={onPickTo} />);
    await user.click(screen.getByRole("button", { name: "Từ chương #2" }));
    expect(onPickFrom).toHaveBeenCalledWith(2);
    await user.click(screen.getByRole("button", { name: /Xong 3/ }));
    expect(screen.getAllByRole("listitem")).toHaveLength(3);
    await user.click(screen.getByRole("button", { name: "Đến chương #4" }));
    expect(onPickTo).toHaveBeenCalledWith(4);
    await user.type(screen.getByRole("searchbox"), "#1");
    expect(screen.getAllByRole("listitem")).toHaveLength(1);
    expect(screen.getByText("0001")).toBeInTheDocument();
  });
});
