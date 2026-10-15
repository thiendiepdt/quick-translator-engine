import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ExportPage } from "@/components/pages/export-page";
import {
  bookInfoGet,
  bookInfoSave,
  exportChapters,
  exportEpub,
  hanVietName,
  pickSaveFile,
  sourceFetchImage,
  sourceFetchText,
} from "@/lib/api";
import { imageToJpeg, renderDefaultCover } from "@/lib/epub-cover";
import type { ChapterRow } from "@/lib/types";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  exportChapters: vi.fn(),
  exportEpub: vi.fn(),
  pickSaveFile: vi.fn(),
  revealFolder: vi.fn(),
  bookInfoGet: vi.fn(),
  bookInfoSave: vi.fn(),
  hanVietName: vi.fn(),
  sourceFetchText: vi.fn(),
  sourceFetchImage: vi.fn(),
}));
// Canvas không có trong jsdom: thay phần vẽ, giữ splitStoryName thật.
vi.mock("@/lib/epub-cover", async (importOriginal) => ({
  ...(await importOriginal<typeof import("@/lib/epub-cover")>()),
  imageToJpeg: vi.fn(),
  renderDefaultCover: vi.fn(),
}));

const row = (id: string, status: ChapterRow["status"]): ChapterRow => ({ id, status, reviewRound: 0, reason: null, warnings: [] });
const chapters = [row("c-0001", "queued"), row("c-0002", "done"), row("c-0003", "skipped"), row("c-0004", "done"), row("c-0005", "queued")];

describe("ExportPage", () => {
  beforeEach(() => {
    vi.mocked(exportChapters).mockReset().mockResolvedValue({ outPath: "D:\\t\\export\\a.txt", ids: ["c-0002", "c-0004"], gaps: ["c-0003"] });
    vi.mocked(pickSaveFile).mockReset();
    useStoryStore.setState({
      root: "D:\\t",
      snapshot: { root: "D:\\t", chapters, counts: {}, settings: {}, story: { name: "" }, sessionRunning: false } as never,
    });
  });

  it("điền sẵn số thứ tự chương done đầu/cuối, có gợi ý mã, export gửi mã chương", async () => {
    const user = userEvent.setup();
    render(<ExportPage />);
    expect(screen.getByLabelText("Từ chương")).toHaveValue("2");
    expect(screen.getByLabelText("Đến chương")).toHaveValue("4");
    expect(screen.getByText("→ c-0002")).toBeInTheDocument();
    expect(screen.getByText("→ c-0004")).toBeInTheDocument();
    expect(screen.getByText(/Sẽ gộp/)).toHaveTextContent("2 chương done");
    expect(screen.getByText(/Hổng 1 chương/)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /Export vào export\// }));
    await waitFor(() => expect(exportChapters).toHaveBeenCalledWith("D:\\t", { from: "c-0002", to: "c-0004", out: undefined }));
    expect(await screen.findByText(/Đã ghi/)).toBeInTheDocument();
  });

  it("bấm Từ/Đến trong danh sách đổi ô nhập và tô khoảng", async () => {
    const user = userEvent.setup();
    render(<ExportPage />);
    await user.click(screen.getByRole("button", { name: "Từ chương #1" }));
    await user.click(screen.getByRole("button", { name: "Đến chương #5" }));
    expect(screen.getByLabelText("Từ chương")).toHaveValue("1");
    expect(screen.getByLabelText("Đến chương")).toHaveValue("5");
    expect(screen.getByText(/Sẽ gộp/)).toHaveTextContent("2 chương done");
    expect(screen.getAllByRole("listitem").every((li) => li.getAttribute("data-in-range") === "true")).toBe(true);
  });

  it("số không tồn tại thì báo và khoá nút; khoảng ngược cũng khoá", async () => {
    const user = userEvent.setup();
    render(<ExportPage />);
    const to = screen.getByLabelText("Đến chương");
    await user.clear(to);
    await user.type(to, "9");
    expect(screen.getByText("không có chương này")).toBeInTheDocument();
    expect(screen.getByText(/không hợp lệ/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Export vào export\// })).toBeDisabled();
    await user.clear(to);
    await user.type(to, "1");
    expect(screen.getByText(/không hợp lệ/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Export vào export\// })).toBeDisabled();
  });

  it("Chọn nơi lưu gợi ý tên file theo mã chương", async () => {
    const user = userEvent.setup();
    vi.mocked(pickSaveFile).mockResolvedValue("D:\\out\\x.txt");
    render(<ExportPage />);
    await user.click(screen.getByRole("button", { name: /Chọn nơi lưu/ }));
    await waitFor(() => expect(pickSaveFile).toHaveBeenCalledWith("c-0002-c-0004.txt"));
    await waitFor(() => expect(exportChapters).toHaveBeenCalledWith("D:\\t", { from: "c-0002", to: "c-0004", out: "D:\\out\\x.txt" }));
  });

  describe("EPUB", () => {
    const JPEG = Uint8Array.from([0xff, 0xd8, 0xff, 1]);
    const DEFAULT_JPEG = Uint8Array.from([0xff, 0xd8, 0xff, 2]);
    const emptyMeta = { author: "", authorOriginal: "", source: "", coverUrl: "", fetchedAt: 0 };
    const QQ_BODY = JSON.stringify({
      introinfo: { book: { title: "方寸道主", author: "愤怒的乌贼", coverUrl: "https://c.example.com/a.jpg" } },
    });
    const QQ_CDN = "https://bookcover.reader.qq.com/cover/836/58257836/t9_58257836.webp";

    function epubStory(sourceUrl: string) {
      useStoryStore.setState({
        root: "D:\\t",
        snapshot: {
          root: "D:\\t",
          chapters,
          counts: {},
          settings: {},
          story: { name: "Phương Thốn Đạo Chủ (方寸道主)", sourceUrl },
          sessionRunning: false,
        } as never,
      });
    }

    beforeEach(() => {
      URL.createObjectURL = vi.fn(() => "blob:cover");
      URL.revokeObjectURL = vi.fn();
      vi.mocked(bookInfoGet).mockReset().mockResolvedValue({ meta: emptyMeta, cover: undefined });
      vi.mocked(bookInfoSave).mockReset().mockResolvedValue(undefined);
      vi.mocked(hanVietName).mockReset().mockImplementation((text) => Promise.resolve(text === "愤怒的乌贼" ? "Phẫn Nộ Đích Ô Tặc" : text));
      vi.mocked(sourceFetchText).mockReset();
      vi.mocked(sourceFetchImage).mockReset();
      vi.mocked(imageToJpeg).mockReset().mockResolvedValue(JPEG);
      vi.mocked(renderDefaultCover).mockReset().mockResolvedValue(DEFAULT_JPEG);
      vi.mocked(exportEpub).mockReset().mockResolvedValue({ outPath: "D:\\t\\export\\a.epub", ids: ["c-0002", "c-0004"], gaps: [] });
    });

    it("lần đầu tự lấy tác giả (đổi Hán-Việt) + ảnh bìa QQ, lưu cover.jpg, export gửi tác giả và bìa", async () => {
      const user = userEvent.setup();
      epubStory("https://book.qq.com/book-detail/58257836");
      vi.mocked(sourceFetchText).mockResolvedValue(QQ_BODY);
      vi.mocked(sourceFetchImage).mockRejectedValueOnce(new Error("HTTP 404")).mockResolvedValue({ bytes: JPEG, mime: "image/webp" });
      render(<ExportPage />);
      await user.click(screen.getByRole("tab", { name: "EPUB" }));

      expect(await screen.findByText("Ảnh bìa lấy từ QQ đọc sách.")).toBeInTheDocument();
      expect(sourceFetchText).toHaveBeenCalledWith(
        "https://detailadr.reader.qq.com/book/queryDetailPage?bid=58257836&sex=1&qmk=1,3,7,9,10",
      );
      // Link đầu lỗi → thử link dựng từ bid.
      expect(sourceFetchImage).toHaveBeenLastCalledWith(QQ_CDN);
      expect(bookInfoSave).toHaveBeenCalledWith(
        "D:\\t",
        expect.objectContaining({ author: "Phẫn Nộ Đích Ô Tặc", authorOriginal: "愤怒的乌贼", source: "qq", coverUrl: QQ_CDN }),
        JPEG,
        false,
      );
      expect(screen.getByLabelText("Tác giả")).toHaveValue("Phẫn Nộ Đích Ô Tặc");
      expect(screen.getByRole("img", { name: "Ảnh bìa" })).toHaveAttribute("src", "blob:cover");

      await user.click(screen.getByRole("button", { name: /Tạo EPUB vào export\// }));
      await waitFor(() =>
        expect(exportEpub).toHaveBeenCalledWith(
          "D:\\t",
          { from: "c-0002", to: "c-0004", out: undefined },
          { author: "Phẫn Nộ Đích Ô Tặc", cover: JPEG },
        ),
      );
    });

    it("link không hỗ trợ: không gọi mạng, dùng bìa mặc định có chữ khi export", async () => {
      const user = userEvent.setup();
      epubStory("https://www.piaotia.com/html/7/7028/");
      render(<ExportPage />);
      await user.click(screen.getByRole("tab", { name: "EPUB" }));
      expect(await screen.findByText(/Link nguồn không hỗ trợ/)).toBeInTheDocument();
      expect(sourceFetchText).not.toHaveBeenCalled();
      expect(bookInfoSave).toHaveBeenCalledWith("D:\\t", expect.objectContaining({ source: "" }), undefined, true);
      await user.type(screen.getByLabelText("Tác giả"), "Tác Giả");
      await user.click(screen.getByRole("button", { name: /Tạo EPUB vào export\// }));
      await waitFor(() => expect(exportEpub).toHaveBeenCalled());
      expect(renderDefaultCover).toHaveBeenLastCalledWith({
        title: "Phương Thốn Đạo Chủ",
        originalTitle: "方寸道主",
        author: "Tác Giả",
      });
      expect(vi.mocked(exportEpub).mock.calls[0][2]).toEqual({ author: "Tác Giả", cover: DEFAULT_JPEG });
    });

    it("tải lỗi thì báo lỗi, bấm Tải lại ảnh bìa thì thử lại", async () => {
      const user = userEvent.setup();
      epubStory("https://book.qq.com/book-detail/58257836");
      vi.mocked(sourceFetchText).mockRejectedValueOnce(new Error("mất mạng")).mockResolvedValue(QQ_BODY);
      vi.mocked(sourceFetchImage).mockResolvedValue({ bytes: JPEG, mime: "image/jpeg" });
      render(<ExportPage />);
      await user.click(screen.getByRole("tab", { name: "EPUB" }));
      expect(await screen.findByText(/Lỗi: mất mạng/)).toBeInTheDocument();
      await user.click(screen.getByRole("button", { name: /Tải lại ảnh bìa/ }));
      expect(await screen.findByText("Ảnh bìa lấy từ QQ đọc sách.")).toBeInTheDocument();
      expect(sourceFetchText).toHaveBeenCalledTimes(2);
    });

    it("đã có cover.jpg thì hiện luôn, không gọi nguồn", async () => {
      const user = userEvent.setup();
      epubStory("https://book.qq.com/book-detail/58257836");
      vi.mocked(bookInfoGet).mockResolvedValue({ meta: { ...emptyMeta, author: "A", source: "qq", fetchedAt: 5 }, cover: JPEG });
      render(<ExportPage />);
      await user.click(screen.getByRole("tab", { name: "EPUB" }));
      expect(await screen.findByText("Ảnh bìa lấy từ QQ đọc sách.")).toBeInTheDocument();
      expect(sourceFetchText).not.toHaveBeenCalled();
      expect(screen.getByLabelText("Tác giả")).toHaveValue("A");
    });

    it("book.json cũ còn tác giả chữ Hán thì đổi sang Hán-Việt lúc mở, giữ tên gốc", async () => {
      const user = userEvent.setup();
      epubStory("https://book.qq.com/book-detail/58257836");
      const saved = { ...emptyMeta, author: "愤怒的乌贼", source: "qq", fetchedAt: 5 };
      vi.mocked(bookInfoGet).mockResolvedValue({ meta: saved, cover: JPEG });
      render(<ExportPage />);
      await user.click(screen.getByRole("tab", { name: "EPUB" }));
      await waitFor(() => expect(screen.getByLabelText("Tác giả")).toHaveValue("Phẫn Nộ Đích Ô Tặc"));
      expect(bookInfoSave).toHaveBeenCalledWith(
        "D:\\t",
        { ...saved, author: "Phẫn Nộ Đích Ô Tặc", authorOriginal: "愤怒的乌贼" },
        undefined,
        false,
      );
      expect(sourceFetchText).not.toHaveBeenCalled();
    });

    it("Chọn nơi lưu gợi ý tên truyện + khoảng, lọc .epub", async () => {
      const user = userEvent.setup();
      epubStory("");
      vi.mocked(pickSaveFile).mockResolvedValue("D:\\out\\x.epub");
      render(<ExportPage />);
      await user.click(screen.getByRole("tab", { name: "EPUB" }));
      await screen.findByText(/Link nguồn không hỗ trợ/);
      await user.click(screen.getByRole("button", { name: /Chọn nơi lưu/ }));
      await waitFor(() =>
        expect(pickSaveFile).toHaveBeenCalledWith("Phương Thốn Đạo Chủ_c-0002-c-0004.epub", "Lưu file EPUB", {
          name: "Sách EPUB",
          extensions: ["epub"],
        }),
      );
      await waitFor(() =>
        expect(exportEpub).toHaveBeenCalledWith(
          "D:\\t",
          { from: "c-0002", to: "c-0004", out: "D:\\out\\x.epub" },
          expect.anything(),
        ),
      );
    });
  });
});
