import { renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useStoryChecked } from "@/hooks/use-story-checked";
import { storySnapshot } from "@/lib/api";
import { useStoryStore } from "@/store/story";

type Handler = (event: { payload: unknown }) => void;
let handler: Handler | undefined;

vi.mock("@tauri-apps/api/event", () => ({
  listen: (_name: string, cb: Handler) => {
    handler = cb;
    return Promise.resolve(() => undefined);
  },
}));
vi.mock("@/lib/api", () => ({ storySnapshot: vi.fn() }));

const A = "D:\\lib\\a";
const fresh = { root: A, chapters: [], counts: { skipped: 0, error: 0 }, progressDrift: null };

describe("useStoryChecked", () => {
  beforeEach(() => {
    handler = undefined;
    vi.mocked(storySnapshot).mockReset().mockResolvedValue(fresh as never);
    useStoryStore.setState({ root: A, snapshot: { ...fresh, progressDrift: null } as never, issues: {} });
  });

  it("quét nền xong của truyện đang mở: nạp lại snapshot kèm số chương lệch", async () => {
    renderHook(() => useStoryChecked());
    await waitFor(() => expect(handler).toBeDefined());
    handler?.({ payload: { root: "d:/lib/a/", progressDrift: { untracked: 3, missing: 1 } } });
    await waitFor(() => expect(useStoryStore.getState().snapshot?.progressDrift).toEqual({ untracked: 3, missing: 1 }));
    expect(storySnapshot).toHaveBeenCalledWith(A);
  });

  it("truyện khác hoặc event sai dạng thì bỏ qua", async () => {
    renderHook(() => useStoryChecked());
    await waitFor(() => expect(handler).toBeDefined());
    handler?.({ payload: { root: "D:\\lib\\b", progressDrift: { untracked: 2, missing: 0 } } });
    handler?.({ payload: { root: A } });
    expect(storySnapshot).not.toHaveBeenCalled();
  });
});
