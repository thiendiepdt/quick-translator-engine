import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { TranslatePage } from "@/components/pages/translate-page";
import { rescanStory, revealFolder } from "@/lib/api";
import { appConfigSchema, storySnapshotSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

type DropHandler = (event: { payload: { type: string; paths?: string[] } }) => void;
let dropHandler: DropHandler | undefined;

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({
    onDragDropEvent: (cb: DropHandler) => {
      dropHandler = cb;
      return Promise.resolve(() => undefined);
    },
  }),
}));

vi.mock("@/lib/api", () => ({
  importChapters: vi.fn(),
  readChapter: vi.fn(),
  rescanStory: vi.fn(),
  revealFolder: vi.fn(() => Promise.resolve()),
  sessionStart: vi.fn(),
  sessionStop: vi.fn(),
  storySnapshot: vi.fn(),
}));

const story = {
  name: "Truyện T",
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
};
const settings = { minLengthRatio: 0.75, maxReviewRounds: 3, chaptersPerSession: 10 };
const emptySnapshot = storySnapshotSchema.parse({
  root: "D:\\lib\\t",
  chapters: [],
  counts: { total: 0, queued: 0, translating: 0, done: 0, error: 0, skipped: 0, withWarnings: 0 },
  settings,
  story,
  sessionRunning: false,
});
const oneChapter = storySnapshotSchema.parse({
  ...emptySnapshot,
  chapters: [{ id: "0001", status: "queued", reviewRound: 0, reason: null, warnings: [] }],
  counts: { ...emptySnapshot.counts, total: 1, queued: 1 },
});
const config = appConfigSchema.parse({ agyPath: null, model: null, maxSessions: 50, recent: [] });

describe("TranslatePage · chưa có chương, Quét lại, kéo thả", () => {
  beforeEach(() => {
    dropHandler = undefined;
    vi.mocked(rescanStory).mockReset();
    useStoryStore.setState({
      screen: "workbench",
      root: emptySnapshot.root,
      snapshot: emptySnapshot,
      config,
      sessions: {},
      progress: {},
      logs: {},
      roots: {},
      selectedId: undefined,
    });
  });

  it("0 chương: hiện hướng dẫn thả file + nút mở raw/; Quét lại thấy chương mới thì cập nhật snapshot", async () => {
    const user = userEvent.setup();
    vi.mocked(rescanStory).mockResolvedValue(oneChapter);
    render(<TranslatePage />);
    expect(screen.getByText("Chưa có chương")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Mở folder raw/" }));
    expect(revealFolder).toHaveBeenCalledWith("D:\\lib\\t\\raw");

    await user.click(screen.getByRole("button", { name: "Quét lại" }));
    await waitFor(() => expect(rescanStory).toHaveBeenCalledWith("D:\\lib\\t"));
    await waitFor(() => expect(useStoryStore.getState().snapshot?.counts.total).toBe(1));
    expect(screen.queryByText("Chưa có chương")).not.toBeInTheDocument();
    expect(screen.getByRole("option", { name: /0001/ })).toBeInTheDocument();
  });

  it("kéo file vào cửa sổ hiện lớp phủ; thả xong snapshot mới thay danh sách", async () => {
    const { importChapters } = await import("@/lib/api");
    vi.mocked(importChapters).mockResolvedValue({ added: ["0001.txt"], skippedExisting: [], ignored: [], snapshot: oneChapter });
    render(<TranslatePage />);
    await waitFor(() => expect(dropHandler).toBeDefined());
    const overlay = screen.getByText("Thả để thêm vào raw/").closest("[aria-hidden]");
    expect(overlay).toHaveAttribute("aria-hidden", "true");

    dropHandler?.({ payload: { type: "enter", paths: ["D:\\x\\0001.txt"] } });
    await waitFor(() => expect(overlay).toHaveAttribute("aria-hidden", "false"));
    dropHandler?.({ payload: { type: "drop", paths: ["D:\\x\\0001.txt"] } });
    await waitFor(() => expect(importChapters).toHaveBeenCalledWith("D:\\lib\\t", ["D:\\x\\0001.txt"]));
    await waitFor(() => expect(useStoryStore.getState().snapshot?.counts.total).toBe(1));
    expect(overlay).toHaveAttribute("aria-hidden", "true");
  });

  it("đang chạy phiên thì Quét lại bị khoá", () => {
    useStoryStore.setState({ sessions: { "d:\\lib\\t": { status: "running", sessionNo: 1 } } });
    render(<TranslatePage />);
    expect(screen.getByRole("button", { name: "Quét lại" })).toBeDisabled();
  });
});
