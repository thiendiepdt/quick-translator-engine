import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { StoryPicker } from "@/components/story-picker";
import { appConfigSet, recentSummaries } from "@/lib/api";
import { appConfigSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  ApiError: class ApiError extends Error {},
  appConfigSet: vi.fn((config: unknown) => Promise.resolve(config)),
  initStory: vi.fn(),
  openStory: vi.fn(),
  pickFolder: vi.fn(),
  recentSummaries: vi.fn(() => Promise.resolve([])),
}));

const config = appConfigSchema.parse({
  agyPath: null,
  model: null,
  maxSessions: 50,
  recent: ["D:\\truyen-a", "D:\\truyen-hong"],
});

describe("StoryPicker · Mở gần đây", () => {
  beforeEach(() => {
    vi.mocked(appConfigSet).mockClear();
    vi.mocked(recentSummaries).mockResolvedValue([
      { root: "D:\\truyen-a", name: "Truyện A", done: 2, total: 10 },
      { root: "D:\\truyen-hong", name: null, done: null, total: null },
    ]);
    useStoryStore.setState({ screen: "picker", config, session: { status: "idle" } });
  });

  it("bấm X bỏ truyện khỏi danh sách: chỉ ghi config.recent, không gọi gì xoá file", async () => {
    const user = userEvent.setup();
    render(<StoryPicker />);
    expect(await screen.findByText("Truyện A")).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: /Bỏ .* khỏi danh sách/ })).toHaveLength(2);

    await user.click(screen.getByRole("button", { name: "Bỏ D:\\truyen-a khỏi danh sách" }));

    await waitFor(() =>
      expect(appConfigSet).toHaveBeenCalledWith(expect.objectContaining({ recent: ["D:\\truyen-hong"] })),
    );
    expect(useStoryStore.getState().config?.recent).toEqual(["D:\\truyen-hong"]);
    expect(screen.queryByText("Truyện A")).not.toBeInTheDocument();
    // Folder hỏng (không đọc được) vẫn bỏ được — đó là lúc cần nút này nhất.
    await user.click(screen.getByRole("button", { name: "Bỏ D:\\truyen-hong khỏi danh sách" }));
    await waitFor(() => expect(useStoryStore.getState().config?.recent).toEqual([]));
    expect(screen.queryByText("Mở gần đây")).not.toBeInTheDocument();
  });
});
