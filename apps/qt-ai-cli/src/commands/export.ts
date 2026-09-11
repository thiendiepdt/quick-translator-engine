import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { naturalChapterCompare } from "@/lib/ai-story";
import { loadState, storyPaths } from "../story-fs.ts";

export interface ExportOptions {
  from?: string;
  to?: string;
  out?: string;
}

export interface ExportResult {
  outPath: string;
  ids: string[];
  /** Chương trong khoảng nhưng chưa done (queued/error/skipped) — file gộp bị hổng ở đó. */
  gaps: string[];
}

/**
 * Gộp out/<id>.txt của các chương done trong khoảng [from..to] thành một file,
 * mỗi chương cách nhau đúng một dòng trống — cùng format với "Tải chương" của web.
 * Mặc định khoảng = từ chương done nhỏ nhất tới lớn nhất; file ra export/<from>-<to>.txt.
 */
export function runExport(root: string, options: ExportOptions = {}): ExportResult {
  const paths = storyPaths(resolve(root));
  const state = loadState(paths);
  const all = Object.keys(state.chapters).sort(naturalChapterCompare);
  const done = all.filter((id) => state.chapters[id]!.status === "done");
  if (done.length === 0) throw new Error("Chưa có chương nào done để export.");

  const from = options.from ?? done[0]!;
  const to = options.to ?? done[done.length - 1]!;
  for (const bound of [from, to]) {
    if (!state.chapters[bound]) throw new Error(`Không có chương ${bound} trong state.json.`);
  }
  if (naturalChapterCompare(from, to) > 0) throw new Error(`Khoảng ngược: --from ${from} sau --to ${to}.`);

  const inRange = all.filter(
    (id) => naturalChapterCompare(id, from) >= 0 && naturalChapterCompare(id, to) <= 0,
  );
  const ids = inRange.filter((id) => state.chapters[id]!.status === "done");
  const gaps = inRange.filter((id) => state.chapters[id]!.status !== "done");
  if (ids.length === 0) throw new Error(`Không có chương done nào trong khoảng ${from}..${to}.`);

  const merged = ids
    .map((id) => {
      const file = join(paths.outDir, `${id}.txt`);
      if (!existsSync(file)) throw new Error(`Chương ${id} done nhưng thiếu ${file}.`);
      return readFileSync(file, "utf8").replace(/\s+$/u, "");
    })
    .filter(Boolean)
    .join("\n\n");

  const outPath = options.out
    ? resolve(options.out)
    : join(paths.root, "export", `${from}-${to}.txt`);
  mkdirSync(dirname(outPath), { recursive: true });
  writeFileSync(outPath, `${merged}\n`, "utf8");
  return { outPath, ids, gaps };
}
