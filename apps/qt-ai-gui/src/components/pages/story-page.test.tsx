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
  storyDefaults: vi.fn((genre: { setting: string }) =>
    Promise.resolve({
      basePrompt: genre.setting === "modern" ? "Prompt hiện đại." : "Prompt gốc.",
      promptSuffix: "Đuôi.",
      checkRules: [],
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
    glossary: { names: { 赵静文: "Triệu Tĩnh Văn" }, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {} },
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
    useStoryStore.setState({ root: snapshot.root, snapshot, session: { status: "idle" } });
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

    await user.click(screen.getByRole("tab", { name: "Prompt" }));
    expect(await screen.findByRole("textbox", { name: "Prompt dịch thuật" }, { timeout: 5000 })).toBeInTheDocument();
    expect(screen.queryByLabelText("Tên nhân vật CN 1")).not.toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Thông tin" }));
    expect(screen.getByLabelText("Tên truyện")).toHaveValue("Truyện A sửa");
    expect(screen.getByText("Có thay đổi chưa lưu")).toBeInTheDocument();
  });

  it("mục Thể loại đổi bối cảnh làm form dirty và prompt mặc định nạp lại theo genre", async () => {
    const user = userEvent.setup();
    render(<StoryPage />);
    await user.click(screen.getByRole("tab", { name: "Thể loại" }));
    await user.click(screen.getByRole("combobox", { name: "Bối cảnh" }));
    await user.click(await screen.findByRole("option", { name: /Hiện đại/ }));
    expect(screen.getByText("Có thay đổi chưa lưu")).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Prompt" }));
    expect(await screen.findByRole("textbox", { name: "Prompt dịch thuật" }, { timeout: 5000 })).toBeInTheDocument();
    const { storyDefaults } = await import("@/lib/api");
    expect(storyDefaults).toHaveBeenCalledWith({ setting: "modern", names: "han" });
  });

  it("chọn Hỗn hợp gọi defaults với setting mixed", async () => {
    const user = userEvent.setup();
    render(<StoryPage />);
    await user.click(screen.getByRole("tab", { name: "Thể loại" }));
    await user.click(screen.getByRole("combobox", { name: "Bối cảnh" }));
    await user.click(await screen.findByRole("option", { name: /Hỗn hợp/ }));
    const { storyDefaults } = await import("@/lib/api");
    expect(storyDefaults).toHaveBeenCalledWith({ setting: "mixed", names: "han" });
  });
});
