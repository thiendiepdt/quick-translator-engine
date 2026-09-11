import { render, screen, within } from "@testing-library/react";
import { MarkdownPlugin } from "@platejs/markdown";
import { createPlateEditor } from "platejs/react";
import { describe, expect, it, vi } from "vitest";

import { PlatePromptEditor } from "@/components/plate-prompt-editor";
import { normalizeMarkdown, promptEditorPlugins } from "@/components/plate-prompt-editor-plugins";

describe("Plate prompt editor", () => {
  it("renders Markdown as an inline WYSIWYG document", () => {
    render(
      <PlatePromptEditor
        initialValue={"# Prompt riêng\n\nNội dung **quan trọng**.\n\n- Quy tắc một"}
        onChange={vi.fn()}
      />,
    );

    const editor = screen.getByRole("textbox", { name: "Prompt dịch thuật" });
    expect(editor).toHaveAttribute("contenteditable", "true");
    expect(within(editor).getByRole("heading", { name: "Prompt riêng" })).toBeVisible();
    expect(within(editor).getByText("quan trọng").closest("strong")).not.toBeNull();
    expect(within(editor).getByText("Quy tắc một")).toBeVisible();
  });

  it("renders GFM tables as real table elements", () => {
    render(
      <PlatePromptEditor
        initialValue={"| Tiếng Trung | Dùng |\n| --- | --- |\n| 他 | **hắn** |"}
        onChange={vi.fn()}
      />,
    );

    const editor = screen.getByRole("textbox", { name: "Prompt dịch thuật" });
    expect(within(editor).getByRole("table")).toBeVisible();
    expect(within(editor).getByRole("columnheader", { name: "Tiếng Trung" })).toBeVisible();
    expect(within(editor).getByRole("cell", { name: "hắn" })).toBeVisible();
    expect(editor.textContent).not.toContain("|");
  });

  it("round-trips GFM tables back to Markdown pipes", () => {
    const markdown = "| CN | VN |\n| --- | --- |\n| 他 | hắn |";
    const editor = createPlateEditor({
      plugins: promptEditorPlugins,
      value: (currentEditor) => currentEditor
        .getApi(MarkdownPlugin)
        .markdown.deserialize(markdown),
    });

    expect(editor.children.some((node) => node.type === "table")).toBe(true);

    const serialized = editor.getApi(MarkdownPlugin).markdown.serialize();
    expect(serialized).toMatch(/\|\s*他\s*\|\s*hắn\s*\|/);
    expect(serialized).toMatch(/\| -+ \| -+ \|/);
  });

  it("serializes edits back to Markdown", () => {
    const editor = createPlateEditor({
      plugins: promptEditorPlugins,
      value: (currentEditor) => currentEditor
        .getApi(MarkdownPlugin)
        .markdown.deserialize("Prompt gốc"),
    });

    editor.tf.insertText(" bổ sung", {
      at: { offset: "Prompt gốc".length, path: [0, 0] },
    });

    expect(editor.getApi(MarkdownPlugin).markdown.serialize()).toBe("Prompt gốc bổ sung\n");
  });

  it("normalizeMarkdown làm prompt gốc và markdown editor xuất ra so được với nhau", () => {
    const raw = "# Tiêu đề\n\n* mục một\n* mục hai\n\n\nĐoạn.";
    const normalized = normalizeMarkdown(raw);
    expect(normalized).toBe(normalizeMarkdown(normalized));
    expect(normalized).toContain("mục một");
    expect(normalizeMarkdown("Khác")).not.toBe(normalized);
  });
});
