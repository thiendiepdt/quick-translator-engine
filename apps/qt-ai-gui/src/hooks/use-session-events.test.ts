import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { storyLabel, useSessionEvents } from "@/hooks/use-session-events";
import { storySnapshot } from "@/lib/api";
import { sessionOf, useStoryStore } from "@/store/story";

type Handler = (event: { payload: unknown }) => void;
let handler: Handler | undefined;

vi.mock("@tauri-apps/api/event", () => ({
  listen: (_name: string, cb: Handler) => {
    handler = cb;
    return Promise.resolve(() => undefined);
  },
}));

vi.mock("@/lib/api", () => ({
  storySnapshot: vi.fn(),
  progressDrift: vi.fn(),
}));

const A = "D:\\lib\\a";
const B = "D:\\lib\\b";

describe("useSessionEvents", () => {
  beforeEach(() => {
    handler = undefined;
    vi.mocked(storySnapshot).mockReset();
    useStoryStore.setState({ sessions: {}, progress: {}, logs: {}, roots: {}, root: A, snapshot: undefined });
  });

  it("event mang root: vào store đúng truyện; chỉ truyện đang mở mới nạp lại snapshot", async () => {
    renderHook(() => useSessionEvents());
    await waitFor(() => expect(handler).toBeDefined());
    handler?.({ payload: { root: B, type: "started", session_no: 3 } });
    handler?.({ payload: { root: B, type: "stopped", kind: "finished" } });
    expect(sessionOf(useStoryStore.getState(), B)).toEqual({ status: "stopped", reason: { kind: "finished" } });
    expect(sessionOf(useStoryStore.getState(), A)).toEqual({ status: "idle" });
    expect(storySnapshot).not.toHaveBeenCalled(); // truyện B không phải truyện đang mở

    vi.mocked(storySnapshot).mockResolvedValue({ root: A } as never);
    handler?.({ payload: { root: "d:/lib/a/", type: "started", session_no: 1 } });
    handler?.({ payload: { root: "d:/lib/a/", type: "stopped", kind: "user_cancelled" } });
    await waitFor(() => expect(storySnapshot).toHaveBeenCalledWith(A));
    expect(sessionOf(useStoryStore.getState(), A)).toEqual({ status: "stopped", reason: { kind: "user_cancelled" } });

    handler?.({ payload: { type: "started", session_no: 9 } }); // thiếu root → bỏ qua
    expect(Object.keys(useStoryStore.getState().sessions)).toHaveLength(2);
  });

  it("storyLabel lấy tên folder cuối", () => {
    expect(storyLabel("D:\\lib\\ta-tuyet-the\\")).toBe("ta-tuyet-the");
    expect(storyLabel("/srv/books/x")).toBe("x");
  });

  it("phiên của truyện đang mở dừng/xong thì hỏi lại độ lệch tiến độ đang nhắc", async () => {
    const { progressDrift } = await import("@/lib/api");
    vi.mocked(progressDrift).mockReset().mockResolvedValue({ untracked: [], missing: [] });
    vi.mocked(storySnapshot).mockResolvedValue({ root: A } as never);
    useStoryStore.setState({ snapshot: { root: A, progressDrift: { untracked: 5, missing: 0 } } as never });
    renderHook(() => useSessionEvents());
    await waitFor(() => expect(handler).toBeDefined());
    handler?.({ payload: { root: A, type: "progress", done: 1, total: 2 } });
    expect(progressDrift).not.toHaveBeenCalled();
    handler?.({ payload: { root: A, type: "stopped", kind: "finished" } });
    await waitFor(() => expect(progressDrift).toHaveBeenCalledWith(A));
    await waitFor(() => expect(useStoryStore.getState().snapshot?.progressDrift).toEqual({ untracked: 0, missing: 0 }));
  });
});
