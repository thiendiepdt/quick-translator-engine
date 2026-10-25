import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ConverterPage } from "@/components/pages/converter-page";
import { pickFiles, pickFolder, zhConvertFiles, zhConvertText } from "@/lib/api";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({
  pickFiles: vi.fn(),
  pickFolder: vi.fn(),
  revealFolder: vi.fn(() => Promise.resolve()),
  zhConvertFiles: vi.fn(),
  zhConvertText: vi.fn(),
}));
vi.mock("sonner", () => ({ toast: { success: vi.fn(), error: vi.fn() } }));

const report = { converted: 3, unchanged: 1, written: ["D:/ra/a.txt"], skipped: [{ path: "D:/nguon/anh.png", reason: "không phải file text" }] };

describe("ConverterPage", () => {
  beforeEach(() => {
    useStoryStore.setState({ screen: "converter", toolFrom: "picker", root: undefined });
    vi.mocked(zhConvertText).mockReset().mockImplementation((text: string) => Promise.resolve(text.replace("後臺", "后台")));
    vi.mocked(zhConvertFiles).mockReset().mockResolvedValue(report);
    vi.mocked(pickFolder).mockReset();
    vi.mocked(pickFiles).mockReset();
  });

  it("ô cụm từ chuyển ngay khi gõ", async () => {
    render(<ConverterPage />);
    await userEvent.type(screen.getByLabelText("Phồn thể"), "看著後臺");
    await vi.waitFor(() => expect(screen.getByLabelText("Giản thể")).toHaveValue("看著后台"));
    expect(zhConvertText).toHaveBeenLastCalledWith("看著後臺");
  });

  it("chuyển cả folder ra folder khác, lọc đuôi, báo số file", async () => {
    vi.mocked(pickFolder).mockResolvedValueOnce("D:/nguon").mockResolvedValueOnce("D:/ra");
    render(<ConverterPage />);
    await userEvent.click(screen.getByRole("tab", { name: "Folder" }));
    const run = screen.getByRole("button", { name: "Chuyển sang giản thể" });
    expect(run).toBeDisabled();
    await userEvent.click(screen.getByRole("button", { name: "Chọn folder…" }));
    expect(await screen.findByText("D:/nguon")).toBeInTheDocument();
    expect(run).toBeDisabled();
    await userEvent.click(screen.getByRole("button", { name: "Chọn folder đích…" }));
    await userEvent.clear(screen.getByLabelText("Đuôi file"));
    await userEvent.type(screen.getByLabelText("Đuôi file"), ".txt, md");
    await userEvent.click(run);
    expect(zhConvertFiles).toHaveBeenCalledWith({
      inputs: ["D:/nguon"],
      recursive: true,
      extensions: ["txt", "md"],
      mode: { kind: "folder", dir: "D:/ra" },
    });
    expect(
      await screen.findByText("Đã chuyển 3 file sang giản thể; 1 file vốn là giản thể UTF-8 vẫn ghi bản sao. Bỏ qua 1 file."),
    ).toBeInTheDocument();
    expect(screen.getByText("D:/nguon/anh.png")).toBeInTheDocument();
  });

  it("file chọn riêng mặc định thêm hậu tố; ghi đè phải xác nhận", async () => {
    vi.mocked(pickFiles).mockResolvedValue(["D:/a.txt", "D:/b.txt"]);
    const confirm = vi.spyOn(window, "confirm").mockReturnValueOnce(false).mockReturnValueOnce(true);
    render(<ConverterPage />);
    await userEvent.click(screen.getByRole("tab", { name: "File" }));
    await userEvent.click(screen.getByRole("button", { name: "Chọn file…" }));
    await userEvent.click(screen.getByRole("button", { name: "Chuyển sang giản thể" }));
    expect(zhConvertFiles).toHaveBeenLastCalledWith({
      inputs: ["D:/a.txt", "D:/b.txt"],
      recursive: true,
      extensions: [],
      mode: { kind: "suffix", suffix: "_gian" },
    });
    await userEvent.click(screen.getByLabelText(/Ghi đè file gốc/));
    await userEvent.click(screen.getByRole("button", { name: "Chuyển sang giản thể" }));
    expect(zhConvertFiles).toHaveBeenCalledTimes(1);
    await userEvent.click(screen.getByRole("button", { name: "Chuyển sang giản thể" }));
    expect(zhConvertFiles).toHaveBeenLastCalledWith(expect.objectContaining({ mode: { kind: "replace" } }));
    expect(confirm).toHaveBeenCalledTimes(2);
  });

  it("nút quay lại về màn đã mở công cụ", async () => {
    render(<ConverterPage />);
    await userEvent.click(screen.getByRole("button", { name: "Về danh sách truyện" }));
    expect(useStoryStore.getState().screen).toBe("picker");
  });
});
