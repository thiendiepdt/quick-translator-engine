import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ChapterReader } from "@/components/chapter-reader";
import { chapterRetry, chaptersDelete, readChapter, saveChapterOutput, storySnapshot } from "@/lib/api";
import { appConfigSchema } from "@/lib/schema";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  appConfigSet: vi.fn((config: unknown) => Promise.resolve(config)),
  chapterForceAccept: vi.fn(),
  chapterRetry: vi.fn(),
  chaptersDelete: vi.fn(),
  chapterSkip: vi.fn(),
  readChapter: vi.fn(() =>
    Promise.resolve({ id: "0002", status: "done", raw: "原文", output: "Bản dịch.", draft: null, review: null, warnings: [], reason: null }),
  ),
  revealFolder: vi.fn(),
  saveChapterOutput: vi.fn(),
  storySnapshot: vi.fn(),
}));

const config = appConfigSchema.parse({ agyPath: null, model: null, maxSessions: 50, recent: [], readingWidth: "wide" });
const ROOT = "D:\\t";
const row = { id: "0002", status: "done" as const, reviewRound: 0, reason: null, warnings: [] };

describe("ChapterReader", () => {
  beforeEach(() => {
    useStoryStore.setState({ config, sessions: {} });
    vi.mocked(chapterRetry).mockReset();
    vi.mocked(chapterRetry).mockResolvedValue(undefined);
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

  it("chương done: Dịch lại bật, hỏi xác nhận (nêu .bak) rồi mới gọi chapterRetry", async () => {
    const user = userEvent.setup();
    vi.mocked(storySnapshot).mockResolvedValue({
      root: ROOT,
      chapters: [],
      counts: { total: 0, queued: 0, translating: 0, done: 0, error: 0, skipped: 0, withWarnings: 0 },
      settings: { maxReviewRounds: 3 },
      story: { name: "" } as never,
      sessionRunning: false,
    } as never);
    render(<ChapterReader root={ROOT} row={row} hasPrev hasNext onPrev={vi.fn()} onNext={vi.fn()} />);
    await waitFor(() => expect(screen.getByText("Bản dịch.")).toBeInTheDocument());
    const button = screen.getByRole("button", { name: "Dịch lại" });
    expect(button).toBeEnabled();
    await user.click(button);
    expect(chapterRetry).not.toHaveBeenCalled();
    expect(screen.getByText(/out\/0002\.txt\.bak/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Huỷ" }));
    expect(chapterRetry).not.toHaveBeenCalled();
    await user.click(button);
    await user.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Dịch lại" }));
    expect(chapterRetry).toHaveBeenCalledWith(ROOT, "0002");
  });

  it("Xoá chương: hỏi xác nhận (nêu out/ giữ nguyên) rồi gọi chapters_delete với đúng id và nạp lại snapshot", async () => {
    const user = userEvent.setup();
    vi.mocked(chaptersDelete).mockReset().mockResolvedValue({ removed: ["0002"], keptOutputs: ["0002"] });
    vi.mocked(storySnapshot).mockReset().mockResolvedValue({} as never);
    render(<ChapterReader root={ROOT} row={row} ordinal={2} hasPrev hasNext onPrev={vi.fn()} onNext={vi.fn()} />);
    await waitFor(() => expect(screen.getByText("Bản dịch.")).toBeInTheDocument());
    await user.click(screen.getByRole("button", { name: "Xoá chương" }));
    expect(chaptersDelete).not.toHaveBeenCalled();
    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getByText(/Xoá chương #2 0002\?/)).toBeInTheDocument();
    expect(within(dialog).getByText(/out\/0002\.txt giữ nguyên/)).toBeInTheDocument();
    await user.click(within(dialog).getByRole("button", { name: "Huỷ" }));
    expect(chaptersDelete).not.toHaveBeenCalled();
    await user.click(screen.getByRole("button", { name: "Xoá chương" }));
    await user.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Xoá" }));
    await waitFor(() => expect(chaptersDelete).toHaveBeenCalledWith(ROOT, ["0002"]));
    await waitFor(() => expect(storySnapshot).toHaveBeenCalledWith(ROOT));
  });

  it("đang chạy phiên thì Xoá chương khoá", async () => {
    useStoryStore.setState({ root: ROOT, sessions: { "d:\\t": { status: "running", sessionNo: 1 } } });
    render(<ChapterReader root={ROOT} row={row} hasPrev hasNext onPrev={vi.fn()} onNext={vi.fn()} />);
    await waitFor(() => expect(screen.getByText("Bản dịch.")).toBeInTheDocument());
    expect(screen.getByRole("button", { name: "Xoá chương" })).toBeDisabled();
    useStoryStore.setState({ root: undefined });
  });

  it("raw đã mất: tab Gốc nhắc Quét lại thay vì nổ lỗi", async () => {
    const user = userEvent.setup();
    vi.mocked(readChapter).mockResolvedValueOnce({
      id: "0002", status: "done", raw: "", rawMissing: true, output: "Bản dịch.", draft: null, review: null, warnings: [], reason: null,
    });
    render(<ChapterReader root={ROOT} row={row} hasPrev hasNext onPrev={vi.fn()} onNext={vi.fn()} />);
    await waitFor(() => expect(screen.getByText("Bản dịch.")).toBeInTheDocument());
    await user.click(screen.getByRole("tab", { name: "Gốc" }));
    expect(screen.getByText(/raw\/0002\.txt đã mất/)).toBeInTheDocument();
  });

  it("chương translating: Dịch lại gọi thẳng không hỏi; queued thì tắt", async () => {
    const user = userEvent.setup();
    vi.mocked(storySnapshot).mockResolvedValue({} as never);
    const translating = { ...row, status: "translating" as const };
    const { rerender } = render(
      <ChapterReader root={ROOT} row={translating} hasPrev hasNext onPrev={vi.fn()} onNext={vi.fn()} />,
    );
    await user.click(screen.getByRole("button", { name: "Dịch lại" }));
    expect(chapterRetry).toHaveBeenCalledWith(ROOT, "0002");
    rerender(<ChapterReader root={ROOT} row={{ ...row, status: "queued" }} hasPrev hasNext onPrev={vi.fn()} onNext={vi.fn()} />);
    expect(screen.getByRole("button", { name: "Dịch lại" })).toBeDisabled();
  });

  it("bản dịch mặc định chỉ đọc; Sửa → textarea, Lưu gọi save_chapter_output rồi đọc lại; Huỷ bỏ thay đổi", async () => {
    const user = userEvent.setup();
    vi.mocked(saveChapterOutput).mockResolvedValue(undefined);
    render(<ChapterReader root={ROOT} row={row} hasPrev hasNext onPrev={vi.fn()} onNext={vi.fn()} />);
    await waitFor(() => expect(screen.getByText("Bản dịch.")).toBeInTheDocument());
    expect(screen.queryByRole("textbox", { name: "Bản dịch đang sửa" })).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Sửa" }));
    const box = screen.getByRole("textbox", { name: "Bản dịch đang sửa" });
    expect(box).toHaveValue("Bản dịch.");
    expect(screen.getByRole("button", { name: "Lưu" })).toBeDisabled(); // chưa đổi gì
    await user.clear(box);
    await user.type(box, "Bản người sửa.");
    await user.click(screen.getByRole("button", { name: "Huỷ" }));
    expect(screen.queryByRole("textbox", { name: "Bản dịch đang sửa" })).not.toBeInTheDocument();
    expect(saveChapterOutput).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "Sửa" }));
    await user.clear(screen.getByRole("textbox", { name: "Bản dịch đang sửa" }));
    await user.type(screen.getByRole("textbox", { name: "Bản dịch đang sửa" }), "Bản người sửa.");
    vi.mocked(readChapter).mockResolvedValueOnce({
      id: "0002", status: "done", raw: "原文", rawMissing: false, output: "Bản người sửa.", draft: null, review: null, warnings: [], reason: null,
    });
    await user.keyboard("{Control>}s{/Control}"); // Ctrl+S lưu, không cần bấm nút
    await waitFor(() => expect(saveChapterOutput).toHaveBeenCalledWith(ROOT, "0002", "Bản người sửa."));
    expect(await screen.findByText("Bản người sửa.")).toBeInTheDocument();
    expect(screen.queryByRole("textbox", { name: "Bản dịch đang sửa" })).not.toBeInTheDocument();
  }, 15000);

  it("nút Chép đưa nội dung tab hiện tại vào clipboard", async () => {
    const user = userEvent.setup();
    const writeText = vi.fn(() => Promise.resolve());
    vi.stubGlobal("navigator", { ...navigator, clipboard: { writeText } });
    render(<ChapterReader root={ROOT} row={row} hasPrev hasNext onPrev={vi.fn()} onNext={vi.fn()} />);
    await waitFor(() => expect(screen.getByText("Bản dịch.")).toBeInTheDocument());
    await user.click(screen.getByRole("button", { name: "Chép bản dịch" }));
    expect(writeText).toHaveBeenCalledWith("Bản dịch.");
    vi.unstubAllGlobals();
  });
});
