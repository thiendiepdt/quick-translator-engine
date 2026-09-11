import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ExportPage } from "@/components/pages/export-page";
import { exportChapters, pickSaveFile } from "@/lib/api";
import type { ChapterRow } from "@/lib/types";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ exportChapters: vi.fn(), pickSaveFile: vi.fn(), revealFolder: vi.fn() }));

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
});
