import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { BaseRulesDialog } from "@/components/base-rules-dialog";
import { baseGet, baseReset, baseSave } from "@/lib/api";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ baseGet: vi.fn(), baseSave: vi.fn(), baseReset: vi.fn() }));

const view = (setting: string, source: "builtin" | "file", rules: { pattern: string; flags?: string; message: string }[]) =>
  ({ kind: "rules", setting, source, rules }) as never;

describe("BaseRulesDialog", () => {
  beforeEach(() => {
    vi.mocked(baseGet).mockReset();
    vi.mocked(baseSave).mockReset();
    vi.mocked(baseReset).mockReset();
    useStoryStore.setState({ baseVersion: 0 });
  });

  it("nạp bộ cổ đại, sửa dòng và Lưu gửi rules đã chuẩn hoá (flags trống → bỏ), bump version", async () => {
    const user = userEvent.setup();
    vi.mocked(baseGet).mockResolvedValue(view("ancient", "builtin", [{ pattern: "a", flags: "i", message: "m" }]));
    vi.mocked(baseSave).mockResolvedValue(view("ancient", "file", [{ pattern: "a", flags: "i", message: "m!" }]));
    render(<BaseRulesDialog open onOpenChange={() => undefined} />);
    expect(await screen.findByLabelText("Mô tả rule 1")).toHaveValue("m");
    expect(baseGet).toHaveBeenCalledWith("rules", "ancient", undefined);
    expect(screen.getByRole("button", { name: "Lưu" })).toBeDisabled();
    await user.type(screen.getByLabelText("Mô tả rule 1"), "!");
    await user.click(screen.getByRole("button", { name: "Thêm" }));
    await user.type(screen.getByLabelText("Regex 2"), "b");
    await user.type(screen.getByLabelText("Mô tả rule 2"), "n");
    await user.click(screen.getByRole("button", { name: "Lưu" }));
    await waitFor(() =>
      expect(baseSave).toHaveBeenCalledWith("rules", "ancient", undefined, {
        rules: [
          { pattern: "a", flags: "i", message: "m!" },
          { pattern: "b", message: "n" },
        ],
      }),
    );
    expect(await screen.findByText(/đã sửa/)).toBeInTheDocument();
    expect(useStoryStore.getState().baseVersion).toBe(1);
  });

  it("đổi bối cảnh nạp lại; Về mặc định gọi base_reset", async () => {
    const user = userEvent.setup();
    vi.mocked(baseGet)
      .mockResolvedValueOnce(view("ancient", "builtin", []))
      .mockResolvedValueOnce(view("modern", "file", [{ pattern: "x", message: "y" }]));
    vi.mocked(baseReset).mockResolvedValue(view("modern", "builtin", []));
    render(<BaseRulesDialog open onOpenChange={() => undefined} />);
    await screen.findByText(/bản cứng/);
    await user.click(screen.getByRole("radio", { name: "Hiện đại" }));
    expect(await screen.findByLabelText("Regex 1")).toHaveValue("x");
    await user.click(screen.getByRole("button", { name: "Về mặc định" }));
    await waitFor(() => expect(baseReset).toHaveBeenCalledWith("rules", "modern", undefined));
    expect(await screen.findByText(/bản cứng/)).toBeInTheDocument();
  });
});
