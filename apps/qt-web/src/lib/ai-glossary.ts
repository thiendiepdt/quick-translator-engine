/**
 * Vòng phản hồi glossary cho dịch AI: sau mỗi chương, trích các tên riêng mới
 * từ cặp raw ↔ bản dịch rồi tự nạp vào glossary truyện để các chương sau dịch
 * nhất quán. Chạy không người trông nên mọi guard nằm ở phía client: chỉ thêm
 * key mới, chỉ nhận cặp thực sự xuất hiện ở cả hai phía.
 */

import {
  ADDRESSING_ARROW,
  addressingSides,
  storyGlossaryCategories,
  type AiStoryConfig,
  type AutoGlossaryEntry,
  type StoryAutoGlossarySetting,
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

/** Key glossary có mặt trong chương: tên thường thì nguyên văn; cặp `甲→乙` thì cả hai bên đều có. */
export function glossaryKeyTouchesSource(key: string, text: string): boolean {
  const sides = addressingSides(key);
  return sides.length > 1 ? sides.every((side) => text.includes(side)) : text.includes(key);
}

/**
 * Cặp xưng hô do model đề xuất: `甲→乙` với hai tên Hán đều có trong raw, target dạng `X–Y`
 * (nhận cả `-`, `—`, `/` rồi chuẩn về `–`). Không đòi target nằm nguyên văn trong bản dịch —
 * `anh–em` là hai từ rời.
 */
function sanitizeAddressing(source: string, target: string, raw: string): [string, string] | undefined {
  const sides = source.split(/→|->|=>/).map((side) => side.trim());
  if (sides.length !== 2 || sides.some((side) => !side || !/\p{Script=Han}/u.test(side) || !raw.includes(side))) {
    return undefined;
  }
  const parts = target.split(/\s*[–—\-/]\s*/).map((part) => part.trim()).filter(Boolean);
  if (parts.length !== 2) return undefined;
  return [sides.join(ADDRESSING_ARROW), parts.join("–")];
}

/**
 * Model chỉ được đề xuất; quyền quyết ở đây: source phải là Hán tự có mặt
 * trong raw, target phải xuất hiện nguyên văn trong bản dịch, key chưa tồn
 * tại, không trùng lặp. Category lạ rơi về "names".
 */
/**
 * Danh xưng gắn họ (`老张`, `李老师`, `苏夫人`, `周七小姐`…) không phải tên riêng: không tự thêm vào names. Trước
 * danh xưng chỉ là họ một chữ hoặc họ kép — biệt danh hai chữ (`东溟夫人`, `厄运小姐`) là tên, vẫn thêm.
 */
const TITLE_NAME =
  /^(?:老|小|阿)?(?:欧阳|上官|司马|诸葛|慕容|东方|南宫|西门|公孙|令狐|皇甫|尉迟|长孙|宇文|独孤|端木|夏侯|轩辕|拓跋|百里|呼延|澹台|司徒|司空|万俟|闻人|太史|申屠|东郭|\p{Script=Han})[一二三四五六七八九十大]?(?:姑娘|夫人|管家|管事|阿姨|姨|老师|大夫|捕头|长老|女士|先生|小姐|奶奶|爷爷|老头|师姐|师兄|师妹|师弟|哥|姐|少|导|总|董|经理)$|^老\p{Script=Han}$/u;
/** `《X》` lưu thành `X` để tên sách/bài hát chỉ có một mục, một cách dịch. */
const BOOK_TITLE = /^《(.+)》$/u;

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
    const trimmedSource = typeof record.source === "string" ? record.source.trim() : "";
    const trimmedTarget = typeof record.target === "string" ? record.target.trim() : "";
    if (!trimmedSource || !trimmedTarget) continue;
    if (record.category === "addressing") {
      const pair = sanitizeAddressing(trimmedSource, trimmedTarget, raw);
      if (!pair || seen.has(pair[0]) || existingKeys.has(pair[0])) continue;
      seen.add(pair[0]);
      pairs.push({ source: pair[0], target: pair[1], category: "addressing" });
      continue;
    }
    const source = trimmedSource.replace(BOOK_TITLE, "$1");
    const target = trimmedTarget.replace(BOOK_TITLE, "$1");
    // Tối thiểu 2 chữ Hán: chữ đơn (雷, 炎…) đa nghĩa quá — vừa dễ ghim sai
    // vào văn tả cảnh, vừa match mọi chương nên không bao giờ được lọc bớt.
    if ((source.match(/\p{Script=Han}/gu) ?? []).length < 2) continue;
    if (seen.has(source) || existingKeys.has(source)) continue;
    if (!raw.includes(source) || !translation.includes(target)) continue;
    // Nhóm ghi sai (không thuộc nhóm nào) thì bỏ — rơi vào names sẽ biến từ chung thành tên riêng.
    if (typeof record.category === "string" && !CATEGORY_KEYS.has(record.category)) continue;
    const category = typeof record.category === "string" ? (record.category as StoryGlossaryKey) : "names";
    if (category === "names" && TITLE_NAME.test(source)) continue;
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

/** Cài đặt theo truyện thắng; "inherit" mới rơi về toggle chung trong Cấu hình AI. */
export function resolveAutoGlossaryEnabled(
  storySetting: StoryAutoGlossarySetting,
  settingsEnabled: boolean,
): boolean {
  if (storySetting === "on") return true;
  if (storySetting === "off") return false;
  return settingsEnabled;
}
