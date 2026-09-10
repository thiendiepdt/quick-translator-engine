import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { SIDEBAR_PAGE, StorySidebar } from "@/components/story-sidebar";
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
const D = "D:\\lib\\delta";

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

function ui() {
  return render(
    <TooltipProvider>
      <StorySidebar />
    </TooltipProvider>,
  );
}

describe("StorySidebar", () => {
  beforeEach(() => {
    vi.mocked(openStory).mockReset();
    useStoryStore.setState({
      screen: "workbench",
      page: "story",
      root: A,
      snapshot: snapshot(A, "Alpha Beta"),
      // Mở theo thứ tự D → C → B → A (A mới nhất); B đang dịch, D đang dịch nhưng đã đóng.
      opened: [A, B, C, D],
      sessions: {
        "d:\\lib\\bravo": { status: "running", sessionNo: 1 },
        "d:\\lib\\delta": { status: "running", sessionNo: 2 },
        "d:\\lib\\charlie": { status: "idle" },
      },
      roots: { "d:\\lib\\bravo": B, "d:\\lib\\delta": D, "d:\\lib\\charlie": C },
      names: { "d:\\lib\\alpha-beta": "Alpha Beta", "d:\\lib\\bravo": "Bravo", "d:\\lib\\charlie": "Charlie", "d:\\lib\\delta": "Delta" },
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

  it("dockEntries: đang dịch trước (theo tên), rồi các truyện đã mở theo lần mở gần nhất", () => {
    const entries = dockEntries(useStoryStore.getState());
    expect(entries.map((e) => e.root)).toEqual([B, D, A, C]);
    expect(entries[0]).toMatchObject({ running: true, percent: 50, currentChapter: "0006", current: false });
    expect(entries[2]).toMatchObject({ current: true, running: false, name: "Alpha Beta" });
  });

  it("bấm ô truyện khác → open_story rồi switchStory giữ trang; bấm truyện đang mở thì không gọi", async () => {
    vi.mocked(openStory).mockResolvedValue(snapshot(B, "Bravo", true));
    const user = userEvent.setup();
    ui();
    const items = screen.getAllByRole("listitem");
    expect(items).toHaveLength(4);
    expect(items[0]).toHaveAccessibleName("Bravo — Đang dịch · 0006 · 50%");
    expect(items[2]).toHaveAttribute("aria-current", "true");
    const [runningItem, , currentItem] = items;
    if (!runningItem || !currentItem) throw new Error("thiếu ô sidebar");

    await user.click(currentItem);
    expect(openStory).not.toHaveBeenCalled();

    await user.click(runningItem);
    await waitFor(() => expect(openStory).toHaveBeenCalledWith(B));
    await waitFor(() => expect(useStoryStore.getState().root).toBe(B));
    expect(useStoryStore.getState().page).toBe("story"); // giữ trang, không về Dịch
    expect(screen.getAllByRole("listitem")[0]).toHaveAttribute("aria-current", "true");
  });

  it("danh sách dài: hiện 12 ô rồi nút +N nạp thêm", async () => {
    const many = Array.from({ length: 30 }, (_, i) => `D:\\lib\\t${String(i).padStart(2, "0")}`);
    useStoryStore.setState({ opened: many, sessions: {}, roots: {}, names: {}, progress: {}, root: many[0] });
    const user = userEvent.setup();
    ui();
    expect(screen.getAllByRole("listitem")).toHaveLength(SIDEBAR_PAGE);
    await user.click(screen.getByRole("button", { name: `Xem thêm ${30 - SIDEBAR_PAGE} truyện` }));
    expect(screen.getAllByRole("listitem")).toHaveLength(SIDEBAR_PAGE * 2);
    await user.click(screen.getByRole("button", { name: /Xem thêm/ }));
    expect(screen.getAllByRole("listitem")).toHaveLength(30);
    expect(screen.queryByRole("button", { name: /Xem thêm/ })).not.toBeInTheDocument();
  });

  it("Ctrl+K mở dialog chuyển truyện", async () => {
    const user = userEvent.setup();
    ui();
    await user.keyboard("{Control>}k{/Control}");
    expect(await screen.findByRole("dialog", { name: "Chuyển truyện" })).toBeInTheDocument();
  });
});
