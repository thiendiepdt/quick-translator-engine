import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ChapterReader } from "@/components/chapter-reader";
import { appConfigSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  appConfigSet: vi.fn((config: unknown) => Promise.resolve(config)),
  chapterForceAccept: vi.fn(),
  chapterRetry: vi.fn(),
  chapterSkip: vi.fn(),
  readChapter: vi.fn(() =>
    Promise.resolve({ id: "0002", status: "done", raw: "原文", output: "Bản dịch.", draft: null, review: null, warnings: [], reason: null }),
  ),
  revealFolder: vi.fn(),
  storySnapshot: vi.fn(),
}));

const config = appConfigSchema.parse({ agyPath: null, model: null, maxSessions: 50, recent: [], readingWidth: "wide" });
const row = { id: "0002", status: "done" as const, reviewRound: 0, reason: null, warnings: [] };

describe("ChapterReader", () => {
  beforeEach(() => {
    useStoryStore.setState({ config, session: { status: "idle" } });
  });

  it("nút chương trước/sau nằm trên đầu trang, theo hasPrev/hasNext; vùng đọc theo readingWidth", async () => {
    const user = userEvent.setup();
    const onPrev = vi.fn();
    const onNext = vi.fn();
    render(<ChapterReader root="D:\\t" row={row} hasPrev hasNext={false} onPrev={onPrev} onNext={onNext} />);
    await waitFor(() => expect(screen.getByText("Bản dịch.")).toBeInTheDocument());
    // Cặp nút đầu trang (index 0) + cặp cuối bài: cả hai cùng nhãn, cùng trạng thái.
    const [headerPrev, footerPrev] = screen.getAllByRole("button", { name: "Chương trước" });
    expect(footerPrev).toBeDefined();
    const [headerNext] = screen.getAllByRole("button", { name: "Chương sau" });
    expect(headerNext).toBeDisabled();
    expect(headerPrev).toBeDefined();
    if (!headerPrev) throw new Error("thiếu nút đầu trang");
    expect(headerPrev.closest("article")).toBeNull();
    await user.click(headerPrev);
    expect(onPrev).toHaveBeenCalledTimes(1);
    expect(document.querySelector("article.reading")).toHaveAttribute("data-width", "wide");
    expect(screen.getByRole("combobox", { name: "Chiều ngang văn bản" })).toHaveTextContent("Rộng");
  });
});
