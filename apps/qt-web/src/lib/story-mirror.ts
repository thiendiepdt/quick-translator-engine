/**
 * Bản sao dự phòng của cấu hình truyện trong localStorage. IndexedDB (nơi
 * lưu chính) trong thực tế mong manh hơn — backend corrupt, private mode,
 * version deadlock — trong khi cấu hình truyện (glossary tích lũy, style,
 * prompt) là phần khó gây dựng lại nhất. Mirror không chống được eviction
 * cả origin (localStorage bay cùng), chỉ cứu ca "IDB hỏng riêng lẻ".
 */

import {
  countStoryGlossaryEntries,
  normalizeAiStoryConfig,
  type AiStoryConfig,
} from "@/lib/ai-story";

export function storyMirrorKeyFor(workspaceStorageKey: string): string {
  return `${workspaceStorageKey}:story-mirror`;
}

/** Config chưa có dấu tay người dùng thì không đáng ghi đè bởi/lên mirror. */
export function aiStoryIsEmpty(story: AiStoryConfig): boolean {
  return (
    !story.name.trim() &&
    !story.sourceUrl.trim() &&
    !story.protagonist.trim() &&
    !story.summary.trim() &&
    !story.customPrompt.trim() &&
    story.checkRules.length === 0 &&
    story.autoGlossaryLog.length === 0 &&
    story.autoGlossary === "inherit" &&
    countStoryGlossaryEntries(story.glossary) === 0 &&
    !story.style.voice.trim() &&
    story.style.toneRules.length === 0 &&
    Object.keys(story.style.signaturePhrases).length === 0 &&
    story.style.avoid.length === 0
  );
}

export function writeStoryMirror(
  workspaceStorageKey: string,
  story: AiStoryConfig,
): void {
  try {
    window.localStorage.setItem(
      storyMirrorKeyFor(workspaceStorageKey),
      JSON.stringify(story),
    );
  } catch {
    // localStorage đầy hoặc bị chặn — mirror là dự phòng, không được gây lỗi.
  }
}

export function readStoryMirror(
  workspaceStorageKey: string,
): AiStoryConfig | undefined {
  try {
    const raw = window.localStorage.getItem(storyMirrorKeyFor(workspaceStorageKey));
    if (!raw) return undefined;
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
      return undefined;
    }
    return normalizeAiStoryConfig(parsed);
  } catch {
    return undefined;
  }
}
