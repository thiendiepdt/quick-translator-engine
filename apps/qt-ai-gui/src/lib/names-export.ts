import type { StoryFormValues } from "@/lib/story-form";
import { GLOSSARY_LABELS, type GlossaryKey } from "@/lib/types";

/** Nhóm được export, theo thứ tự ghi file. `addressing` (khoá `甲→乙`) không phải cụm Hán nên không có mặt. */
export type NamesExportKey = Exclude<GlossaryKey, "addressing">;
export const NAMES_EXPORT_KEYS: readonly NamesExportKey[] = [
  "names",
  "places",
  "items",
  "creatures",
  "skills",
  "common",
  "signature_phrases",
];
/** Nhóm không tick sẵn: cụm đặc trưng là câu/thành ngữ, hiếm khi muốn thành tên trong bản convert. */
const DEFAULT_UNCHECKED: readonly NamesExportKey[] = ["signature_phrases"];

export const NAMES_FILE_NAMES = ["Names.txt", "Names2.txt"] as const;
export type NamesFileName = (typeof NAMES_FILE_NAMES)[number];

export interface NamesExportRow {
  /** `${group}:${index}` — ổn định trong một lần mở dialog. */
  id: string;
  group: NamesExportKey;
  source: string;
  target: string;
  /** Có giá trị = dòng không export được, kèm lý do ngắn để hiện. */
  skip?: string;
}

const BOM = "﻿";
const EOL = "\r\n";

/**
 * Lập danh sách dòng cho dialog: trim hai cột; đánh dấu bỏ theo cách QT đọc Names.txt
 * (tách mọi `=`, mỗi dòng một mục, khoá trùng chỉ giữ dòng đầu — xem docs/engine/dictionaries.md).
 */
export function planNamesExport(glossary: StoryFormValues["glossary"]): NamesExportRow[] {
  const rows: NamesExportRow[] = [];
  const seen = new Map<string, NamesExportKey>();
  for (const group of NAMES_EXPORT_KEYS) {
    glossary[group].forEach((pair, index) => {
      const source = pair.source.trim();
      const target = pair.target.trim();
      const row: NamesExportRow = { id: `${group}:${index}`, group, source, target };
      if (!source || !target) row.skip = "thiếu Hán/Việt";
      else if (source.includes("=") || target.includes("=")) row.skip = "có dấu =";
      else if (/[\r\n]/.test(source) || /[\r\n]/.test(target)) row.skip = "nhiều dòng";
      else {
        const kept = seen.get(source);
        if (kept) row.skip = `trùng nhóm ${GLOSSARY_LABELS[kept]}`;
        else seen.set(source, group);
      }
      rows.push(row);
    });
  }
  return rows;
}

/** Tick sẵn mọi dòng hợp lệ của các nhóm không nằm trong DEFAULT_UNCHECKED. */
export function defaultSelection(rows: NamesExportRow[]): Set<string> {
  return new Set(rows.filter((r) => !r.skip && !DEFAULT_UNCHECKED.includes(r.group)).map((r) => r.id));
}

/** Nội dung file: mỗi dòng `Hán=Việt` CRLF; `bom` cho file ghi ra đĩa (QT ghi có BOM), clipboard thì không. */
export function renderNames(
  rows: NamesExportRow[],
  selected: ReadonlySet<string>,
  options: { bom?: boolean } = {},
): string {
  const body = rows
    .filter((r) => !r.skip && selected.has(r.id))
    .map((r) => `${r.source}=${r.target}${EOL}`)
    .join("");
  if (!body) return "";
  return options.bom ? BOM + body : body;
}
