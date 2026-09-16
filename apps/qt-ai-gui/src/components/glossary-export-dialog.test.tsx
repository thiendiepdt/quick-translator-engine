import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { GlossaryExportDialog } from "@/components/glossary-export-dialog";
import { pickSaveFile, writeTextFile } from "@/lib/api";
import { copyText } from "@/lib/clipboard";
import type { StoryFormValues } from "@/lib/story-form";

vi.mock("@/lib/api", () => ({ pickSaveFile: vi.fn(), writeTextFile: vi.fn() }));
vi.mock("@/lib/clipboard", () => ({ copyText: vi.fn() }));

const glossary = (): StoryFormValues["glossary"] => ({
  names: [
    { source: "赵静文", target: "Triệu Tĩnh Văn" },
    { source: "a=b", target: "x" },
  ],
  places: [{ source: "京城", target: "Kinh Thành" }],
  items: [],
  creatures: [],
  skills: [],
  common: [],
  signature_phrases: [{ source: "天道酬勤", target: "Thiên đạo thù cần" }],
  addressing: [{ source: "甲→乙", target: "ta–ngươi" }],
});

/** Textarea chuẩn hoá CRLF → LF ở `value`; nội dung chép/lưu vẫn CRLF (test riêng bên dưới). */
function preview(): HTMLTextAreaElement {
  return screen.getByLabelText<HTMLTextAreaElement>("Xem trước");
}

describe("GlossaryExportDialog", () => {
  beforeEach(() => {
    vi.mocked(pickSaveFile).mockReset();
    vi.mocked(writeTextFile).mockReset().mockResolvedValue(undefined);
    vi.mocked(copyText).mockReset().mockResolvedValue(undefined);
  });

  it("mặc định tick hết trừ Cụm từ đặc trưng, nhóm gọn hết; addressing không hiện; dòng có = bị khoá kèm lý do", async () => {
    const user = userEvent.setup();
    render(<GlossaryExportDialog open onOpenChange={vi.fn()} glossary={glossary()} />);
    expect(preview().value).toBe("赵静文=Triệu Tĩnh Văn\n京城=Kinh Thành\n");
    expect(screen.getByText(/Sẽ ghi/)).toHaveTextContent("Sẽ ghi 2 dòng · bỏ 1 dòng");
    expect(screen.queryByText("Xưng hô theo cặp")).not.toBeInTheDocument();
    expect(screen.queryByText("甲→乙")).not.toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: "Cụm từ đặc trưng" })).not.toBeChecked();
    // Nhóm gọn sẵn; bung Tên nhân vật mới thấy dòng.
    expect(screen.queryByText("赵静文")).not.toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Tên nhân vật" }));
    const bad = screen.getByRole("checkbox", { name: "a=b = x" });
    expect(bad).toBeDisabled();
    expect(screen.getByText("có dấu =")).toBeInTheDocument();
  });

  it("tick nhóm và tick dòng đổi xem trước; bỏ hết thì khoá nút", async () => {
    const user = userEvent.setup();
    render(<GlossaryExportDialog open onOpenChange={vi.fn()} glossary={glossary()} />);
    await user.click(screen.getByRole("checkbox", { name: "Cụm từ đặc trưng" }));
    expect(preview().value).toContain("天道酬勤=Thiên đạo thù cần\n");
    await user.click(screen.getByRole("button", { name: "Tên nhân vật" }));
    await user.click(screen.getByRole("checkbox", { name: "赵静文 = Triệu Tĩnh Văn" }));
    expect(preview().value).not.toContain("赵静文");
    const names = screen.getByRole<HTMLInputElement>("checkbox", { name: "Tên nhân vật" });
    expect(names.indeterminate).toBe(false);
    expect(names).not.toBeChecked();
    await user.click(screen.getByRole("checkbox", { name: "Địa danh" }));
    await user.click(screen.getByRole("checkbox", { name: "Cụm từ đặc trưng" }));
    expect(preview().value).toBe("");
    expect(screen.getByRole("button", { name: /Chép/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Lưu/ })).toBeDisabled();
  });

  it("tìm nhanh hiện dòng khớp dù nhóm đang gọn, chỉ lọc hiển thị, không đổi tick", async () => {
    const user = userEvent.setup();
    render(<GlossaryExportDialog open onOpenChange={vi.fn()} glossary={glossary()} />);
    expect(screen.queryByText("京城")).not.toBeInTheDocument();
    await user.type(screen.getByPlaceholderText("Tìm Hán hoặc Việt…"), "Kinh");
    expect(screen.queryByText("赵静文")).not.toBeInTheDocument();
    expect(screen.getByText("京城")).toBeInTheDocument();
    expect(preview().value).toContain("赵静文=Triệu Tĩnh Văn");
  });

  it("Chép đưa nội dung không BOM vào clipboard", async () => {
    const user = userEvent.setup();
    render(<GlossaryExportDialog open onOpenChange={vi.fn()} glossary={glossary()} />);
    await user.click(screen.getByRole("button", { name: /Chép/ }));
    await waitFor(() => expect(copyText).toHaveBeenCalledWith("赵静文=Triệu Tĩnh Văn\r\n京城=Kinh Thành\r\n"));
  });

  it("Lưu… mở hộp thoại với tên đã chọn rồi ghi file có BOM; huỷ hộp thoại thì không ghi", async () => {
    const user = userEvent.setup();
    vi.mocked(pickSaveFile).mockResolvedValueOnce(undefined).mockResolvedValueOnce("D:\\qt\\Names2.txt");
    render(<GlossaryExportDialog open onOpenChange={vi.fn()} glossary={glossary()} />);
    await user.click(screen.getByRole("button", { name: /Lưu/ }));
    await waitFor(() => expect(pickSaveFile).toHaveBeenCalledWith("Names.txt", "Lưu Names.txt"));
    expect(writeTextFile).not.toHaveBeenCalled();
    await user.click(
      within(screen.getByRole("radiogroup", { name: "Tên file" })).getByRole("radio", { name: "Names2.txt" }),
    );
    await user.click(screen.getByRole("button", { name: /Lưu/ }));
    await waitFor(() => expect(pickSaveFile).toHaveBeenLastCalledWith("Names2.txt", "Lưu Names2.txt"));
    await waitFor(() =>
      expect(writeTextFile).toHaveBeenCalledWith(
        "D:\\qt\\Names2.txt",
        "\ufeff赵静文=Triệu Tĩnh Văn\r\n京城=Kinh Thành\r\n",
      ),
    );
    expect(await screen.findByText(/Đã ghi/)).toHaveTextContent("D:\\qt\\Names2.txt");
  });
});
