import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ReplacePage } from "@/components/pages/replace-page";
import { replaceApply, replaceFind, replaceUndo, replaceUndoInfo } from "@/lib/api";
import { storySnapshotSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  replaceFind: vi.fn(),
  replaceApply: vi.fn(),
  replaceUndoInfo: vi.fn(),
  replaceUndo: vi.fn(),
}));

const row = (id: string) => ({ id, status: "done", reviewRound: 0, reason: null, warnings: [] });
const snapshot = storySnapshotSchema.parse({
  root: "D:\\t",
  chapters: [row("0001"), row("0002"), row("0003")],
  counts: { total: 3, queued: 0, translating: 0, done: 3, error: 0, skipped: 0, withWarnings: 0 },
  settings: { minLengthRatio: 0.75, maxReviewRounds: 3, chaptersPerSession: 10 },
  story: {
    name: "Truyện A",
    sourceUrl: "",
    protagonist: "",
    summary: "",
    genre: { setting: "ancient", names: "han" },
    glossary: { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {}, addressing: {} },
    style: { voice: "", toneRules: [], signaturePhrases: {}, avoid: [] },
    customPrompt: "",
    checkRules: [],
    autoGlossaryLog: [],
    autoGlossary: "inherit",
  },
  sessionRunning: false,
});

const found = {
  total: 3,
  skipped: 0,
  chapters: [
    { chapter: "0001", count: 2 },
    { chapter: "0003", count: 1 },
  ],
  hits: [
    { chapter: "0001", paragraph: 4, occurrence: 0, before: "…giữa hàng nghìn ", matched: "người tộc", after: " và viên yêu…", raw: "数千人族与猿妖" },
    { chapter: "0001", paragraph: 9, occurrence: 0, before: "Bốn ", matched: "người tộc", after: " Hải Cự Nhân", raw: "四名海巨人族" },
    { chapter: "0003", paragraph: 0, occurrence: 0, before: "", matched: "người tộc", after: " ta.", raw: null },
  ],
};

describe("ReplacePage", () => {
  beforeEach(() => {
    useStoryStore.setState({ root: snapshot.root, snapshot, sessions: {}, page: "replace", selectedId: undefined });
    vi.mocked(replaceFind).mockReset().mockResolvedValue(found);
    vi.mocked(replaceApply).mockReset().mockResolvedValue({ chapters: 1, occurrences: 2 });
    vi.mocked(replaceUndoInfo).mockReset().mockResolvedValue(null);
    vi.mocked(replaceUndo).mockReset().mockResolvedValue({ restored: 1, conflicts: [] });
  });

  it("gõ để tìm, xem ngữ cảnh + raw, bỏ chọn một chỗ rồi thay phần còn lại", async () => {
    const user = userEvent.setup();
    render(<ReplacePage />);
    await user.type(screen.getByLabelText("Tìm"), "người tộc");
    await user.type(screen.getByLabelText("Thay bằng"), "nhân tộc");
    expect(await screen.findByText("3 chỗ trong 2 chương — đang chọn 3.")).toBeInTheDocument();
    expect(vi.mocked(replaceFind).mock.lastCall?.[1]).toMatchObject({ find: "người tộc", wholeWord: true, caseSensitive: false, replace: "" });
    expect(screen.getByText("raw: 四名海巨人族")).toBeInTheDocument();
    expect(screen.getAllByText("nhân tộc").length).toBeGreaterThan(0);

    await user.click(screen.getByLabelText("Chọn chỗ Chương #1 đoạn 10"));
    expect(screen.getByText("3 chỗ trong 2 chương — đang chọn 2.")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thay 2 chỗ" }));
    expect(replaceApply).toHaveBeenCalledWith(
      snapshot.root,
      expect.objectContaining({ find: "người tộc", replace: "nhân tộc" }),
      [],
      [{ chapter: "0001", paragraph: 9, occurrence: 0 }],
    );
    expect(await screen.findByText("Đã thay 2 chỗ trong 1 chương.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Hồ sơ truyện → Glossary" })).toBeInTheDocument();
  });

  it("bỏ chọn cả chương thì gửi chương đó, không gửi từng chỗ của nó", async () => {
    const user = userEvent.setup();
    render(<ReplacePage />);
    await user.type(screen.getByLabelText("Tìm"), "người tộc");
    await user.type(screen.getByLabelText("Thay bằng"), "nhân tộc");
    await screen.findByText("3 chỗ trong 2 chương — đang chọn 3.");
    await user.click(screen.getByLabelText("Chọn chỗ Chương #1 đoạn 5"));
    await user.click(screen.getByLabelText("Chọn Chương #1"));
    expect(screen.getByText("3 chỗ trong 2 chương — đang chọn 1.")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Thay 1 chỗ" }));
    expect(vi.mocked(replaceApply).mock.lastCall?.slice(2)).toEqual([["0001"], []]);
  });

  it("bấm vào một chỗ thì mở chương đó ở trang Dịch", async () => {
    const user = userEvent.setup();
    render(<ReplacePage />);
    await user.type(screen.getByLabelText("Tìm"), "người tộc");
    const list = await screen.findByRole("list", { name: "Chỗ khớp theo chương" });
    await user.click(within(list).getByText("ta."));
    expect(useStoryStore.getState().selectedId).toBe("0003");
    expect(useStoryStore.getState().page).toBe("translate");
  });

  it("hoàn tác lần thay gần nhất", async () => {
    const user = userEvent.setup();
    vi.mocked(replaceUndoInfo).mockResolvedValue({ find: "Lục Đình Vân", replace: "Lục Vân Đình", chapters: 2, occurrences: 5, createdAt: 1 });
    render(<ReplacePage />);
    expect(await screen.findByText("Lần thay gần nhất: “Lục Đình Vân” → “Lục Vân Đình” (5 chỗ, 2 chương).")).toBeInTheDocument();
    vi.mocked(replaceUndoInfo).mockResolvedValue(null);
    await user.click(screen.getByRole("button", { name: "Hoàn tác" }));
    expect(replaceUndo).toHaveBeenCalledWith(snapshot.root);
    expect(await screen.findByText("Đã hoàn tác 1 chương.")).toBeInTheDocument();
    await waitFor(() => expect(screen.queryByRole("button", { name: "Hoàn tác" })).not.toBeInTheDocument());
  });
});
