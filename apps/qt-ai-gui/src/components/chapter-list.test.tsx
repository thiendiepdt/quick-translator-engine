import { render, screen } from "@testing-library/react";
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
    await userEvent.click(screen.getByRole("button", { name: /Lỗi 1/ }));
    expect(onFilter).toHaveBeenCalledWith("error");
    await userEvent.click(screen.getByRole("option", { name: /0003/ }));
    expect(onSelect).toHaveBeenCalledWith("0003");
    await userEvent.type(screen.getByRole("searchbox"), "2");
    expect(onQuery).toHaveBeenLastCalledWith("2");
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
