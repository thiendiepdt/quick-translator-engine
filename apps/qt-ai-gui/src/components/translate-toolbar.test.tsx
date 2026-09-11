import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { TranslateToolbar } from "@/components/translate-toolbar";
import { sessionStop, storySnapshot } from "@/lib/api";
import { pathKey } from "@/lib/paths";
import { appConfigSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
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
