import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { StorySwitcherDialog } from "@/components/story-switcher-dialog";
import { libraryList, recentSummaries } from "@/lib/api";
import { filterCards, mergeCandidates, PAGE_SIZE } from "@/lib/switcher";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  libraryList: vi.fn(() => Promise.resolve([])),
  recentSummaries: vi.fn(() => Promise.resolve([])),
}));

const library = Array.from({ length: 30 }, (_, i) => ({
  root: `D:\\lib\\truyen-${String(i).padStart(2, "0")}`,
  name: `Truyện ${i}`,
  done: i,
  total: 30,
}));

describe("mergeCandidates / filterCards", () => {
  it("đang dịch trước, đang mở kế, khử trùng thư viện với gần đây, folder chưa init không mở được", () => {
    const cards = mergeCandidates({
      running: ["d:/lib/truyen-05/"],
      names: { "d:\\lib\\ngoai": "Ngoài" },
      progress: {
        "d:\\lib\\truyen-05": { done: 9, queued: 1, translating: 0, error: 0, skipped: 0, warnings_count: 0, current: null },
      },
      currentRoot: "D:\\lib\\truyen-02",
      library: [...library.slice(0, 4), ...library.slice(5, 6), { root: "D:\\lib\\rac", name: null, done: null, total: null }],
      recent: [{ root: "D:\\LIB\\truyen-01\\", name: "Truyện 1", done: 1, total: 30 }, { root: "D:\\lib\\ngoai", name: null, done: 2, total: 8 }],
    });
    expect(cards[0]).toMatchObject({ root: "D:\\lib\\truyen-05", running: true, done: 9 }); // tiến độ live
    expect(cards[1]).toMatchObject({ root: "D:\\lib\\truyen-02", current: true });
    expect(cards.filter((c) => c.root.toLowerCase().includes("truyen-01"))).toHaveLength(1);
    expect(cards.find((c) => c.root === "D:\\lib\\rac")?.openable).toBe(false);
    expect(cards.find((c) => c.root === "D:\\lib\\ngoai")?.name).toBe("Ngoài"); // tên từ store khi đĩa không có
    expect(filterCards(cards, "ngo")).toHaveLength(1);
    const sourced = mergeCandidates({
      running: [],
      names: {},
      progress: {},
      library: [{ ...library[0], source: { label: "Fanqie", extension: true, url: "https://fanqienovel.com/page/1" } }, library[1]],
      recent: [],
    });
    expect(sourced[0].source?.label).toBe("Fanqie");
    expect(filterCards(sourced, "fanqie").map((c) => c.root)).toEqual([library[0].root]); // tìm theo nguồn
    expect(filterCards(cards, "truyen-0")).toHaveLength(5); // 00–03 + 05 (01 ở gần đây đã khử trùng)
  });
});

describe("mergeCandidates · chương lỗi/bỏ qua", () => {
  it("thẻ mang số chương lỗi/bỏ qua từ đĩa; phiên đang chạy dùng số live", () => {
    const cards = mergeCandidates({
      running: ["D:\\lib\\b"],
      names: {},
      progress: { "d:\\lib\\b": { done: 1, queued: 0, translating: 0, error: 3, skipped: 0, warnings_count: 0, current: null } },
      library: [
        { root: "D:\\lib\\a", name: "A", done: 1, total: 4, error: 1, skipped: 2 },
        { root: "D:\\lib\\b", name: "B", done: 1, total: 4, error: 0, skipped: 0 },
        { root: "D:\\lib\\c", name: "C", done: 1, total: 4 },
      ],
      recent: [],
    });
    expect(cards.find((c) => c.name === "A")).toMatchObject({ error: 1, skipped: 2 });
    expect(cards.find((c) => c.name === "B")).toMatchObject({ error: 3, skipped: 0 });
    expect(cards.find((c) => c.name === "C")).toMatchObject({ error: 0, skipped: 0 });
  });
});

describe("StorySwitcherDialog", () => {
  beforeEach(() => {
    vi.mocked(libraryList).mockResolvedValue(library);
    vi.mocked(recentSummaries).mockResolvedValue([]);
    useStoryStore.setState({ root: "D:\\lib\\truyen-02", sessions: {}, roots: {}, names: {}, progress: {} });
  });

  it("hiện 12 thẻ, Xem thêm nạp thêm, tìm lọc, bấm thẻ trả root", async () => {
    const user = userEvent.setup();
    const onPick = vi.fn();
    render(<StorySwitcherDialog open onOpenChange={() => undefined} onPick={onPick} />);
    await waitFor(() => expect(screen.getAllByRole("listitem")).toHaveLength(PAGE_SIZE));
    expect(screen.getByRole("button", { name: `Xem thêm (${30 - PAGE_SIZE})` })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Xem thêm/ }));
    expect(screen.getAllByRole("listitem")).toHaveLength(2 * PAGE_SIZE);
    await user.click(screen.getByRole("button", { name: `Xem thêm (${30 - 2 * PAGE_SIZE})` }));
    expect(screen.getAllByRole("listitem")).toHaveLength(30);
    expect(screen.queryByRole("button", { name: /Xem thêm/ })).not.toBeInTheDocument();

    await user.type(screen.getByRole("textbox", { name: "Tìm truyện" }), "Truyện 2");
    const remaining = screen.getAllByRole("listitem");
    expect(remaining.length).toBe(11); // 2, 20–29
    await user.click(screen.getByRole("button", { name: /Truyện 21/ }));
    expect(onPick).toHaveBeenCalledWith("D:\\lib\\truyen-21");
  });

  it("thẻ truyện có chương lỗi/bỏ qua hiện chip như màn chọn truyện", async () => {
    vi.mocked(libraryList).mockResolvedValue([
      { root: "D:\\lib\\x", name: "X", done: 1, total: 4, error: 2, skipped: 1 },
      { root: "D:\\lib\\y", name: "Y", done: 4, total: 4, error: 0, skipped: 0 },
    ]);
    render(<StorySwitcherDialog open onOpenChange={() => undefined} onPick={() => undefined} />);
    expect(await screen.findByText("Lỗi 2")).toBeInTheDocument();
    expect(screen.getByText("Bỏ qua 1")).toBeInTheDocument();
    expect(screen.getAllByText(/^(Lỗi|Bỏ qua) \d+$/)).toHaveLength(2);
  });

  it("thẻ truyện hiện nhãn nguồn như màn chọn truyện", async () => {
    vi.mocked(libraryList).mockResolvedValue([
      { root: "D:\\lib\\x", name: "X", done: 1, total: 4, source: { label: "Fanqie", extension: true, url: "https://fanqienovel.com/page/1" } },
      { root: "D:\\lib\\y", name: "Y", done: 4, total: 4, source: { label: "book.qq.com", extension: false, url: "https://book.qq.com/book-detail/2" } },
      { root: "D:\\lib\\z", name: "Z", done: 4, total: 4 },
    ]);
    render(<StorySwitcherDialog open onOpenChange={() => undefined} onPick={() => undefined} />);
    expect(await screen.findByText("Fanqie")).toHaveAttribute("title", "Tải raw qua extension Fanqie — https://fanqienovel.com/page/1");
    expect(screen.getByText("book.qq.com")).toHaveAttribute("title", expect.stringContaining("Chưa có extension"));
  });
});
