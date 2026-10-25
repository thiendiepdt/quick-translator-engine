import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ChapterList } from "@/components/chapter-list";
import type { ChapterRow } from "@/lib/types";

const rows: ChapterRow[] = [
  { id: "0001", status: "done", reviewRound: 0, reason: null, warnings: [] },
  { id: "0002", status: "done", reviewRound: 3, reason: null, warnings: ["[[1]] CJK"] },
  { id: "0003", status: "error", reviewRound: 3, reason: "Quá 3 vòng", warnings: [] },
  { id: "0004", status: "queued", reviewRound: 0, reason: null, warnings: [] },
];

const noop = () => undefined;

describe("ChapterList", () => {
  it("hiện chip đếm, lọc theo prop, gọi onSelect/onFilter/onQuery", async () => {
    const onSelect = vi.fn();
    const onFilter = vi.fn();
    const onQuery = vi.fn();
    render(
      <ChapterList
        rows={rows}
        filter="all"
        query=""
        selectedId={undefined}
        onSelect={onSelect}
        onFilter={onFilter}
        onQuery={onQuery}
      />,
    );
    expect(screen.getByRole("button", { name: /Lỗi 1/ })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Cảnh báo 1/ })).toBeInTheDocument();
    expect(screen.getAllByRole("option")).toHaveLength(4);
    expect(screen.getByRole("option", { name: /#3.*0003/ })).toBeInTheDocument(); // số thứ tự trước mã
    await userEvent.click(screen.getByRole("button", { name: /Lỗi 1/ }));
    expect(onFilter).toHaveBeenCalledWith("error");
    await userEvent.click(screen.getByRole("option", { name: /0003/ }));
    expect(onSelect).toHaveBeenCalledWith("0003");
    await userEvent.type(screen.getByRole("searchbox"), "2");
    expect(onQuery).toHaveBeenLastCalledWith("2");
  });

  it("truyện nhiều chương chỉ vẽ dòng đang thấy, cuộn tới đâu vẽ tới đó", () => {
    const many: ChapterRow[] = Array.from({ length: 2500 }, (_, i) => ({
      id: String(i + 1).padStart(4, "0"),
      status: "done",
      reviewRound: 0,
      reason: null,
      warnings: [],
    }));
    render(<ChapterList rows={many} filter="all" query="" selectedId={undefined} onSelect={noop} onFilter={noop} onQuery={noop} />);
    expect(screen.getAllByRole("option").length).toBeLessThan(60);
    expect(screen.getByRole("option", { name: /^#1\s*0001$/ })).toBeInTheDocument();
    const list = screen.getByRole("listbox", { name: "Danh sách chương" });
    list.scrollTop = 2000 * 36;
    fireEvent.scroll(list);
    expect(screen.getByRole("option", { name: /^#2001\s*2001$/ })).toBeInTheDocument();
    expect(screen.queryByRole("option", { name: /^#1\s*0001$/ })).not.toBeInTheDocument();
  });

  it("selectedId đổi từ ngoài thì cuộn tới hàng đó; hàng đang thấy thì không cuộn", () => {
    const many: ChapterRow[] = Array.from({ length: 300 }, (_, i) => ({
      id: String(i + 1).padStart(4, "0"),
      status: "queued",
      reviewRound: 0,
      reason: null,
      warnings: [],
    }));
    const { rerender } = render(
      <ChapterList rows={many} filter="all" query="" selectedId="0005" onSelect={noop} onFilter={noop} onQuery={noop} />,
    );
    const list = screen.getByRole("listbox", { name: "Danh sách chương" });
    expect(list.scrollTop).toBe(0);
    rerender(<ChapterList rows={many} filter="all" query="" selectedId="0200" onSelect={noop} onFilter={noop} onQuery={noop} />);
    // Khung chưa đo được (jsdom) coi như cao 800 px: hàng 200 nằm sát đáy khung.
    expect(list.scrollTop).toBe(200 * 36 - 800);
    fireEvent.scroll(list);
    expect(screen.getByRole("option", { name: /0200/ })).toHaveAttribute("aria-selected", "true");
    rerender(<ChapterList rows={many} filter="all" query="" selectedId="0010" onSelect={noop} onFilter={noop} onQuery={noop} />);
    expect(list.scrollTop).toBe(9 * 36);
  });

  it("lọc error chỉ còn 1 hàng, rỗng thì báo", () => {
    const { rerender } = render(
      <ChapterList rows={rows} filter="error" query="" selectedId="0003" onSelect={noop} onFilter={noop} onQuery={noop} />,
    );
    expect(screen.getAllByRole("option")).toHaveLength(1);
    expect(screen.getByRole("option", { name: /0003/ })).toHaveAttribute("aria-selected", "true");
    rerender(
      <ChapterList rows={rows} filter="skipped" query="" selectedId={undefined} onSelect={noop} onFilter={noop} onQuery={noop} />,
    );
    expect(screen.getByText("Không có chương nào khớp.")).toBeInTheDocument();
  });
});
