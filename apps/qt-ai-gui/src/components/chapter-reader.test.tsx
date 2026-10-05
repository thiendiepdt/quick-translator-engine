import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ChapterReader } from "@/components/chapter-reader";
import { appConfigSet, chapterRetry, chaptersDelete, readChapter, saveChapterOutput, storySnapshot } from "@/lib/api";
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

  it("nút Số dòng bật/tắt đánh số đoạn ở Bản dịch và Gốc, lưu vào AppConfig; số không lẫn vào nội dung chép", async () => {
    vi.mocked(readChapter).mockResolvedValueOnce({
      id: "0002",
      status: "done",
      raw: "第1章 甲\n\n二。\n三。",
      rawMissing: false,
      output: "Chương 1: A\n\nĐoạn hai.\n\nĐoạn ba.\n",
      draft: "[[1]] nháp",
      review: null,
      warnings: [],
      reason: null,
    });
    // Config mới (hoặc config cũ chưa có trường này) mặc định bật; ở đây tắt trước để thử cả hai chiều.
    expect(config.lineNumbers).toBe(true);
    useStoryStore.setState({ config: { ...config, lineNumbers: false } });
    const user = userEvent.setup();
    const writeText = vi.fn(() => Promise.resolve());
    vi.stubGlobal("navigator", { ...navigator, clipboard: { writeText } });
    render(<ChapterReader root={ROOT} row={row} hasPrev hasNext onPrev={vi.fn()} onNext={vi.fn()} />);
    await waitFor(() => expect(screen.getByText(/Đoạn hai\./)).toBeInTheDocument());
    const toggle = screen.getByRole("button", { name: "Số dòng" });
    expect(toggle).toHaveAttribute("aria-pressed", "false");
    expect(screen.getByTestId("reading-text").querySelectorAll("[aria-hidden]")).toHaveLength(0);

    await user.click(toggle);
    await waitFor(() => expect(appConfigSet).toHaveBeenLastCalledWith(expect.objectContaining({ lineNumbers: true })));
    expect(useStoryStore.getState().config?.lineNumbers).toBe(true);
    expect(screen.getByRole("button", { name: "Số dòng" })).toHaveAttribute("aria-pressed", "true");
    const gutter = (panel: HTMLElement) =>
      [...panel.querySelectorAll("[aria-hidden]")].map((cell) => cell.textContent).filter((text) => text !== "");
    const output = screen.getByTestId("reading-text");
    expect(gutter(output)).toEqual(["1", "2", "3"]);
    expect(within(output).getByText("Đoạn ba.")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Chép bản dịch" }));
    expect(writeText).toHaveBeenCalledWith("Chương 1: A\n\nĐoạn hai.\n\nĐoạn ba.\n");

    // Gốc: cùng cách đếm nên dòng 3 của gốc ứng với dòng 3 của bản dịch. Nháp đã có nhãn [[n]] nên không đánh số.
    await user.click(screen.getByRole("tab", { name: "Gốc" }));
    expect(gutter(screen.getByTestId("reading-text"))).toEqual(["1", "2", "3"]);
    expect(within(screen.getByTestId("reading-text")).getByText("三。")).toBeInTheDocument();
    // Gốc toàn chữ Hán: không tô, không báo.
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Nháp" }));
    expect(screen.queryByTestId("reading-text")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Số dòng" }));
    await waitFor(() => expect(useStoryStore.getState().config?.lineNumbers).toBe(false));
    vi.unstubAllGlobals();
  });

  it("bản dịch còn sót chữ Hán: tô dòng + tô riêng chữ Hán, thanh báo liệt kê số dòng; tắt số dòng vẫn tô", async () => {
    vi.mocked(readChapter).mockResolvedValueOnce({
      id: "0002",
      status: "done",
      raw: "第1章 甲\n\n二。\n\n三。\n\n四。",
      rawMissing: false,
      output: "Chương 1: A\n\nNàng nói: 好的, đi thôi.\n\n【Hệ thống】 Đoạn sạch.\n\n魔法 mở đầu.\n",
      draft: null,
      review: null,
      warnings: [],
      reason: null,
    });
    const user = userEvent.setup();
    render(<ChapterReader root={ROOT} row={row} hasPrev hasNext onPrev={vi.fn()} onNext={vi.fn()} />);
    const alert = await screen.findByRole("alert");
    expect(alert).toHaveTextContent("2 dòng còn sót chữ Hán:");
    // Độ dài: gốc đếm ký tự không khoảng trắng, bản dịch đếm chữ; tooltip có số ký tự và tỉ lệ dịch/gốc.
    const stats = screen.getByLabelText("Độ dài chương");
    expect(stats).toHaveTextContent("Gốc 10 chữ · Dịch 15 chữ");
    expect(stats).toHaveAttribute(
      "title",
      "Gốc: 10 ký tự, không tính khoảng trắng. Bản dịch: 15 chữ (53 ký tự). Tỉ lệ ký tự dịch/gốc: 5,30.",
    );
    expect(within(alert).getAllByRole("button").map((button) => button.textContent)).toEqual(["2", "4"]);
    const body = screen.getByTestId("reading-text");
    const flagged = [...body.querySelectorAll("[data-han]")];
    expect(flagged.map((line) => line.getAttribute("data-line"))).toEqual(["2", "4"]);
    expect([...body.querySelectorAll("mark")].map((mark) => mark.textContent)).toEqual(["好的", "魔法"]);
    // Dòng sạch (kể cả có ngoặc 【】) không bị tô; nội dung dòng bị tô vẫn nguyên chữ.
    expect(body.querySelector('[data-line="3"]')).not.toHaveAttribute("data-han");
    expect(body.querySelector('[data-line="2"]')).toHaveTextContent("Nàng nói: 好的, đi thôi.");
    await user.click(within(alert).getByRole("button", { name: "Tới dòng 4" }));

    await user.click(screen.getByRole("button", { name: "Số dòng" }));
    await waitFor(() => expect(useStoryStore.getState().config?.lineNumbers).toBe(false));
    expect(screen.getByTestId("reading-text").querySelectorAll("[data-han]")).toHaveLength(2);
    expect(screen.getByRole("alert")).toHaveTextContent("2 dòng còn sót chữ Hán:");
  });

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
