import { rmSync } from "node:fs";
import { resolve } from "node:path";
import { loadState, saveState, storyPaths, workFile, type WorkKind } from "../story-fs.ts";

const WORK_KINDS: WorkKind[] = ["prompt", "draft", "glossary", "check", "review"];

export function runSkip(root: string, id: string, reason: string): void {
  if (!reason.trim()) throw new Error("skip cần --reason <lý do> không rỗng.");
  const paths = storyPaths(resolve(root));
  const state = loadState(paths);
  const chapter = state.chapters[id];
  if (!chapter) throw new Error(`Không có chương ${id} trong state.json.`);
  if (chapter.status === "done") throw new Error(`Chương ${id} đã done, không skip được.`);
  state.chapters[id] = { ...chapter, status: "skipped", reason: reason.trim(), updatedAt: Date.now() };
  saveState(paths, state);
  for (const kind of WORK_KINDS) rmSync(workFile(paths, id, kind), { force: true });
}
