import {
  BasicBlocksPlugin,
  BasicMarksPlugin,
  HorizontalRulePlugin,
} from "@platejs/basic-nodes/react";
import { LinkPlugin } from "@platejs/link/react";
import { ListPlugin } from "@platejs/list/react";
import { MarkdownPlugin } from "@platejs/markdown";
import { TablePlugin } from "@platejs/table/react";
import { createPlateEditor } from "platejs/react";
import remarkGfm from "remark-gfm";

export const promptEditorPlugins = [
  BasicBlocksPlugin,
  BasicMarksPlugin,
  HorizontalRulePlugin,
  ListPlugin,
  TablePlugin,
  LinkPlugin,
  // Bảng, gạch ngang… là cú pháp GFM — thiếu remark-gfm thì "|" chỉ là text.
  MarkdownPlugin.configure({ options: { remarkPlugins: [remarkGfm] } }),
];

/**
 * Markdown sau khi đi qua editor (deserialize → serialize) — dùng để so "đã sửa hay chưa" với
 * prompt gốc, vì editor chuẩn hoá dấu list, dòng trống… nên không so chuỗi thô được.
 */
export function normalizeMarkdown(markdown: string): string {
  const editor = createPlateEditor({
    plugins: promptEditorPlugins,
    value: (currentEditor) => currentEditor.getApi(MarkdownPlugin).markdown.deserialize(markdown),
  });
  return editor.getApi(MarkdownPlugin).markdown.serialize().trim();
}
