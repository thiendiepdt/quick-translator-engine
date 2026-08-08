import {
  BasicBlocksPlugin,
  BasicMarksPlugin,
  HorizontalRulePlugin,
} from "@platejs/basic-nodes/react";
import { LinkPlugin } from "@platejs/link/react";
import { ListPlugin } from "@platejs/list/react";
import { MarkdownPlugin } from "@platejs/markdown";
import { TablePlugin } from "@platejs/table/react";
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
