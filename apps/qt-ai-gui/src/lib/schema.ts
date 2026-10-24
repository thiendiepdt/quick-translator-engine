import { z } from "zod";

export const GLOSSARY_KEYS = ["names", "places", "items", "creatures", "skills", "common", "signature_phrases", "addressing"] as const;
const stringRecord = z.record(z.string(), z.string());

export const chapterStatusSchema = z.enum(["queued", "translating", "done", "error", "skipped"]);
export const autoGlossarySchema = z.enum(["inherit", "on", "off"]);

export const checkRuleSchema = z.object({ pattern: z.string(), flags: z.string().optional(), message: z.string() });

export const GENRE_SETTINGS = ["ancient", "modern", "mixed"] as const;
export const GENRE_NAMES = ["han", "foreign", "mixed"] as const;
export const GENRE_TONES = ["neutral", "romance", "witty", "punchy", "lyrical", "erotic", "youth"] as const;
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

/** `cast.json` — bảng nhân vật (giới tính) + mốc đổi xưng hô; file riêng cạnh story.json. */
/** `swap`: biến thân — raw đổi 他/她 theo câu, lời kể theo raw, check theo giới bỏ qua. */
export const genderSchema = z.enum(["male", "female", "swap"]);
export const entrySourceSchema = z.enum(["auto", "user"]);
export const castSchema = z.object({
  version: z.number().default(1),
  characters: z
    .record(
      z.string(),
      z.object({
        gender: genderSchema.optional(),
        source: entrySourceSchema.default("auto"),
        chapter: z.string().optional(),
        disputed: z.array(z.string()).optional(),
        /** Đại từ lời kể chốt riêng (lão, bà, Ngài…); trống thì theo giới. */
        narration: z.string().optional(),
      }),
    )
    .default({}),
  addressing: z
    .record(
      z.string(),
      z.object({
        pinned: z.boolean().default(false),
        changes: z
          .array(
            z.object({
              from: z.string(),
              target: z.string(),
              note: z.string().optional(),
              source: entrySourceSchema.default("auto"),
            }),
          )
          .default([]),
      }),
    )
    .default({}),
});
/** Rà ngược một mục glossary: đoạn đã dịch có khoá trong raw mà bản dịch thiếu cách dịch hiện tại. */
export const glossaryUsageSchema = z.object({
  total: z.number(),
  chapters: z.number(),
  skipped: z.number(),
  hits: z.array(z.object({ chapter: z.string(), paragraph: z.number(), text: z.string() })),
});
export const glossaryBackfillSchema = z.object({
  chapters: z.number(),
  paragraphs: z.number(),
  occurrences: z.number(),
  skipped: z.number(),
});
/** Trang Tìm & thay: kết quả tìm trong bản dịch đã chốt và hồ sơ truyện. */
export const replaceFindSchema = z.object({
  total: z.number(),
  chapters: z.array(z.object({ chapter: z.string(), count: z.number() })),
  hits: z.array(
    z.object({
      chapter: z.string(),
      paragraph: z.number(),
      occurrence: z.number(),
      before: z.string(),
      matched: z.string(),
      after: z.string(),
      raw: z.string().nullable(),
    }),
  ),
  skipped: z.number(),
  /** Chỗ khớp trong hồ sơ (Rust `ProfileHit`): `field` là ô chứa, `key` khoá Hán của mục (rỗng ở ô chữ trơn). */
  profile: z.array(
    z.object({
      field: z.string(),
      key: z.string(),
      index: z.number(),
      occurrence: z.number(),
      before: z.string(),
      matched: z.string(),
      after: z.string(),
      chapter: z.string().nullable(),
    }),
  ),
});
/** `profile`: số chỗ đã thay trong hồ sơ (thay nhiều cặp ở Kiểm tra VN luôn 0). */
export const replaceOutcomeSchema = z.object({ chapters: z.number(), occurrences: z.number(), profile: z.number() });
export const replaceUndoInfoSchema = z
  .object({
    find: z.string(),
    replace: z.string(),
    chapters: z.number(),
    occurrences: z.number(),
    profile: z.number(),
    createdAt: z.number(),
  })
  .nullable();
/** `profileConflicts`: tên file hồ sơ (story.json, cast.json, titles.json) đã sửa tiếp nên giữ nguyên. */
export const replaceUndoOutcomeSchema = z.object({
  restored: z.number(),
  conflicts: z.array(z.string()),
  profileRestored: z.number(),
  profileConflicts: z.array(z.string()),
});
export const scanCastViewSchema = z.object({
  asked: z.number(),
  filled: z.number(),
  failedBatches: z.number(),
  cast: castSchema,
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
  /** Độ lệch state.json ↔ out/ (Drive sync lệch) — chỉ có khi mở / quét lại / vừa đồng bộ; snapshot thường là null. */
  progressDrift: z.object({ untracked: z.number(), missing: z.number() }).nullable().optional(),
});

/** Chương chưa done mà out/ đã có bản dịch (untracked) và chương done mà chưa thấy bản dịch (missing). */
export const progressDriftSchema = z.object({ untracked: z.array(z.string()), missing: z.array(z.string()) });
export const requeueUntrackedResultSchema = z.object({ requeued: z.array(z.string()), snapshot: storySnapshotSchema });
export const syncProgressResultSchema = z.object({
  markedDone: z.array(z.string()),
  missing: z.array(z.string()),
  snapshot: storySnapshotSchema,
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
  /** `id` đưa vào `--model`; `label` chỉ hiển thị (agy 1.3 liệt kê `id<TAB>nhãn`). */
  models: z.array(z.object({ id: z.string(), label: z.string() })),
  message: z.string().nullable(),
});

/** Một tên tiêu đề lặp ở từ hai chương raw trở lên (Rust `titles::TitleGroup`). */
export const titleGroupSchema = z.object({
  source: z.string(),
  target: z.string(),
  settled: z.boolean(),
  chapters: z.number(),
  translated: z.number(),
  variants: z.array(z.object({ text: z.string(), count: z.number() })),
  pending: z.number(),
});
export const titleSurveySchema = z.object({ groups: z.array(titleGroupSchema), unparsed: z.number() });
export const titlesAppliedSchema = z.object({ changed: z.number(), survey: titleSurveySchema });

/** Hội thoại agy trên đĩa (Rust `agy_data_stats`). `dataDir` null = máy chưa từng chạy agy CLI. */
export const agyDataStatsSchema = z.object({
  dataDir: z.string().nullable(),
  appCount: z.number(),
  appBytes: z.number(),
  otherCount: z.number(),
  otherBytes: z.number(),
});
export const agyDataCleanedSchema = z.object({
  removed: z.number(),
  freedBytes: z.number(),
  kept: z.number(),
  stats: agyDataStatsSchema,
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
  /** Trang đọc đánh số từng đoạn ở tab Bản dịch và Gốc; mặc định bật. */
  lineNumbers: z.boolean().default(true),
  /** Phiên agy hết quota thì chờ tới giờ reset rồi tự dịch tiếp; mặc định tắt. */
  resumeOnQuota: z.boolean().default(false),
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
  /** Chương còn phải dịch (queued + translating); > 0 là truyện chưa dịch xong. */
  pending: z.number().nullable().optional(),
  /** Chỗ nhạy cảm VN chưa xét + cần xử lý; null = chưa quét. */
  vnFlags: z.number().nullable().optional(),
});

/** Kết quả quét raw/ của cả thư viện (Rust `library_rescan`). */
export const libraryRescanSchema = z.object({
  changed: z.array(z.object({ root: z.string(), name: z.string().nullable(), added: z.number(), removed: z.number() })),
  skippedRunning: z.array(z.string()),
  failed: z.array(z.string()),
  stories: z.array(recentSummarySchema),
});

/** Quét nhạy cảm VN cả thư viện (màn chọn truyện). */
export const vnLibraryScanSchema = z.object({
  scanned: z.number(),
  total: z.number(),
  failed: z.array(z.string()),
  cancelled: z.boolean(),
  stories: z.array(recentSummarySchema),
});
export const vnLibraryProgressSchema = z.object({ done: z.number(), total: z.number(), root: z.string() });

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
  /** agy hết quota và không bật tự chạy lại; `reset_at` là ms epoch lúc quota mở lại. */
  z.object({ kind: z.literal("quota_exhausted"), reset_at: z.number() }),
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
  z.object({ type: z.literal("stopped"), kind: z.literal("quota_exhausted"), reset_at: z.number() }),
  z.object({ type: z.literal("stopped"), kind: z.literal("internal"), message: z.string() }),
]);

export const sessionEventSchema = z.union([
  z.object({ type: z.literal("started"), session_no: z.number() }),
  progressSchema.extend({ type: z.literal("progress") }),
  z.object({ type: z.literal("agy_log"), line: z.string(), stream: z.enum(["stdout", "stderr"]) }),
  /** Hết quota agy: phiên chờ tới `until` (ms epoch) rồi tự chạy tiếp. */
  z.object({ type: z.literal("quota_wait"), until: z.number() }),
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
/** `book.json` của truyện: tác giả + nguồn ảnh bìa cho export EPUB. */
export const bookMetaSchema = z.object({
  author: z.string(),
  authorOriginal: z.string(),
  source: z.string(),
  coverUrl: z.string(),
  fetchedAt: z.number(),
});
export const bookInfoViewSchema = z.object({
  meta: bookMetaSchema,
  cover: z.array(z.number()).nullable(),
});
export const sourceImageSchema = z.object({ bytes: z.array(z.number()), mime: z.string() });
export const retryRangeOutcomeSchema = z.object({
  retried: z.array(z.string()),
  backedUp: z.array(z.string()),
  alreadyQueued: z.array(z.string()),
});
export const deleteOutcomeSchema = z.object({ removed: z.array(z.string()), keptOutputs: z.array(z.string()) });

export const VN_GROUPS = ["ten", "cq", "ls", "hd"] as const;
export const VN_STATUSES = ["pending", "ok", "action"] as const;
export const VN_LABELS = ["xucpham", "nhaycam", "trungtinh", "khonglienquan"] as const;
export const vnStatusSchema = z.enum(VN_STATUSES);
export const vnEntrySchema = z.object({
  id: z.string(),
  group: z.enum(VN_GROUPS),
  level: z.enum(["mietthi", "canxem"]),
  weak: z.boolean().default(false),
  zh: z.array(z.string()).default([]),
  vi: z.array(z.string()).default([]),
  exclude: z.array(z.string()).default([]),
});
export const vnReplacementSchema = z.object({
  from: z.string(),
  to: z.string(),
  setting: z.enum(["ancient", "modern", "any"]),
});
export const vnSettingsSchema = z.object({
  extra: z.array(vnEntrySchema).default([]),
  disabled: z.array(z.string()).default([]),
  replacements: z.array(vnReplacementSchema).default([]),
});
export const vnHitViewSchema = z.object({
  id: z.string(),
  chapter: z.string(),
  paragraph: z.number(),
  entry: z.string(),
  group: z.enum(VN_GROUPS),
  level: z.enum(["mietthi", "canxem"]),
  side: z.enum(["raw", "out"]),
  matched: z.string(),
  count: z.number(),
  fingerprint: z.string(),
  status: vnStatusSchema,
  ai: z
    .object({
      label: z.enum(VN_LABELS),
      reason: z.string(),
      /** Dịch nhanh các đoạn raw khi chương chưa dịch; rỗng nếu đã có bản dịch (hoặc nhãn lưu từ bản cũ). */
      translation: z.string().optional().default(""),
      suggestion: z.string().optional().default(""),
      model: z.string(),
      at: z.number(),
    })
    .nullable()
    .optional(),
  raw: z.array(z.string()),
  out: z.array(z.string()),
  focus: z.number(),
});
export const vnSummarySchema = z.object({ hits: z.number(), open: z.number(), scanned: z.number() });
export const vnVerifyOutcomeSchema = z.object({ verified: z.number(), failed: z.number(), error: z.string().nullable().optional() });
