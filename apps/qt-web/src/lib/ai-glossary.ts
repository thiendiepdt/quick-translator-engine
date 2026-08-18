/**
 * Vòng phản hồi glossary cho dịch AI: sau mỗi chương, trích các tên riêng mới
 * từ cặp raw ↔ bản dịch rồi tự nạp vào glossary truyện để các chương sau dịch
 * nhất quán. Chạy không người trông nên mọi guard nằm ở phía client: chỉ thêm
 * key mới, chỉ nhận cặp thực sự xuất hiện ở cả hai phía.
 */

import {
  storyGlossaryCategories,
  type AiStoryConfig,
  type AutoGlossaryEntry,
  type StoryGlossary,
  type StoryGlossaryKey,
} from "@/lib/ai-story";
import type { TranslationGlossary } from "@/lib/ai-translation";

export interface ExtractedGlossaryPair {
  source: string;
  target: string;
  category: StoryGlossaryKey;
}

/** Mọi key đã có mặt trong prompt dịch — workspace + truyện — để loại trước. */
export function collectGlossaryKeys(
  workspaceGlossary: TranslationGlossary,
  storyGlossary: StoryGlossary,
): Set<string> {
  const keys = new Set<string>();
  for (const group of Object.values(workspaceGlossary)) {
    for (const key of Object.keys(group)) keys.add(key);
  }
  for (const group of Object.values(storyGlossary)) {
    for (const key of Object.keys(group)) keys.add(key);
  }
  return keys;
}

const CATEGORY_KEYS = new Set<string>(storyGlossaryCategories.map(({ key }) => key));

/**
 * Model chỉ được đề xuất; quyền quyết ở đây: source phải là Hán tự có mặt
 * trong raw, target phải xuất hiện nguyên văn trong bản dịch, key chưa tồn
 * tại, không trùng lặp. Category lạ rơi về "names".
 */
export function sanitizeExtractedGlossary(
  parsed: unknown,
  raw: string,
  translation: string,
  existingKeys: ReadonlySet<string>,
): ExtractedGlossaryPair[] {
  if (!Array.isArray(parsed)) return [];
  const seen = new Set<string>();
  const pairs: ExtractedGlossaryPair[] = [];
  for (const item of parsed) {
    if (typeof item !== "object" || item === null) continue;
    const record = item as Record<string, unknown>;
    const source = typeof record.source === "string" ? record.source.trim() : "";
    const target = typeof record.target === "string" ? record.target.trim() : "";
    if (!source || !target) continue;
    if (!/\p{Script=Han}/u.test(source)) continue;
    if (seen.has(source) || existingKeys.has(source)) continue;
    if (!raw.includes(source) || !translation.includes(target)) continue;
    const category = typeof record.category === "string" && CATEGORY_KEYS.has(record.category)
      ? (record.category as StoryGlossaryKey)
      : "names";
    seen.add(source);
    pairs.push({ source, target, category });
  }
  return pairs;
}

/**
 * Nạp các cặp đã sanitize vào glossary truyện: chỉ thêm key mới (entry sẵn có
 * — kể cả do người dùng điền tay — luôn thắng), append vào cuối nhóm để prefix
 * prompt phía trước còn ăn implicit cache, và ghi nhật ký nguồn gốc.
 */
export function appendAutoGlossary(
  story: AiStoryConfig,
  pairs: ExtractedGlossaryPair[],
  chapter: string,
): AiStoryConfig {
  const glossary = Object.fromEntries(
    Object.entries(story.glossary).map(([key, group]) => [key, { ...group }]),
  ) as StoryGlossary;
  const existing = collectGlossaryKeys({}, glossary);
  const log: AutoGlossaryEntry[] = [...story.autoGlossaryLog];
  for (const { source, target, category } of pairs) {
    if (existing.has(source)) continue;
    existing.add(source);
    glossary[category][source] = target;
    log.push({ source, target, category, chapter });
  }
  return { ...story, glossary, autoGlossaryLog: log };
}
