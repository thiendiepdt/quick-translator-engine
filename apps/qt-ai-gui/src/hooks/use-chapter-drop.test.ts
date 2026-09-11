import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { describeImport, useChapterDrop } from "@/hooks/use-chapter-drop";
import { importChapters } from "@/lib/api";

type DropHandler = (event: { payload: { type: string; paths?: string[] } }) => void;
let handler: DropHandler | undefined;
const unlisten = vi.fn();

vi.mock("@tauri-apps/api/webview", () => ({
  getCurrentWebview: () => ({
    onDragDropEvent: (cb: DropHandler) => {
      handler = cb;
      return Promise.resolve(unlisten);
    },
  }),
}));

vi.mock("@/lib/api", () => ({
  importChapters: vi.fn(),
}));

describe("useChapterDrop", () => {
  beforeEach(() => {
    handler = undefined;
    unlisten.mockClear();
    vi.mocked(importChapters).mockReset();
  });

  it("enter/over bật dragging, leave tắt, drop gọi import_chapters với paths và báo kết quả", async () => {
    const outcome = { added: ["0001.txt"], skippedExisting: [], ignored: [], snapshot: {} } as never;
    vi.mocked(importChapters).mockResolvedValue(outcome);
    const onImported = vi.fn();
    const onError = vi.fn();
    const { result, unmount } = renderHook(() => useChapterDrop("D:\\t", onImported, onError));
    await waitFor(() => expect(handler).toBeDefined());

    act(() => handler?.({ payload: { type: "enter", paths: ["a.txt"] } }));
    expect(result.current).toBe(true);
    act(() => handler?.({ payload: { type: "leave" } }));
    expect(result.current).toBe(false);
    act(() => handler?.({ payload: { type: "over" } }));
    expect(result.current).toBe(true);
    act(() => handler?.({ payload: { type: "drop", paths: ["D:\\x\\0001.txt", "D:\\x\\batch"] } }));
    expect(result.current).toBe(false);
    await waitFor(() => expect(onImported).toHaveBeenCalledWith(outcome));
    expect(importChapters).toHaveBeenCalledWith("D:\\t", ["D:\\x\\0001.txt", "D:\\x\\batch"]);
    expect(onError).not.toHaveBeenCalled();

    unmount();
    expect(unlisten).toHaveBeenCalled();
  });

  it("không có root thì không lắng nghe; import lỗi thì báo onError", async () => {
    const onError = vi.fn();
    renderHook(() => useChapterDrop(undefined, () => undefined, onError));
    expect(handler).toBeUndefined();

    vi.mocked(importChapters).mockRejectedValue(new Error("hỏng"));
    renderHook(() => useChapterDrop("D:\\t", () => undefined, onError));
    await waitFor(() => expect(handler).toBeDefined());
    act(() => handler?.({ payload: { type: "drop", paths: ["x"] } }));
    await waitFor(() => expect(onError).toHaveBeenCalledWith("hỏng"));
  });

  it("describeImport chỉ nêu phần khác 0", () => {
    expect(describeImport({ added: ["a", "b"], skippedExisting: [], ignored: [] })).toBe("Đã thêm 2 chương");
    expect(describeImport({ added: [], skippedExisting: ["a"], ignored: ["b", "c"] })).toBe(
      "Đã thêm 0 chương, bỏ qua 1 trùng tên, 2 file không phải .txt",
    );
  });
});
