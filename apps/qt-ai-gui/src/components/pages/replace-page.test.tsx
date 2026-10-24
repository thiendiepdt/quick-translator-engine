import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ReplacePage } from "@/components/pages/replace-page";
import { replaceApply, replaceFind, replaceUndo, replaceUndoInfo, storySnapshot } from "@/lib/api";
import { storySnapshotSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  replaceFind: vi.fn(),
  replaceApply: vi.fn(),
  replaceUndoInfo: vi.fn(),
  replaceUndo: vi.fn(),
  storySnapshot: vi.fn(),
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
  profile: [],
};

const inProfile = {
  ...found,
  profile: [
    { field: "glossary.common", key: "人族", index: 0, occurrence: 0, before: "", matched: "người tộc", after: "", chapter: null },
    { field: "toneRules", key: "", index: 1, occurrence: 0, before: "Gọi ", matched: "người tộc", after: " là…", chapter: null },
    { field: "addressing", key: "林枫→人族", index: 0, occurrence: 0, before: "ta–", matched: "người tộc", after: "", chapter: "0002" },
    { field: "titles", key: "人族大军", index: 0, occurrence: 0, before: "Đại quân ", matched: "người tộc", after: "", chapter: null },
    { field: "titles", key: "人族之耻", index: 0, occurrence: 0, before: "Nỗi nhục ", matched: "người tộc", after: "", chapter: null },
  ],
};

describe("ReplacePage", () => {
  beforeEach(() => {
    useStoryStore.setState({ root: snapshot.root, snapshot, sessions: {}, page: "replace", selectedId: undefined });
    vi.mocked(replaceFind).mockReset().mockResolvedValue(found);
    vi.mocked(replaceApply).mockReset().mockResolvedValue({ chapters: 1, occurrences: 2, profile: 0 });
    vi.mocked(replaceUndoInfo).mockReset().mockResolvedValue(null);
    vi.mocked(replaceUndo).mockReset().mockResolvedValue({ restored: 1, conflicts: [], profileRestored: 0, profileConflicts: [] });
    vi.mocked(storySnapshot).mockReset().mockResolvedValue(snapshot);
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
      [],
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
    expect(vi.mocked(replaceApply).mock.lastCall?.slice(2)).toEqual([["0001"], [], []]);
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
    vi.mocked(replaceUndoInfo).mockResolvedValue({ find: "Lục Đình Vân", replace: "Lục Vân Đình", chapters: 2, occurrences: 5, profile: 0, createdAt: 1 });
    render(<ReplacePage />);
    expect(await screen.findByText("Lần thay gần nhất: “Lục Đình Vân” → “Lục Vân Đình” (5 chỗ, 2 chương).")).toBeInTheDocument();
    vi.mocked(replaceUndoInfo).mockResolvedValue(null);
    await user.click(screen.getByRole("button", { name: "Hoàn tác" }));
    expect(replaceUndo).toHaveBeenCalledWith(snapshot.root);
    expect(await screen.findByText("Đã hoàn tác 1 chương.")).toBeInTheDocument();
    await waitFor(() => expect(screen.queryByRole("button", { name: "Hoàn tác" })).not.toBeInTheDocument());
  });

  it("chỗ khớp trong hồ sơ chia tab Hồ sơ / Nhân vật / Tiêu đề chương, bỏ chọn rồi thay chung một lần", async () => {
    const user = userEvent.setup();
    vi.mocked(replaceFind).mockResolvedValue(inProfile);
    vi.mocked(replaceApply).mockResolvedValue({ chapters: 2, occurrences: 3, profile: 4 });
    render(<ReplacePage />);
    await user.type(screen.getByLabelText("Tìm"), "người tộc");
    await user.type(screen.getByLabelText("Thay bằng"), "nhân tộc");
    expect(await screen.findByText("3 chỗ trong 2 chương, 5 chỗ trong hồ sơ — đang chọn 8.")).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Chương 3" })).toHaveAttribute("aria-selected", "true");

    await user.click(screen.getByRole("tab", { name: "Hồ sơ 2" }));
    expect(screen.getByRole("region", { name: "Glossary · Từ thông dụng" })).toBeInTheDocument();
    await user.click(screen.getByLabelText("Chọn Tone rules dòng 2"));
    await user.click(screen.getByRole("tab", { name: "Nhân vật 1" }));
    expect(screen.getByText("林枫→人族 · từ ch.2")).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Tiêu đề chương 2" }));
    await user.click(screen.getByLabelText("Chọn cả nhóm Sổ tiêu đề chương"));
    expect(screen.getByText("3 chỗ trong 2 chương, 5 chỗ trong hồ sơ — đang chọn 5.")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Thay 5 chỗ" }));
    expect(vi.mocked(replaceApply).mock.lastCall?.[4]).toEqual([
      { field: "toneRules", key: "", index: 1, occurrence: 0 },
      { field: "titles", key: "人族大军", index: 0, occurrence: 0 },
      { field: "titles", key: "人族之耻", index: 0, occurrence: 0 },
    ]);
    expect(await screen.findByText("Đã thay 3 chỗ trong 2 chương và 4 chỗ trong hồ sơ.")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Hồ sơ truyện → Glossary" })).not.toBeInTheDocument();
    expect(storySnapshot).toHaveBeenCalledWith(snapshot.root);
  });

  it("chỉ khớp trong hồ sơ thì mở sẵn tab có chỗ khớp", async () => {
    const user = userEvent.setup();
    vi.mocked(replaceFind).mockResolvedValue({ ...inProfile, total: 0, chapters: [], hits: [] });
    render(<ReplacePage />);
    await user.type(screen.getByLabelText("Tìm"), "người tộc");
    expect(await screen.findByText("5 chỗ trong hồ sơ — đang chọn 5.")).toBeInTheDocument();
    expect(screen.getByRole("tab", { name: "Chương 0" })).toBeDisabled();
    expect(screen.getByRole("tab", { name: "Hồ sơ 2" })).toHaveAttribute("aria-selected", "true");
  });

  it("hoàn tác báo cả file hồ sơ trả về và file đã sửa tiếp", async () => {
    const user = userEvent.setup();
    vi.mocked(replaceUndoInfo).mockResolvedValue({ find: "a", replace: "b", chapters: 1, occurrences: 2, profile: 3, createdAt: 1 });
    vi.mocked(replaceUndo).mockResolvedValue({ restored: 1, conflicts: [], profileRestored: 1, profileConflicts: ["cast.json"] });
    render(<ReplacePage />);
    expect(await screen.findByText("Lần thay gần nhất: “a” → “b” (2 chỗ, 1 chương, 3 chỗ trong hồ sơ).")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Hoàn tác" }));
    expect(await screen.findByText("Đã hoàn tác 1 chương và 1 file hồ sơ. bảng nhân vật đã sửa tiếp sau đó nên giữ nguyên.")).toBeInTheDocument();
    expect(storySnapshot).toHaveBeenCalledWith(snapshot.root);
  });
});
