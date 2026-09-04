import { z } from "zod";

export const GLOSSARY_KEYS = ["names", "places", "items", "creatures", "skills", "common", "signature_phrases"] as const;
const stringRecord = z.record(z.string(), z.string());

export const chapterStatusSchema = z.enum(["queued", "translating", "done", "error", "skipped"]);
export const autoGlossarySchema = z.enum(["inherit", "on", "off"]);

export const checkRuleSchema = z.object({ pattern: z.string(), flags: z.string().optional(), message: z.string() });

export const GENRE_SETTINGS = ["ancient", "modern", "mixed"] as const;
export const GENRE_NAMES = ["han", "foreign", "mixed"] as const;
/** Hai trục thể loại (port `StoryGenre` của qt-web): bối cảnh quyết xưng hô/rule, tên riêng quyết cách phiên. */
export const storyGenreSchema = z.object({ setting: z.enum(GENRE_SETTINGS), names: z.enum(GENRE_NAMES) });

export const storyConfigSchema = z.object({
  name: z.string(),
  sourceUrl: z.string(),
  protagonist: z.string(),
  summary: z.string(),
  genre: storyGenreSchema,
  glossary: z.object({
    names: stringRecord,
    places: stringRecord,
    items: stringRecord,
    creatures: stringRecord,
    skills: stringRecord,
    common: stringRecord,
    signature_phrases: stringRecord,
  }),
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
export const OPENAI_REASONING_EFFORTS = ["none", "low", "medium", "high", "xhigh", "max"] as const;
export const reasoningEffortSchema = z.enum(OPENAI_REASONING_EFFORTS);

export const providerCredentialsSchema = z.object({
  apiKey: z.string().default(""),
  model: z.string().default(""),
  baseUrl: z.string().default(""),
});

export const DEFAULT_API_MODELS = { gemini: "gemini-3.7-flash", openai: "gpt-5.6-sol" } as const;

/** Cùng default với `ApiSettings::default()` bên Rust; config cũ thiếu cả khối vẫn parse. */
export const apiSettingsSchema = z.object({
  provider: apiProviderSchema.default("gemini"),
  gemini: providerCredentialsSchema.default({ apiKey: "", model: DEFAULT_API_MODELS.gemini, baseUrl: "" }),
  openai: providerCredentialsSchema.default({ apiKey: "", model: DEFAULT_API_MODELS.openai, baseUrl: "" }),
  thinking: z.boolean().default(true),
  reasoningEffort: reasoningEffortSchema.default("high"),
});

export const appConfigSchema = z.object({
  engine: engineSchema.default("agy"),
  api: apiSettingsSchema.default({
    provider: "gemini",
    gemini: { apiKey: "", model: DEFAULT_API_MODELS.gemini, baseUrl: "" },
    openai: { apiKey: "", model: DEFAULT_API_MODELS.openai, baseUrl: "" },
    thinking: true,
    reasoningEffort: "high",
  }),
  agyPath: z.string().nullable(),
  model: z.string().nullable(),
  maxSessions: z.number().int().min(1).max(1000),
  recent: z.array(z.string()),
  palette: z.string().default("editorial"),
  themeMode: z.string().default("system"),
  readingWidth: z.string().default("normal"),
});

/** Prompt gốc + rule mặc định của hệ (Rust `story_defaults`). */
export const storyDefaultsSchema = z.object({
  basePrompt: z.string(),
  promptSuffix: z.string(),
  checkRules: z.array(checkRuleSchema),
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

export const sessionStatusSchema = z.object({ running: z.boolean() });
export const aiFillResultSchema = z.object({
  before: storyConfigSchema,
  after: storyConfigSchema,
  exitCode: z.number(),
  log: z.array(z.string()),
});
export const exportOutcomeSchema = z.object({ outPath: z.string(), ids: z.array(z.string()), gaps: z.array(z.string()) });
