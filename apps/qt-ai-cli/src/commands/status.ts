import { resolve } from "node:path";
import { naturalChapterCompare } from "@/lib/ai-story";
import { loadState, storyPaths, type ChapterStatus } from "../story-fs.ts";

export function runStatus(root: string): string {
  const state = loadState(storyPaths(resolve(root)));
  const counts: Record<ChapterStatus, number> = {
    queued: 0, translating: 0, done: 0, error: 0, skipped: 0,
  };
  let withWarnings = 0;
  const flagged: string[] = [];
  const ids = Object.keys(state.chapters).sort(naturalChapterCompare);
  for (const id of ids) {
    const chapter = state.chapters[id]!;
    counts[chapter.status] += 1;
    if (chapter.status === "error" || chapter.status === "skipped") {
      flagged.push(`  ${id} [${chapter.status}] ${chapter.reason ?? ""}`.trimEnd());
    }
    if (chapter.status === "translating") flagged.push(`  ${id} [translating] — đang dở, check/accept/skip trước`);
    if (chapter.status === "done" && chapter.warnings && chapter.warnings.length > 0) {
      withWarnings += 1;
      flagged.push(`  ${id} [done, ${chapter.warnings.length} cảnh báo] ${chapter.warnings[0]}`);
    }
  }
  const lines = [
    `Tổng ${ids.length} chương — done: ${counts.done}, queued: ${counts.queued}, translating: ${counts.translating}, error: ${counts.error}, skipped: ${counts.skipped}` +
      (withWarnings > 0 ? `, done kèm cảnh báo: ${withWarnings}` : ""),
  ];
  if (flagged.length > 0) lines.push("Cần chú ý:", ...flagged);
  lines.push(`Giới hạn phiên: dịch tối đa ${state.settings.chaptersPerSession} chương/phiên rồi nghỉ.`);
  return lines.join("\n");
}
