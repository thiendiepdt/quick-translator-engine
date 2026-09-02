import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import { ChapterTable, filterRows } from "@/components/chapter-table";
import type { ChapterRow } from "@/lib/types";

const rows: ChapterRow[] = [
  { id: "0001", status: "done", reviewRound: 0, reason: null, warnings: [] },
  { id: "0002", status: "done", reviewRound: 3, reason: null, warnings: ["[[1]] CJK"] },
  { id: "0003", status: "error", reviewRound: 3, reason: "Quá 3 vòng", warnings: [] },
  { id: "0004", status: "queued", reviewRound: 0, reason: null, warnings: [] },
];

describe("ChapterTable", () => {
  it("filterRows lọc theo trạng thái, 'all' giữ hết", () => {
    expect(filterRows(rows, "all")).toHaveLength(4);
    expect(filterRows(rows, "error").map((r) => r.id)).toEqual(["0003"]);
  });

  it("hiện badge trạng thái tiếng Việt, số cảnh báo, và gọi onSelect khi click", async () => {
    const onSelect = vi.fn();
    render(
      <ChapterTable rows={rows} filter="all" selectedId={undefined} onSelect={onSelect} onFilter={() => undefined} />,
    );
    expect(screen.getAllByText("Xong")).toHaveLength(2);
    expect(screen.getByText("Lỗi")).toBeInTheDocument();
    expect(screen.getByText("1 cảnh báo")).toBeInTheDocument();
    await userEvent.click(screen.getByText("0003"));
    expect(onSelect).toHaveBeenCalledWith("0003");
  });
});
