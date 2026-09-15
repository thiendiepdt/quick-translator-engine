import { z } from "zod";

export const GLOSSARY_KEYS = ["names", "places", "items", "creatures", "skills", "common", "signature_phrases", "addressing"] as const;
const stringRecord = z.record(z.string(), z.string());

export const chapterStatusSchema = z.enum(["queued", "translating", "done", "error", "skipped"]);
export const autoGlossarySchema = z.enum(["inherit", "on", "off"]);

export const checkRuleSchema = z.object({ pattern: z.string(), flags: z.string().optional(), message: z.string() });

export const GENRE_SETTINGS = ["ancient", "modern", "mixed"] as const;
export const GENRE_NAMES = ["han", "foreign", "mixed"] as const;
export const GENRE_TONES = ["neutral", "romance"] as const;
/** Hai trục thể loại (port `StoryGenre` của qt-web): bối cảnh quyết xưng hô/rule, tên riêng quyết cách phiên. */
/** `tone` default neutral: story.json cũ chưa có field, prompt không đổi. */
export const storyGenreSchema = z.object({
  setting: z.enum(GENRE_SETTINGS),
  names: z.enum(GENRE_NAMES),
  tone: z.enum(GENRE_TONES).default("neutral"),
});

/** 8 nhóm glossary (truyện lẫn kho chung của app). */
export const glossaryRecordSchema = z.object({
  names: stringRecord,
  places: stringRecord,
  items: stringRecord,
  creatures: stringRecord,
  skills: stringRecord,
  common: stringRecord,
  signature_phrases: stringRecord,
  /** `甲→乙: X–Y` xưng hô theo cặp; story.json cũ thiếu nhóm này. */
  addressing: stringRecord.default({}),
});

export const storyConfigSchema = z.object({
  name: z.string(),
  sourceUrl: z.string(),
  protagonist: z.string(),
  summary: z.string(),
  genre: storyGenreSchema,
  glossary: glossaryRecordSchema,
  style: z.object({
    voice: z.string(),
    toneRules: z.array(z.string()),
    signaturePhrases: stringRecord,
    avoid: z.array(z.string()),
  }),
  customPrompt: z.string(),
  checkRules: z.array(checkRuleSchema),
  autoGlossaryLog: z.array(
    z.object({ source: z.string(), target: z.string(), category: z.string(), chapter: z.string() }),
  ),
  autoGlossary: autoGlossarySchema,
});

export const harnessSettingsSchema = z.object({
  minLengthRatio: z.number().min(0.1).max(3),
  maxReviewRounds: z.number().int().min(0).max(10),
  chaptersPerSession: z.number().int().min(1).max(100),
});

export const chapterRowSchema = z.object({
  id: z.string(),
  status: chapterStatusSchema,
  reviewRound: z.number(),
  reason: z.string().nullable(),
  warnings: z.array(z.string()),
});

export const countsSchema = z.object({
  total: z.number(),
  queued: z.number(),
  translating: z.number(),
  done: z.number(),
  error: z.number(),
  skipped: z.number(),
  withWarnings: z.number(),
});

export const storySnapshotSchema = z.object({
  root: z.string(),
  chapters: z.array(chapterRowSchema),
  counts: countsSchema,
  settings: harnessSettingsSchema,
  story: storyConfigSchema,
  sessionRunning: z.boolean(),
});

export const chapterViewSchema = z.object({
  id: z.string(),
  status: chapterStatusSchema,
  raw: z.string(),
  /** raw/<id>.txt đã mất sau lần quét gần nhất — UI nhắc Quét lại để gỡ chương. */
  rawMissing: z.boolean().default(false),
  output: z.string().nullable(),
  draft: z.string().nullable(),
  review: z.string().nullable(),
  warnings: z.array(z.string()),
  reason: z.string().nullable(),
});

export const agyStatusSchema = z.object({
  found: z.boolean(),
  path: z.string().nullable(),
  version: z.string().nullable(),
  models: z.array(z.string()),
  message: z.string().nullable(),
});

export const engineSchema = z.enum(["agy", "api"]);
export const apiProviderSchema = z.enum(["gemini", "openai"]);

/** Bước gọi model có mức nghĩ riêng — cùng thứ tự với `ApiStep` bên Rust. */
export const API_STEPS = ["translate", "review", "glossary", "fill"] as const;
export type ApiStep = (typeof API_STEPS)[number];
/** "" = không gửi tham số nghĩ (model tự quyết). Gemini 3.x → thinkingLevel; 2.5: minimal = tắt, khác = tự động. */
export const GEMINI_EFFORTS = ["", "minimal", "low", "medium", "high"] as const;
/** "" = không gửi `reasoning_effort`. */
export const OPENAI_EFFORTS = ["", "none", "low", "medium", "high", "xhigh", "max"] as const;
/** Dịch/soát và AI điền hồ sơ nghĩ cao; trích glossary chỉ đối chiếu tên nên low cho nhanh, rẻ. */
export const DEFAULT_STEP_EFFORTS = { translate: "high", review: "high", glossary: "low", fill: "high" } as const;

/**
 * `strict`: 4 khoá bắt buộc (form Cài đặt). `stored`: config trên đĩa — thiếu khoá/thiếu cả khối thì lấy mặc định.
 */
function stepEffortsSchemas<const T extends readonly ["", string, ...string[]]>(values: T) {
  const level = z.enum(values);
  type Level = z.infer<typeof level>;
  const strict = z.object({ translate: level, review: level, glossary: level, fill: level });
  const stored = z
    .object({
      translate: level.default("high" as Level),
      review: level.default("high" as Level),
      glossary: level.default("low" as Level),
      fill: level.default("high" as Level),
    })
    .default(DEFAULT_STEP_EFFORTS);
  return { strict, stored };
}
const geminiEfforts = stepEffortsSchemas(GEMINI_EFFORTS);
const openaiEfforts = stepEffortsSchemas(OPENAI_EFFORTS);
export const geminiStepEffortsSchema = geminiEfforts.strict;
export const openaiStepEffortsSchema = openaiEfforts.strict;

function providerCredentialsSchema<T extends z.ZodTypeAny>(effort: T) {
  return z.object({
    apiKey: z.string().default(""),
    model: z.string().default(""),
    baseUrl: z.string().default(""),
    effort,
  });
}
export const geminiCredentialsSchema = providerCredentialsSchema(geminiEfforts.stored);
export const openaiCredentialsSchema = providerCredentialsSchema(openaiEfforts.stored);

export const DEFAULT_API_MODELS = { gemini: "gemini-3.7-flash", openai: "gpt-5.6-sol" } as const;

/** Cùng default với `ApiSettings::default()` bên Rust; config cũ thiếu cả khối vẫn parse (Rust đã
 * chuyển `thinking`/`reasoningEffort` cũ sang `effort`, zod bỏ hai khoá đó nếu còn). */
export const apiSettingsSchema = z.object({
  provider: apiProviderSchema.default("gemini"),
  gemini: geminiCredentialsSchema.default({ apiKey: "", model: DEFAULT_API_MODELS.gemini, baseUrl: "", effort: DEFAULT_STEP_EFFORTS }),
  openai: openaiCredentialsSchema.default({ apiKey: "", model: DEFAULT_API_MODELS.openai, baseUrl: "", effort: DEFAULT_STEP_EFFORTS }),
});

export const appConfigSchema = z.object({
  engine: engineSchema.default("api"),
  api: apiSettingsSchema.default({
    provider: "gemini",
    gemini: { apiKey: "", model: DEFAULT_API_MODELS.gemini, baseUrl: "", effort: DEFAULT_STEP_EFFORTS },
    openai: { apiKey: "", model: DEFAULT_API_MODELS.openai, baseUrl: "", effort: DEFAULT_STEP_EFFORTS },
  }),
  agyPath: z.string().nullable(),
  model: z.string().nullable(),
  maxSessions: z.number().int().min(1).max(1000),
  /** Số truyện dịch song song (mỗi truyện một phiên). */
  maxParallel: z.number().int().min(1).max(20).default(20),
  recent: z.array(z.string()),
  /** Thư viện: folder cha chứa mọi truyện; null = chưa chọn. */
  libraryRoot: z.string().nullable().default(null),
  palette: z.string().default("editorial"),
  themeMode: z.string().default("system"),
  readingWidth: z.string().default("normal"),
});

/** "builtin" = bản cứng trong binary; "file" = người dùng đã sửa ở Cài đặt → Bản mặc định. */
export const BASE_SOURCES = ["builtin", "file"] as const;
/** Prompt gốc + rule mặc định của app (Rust `story_defaults`). */
export const storyDefaultsSchema = z.object({
  basePrompt: z.string(),
  promptSource: z.enum(BASE_SOURCES),
  promptSuffix: z.string(),
  checkRules: z.array(checkRuleSchema),
  rulesSource: z.enum(BASE_SOURCES),
});
export const BASE_KINDS = ["prompt", "rules", "glossary"] as const;
/** Một bản mặc định (Rust `base_get/base_save/base_reset`): prompt theo genre, rules/glossary theo bối cảnh. */
export const baseViewSchema = z.object({
  kind: z.enum(BASE_KINDS),
  setting: z.enum(GENRE_SETTINGS),
  names: z.enum(GENRE_NAMES).optional(),
  source: z.enum(BASE_SOURCES),
  text: z.string().optional(),
  rules: z.array(checkRuleSchema).optional(),
  glossary: glossaryRecordSchema.optional(),
});

export const recentSummarySchema = z.object({
  root: z.string(),
  name: z.string().nullable(),
  done: z.number().nullable(),
  total: z.number().nullable(),
});

export const progressSchema = z.object({
  done: z.number(),
  queued: z.number(),
  translating: z.number(),
  error: z.number(),
  skipped: z.number(),
  warnings_count: z.number(),
  current: z.string().nullable(),
});

export const stopReasonSchema = z.discriminatedUnion("kind", [
  z.object({ kind: z.literal("finished") }),
  z.object({ kind: z.literal("no_progress") }),
  z.object({ kind: z.literal("agy_failed"), code: z.number() }),
  z.object({ kind: z.literal("api_failed"), message: z.string() }),
  z.object({ kind: z.literal("user_cancelled") }),
  z.object({ kind: z.literal("max_sessions") }),
  z.object({ kind: z.literal("internal"), message: z.string() }),
]);

// `Stopped(StopReason)` bên Rust là newtype trong enum tagged `type` → serde flatten biến thể trong:
// {type:"stopped", kind:"agy_failed", code:3}. Các biến thể stopped cùng giá trị `type` nên không
// dùng được discriminatedUnion("type") phẳng; gom bằng union thường.
const stoppedEventSchema = z.discriminatedUnion("kind", [
  z.object({ type: z.literal("stopped"), kind: z.literal("finished") }),
  z.object({ type: z.literal("stopped"), kind: z.literal("no_progress") }),
  z.object({ type: z.literal("stopped"), kind: z.literal("agy_failed"), code: z.number() }),
  z.object({ type: z.literal("stopped"), kind: z.literal("api_failed"), message: z.string() }),
  z.object({ type: z.literal("stopped"), kind: z.literal("user_cancelled") }),
  z.object({ type: z.literal("stopped"), kind: z.literal("max_sessions") }),
  z.object({ type: z.literal("stopped"), kind: z.literal("internal"), message: z.string() }),
]);

export const sessionEventSchema = z.union([
  z.object({ type: z.literal("started"), session_no: z.number() }),
  progressSchema.extend({ type: z.literal("progress") }),
  z.object({ type: z.literal("agy_log"), line: z.string(), stream: z.enum(["stdout", "stderr"]) }),
  stoppedEventSchema,
]);

/** Root của mọi truyện đang có phiên chạy. */
export const sessionStatusSchema = z.object({ running: z.array(z.string()) });
/** Event phiên từ Rust kèm root: `{root, type, ...}`; phần event parse riêng bằng sessionEventSchema. */
export const rootedSessionEventSchema = z.object({ root: z.string() });
/** Kết quả kéo thả chương vào raw/ (Rust `import_chapters`). */
export const importOutcomeSchema = z.object({
  added: z.array(z.string()),
  skippedExisting: z.array(z.string()),
  ignored: z.array(z.string()),
  snapshot: storySnapshotSchema,
});

export const aiFillResultSchema = z.object({
  before: storyConfigSchema,
  after: storyConfigSchema,
  exitCode: z.number(),
  log: z.array(z.string()),
});
export const exportOutcomeSchema = z.object({ outPath: z.string(), ids: z.array(z.string()), gaps: z.array(z.string()) });
export const retryRangeOutcomeSchema = z.object({
  retried: z.array(z.string()),
  backedUp: z.array(z.string()),
  alreadyQueued: z.array(z.string()),
});
export const deleteOutcomeSchema = z.object({ removed: z.array(z.string()), keptOutputs: z.array(z.string()) });
