import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { CloseGuard } from "@/components/close-guard";
import { sessionsStopAll } from "@/lib/api";
import { pathKey } from "@/lib/paths";
import { useStoryStore } from "@/store/story";

const win = vi.hoisted(() => ({
  handler: undefined as ((event: { preventDefault: () => void }) => void) | undefined,
  destroy: vi.fn(() => Promise.resolve()),
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    onCloseRequested: (handler: (event: { preventDefault: () => void }) => void) => {
      win.handler = handler;
      return Promise.resolve(() => {});
    },
    destroy: win.destroy,
  }),
}));
vi.mock("@/lib/api", () => ({ sessionsStopAll: vi.fn() }));

const A = "D:\\lib\\dien-phe";
const B = "D:\\lib\\saint-seiya";

async function requestClose() {
  const event = { preventDefault: vi.fn() };
  await act(async () => {
    await Promise.resolve();
    win.handler?.(event);
  });
  return event;
}

describe("CloseGuard", () => {
  beforeEach(() => {
    win.handler = undefined;
    win.destroy.mockClear();
    vi.mocked(sessionsStopAll).mockReset().mockResolvedValue({ running: [] });
    useStoryStore.setState({ sessions: {}, roots: {}, names: {} });
  });

  it("không có phiên chạy thì để đóng bình thường", async () => {
    render(<CloseGuard />);
    const event = await requestClose();
    expect(event.preventDefault).not.toHaveBeenCalled();
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("đang dịch thì chặn đóng, hỏi kèm tên truyện; Dừng và đóng thì dừng hết phiên rồi đóng", async () => {
    useStoryStore.setState({
      sessions: { [pathKey(A)]: { status: "running", sessionNo: 1 }, [pathKey(B)]: { status: "running", sessionNo: 2 } },
      roots: { [pathKey(A)]: A, [pathKey(B)]: B },
      names: { [pathKey(A)]: "Điên phê đại lão" },
    });
    render(<CloseGuard />);
    const event = await requestClose();
    expect(event.preventDefault).toHaveBeenCalled();
    expect(screen.getByRole("dialog", { name: "Đang dịch 2 truyện" })).toBeInTheDocument();
    expect(screen.getByText("Điên phê đại lão")).toBeInTheDocument();
    expect(screen.getByText("saint-seiya")).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Dừng và đóng" }));
    expect(sessionsStopAll).toHaveBeenCalled();
    expect(win.destroy).toHaveBeenCalled();
  });

  it("Huỷ thì giữ app, không dừng phiên", async () => {
    useStoryStore.setState({ sessions: { [pathKey(A)]: { status: "running", sessionNo: 1 } }, roots: { [pathKey(A)]: A } });
    render(<CloseGuard />);
    await requestClose();
    await userEvent.click(screen.getByRole("button", { name: "Huỷ" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    expect(sessionsStopAll).not.toHaveBeenCalled();
    expect(win.destroy).not.toHaveBeenCalled();
  });
});
