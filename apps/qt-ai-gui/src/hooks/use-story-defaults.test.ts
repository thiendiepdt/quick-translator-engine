import { renderHook, waitFor } from "@testing-library/react";
import { act } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useStoryDefaults } from "@/hooks/use-story-defaults";
import { storyDefaults } from "@/lib/api";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ storyDefaults: vi.fn() }));

const value = (tag: string) => ({
  basePrompt: tag,
  promptSource: "file" as const,
  promptSuffix: "s",
  checkRules: [],
  rulesSource: "builtin" as const,
});

describe("useStoryDefaults", () => {
  beforeEach(() => {
    vi.mocked(storyDefaults).mockReset();
    useStoryStore.setState({ baseVersion: 0 });
  });

  it("nạp theo genre, dùng lại kết quả cho cùng genre, nạp lại khi baseVersion tăng", async () => {
    vi.mocked(storyDefaults).mockResolvedValueOnce(value("v1")).mockResolvedValueOnce(value("v2"));
    const genre = { setting: "ancient" as const, names: "han" as const, tone: "neutral" as const };
    const { result, rerender } = renderHook(() => useStoryDefaults(genre));
    await waitFor(() => expect(result.current?.basePrompt).toBe("v1"));
    rerender();
    expect(storyDefaults).toHaveBeenCalledTimes(1);
    act(() => useStoryStore.getState().bumpBaseVersion());
    await waitFor(() => expect(result.current?.basePrompt).toBe("v2"));
    expect(storyDefaults).toHaveBeenCalledTimes(2);
  });
});
