import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { BasePromptDialog } from "@/components/base-prompt-dialog";
import { baseGet, baseReset, baseSave } from "@/lib/api";
import { useStoryStore } from "@/store/story";

vi.mock("@/lib/api", () => ({ baseGet: vi.fn(), baseSave: vi.fn(), baseReset: vi.fn() }));
// Plate nặng và không cần cho test: editor giả là textarea gọi onChange.
vi.mock("@/components/plate-prompt-editor", () => ({
  PlatePromptEditor: ({ initialValue, onChange }: { initialValue: string; onChange: (v: string) => void }) => (
    <textarea aria-label="Editor" defaultValue={initialValue} onChange={(e) => onChange(e.target.value)} />
  ),
}));

const view = (setting: string, names: string, source: "builtin" | "file", text: string) =>
  ({ kind: "prompt", setting, names, source, text }) as never;

describe("BasePromptDialog", () => {
  beforeEach(() => {
    vi.mocked(baseGet).mockReset();
    vi.mocked(baseSave).mockReset();
    vi.mocked(baseReset).mockReset();
    useStoryStore.setState({ baseVersion: 0 });
  });

  it("nạp genre mặc định, sửa rồi Lưu gọi base_save, nhãn đổi sang đã sửa, baseVersion tăng", async () => {
    const user = userEvent.setup();
    vi.mocked(baseGet).mockResolvedValue(view("ancient", "han", "builtin", "# gốc"));
    vi.mocked(baseSave).mockResolvedValue(view("ancient", "han", "file", "# gốc!"));
    render(<BasePromptDialog open onOpenChange={() => undefined} />);
    expect(await screen.findByText("bản cứng")).toBeInTheDocument();
    expect(baseGet).toHaveBeenCalledWith("prompt", "ancient", "han");
    expect(screen.getByRole("button", { name: "Lưu" })).toBeDisabled();

    await user.type(await screen.findByLabelText("Editor"), "!");
    await user.click(screen.getByRole("button", { name: "Lưu" }));
    await waitFor(() => expect(baseSave).toHaveBeenCalledWith("prompt", "ancient", "han", { text: "# gốc!" }));
    expect(await screen.findByText("đã sửa")).toBeInTheDocument();
    expect(useStoryStore.getState().baseVersion).toBe(1);
  });

  it("đổi genre nạp lại; Về mặc định gọi base_reset", async () => {
    const user = userEvent.setup();
    vi.mocked(baseGet)
      .mockResolvedValueOnce(view("ancient", "han", "builtin", "# gốc"))
      .mockResolvedValueOnce(view("modern", "han", "file", "# hiện đại"));
    vi.mocked(baseReset).mockResolvedValue(view("modern", "han", "builtin", "# cứng"));
    render(<BasePromptDialog open onOpenChange={() => undefined} />);
    await screen.findByText("bản cứng");
    await user.click(screen.getByRole("radio", { name: "Hiện đại" }));
    expect(await screen.findByText("đã sửa")).toBeInTheDocument();
    expect(baseGet).toHaveBeenLastCalledWith("prompt", "modern", "han");
    await user.click(screen.getByRole("button", { name: "Về mặc định" }));
    await waitFor(() => expect(baseReset).toHaveBeenCalledWith("prompt", "modern", "han"));
    expect(await screen.findByText("bản cứng")).toBeInTheDocument();
    expect(useStoryStore.getState().baseVersion).toBe(1);
  });
});
