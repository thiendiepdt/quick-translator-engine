import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { GlossaryPendingPanel } from "@/components/glossary-pending-panel";
import { glossaryPendingList, glossaryPendingResolve, glossaryScan } from "@/lib/api";

vi.mock("@/lib/api", () => ({
  glossaryPendingList: vi.fn(),
  glossaryPendingResolve: vi.fn(),
  glossaryScan: vi.fn(),
  glossaryBackfill: vi.fn(),
}));

const twin = {
  source: "萧姹",
  target: "Tiêu Xá",
  category: "names",
  chapter: "0276",
  similarSource: "萧诧",
  similarTarget: "Tiêu Sá",
  similarCategory: "names",
  kind: "homophone" as const,
  sourceCount: 7,
  similarCount: 955,
};

describe("GlossaryPendingPanel", () => {
  beforeEach(() => {
    vi.mocked(glossaryPendingList).mockReset();
    vi.mocked(glossaryPendingResolve).mockReset();
    vi.mocked(glossaryScan).mockResolvedValue({ total: 0, chapters: 0, skipped: 0, hits: [] });
  });

  it("không có câu hỏi thì không hiện gì", async () => {
    vi.mocked(glossaryPendingList).mockResolvedValue([]);
    const { container } = render(<GlossaryPendingPanel root={"D:\\t"}running={false} dirty={false} onStoryChanged={vi.fn()} />);
    await waitFor(() => expect(glossaryPendingList).toHaveBeenCalled());
    expect(container).toBeEmptyDOMElement();
  });

  it("dùng tên cũ: gọi resolve, nạp lại hồ sơ và mở Rà chương cũ điền sẵn tên cần thay", async () => {
    const user = userEvent.setup();
    const onStoryChanged = vi.fn();
    vi.mocked(glossaryPendingList).mockResolvedValueOnce([twin]).mockResolvedValue([]);
    vi.mocked(glossaryPendingResolve).mockResolvedValue({ key: "萧姹", from: "Tiêu Xá", to: "Tiêu Sá" });
    render(<GlossaryPendingPanel root={"D:\\t"}running={false} dirty={false} onStoryChanged={onStoryChanged} />);
    expect(await screen.findByText(/raw 955 lần/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Cùng một — dùng “Tiêu Sá”" }));
    expect(glossaryPendingResolve).toHaveBeenCalledWith("D:\\t", "萧姹", "keepOld");
    expect(onStoryChanged).toHaveBeenCalled();
    expect(await screen.findByRole("dialog")).toBeInTheDocument();
    expect(screen.getByDisplayValue("Tiêu Xá")).toBeInTheDocument();
  });

  it("form chưa lưu thì khoá nút; tên có · chỉ có nút Đã xem", async () => {
    vi.mocked(glossaryPendingList).mockResolvedValue([{ ...twin, kind: "dotSegment" }]);
    render(<GlossaryPendingPanel root={"D:\\t"}running={false} dirty onStoryChanged={vi.fn()} />);
    const seen = await screen.findByRole("button", { name: /Đã xem/ });
    expect(seen).toBeDisabled();
    expect(screen.queryByRole("button", { name: /Cùng một/ })).not.toBeInTheDocument();
    expect(screen.getByText(/Lưu thay đổi trên form trước/)).toBeInTheDocument();
  });
});
