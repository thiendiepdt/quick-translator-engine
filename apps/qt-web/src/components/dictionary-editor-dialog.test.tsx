import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { DictionaryEditorDialog } from "@/components/dictionary-editor-dialog";
import { dictionaryDefinitions } from "@/lib/types";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

describe("dictionary editor import", () => {
  it("appends clipboard content after the existing records", async () => {
    Object.defineProperty(window.navigator, "clipboard", {
      configurable: true,
      value: { readText: vi.fn().mockResolvedValue("药老=Dược Lão") },
    });
    const onSave = vi.fn();
    render(
      <DictionaryEditorDialog
        open
        onOpenChange={() => undefined}
        definition={dictionaryDefinitions[0]}
        value="萧炎=Tiêu Viêm"
        onSave={onSave}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Dán" }));

    await waitFor(() => {
      expect(onSave).toHaveBeenCalledWith("萧炎=Tiêu Viêm\n药老=Dược Lão");
    });
  });

  it("empties the dictionary with undo restoring the session content", async () => {
    const onSave = vi.fn();
    render(
      <DictionaryEditorDialog
        open
        onOpenChange={() => undefined}
        definition={dictionaryDefinitions[0]}
        value="萧炎=Tiêu Viêm"
        onSave={onSave}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Đặt rỗng" }));

    await waitFor(() => {
      expect(onSave).toHaveBeenCalledWith("");
    });
  });
});
