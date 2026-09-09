import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { StoryDock } from "@/components/story-dock";
import { TooltipProvider } from "@/components/ui/tooltip";
import { openStory } from "@/lib/api";
import { dockEntries, initials } from "@/lib/dock";
import { storySnapshotSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  libraryList: vi.fn(() => Promise.resolve([])),
  openStory: vi.fn(),
  recentSummaries: vi.fn(() => Promise.resolve([])),
}));

const A = "D:\\lib\\alpha-beta";
const B = "D:\\lib\\bravo";
const C = "D:\\lib\\charlie";

function snapshot(root: string, name: string, sessionRunning = false) {
  return storySnapshotSchema.parse({
    root,
    chapters: [],
    counts: { total: 0, queued: 0, translating: 0, done: 0, error: 0, skipped: 0, withWarnings: 0 },
    settings: { minLengthRatio: 0.75, maxReviewRounds: 3, chaptersPerSession: 10 },
    story: {
      name,
      sourceUrl: "",
      protagonist: "",
      summary: "",
      genre: { setting: "ancient", names: "han" },
      glossary: { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {} },
      style: { voice: "", toneRules: [], signaturePhrases: {}, avoid: [] },
      customPrompt: "",
      checkRules: [],
      autoGlossaryLog: [],
      autoGlossary: "inherit",
    },
    sessionRunning,
  });
}

describe("StoryDock", () => {
  beforeEach(() => {
    vi.mocked(openStory).mockReset();
    useStoryStore.setState({
      screen: "workbench",
      page: "story",
      root: A,
      snapshot: snapshot(A, "Alpha Beta"),
      sessions: { "d:\\lib\\bravo": { status: "running", sessionNo: 1 }, "d:\\lib\\charlie": { status: "idle" } },
      roots: { "d:\\lib\\bravo": B, "d:\\lib\\charlie": C },
      names: { "d:\\lib\\alpha-beta": "Alpha Beta", "d:\\lib\\bravo": "Bravo", "d:\\lib\\charlie": "Charlie" },
      progress: {
        "d:\\lib\\bravo": { done: 5, queued: 5, translating: 0, error: 0, skipped: 0, warnings_count: 0, current: "0006" },
      },
      logs: {},
    });
  });

  it("initials: hai từ đầu, một từ lấy 2 chữ, không tên thì lấy folder", () => {
    expect(initials("Alpha Beta", A)).toBe("AB");
    expect(initials("Bravo", B)).toBe("BR");
    expect(initials("", "D:\\lib\\ta-tuyet\\")).toBe("TT");
  });

  it("dockEntries: đang mở trước rồi đang dịch; truyện idle không vào dock; tiến độ tính từ progress", () => {
    const entries = dockEntries(useStoryStore.getState());
    expect(entries.map((e) => e.root)).toEqual([A, B]);
    expect(entries[0]).toMatchObject({ current: true, running: false, name: "Alpha Beta" });
    expect(entries[1]).toMatchObject({ current: false, running: true, percent: 50, currentChapter: "0006" });
  });

  it("bấm ô truyện đang dịch → open_story rồi switchStory giữ trang; bấm truyện đang mở thì không gọi", async () => {
    vi.mocked(openStory).mockResolvedValue(snapshot(B, "Bravo", true));
    const user = userEvent.setup();
    render(
      <TooltipProvider>
        <StoryDock />
      </TooltipProvider>,
    );
    const items = screen.getAllByRole("listitem");
    expect(items).toHaveLength(2);
    expect(items[0]).toHaveAttribute("aria-current", "true");
    expect(items[1]).toHaveAccessibleName("Bravo — Đang dịch · 0006 · 50%");

    const [currentItem, runningItem] = items;
    if (!currentItem || !runningItem) throw new Error("thiếu ô dock");
    await user.click(currentItem);
    expect(openStory).not.toHaveBeenCalled();

    await user.click(runningItem);
    await waitFor(() => expect(openStory).toHaveBeenCalledWith(B));
    await waitFor(() => expect(useStoryStore.getState().root).toBe(B));
    expect(useStoryStore.getState().page).toBe("story"); // giữ trang, không về Dịch
    expect(screen.getAllByRole("listitem")[0]).toHaveAttribute("aria-current", "true");
  });

  it("Ctrl+K mở dialog chuyển truyện", async () => {
    const user = userEvent.setup();
    render(
      <TooltipProvider>
        <StoryDock />
      </TooltipProvider>,
    );
    await user.keyboard("{Control>}k{/Control}");
    expect(await screen.findByRole("dialog", { name: "Chuyển truyện" })).toBeInTheDocument();
  });
});
