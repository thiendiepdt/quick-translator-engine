import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { StoryPage } from "@/components/pages/story-page";
import { storySnapshotSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  saveStory: vi.fn(),
  storySnapshot: vi.fn(),
  aiFillStory: vi.fn(),
  storyReset: vi.fn(),
  castLoad: vi.fn(() => Promise.resolve({ version: 1, characters: { 赵静文: { gender: "female", source: "auto" } }, addressing: {} })),
  castSave: vi.fn(),
  castScan: vi.fn(),
  castCleanAddressing: vi.fn(),
  pickSaveFile: vi.fn(),
  writeTextFile: vi.fn(),
  storyDefaults: vi.fn((genre: { setting: string }) =>
    Promise.resolve({
      basePrompt: genre.setting === "modern" ? "Prompt hiện đại." : "Prompt gốc.",
      promptSource: "file",
      promptSuffix: "Đuôi.",
      checkRules: [],
      rulesSource: "builtin",
    }),
  ),
}));

const snapshot = storySnapshotSchema.parse({
  root: "D:\\t",
  chapters: [],
  counts: { total: 0, queued: 0, translating: 0, done: 0, error: 0, skipped: 0, withWarnings: 0 },
  settings: { minLengthRatio: 0.75, maxReviewRounds: 3, chaptersPerSession: 10 },
  story: {
    name: "Truyện A",
    sourceUrl: "",
    protagonist: "",
    summary: "",
    genre: { setting: "ancient", names: "han" },
    glossary: { names: { 赵静文: "Triệu Tĩnh Văn" }, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {}, addressing: {} },
    style: { voice: "", toneRules: [], signaturePhrases: {}, avoid: [] },
    customPrompt: "",
    checkRules: [],
    autoGlossaryLog: [],
    autoGlossary: "inherit",
  },
  sessionRunning: false,
});

describe("StoryPage", () => {
  beforeEach(() => {
    useStoryStore.setState({ root: snapshot.root, snapshot, sessions: {} });
  });

  it("mỗi mục là một tab, chỉ mục đang chọn được render; giá trị mục khác vẫn giữ", async () => {
    const user = userEvent.setup();
    render(<StoryPage />);
    expect(screen.getByRole("tab", { name: "Thông tin", selected: true })).toBeInTheDocument();
    expect(screen.getByLabelText("Tên truyện")).toHaveValue("Truyện A");
    expect(screen.queryByLabelText("Tên nhân vật CN 1")).not.toBeInTheDocument();

    await user.type(screen.getByLabelText("Tên truyện"), " sửa");
    await user.click(screen.getByRole("tab", { name: "Glossary" }));
    expect(screen.queryByLabelText("Tên truyện")).not.toBeInTheDocument();
    expect(screen.getByLabelText("Tên nhân vật CN 1")).toHaveValue("赵静文");
    expect(screen.getByText(/Kho chung theo bối cảnh/)).toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Prompt" }));
    expect(await screen.findByRole("textbox", { name: "Prompt dịch thuật" }, { timeout: 5000 })).toBeInTheDocument();
    expect(screen.getByText("mặc định của app (đã sửa)")).toBeInTheDocument();
    expect(screen.queryByLabelText("Tên nhân vật CN 1")).not.toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Thông tin" }));
    expect(screen.getByLabelText("Tên truyện")).toHaveValue("Truyện A sửa");
    expect(screen.getByText("Có thay đổi chưa lưu")).toBeInTheDocument();
  });

  it("tab Nhân vật hiện bảng giới tính theo glossary names đã lưu", async () => {
    const user = userEvent.setup();
    render(<StoryPage />);
    await user.click(screen.getByRole("tab", { name: "Nhân vật" }));
    expect(await screen.findByLabelText("Giới tính 赵静文")).toHaveValue("female");
    expect(screen.getByRole("button", { name: "Lưu bảng nhân vật" })).toBeDisabled();
  });

  it("Reset truyện… hỏi lại, nêu rõ cái mất cái giữ, xác nhận thì gọi lệnh và nạp snapshot mới", async () => {
    const user = userEvent.setup();
    const { storyReset } = await import("@/lib/api");
    const fresh = storySnapshotSchema.parse({
      ...snapshot,
      story: { ...snapshot.story, glossary: { ...snapshot.story.glossary, names: {} } },
    });
    vi.mocked(storyReset).mockResolvedValue(fresh);
    render(<StoryPage />);
    await user.click(screen.getByRole("button", { name: "Reset truyện…" }));
    const dialog = screen.getByRole("dialog");
    expect(dialog).toHaveTextContent("raw/, out/ và export/ giữ nguyên");
    expect(dialog).toHaveTextContent("glossary, bảng nhân vật");
    expect(dialog).toHaveTextContent("AGENTS.md và workflow của agent ghi lại theo template mới");
    expect(storyReset).not.toHaveBeenCalled();
    await user.click(screen.getByRole("button", { name: "Reset truyện" }));
    expect(storyReset).toHaveBeenCalledWith(snapshot.root);
    expect(useStoryStore.getState().snapshot?.story.glossary.names).toEqual({});
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("reset khi đang mở tab Nhân vật thì nạp lại bảng nhân vật từ đĩa, không giữ bảng cũ trong bộ nhớ", async () => {
    const user = userEvent.setup();
    const { storyReset, castLoad } = await import("@/lib/api");
    vi.mocked(storyReset).mockResolvedValue(snapshot);
    render(<StoryPage />);
    await user.click(screen.getByRole("tab", { name: "Nhân vật" }));
    await screen.findByLabelText("Giới tính 赵静文");
    const before = vi.mocked(castLoad).mock.calls.length;
    await user.click(screen.getByRole("button", { name: "Reset truyện…" }));
    await user.click(screen.getByRole("button", { name: "Reset truyện" }));
    await screen.findByLabelText("Giới tính 赵静文");
    expect(vi.mocked(castLoad).mock.calls.length).toBe(before + 1);
  });

  it("tab Glossary có nút Export Names.txt… mở dialog với glossary đang hiển thị (kể cả sửa chưa lưu)", async () => {
    const user = userEvent.setup();
    render(<StoryPage />);
    await user.click(screen.getByRole("tab", { name: "Glossary" }));
    await user.type(screen.getByLabelText("Tên nhân vật VN 1"), " sửa");
    await user.click(screen.getByRole("button", { name: /Export Names\.txt/ }));
    expect(await screen.findByRole("dialog", { name: "Export glossary ra Names.txt" })).toBeInTheDocument();
    expect(screen.getByLabelText("Xem trước")).toHaveValue("赵静文=Triệu Tĩnh Văn sửa\n");
  });

  it("Thể loại nằm ngay trong tab Thông tin (không có tab riêng); đổi bối cảnh làm form dirty, prompt mặc định nạp lại theo genre", async () => {
    const user = userEvent.setup();
    render(<StoryPage />);
    expect(screen.queryByRole("tab", { name: "Thể loại" })).not.toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Thể loại" })).toBeInTheDocument();
    await user.click(screen.getByRole("combobox", { name: "Bối cảnh" }));
    await user.click(await screen.findByRole("option", { name: /Hiện đại/ }));
    expect(screen.getByText("Có thay đổi chưa lưu")).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Prompt" }));
    expect(await screen.findByRole("textbox", { name: "Prompt dịch thuật" }, { timeout: 5000 })).toBeInTheDocument();
    const { storyDefaults } = await import("@/lib/api");
    expect(storyDefaults).toHaveBeenCalledWith({ setting: "modern", names: "han", tone: "neutral" });
  });

  it("chọn Hỗn hợp gọi defaults với setting mixed", async () => {
    const user = userEvent.setup();
    render(<StoryPage />);
    await user.click(screen.getByRole("combobox", { name: "Bối cảnh" }));
    await user.click(await screen.findByRole("option", { name: /Hỗn hợp/ }));
    const { storyDefaults } = await import("@/lib/api");
    expect(storyDefaults).toHaveBeenCalledWith({ setting: "mixed", names: "han", tone: "neutral" });
  });

  it("Giọng văn mặc định Trung tính; chọn Ngôn tình làm form dirty và hint nói prompt được chèn mục", async () => {
    const user = userEvent.setup();
    render(<StoryPage />);
    expect(screen.getByRole("combobox", { name: "Giọng văn" })).toHaveTextContent("Trung tính");
    await user.click(screen.getByRole("combobox", { name: "Giọng văn" }));
    await user.click(await screen.findByRole("option", { name: /Ngôn tình/ }));
    expect(screen.getByText("Có thay đổi chưa lưu")).toBeInTheDocument();
    expect(screen.getByText(/chèn thêm mục giọng ngôn tình/)).toBeInTheDocument();
    await user.click(screen.getByRole("combobox", { name: "Giọng văn" }));
    await user.click(await screen.findByRole("option", { name: /Hài hước/ }));
    expect(screen.getByText(/giữ punchline/)).toBeInTheDocument();
  });
});
