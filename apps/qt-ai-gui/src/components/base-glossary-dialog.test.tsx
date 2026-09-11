import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { BaseGlossaryDialog } from "@/components/base-glossary-dialog";
import { baseGet, baseReset, baseSave } from "@/lib/api";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ baseGet: vi.fn(), baseSave: vi.fn(), baseReset: vi.fn() }));

const empty = { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {}, addressing: {} };
const view = (setting: string, source: "builtin" | "file", glossary: Record<string, Record<string, string>>) =>
  ({ kind: "glossary", setting, source, glossary: { ...empty, ...glossary } }) as never;

function last<T>(items: T[]): T {
  const item = items.at(-1);
  if (item === undefined) throw new Error("danh sách rỗng");
  return item;
}

describe("BaseGlossaryDialog", () => {
  beforeEach(() => {
    vi.mocked(baseGet).mockReset();
    vi.mocked(baseSave).mockReset();
    vi.mocked(baseReset).mockReset();
    useStoryStore.setState({ baseVersion: 0 });
  });

  it("nạp kho cổ đại, thêm mục ở Tên nhân vật rồi Lưu gửi glossary đủ 8 nhóm", async () => {
    const user = userEvent.setup();
    vi.mocked(baseGet).mockResolvedValue(view("ancient", "builtin", { names: { 赵: "Triệu" } }));
    vi.mocked(baseSave).mockResolvedValue(view("ancient", "file", { names: { 赵: "Triệu", 钱: "Tiền" } }));
    render(<BaseGlossaryDialog open onOpenChange={() => undefined} />);
    expect(await screen.findByDisplayValue("Triệu")).toBeInTheDocument();
    expect(baseGet).toHaveBeenCalledWith("glossary", "ancient", undefined);
    expect(screen.getByRole("button", { name: "Lưu" })).toBeDisabled();

    const [addFirst] = screen.getAllByRole("button", { name: /Thêm/ });
    if (!addFirst) throw new Error("thiếu nút Thêm");
    await user.click(addFirst);
    await user.type(last(screen.getAllByPlaceholderText("Hán tự")), "钱");
    await user.type(last(screen.getAllByPlaceholderText("Tiếng Việt")), "Tiền");
    await user.click(screen.getByRole("button", { name: "Lưu" }));
    await waitFor(() =>
      expect(baseSave).toHaveBeenCalledWith("glossary", "ancient", undefined, {
        glossary: { ...empty, names: { 赵: "Triệu", 钱: "Tiền" } },
      }),
    );
    expect(await screen.findByText("đã sửa")).toBeInTheDocument();
    expect(useStoryStore.getState().baseVersion).toBe(1);
  });

  it("Về mặc định gọi base_reset cho bối cảnh đang chọn", async () => {
    const user = userEvent.setup();
    vi.mocked(baseGet).mockResolvedValue(view("ancient", "file", { names: { 赵: "Triệu" } }));
    vi.mocked(baseReset).mockResolvedValue(view("ancient", "builtin", {}));
    render(<BaseGlossaryDialog open onOpenChange={() => undefined} />);
    await screen.findByText("đã sửa");
    await user.click(screen.getByRole("button", { name: "Về mặc định" }));
    await waitFor(() => expect(baseReset).toHaveBeenCalledWith("glossary", "ancient", undefined));
    expect(await screen.findByText("bản cứng")).toBeInTheDocument();
  });
});
