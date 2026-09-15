import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { TranslateToolbar } from "@/components/translate-toolbar";
import { chaptersRetryIds, sessionStop, storySnapshot } from "@/lib/api";
import { pathKey } from "@/lib/paths";
import { appConfigSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  chaptersDelete: vi.fn(),
  chaptersRetry: vi.fn(),
  chaptersRetryIds: vi.fn(),
  rescanStory: vi.fn(),
  sessionStart: vi.fn(),
  sessionStop: vi.fn(),
  storySnapshot: vi.fn(),
}));

const ROOT = "D:\\t";
const config = appConfigSchema.parse({ agyPath: null, model: null, maxSessions: 50, recent: [] });

describe("TranslateToolbar · nút Dừng", () => {
  beforeEach(() => {
    useStoryStore.setState({
      config,
      root: ROOT,
      roots: { [pathKey(ROOT)]: ROOT },
      sessions: { [pathKey(ROOT)]: { status: "running", sessionNo: 1 } },
    });
  });

  it("bấm Dừng thì nút đổi 'Đang dừng…' và khoá cho tới khi session_stop trả về", async () => {
    let finish: (value: { running: string[] }) => void = () => undefined;
    vi.mocked(sessionStop).mockReturnValue(new Promise((resolve) => (finish = resolve)));
    vi.mocked(storySnapshot).mockResolvedValue({
      root: ROOT,
      chapters: [],
      counts: { total: 0, queued: 0, translating: 0, done: 0, error: 0, skipped: 0, withWarnings: 0 },
      settings: { minLengthRatio: 0.75, maxReviewRounds: 3, chaptersPerSession: 10 },
      story: { name: "" } as never,
      sessionRunning: false,
    });
    const user = userEvent.setup();
    render(<TranslateToolbar />);
    await user.click(screen.getByRole("button", { name: "Dừng" }));
    const stopping = screen.getByRole("button", { name: /Đang dừng/ });
    expect(stopping).toBeDisabled();
    expect(sessionStop).toHaveBeenCalledWith(ROOT);
    finish({ running: [] });
    // Trạng thái phiên chỉ đổi khi Rust phát event `stopped`; ở đây chỉ kiểm tra nút hết bận.
    expect(await screen.findByRole("button", { name: "Dừng" })).toBeEnabled();
  });
});

describe("TranslateToolbar · cảnh báo hổng chương", () => {
  beforeEach(() => {
    useStoryStore.setState({
      config,
      root: ROOT,
      roots: { [pathKey(ROOT)]: ROOT },
      sessions: {},
      selectedId: undefined,
      snapshot: {
        root: ROOT,
        story: { title: "T" },
        chapters: [
          { id: "c1", status: "done", reviewRound: 0, reason: null, warnings: [] },
          { id: "c2", status: "skipped", reviewRound: 0, reason: "model từ chối", warnings: [] },
          { id: "c3", status: "done", reviewRound: 0, reason: null, warnings: [] },
          { id: "c4", status: "queued", reviewRound: 0, reason: null, warnings: [] },
        ],
      } as never,
    });
  });

  it("nút Xoá… mở dialog xoá nhiều chương; đang chạy phiên thì khoá", async () => {
    const user = userEvent.setup();
    const { unmount } = render(<TranslateToolbar />);
    await user.click(screen.getByRole("button", { name: "Xoá…" }));
    expect(screen.getByRole("dialog", { name: "Xoá nhiều chương" })).toBeInTheDocument();
    unmount();
    useStoryStore.setState({ sessions: { [pathKey(ROOT)]: { status: "running", sessionNo: 1 } } });
    render(<TranslateToolbar />);
    expect(screen.getByRole("button", { name: "Xoá…" })).toBeDisabled();
  });

  it("báo chương chưa dịch đứng trước chương done cuối; bấm số thứ tự thì chọn chương đó", async () => {
    render(<TranslateToolbar />);
    const alert = screen.getByRole("status", { name: /hổng/i });
    expect(alert).toHaveTextContent("1 chương trước #3 chưa dịch");
    await userEvent.click(screen.getByRole("button", { name: "#2 c2" }));
    expect(useStoryStore.getState().selectedId).toBe("c2");
  });

  it("nút 'Dịch lại cả N' đếm chương hổng chưa queued, gọi đúng id rồi tải lại snapshot; đang chạy phiên thì khoá", async () => {
    vi.mocked(chaptersRetryIds).mockResolvedValue({ retried: ["c2"], backedUp: [], alreadyQueued: [] });
    const reloaded = { ...useStoryStore.getState().snapshot!, chapters: [] };
    vi.mocked(storySnapshot).mockResolvedValue(reloaded);
    const user = userEvent.setup();
    const { unmount } = render(<TranslateToolbar />);
    await user.click(screen.getByRole("button", { name: "Dịch lại cả 1" }));
    expect(chaptersRetryIds).toHaveBeenCalledWith(ROOT, ["c2"]);
    expect(storySnapshot).toHaveBeenCalledWith(ROOT);
    await vi.waitFor(() => expect(useStoryStore.getState().snapshot?.chapters).toHaveLength(0));
    unmount();

    useStoryStore.setState((s) => ({
      snapshot: {
        ...s.snapshot!,
        chapters: [
          { id: "c1", status: "done", reviewRound: 0, reason: null, warnings: [] },
          { id: "c2", status: "queued", reviewRound: 0, reason: null, warnings: [] },
          { id: "c3", status: "error", reviewRound: 0, reason: "lỗi", warnings: [] },
          { id: "c4", status: "done", reviewRound: 0, reason: null, warnings: [] },
        ],
      } as never,
      sessions: { [pathKey(ROOT)]: { status: "running", sessionNo: 1 } },
    }));
    render(<TranslateToolbar />);
    // c2 queued sẵn không đếm; chỉ c3.
    expect(screen.getByRole("button", { name: "Dịch lại cả 1" })).toBeDisabled();
  });

  it("hổng toàn chương queued thì không có nút dịch lại", () => {
    useStoryStore.setState((s) => ({
      snapshot: {
        ...s.snapshot!,
        chapters: s.snapshot!.chapters.map((c) => (c.id === "c2" ? { ...c, status: "queued", reason: null } : c)),
      } as never,
    }));
    render(<TranslateToolbar />);
    expect(screen.getByRole("status", { name: /hổng/i })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Dịch lại cả/ })).not.toBeInTheDocument();
  });

  it("không hổng thì không hiện", () => {
    useStoryStore.setState((s) => ({
      snapshot: { ...s.snapshot!, chapters: s.snapshot!.chapters.filter((c) => c.id !== "c2") },
    }));
    render(<TranslateToolbar />);
    expect(screen.queryByRole("status", { name: /hổng/i })).not.toBeInTheDocument();
  });
});
