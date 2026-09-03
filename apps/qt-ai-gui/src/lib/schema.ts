import { z } from "zod";

export const GLOSSARY_KEYS = ["names", "places", "items", "creatures", "skills", "common", "signature_phrases"] as const;
const stringRecord = z.record(z.string(), z.string());

export const chapterStatusSchema = z.enum(["queued", "translating", "done", "error", "skipped"]);
export const autoGlossarySchema = z.enum(["inherit", "on", "off"]);

export const checkRuleSchema = z.object({ pattern: z.string(), flags: z.string().optional(), message: z.string() });

export const storyConfigSchema = z.object({
  name: z.string(),
  sourceUrl: z.string(),
  protagonist: z.string(),
  summary: z.string(),
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

export const appConfigSchema = z.object({
  agyPath: z.string().nullable(),
  model: z.string().nullable(),
  maxSessions: z.number().int().min(1).max(1000),
  recent: z.array(z.string()),
  palette: z.string().default("editorial"),
  themeMode: z.string().default("system"),
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
