import { rmSync } from "node:fs";
import { resolve } from "node:path";
import { loadState, saveState, storyPaths, workFile, type WorkKind } from "../story-fs.ts";

const WORK_KINDS: WorkKind[] = ["prompt", "draft", "glossary", "check", "review"];

/** Đưa chương error/skipped về hàng đợi để dịch lại từ đầu (reviewRound 0, dọn work/). */
export function runRetry(root: string, id: string): void {
  const paths = storyPaths(resolve(root));
  const state = loadState(paths);
  const chapter = state.chapters[id];
  if (!chapter) throw new Error(`Không có chương ${id} trong state.json.`);
  if (chapter.status === "done") throw new Error(`Chương ${id} đã done — muốn dịch lại thì xoá out/${id}.txt trước rồi tính.`);
  if (chapter.status === "queued") throw new Error(`Chương ${id} đang queued sẵn rồi.`);
  if (chapter.status === "translating") {
    throw new Error(`Chương ${id} đang translating — qt-ai next sẽ tự phát lại nó, không cần retry.`);
  }
  state.chapters[id] = { status: "queued", reviewRound: 0, updatedAt: Date.now() };
  saveState(paths, state);
  for (const kind of WORK_KINDS) rmSync(workFile(paths, id, kind), { force: true });
}
