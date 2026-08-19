import type { TranslationViolation } from "@/lib/ai-translation";

export const storyGlossaryCategories = [
  { key: "names", label: "Tên nhân vật" },
  { key: "places", label: "Địa danh" },
  { key: "items", label: "Đồ vật / vũ khí" },
  { key: "creatures", label: "Sinh vật" },
  { key: "skills", label: "Kỹ năng / công pháp" },
  { key: "common", label: "Từ thông dụng" },
  { key: "signature_phrases", label: "Cụm từ đặc trưng" },
] as const;

export type StoryGlossaryKey = (typeof storyGlossaryCategories)[number]["key"];
export type StoryGlossary = Record<StoryGlossaryKey, Record<string, string>>;

export interface StoryStyle {
  voice: string;
  toneRules: string[];
  signaturePhrases: Record<string, string>;
  avoid: string[];
}

export interface AiCheckRule {
  pattern: string;
  flags?: string;
  message: string;
}

/** Một mục glossary do máy tự thêm sau khi dịch xong chương — giữ nguồn gốc để duyệt lại. */
export interface AutoGlossaryEntry {
  source: string;
  target: string;
  category: StoryGlossaryKey;
  chapter: string;
}

export interface AiStoryConfig {
  name: string;
  sourceUrl: string;
  protagonist: string;
  summary: string;
  glossary: StoryGlossary;
  style: StoryStyle;
  /** Trống nghĩa là dùng prompt mặc định được port từ Novel Translator. */
  customPrompt: string;
  /** Trống nghĩa là dùng bộ rule mặc định. */
  checkRules: AiCheckRule[];
  /** Nhật ký các mục glossary tự thêm từ bản dịch, để duyệt/gỡ sau. */
  autoGlossaryLog: AutoGlossaryEntry[];
  /** Bật/tắt tự thêm tên riêng cho truyện này; "inherit" = theo Cấu hình AI. */
  autoGlossary: StoryAutoGlossarySetting;
}

export type StoryAutoGlossarySetting = "inherit" | "on" | "off";

export type AiChapterStatus =
  | "queued"
  | "translating"
  | "reviewing"
  | "done"
  | "error";

export interface AiTranslationChapter {
  id: string;
  filename: string;
  source: string;
  output: string;
  thinking: string;
  violations: TranslationViolation[];
  status: AiChapterStatus;
  reviewRound: number;
  error?: string;
  updatedAt: number;
}

export function emptyStoryGlossary(): StoryGlossary {
  return Object.fromEntries(
    storyGlossaryCategories.map(({ key }) => [key, {}]),
  ) as StoryGlossary;
}

export function emptyAiStoryConfig(): AiStoryConfig {
  return {
    name: "",
    sourceUrl: "",
    protagonist: "",
    summary: "",
    glossary: emptyStoryGlossary(),
    style: {
      voice: "",
      toneRules: [],
      signaturePhrases: {},
      avoid: [],
    },
    customPrompt: "",
    checkRules: [],
    autoGlossaryLog: [],
    autoGlossary: "inherit",
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function stringValue(value: unknown): string {
  return typeof value === "string" ? value : "";
}

function stringList(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((item): item is string => typeof item === "string")
    : [];
}

function stringRecord(value: unknown): Record<string, string> {
  if (!isRecord(value)) return {};
  return Object.fromEntries(
    Object.entries(value).filter(
      (entry): entry is [string, string] =>
        Boolean(entry[0].trim()) && typeof entry[1] === "string",
    ),
  );
}

export function normalizeAiStoryConfig(value: unknown): AiStoryConfig {
  const source = isRecord(value) ? value : {};
  const glossaryValue = isRecord(source.glossary) ? source.glossary : {};
  const glossary = Object.fromEntries(
    storyGlossaryCategories.map(({ key }) => [key, stringRecord(glossaryValue[key])]),
  ) as StoryGlossary;
  const styleValue = isRecord(source.style) ? source.style : {};
  const rules = Array.isArray(source.checkRules)
    ? source.checkRules.flatMap((rule) => {
        if (!isRecord(rule)) return [];
        const pattern = stringValue(rule.pattern);
        const flags = stringValue(rule.flags);
        const message = stringValue(rule.message);
        return pattern && message
          ? [{ pattern, ...(flags ? { flags } : {}), message }]
          : [];
      })
    : [];
  const glossaryKeys = new Set<string>(storyGlossaryCategories.map(({ key }) => key));
  const autoGlossaryLog: AutoGlossaryEntry[] = Array.isArray(source.autoGlossaryLog)
    ? source.autoGlossaryLog.flatMap((entry) => {
        if (!isRecord(entry)) return [];
        const entrySource = stringValue(entry.source).trim();
        const target = stringValue(entry.target).trim();
        const category = stringValue(entry.category);
        if (!entrySource || !target || !glossaryKeys.has(category)) return [];
        return [{
          source: entrySource,
          target,
          category: category as StoryGlossaryKey,
          chapter: stringValue(entry.chapter),
        }];
      })
    : [];
  return {
    name: stringValue(source.name),
    sourceUrl: stringValue(source.sourceUrl),
    protagonist: stringValue(source.protagonist),
    summary: stringValue(source.summary),
    glossary,
    style: {
      voice: stringValue(styleValue.voice),
      toneRules: stringList(styleValue.toneRules ?? styleValue.tone_rules),
      signaturePhrases: stringRecord(
        styleValue.signaturePhrases ?? styleValue.signature_phrases,
      ),
      avoid: stringList(styleValue.avoid),
    },
    customPrompt: stringValue(source.customPrompt),
    checkRules: rules,
    autoGlossaryLog,
    autoGlossary:
      source.autoGlossary === "on" || source.autoGlossary === "off"
        ? source.autoGlossary
        : "inherit",
  };
}

function validViolation(value: unknown): value is TranslationViolation {
  return (
    isRecord(value) &&
    typeof value.line === "number" &&
    typeof value.message === "string" &&
    typeof value.text === "string"
  );
}

export function normalizeAiTranslationChapters(
  value: unknown,
): AiTranslationChapter[] {
  if (!Array.isArray(value)) return [];
  const seen = new Set<string>();
  return value.flatMap((item) => {
    if (!isRecord(item)) return [];
    const id = stringValue(item.id);
    const filename = stringValue(item.filename).trim();
    if (!id || !filename || seen.has(id)) return [];
    seen.add(id);
    const rawStatus = item.status;
    const status: AiChapterStatus =
      rawStatus === "done" || rawStatus === "error" || rawStatus === "queued"
        ? rawStatus
        : "queued";
    return [{
      id,
      filename,
      source: stringValue(item.source),
      output: stringValue(item.output),
      thinking: stringValue(item.thinking),
      violations: Array.isArray(item.violations)
        ? item.violations.filter(validViolation)
        : [],
      status,
      reviewRound:
        typeof item.reviewRound === "number" && Number.isFinite(item.reviewRound)
          ? Math.max(0, Math.trunc(item.reviewRound))
          : 0,
      ...(typeof item.error === "string" && item.error ? { error: item.error } : {}),
      updatedAt:
        typeof item.updatedAt === "number" && Number.isFinite(item.updatedAt)
          ? item.updatedAt
          : Date.now(),
    }];
  });
}

export function naturalChapterCompare(left: string, right: string): number {
  return left.localeCompare(right, "vi", { numeric: true, sensitivity: "base" });
}

export function countStoryGlossaryEntries(glossary: StoryGlossary): number {
  return Object.values(glossary).reduce(
    (total, entries) => total + Object.keys(entries).length,
    0,
  );
}

/** Đọc file JSON cấu hình truyện do người dùng xuất/nhập; hỏng thì undefined. */
export function parseAiStoryConfigJson(text: string): AiStoryConfig | undefined {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    return undefined;
  }
  if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
    return undefined;
  }
  return normalizeAiStoryConfig(parsed);
}
