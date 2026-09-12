import { act, fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MarkdownPlugin } from "@platejs/markdown";
import { createPlateEditor, type PlateEditor } from "platejs/react";
import { describe, expect, it, vi } from "vitest";

import { PlatePromptEditor, SERIALIZE_DEBOUNCE_MS } from "@/components/plate-prompt-editor";
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

describe("Plate prompt editor · serialize trễ và công cụ bảng", () => {
  it("gõ không gọi onChange ngay; ngừng 300ms mới serialize một lần; blur thì flush ngay", async () => {
    vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout"] });
    try {
      const onChange = vi.fn();
      let editor: PlateEditor | undefined;
      render(<PlatePromptEditor initialValue="Prompt gốc" onChange={onChange} onEditor={(e) => (editor = e)} />);
      expect(editor).toBeDefined();
      // Slate báo onChange qua microtask → act async để Plate nhận thay đổi.
      await act(async () => {
        editor?.tf.insertText(" a", { at: { path: [0, 0], offset: "Prompt gốc".length } });
        await Promise.resolve();
      });
      await act(async () => {
        editor?.tf.insertText("b", { at: { path: [0, 0], offset: "Prompt gốc a".length } });
        await Promise.resolve();
      });
      expect(onChange).not.toHaveBeenCalled();
      void act(() => vi.advanceTimersByTime(SERIALIZE_DEBOUNCE_MS - 1));
      expect(onChange).not.toHaveBeenCalled();
      void act(() => vi.advanceTimersByTime(1));
      expect(onChange).toHaveBeenCalledTimes(1);
      expect(onChange).toHaveBeenLastCalledWith("Prompt gốc ab\n");

      await act(async () => {
        editor?.tf.insertText("c", { at: { path: [0, 0], offset: "Prompt gốc ab".length } });
        await Promise.resolve();
      });
      fireEvent.blur(screen.getByRole("textbox", { name: "Prompt dịch thuật" }));
      expect(onChange).toHaveBeenCalledTimes(2);
      expect(onChange).toHaveBeenLastCalledWith("Prompt gốc abc\n");
      // Blur đã flush → hết hạn debounce không gọi thêm.
      void act(() => vi.advanceTimersByTime(SERIALIZE_DEBOUNCE_MS));
      expect(onChange).toHaveBeenCalledTimes(2);
    } finally {
      vi.useRealTimers();
    }
  });

  it("thanh công cụ markdown: tiêu đề, danh sách, trích dẫn, gạch ngang, đậm — ra đúng markdown", async () => {
    const user = userEvent.setup();
    let editor: PlateEditor | undefined;
    render(<PlatePromptEditor initialValue={"Dòng một\n\nDòng hai"} onChange={vi.fn()} onEditor={(e) => (editor = e)} />);
    const box = screen.getByRole("textbox", { name: "Prompt dịch thuật" });
    const toolbar = screen.getByRole("toolbar", { name: "Công cụ prompt" });
    const markdown = () => editor?.getApi(MarkdownPlugin).markdown.serialize() ?? "";
    await act(async () => {
      editor?.tf.select({ path: [0, 0], offset: 0 });
      await Promise.resolve();
    });
    await user.click(within(toolbar).getByRole("button", { name: "Tiêu đề 2" }));
    expect(within(box).getByRole("heading", { level: 2, name: "Dòng một" })).toBeInTheDocument();
    expect(within(toolbar).getByRole("button", { name: "Tiêu đề 2" })).toHaveAttribute("aria-pressed", "true");
    expect(markdown()).toMatch(/^## Dòng một/);
    // Bấm lại → về đoạn văn.
    await user.click(within(toolbar).getByRole("button", { name: "Tiêu đề 2" }));
    expect(within(box).queryByRole("heading")).not.toBeInTheDocument();

    await user.click(within(toolbar).getByRole("button", { name: "Danh sách gạch đầu dòng" }));
    expect(markdown()).toMatch(/^[-*] Dòng một/);
    expect(within(toolbar).getByRole("button", { name: "Danh sách gạch đầu dòng" })).toHaveAttribute("aria-pressed", "true");
    await user.click(within(toolbar).getByRole("button", { name: "Danh sách đánh số" }));
    expect(markdown()).toMatch(/^1\. Dòng một/);
    await user.click(within(toolbar).getByRole("button", { name: "Danh sách đánh số" }));
    expect(markdown()).toMatch(/^Dòng một/);

    await user.click(within(toolbar).getByRole("button", { name: "Trích dẫn" }));
    expect(markdown()).toMatch(/^> Dòng một\n\nDòng hai/);
    expect(within(toolbar).getByRole("button", { name: "Trích dẫn" })).toHaveAttribute("aria-pressed", "true");
    await user.click(within(toolbar).getByRole("button", { name: "Trích dẫn" }));
    expect(markdown()).toMatch(/^Dòng một\n\nDòng hai/);

    // Con trỏ thu gọn: đậm bật cho chữ gõ tiếp theo, nút sáng (bôi đen mở toolbar nổi, jsdom không đo được Range).
    await user.click(within(toolbar).getByRole("button", { name: "In đậm" }));
    expect(within(toolbar).getByRole("button", { name: "In đậm" })).toHaveAttribute("aria-pressed", "true");
    await user.click(within(toolbar).getByRole("button", { name: "In đậm" }));
    expect(within(toolbar).getByRole("button", { name: "In đậm" })).toHaveAttribute("aria-pressed", "false");

    // Gạch ngang chèn ngay sau khối hiện tại, con trỏ nhảy xuống khối kế; không đẻ đoạn rỗng (ZWSP) thừa.
    await user.click(within(toolbar).getByRole("button", { name: "Gạch ngang" }));
    expect(markdown()).toBe("Dòng một\n\n***\n\nDòng hai\n");
    expect(editor?.selection?.anchor.path).toEqual([2, 0]);
  });

  it("thanh công cụ: chèn bảng luôn có; trong bảng mới hiện thêm/xoá dòng, cột, bảng", async () => {
    const user = userEvent.setup();
    let editor: PlateEditor | undefined;
    render(
      <PlatePromptEditor
        initialValue={"Mở đầu\n\n| A | B |\n| --- | --- |\n| 1 | 2 |"}
        onChange={vi.fn()}
        onEditor={(e) => (editor = e)}
      />,
    );
    const box = screen.getByRole("textbox", { name: "Prompt dịch thuật" });
    const toolbar = screen.getByRole("toolbar", { name: "Công cụ prompt" });
    expect(within(toolbar).getByRole("button", { name: "Chèn bảng" })).toBeInTheDocument();
    expect(within(toolbar).queryByRole("button", { name: "Xoá bảng" })).not.toBeInTheDocument();

    // Con trỏ vào ô "1" (bảng ở block 1, dòng 1, ô 0) → công cụ bảng hiện.
    await act(async () => {
      editor?.tf.select({ path: [1, 1, 0, 0, 0], offset: 0 });
      await Promise.resolve();
    });
    expect(await within(toolbar).findByRole("button", { name: "Xoá bảng" })).toBeInTheDocument();
    expect(within(box).getAllByRole("row")).toHaveLength(2);
    await user.click(within(toolbar).getByRole("button", { name: "Thêm dòng dưới" }));
    expect(within(box).getAllByRole("row")).toHaveLength(3);
    const firstRowCells = () => within(box).getAllByRole("row")[0].querySelectorAll("td, th").length;
    await user.click(within(toolbar).getByRole("button", { name: "Thêm cột phải" }));
    expect(firstRowCells()).toBe(3);
    await user.click(within(toolbar).getByRole("button", { name: "Xoá cột" }));
    expect(firstRowCells()).toBe(2);
    await user.click(within(toolbar).getByRole("button", { name: "Xoá bảng" }));
    expect(within(box).queryByRole("table")).not.toBeInTheDocument();
    expect(within(toolbar).queryByRole("button", { name: "Xoá bảng" })).not.toBeInTheDocument();

    // Chèn bảng mới 3 cột × 2 dòng, dòng đầu là tiêu đề.
    await user.click(within(toolbar).getByRole("button", { name: "Chèn bảng" }));
    expect(within(box).getByRole("table")).toBeInTheDocument();
    expect(within(box).getAllByRole("columnheader")).toHaveLength(3);
    expect(within(box).getAllByRole("row")).toHaveLength(2);
  });
});
