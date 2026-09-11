import { existsSync, renameSync, rmSync } from "node:fs";
import { join, resolve } from "node:path";
import { loadState, saveState, storyPaths, workFile, type StoryPaths, type StoryState, type WorkKind } from "../story-fs.ts";
import { naturalChapterCompare } from "@/lib/ai-story";

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
  requeueChapter(paths, state, id);
  saveState(paths, state);
}

export interface RetryRangeOutcome {
  retried: string[];
  backedUp: string[];
  alreadyQueued: string[];
}

/** Dịch lại mọi chương trong [from..to] (undefined = đầu/cuối): done giữ .bak, queued bỏ qua. */
export function runRetryRange(root: string, from?: string, to?: string): RetryRangeOutcome {
  const paths = storyPaths(resolve(root));
  const state = loadState(paths);
  const all = Object.keys(state.chapters).sort(naturalChapterCompare);
  for (const bound of [from, to]) {
    if (bound !== undefined && !state.chapters[bound]) throw new Error(`Không có chương ${bound} trong state.json.`);
  }
  if (from !== undefined && to !== undefined && naturalChapterCompare(from, to) > 0) {
    throw new Error(`Khoảng ngược: --from ${from} sau --to ${to}.`);
  }
  const outcome: RetryRangeOutcome = { retried: [], backedUp: [], alreadyQueued: [] };
  for (const id of all) {
    if (from !== undefined && naturalChapterCompare(id, from) < 0) continue;
    if (to !== undefined && naturalChapterCompare(id, to) > 0) continue;
    const status = state.chapters[id]!.status;
    if (status === "queued") {
      outcome.alreadyQueued.push(id);
      continue;
    }
    if (status === "done") outcome.backedUp.push(id);
    requeueChapter(paths, state, id);
    outcome.retried.push(id);
  }
  if (outcome.retried.length > 0) saveState(paths, state);
  return outcome;
}

/** Một chương về hàng đợi trong state (chưa ghi đĩa): done → out/<id>.txt thành .bak; dọn work/. */
function requeueChapter(paths: StoryPaths, state: StoryState, id: string): void {
  if (state.chapters[id]?.status === "done") {
    const out = join(paths.outDir, `${id}.txt`);
    if (existsSync(out)) renameSync(out, retryBackupPath(paths, id));
  }
  state.chapters[id] = { status: "queued", reviewRound: 0, updatedAt: Date.now() };
  for (const kind of WORK_KINDS) rmSync(workFile(paths, id, kind), { force: true });
}
