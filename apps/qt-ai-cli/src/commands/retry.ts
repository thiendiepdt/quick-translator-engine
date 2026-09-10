import { existsSync, renameSync, rmSync } from "node:fs";
import { join, resolve } from "node:path";
import { loadState, saveState, storyPaths, workFile, type StoryPaths, type WorkKind } from "../story-fs.ts";

const WORK_KINDS: WorkKind[] = ["prompt", "draft", "glossary", "check", "review"];

/** Bản sao lưu của out/<id>.txt khi dịch lại chương đã done (đè bak cũ, không tích luỹ). */
export function retryBackupPath(paths: StoryPaths, id: string): string {
  return join(paths.outDir, `${id}.txt.bak`);
}

/**
 * Đưa chương về hàng đợi dịch lại từ đầu (reviewRound 0, dọn work/).
 * - error/skipped: ca "cứu chương hỏng" thông thường.
 * - done: out/<id>.txt đổi tên thành .txt.bak để lỡ tay còn lấy lại; accept lần sau ghi bản mới.
 * - translating: chương kẹt sau khi phiên chết — caller tự chắc không có phiên đang chạy trên truyện.
 * - queued: từ chối, chương đã ở hàng đợi.
 */
export function runRetry(root: string, id: string): void {
  const paths = storyPaths(resolve(root));
  const state = loadState(paths);
  const chapter = state.chapters[id];
  if (!chapter) throw new Error(`Không có chương ${id} trong state.json.`);
  if (chapter.status === "queued") throw new Error(`Chương ${id} đang queued sẵn rồi.`);
  if (chapter.status === "done") {
    const out = join(paths.outDir, `${id}.txt`);
    if (existsSync(out)) renameSync(out, retryBackupPath(paths, id));
  }
  state.chapters[id] = { status: "queued", reviewRound: 0, updatedAt: Date.now() };
  saveState(paths, state);
  for (const kind of WORK_KINDS) rmSync(workFile(paths, id, kind), { force: true });
}
