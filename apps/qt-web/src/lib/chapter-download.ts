/**
 * Chọn và đóng gói chương đã dịch để tải về. "Số chương" trong UI là số thứ
 * tự 1-based trong danh sách chương (đã sort tự nhiên theo tên file).
 */

import type { AiTranslationChapter } from "@/lib/ai-story";

function isDownloadable(chapter: AiTranslationChapter): boolean {
  return chapter.status === "done" && chapter.output.trim().length > 0;
}

/** Khoảng mặc định: từ chương done nhỏ nhất đến chương done lớn nhất. */
export function doneChapterRange(
  chapters: AiTranslationChapter[],
): { from: number; to: number } | undefined {
  let from: number | undefined;
  let to: number | undefined;
  chapters.forEach((chapter, index) => {
    if (!isDownloadable(chapter)) return;
    from ??= index + 1;
    to = index + 1;
  });
  return from !== undefined && to !== undefined ? { from, to } : undefined;
}

/** Chương done trong khoảng [from..to] 1-based; khoảng lệch được clamp. */
export function selectChaptersForDownload(
  chapters: AiTranslationChapter[],
  from: number,
  to: number,
): AiTranslationChapter[] {
  const start = Math.max(1, Math.trunc(from));
  const end = Math.min(chapters.length, Math.trunc(to));
  return chapters.slice(start - 1, end).filter(isDownloadable);
}

/** Gộp các chương thành một văn bản, mỗi chương cách nhau đúng một dòng trống. */
export function mergeChapterOutputs(chapters: AiTranslationChapter[]): string {
  const merged = chapters
    .map((chapter) => chapter.output.replace(/\s+$/u, ""))
    .filter(Boolean)
    .join("\n\n");
  return merged ? `${merged}\n` : "";
}
