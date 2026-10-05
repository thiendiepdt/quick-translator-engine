import { describe, expect, it } from "vitest";

import { appConfigSchema, sessionEventSchema, storyConfigSchema, storySnapshotSchema } from "@/lib/schema";
import { clockLabel, stopReasonLabel } from "@/lib/types";

const story = {
  name: "A",
  sourceUrl: "",
  protagonist: "",
  summary: "",
  genre: { setting: "ancient", names: "han" },
  glossary: {
    names: { 赵静文: "Triệu Tĩnh Văn" },
    places: {},
    items: {},
    creatures: {},
    skills: {},
    common: {},
    signature_phrases: {},
    addressing: {},
  },
  style: { voice: "", toneRules: [], signaturePhrases: {}, avoid: [] },
  customPrompt: "",
  checkRules: [{ pattern: "x", message: "m" }],
  autoGlossaryLog: [],
  autoGlossary: "inherit",
};

describe("schema", () => {
  it("parse snapshot từ Rust", () => {
    const snap = storySnapshotSchema.parse({
      root: "D:\\t",
      chapters: [{ id: "0001", status: "done", reviewRound: 1, reason: null, warnings: ["[[1]] x"] }],
      counts: { total: 1, queued: 0, translating: 0, done: 1, error: 0, skipped: 0, withWarnings: 1 },
      settings: { minLengthRatio: 0.75, maxReviewRounds: 3, chaptersPerSession: 10 },
      story,
      sessionRunning: false,
    });
    expect(snap.chapters[0]?.reason).toBeNull();
    expect(snap.story.checkRules[0]?.flags).toBeUndefined();
  });

  it("appConfig cũ thiếu palette/themeMode vẫn parse với default", () => {
    const parsed = appConfigSchema.parse({ agyPath: null, model: null, maxSessions: 50, recent: [] });
    expect(parsed.palette).toBe("editorial");
    expect(parsed.themeMode).toBe("system");
    expect(parsed.readingWidth).toBe("normal");
  });

  it("appConfig thiếu engine/api ra api (người mới không cần agy) và api mặc định; engine api giữ key theo provider", () => {
    const parsed = appConfigSchema.parse({ agyPath: null, model: null, maxSessions: 50, recent: [] });
    expect(parsed.engine).toBe("api");
    expect(parsed.libraryRoot).toBeNull();
    const effort = { translate: "high", review: "high", glossary: "low", fill: "high" };
    expect(parsed.api).toEqual({
      provider: "gemini",
      gemini: { apiKey: "", model: "gemini-3.7-flash", baseUrl: "", effort },
      openai: { apiKey: "", model: "gpt-5.6-sol", baseUrl: "", effort },
    });
    // Config cũ còn thinking/reasoningEffort chung: zod bỏ qua (Rust đã chuyển sang effort khi load).
    const api = appConfigSchema.parse({
      engine: "api",
      api: {
        provider: "openai",
        openai: { apiKey: "sk-hub", baseUrl: "http://192.0.2.10/v1", effort: { review: "xhigh" } },
        thinking: false,
        reasoningEffort: "xhigh",
      },
      agyPath: null,
      model: null,
      maxSessions: 50,
      recent: [],
    });
    expect(api.engine).toBe("api");
    expect(api.api.openai).toEqual({
      apiKey: "sk-hub",
      model: "",
      baseUrl: "http://192.0.2.10/v1",
      effort: { ...effort, review: "xhigh" },
    });
    expect(api.api.gemini.model).toBe("gemini-3.7-flash");
    expect("thinking" in api.api).toBe(false);
    expect(() => appConfigSchema.parse({ engine: "cloud", agyPath: null, model: null, maxSessions: 1, recent: [] })).toThrow();
    const bad = (api: unknown) => appConfigSchema.parse({ api, agyPath: null, model: null, maxSessions: 1, recent: [] });
    expect(() => bad({ openai: { effort: { translate: "ultra" } } })).toThrow();
    expect(() => bad({ gemini: { effort: { translate: "xhigh" } } })).toThrow(); // xhigh chỉ có ở OpenAI
    expect(bad({ gemini: { effort: { glossary: "low" } } }).api.gemini.effort.glossary).toBe("low");
  });

  it("parse quota_wait, stopped quota_exhausted và config resumeOnQuota mặc định tắt", () => {
    expect(sessionEventSchema.parse({ type: "quota_wait", until: 1791229542000 })).toEqual({
      type: "quota_wait",
      until: 1791229542000,
    });
    expect(sessionEventSchema.parse({ type: "stopped", kind: "quota_exhausted", reset_at: 1791229482000 })).toEqual({
      type: "stopped",
      kind: "quota_exhausted",
      reset_at: 1791229482000,
    });
    const config = appConfigSchema.parse({ agyPath: null, model: null, maxSessions: 50, recent: [] });
    expect(config.resumeOnQuota).toBe(false);
    expect(config.lineNumbers).toBe(true);
    // Cùng ngày chỉ hiện giờ; khác ngày kèm ngày/tháng.
    const at = new Date(2026, 9, 6, 2, 45).getTime();
    expect(clockLabel(at, new Date(2026, 9, 6, 0, 10).getTime())).toBe("02:45");
    expect(clockLabel(at, new Date(2026, 9, 5, 23, 50).getTime())).toBe("02:45 ngày 06/10");
    expect(stopReasonLabel({ kind: "quota_exhausted", reset_at: at })).toContain("Hết quota agy — quota mở lại lúc 02:45");
  });

  it("parse stopped api_failed", () => {
    expect(sessionEventSchema.parse({ type: "stopped", kind: "api_failed", message: "401" })).toEqual({
      type: "stopped",
      kind: "api_failed",
      message: "401",
    });
  });

  it("story.json cũ thiếu glossary.addressing vẫn parse với {}", () => {
    const { addressing: _dropped, ...legacyGlossary } = story.glossary;
    const parsed = storyConfigSchema.parse({ ...story, glossary: legacyGlossary });
    expect(parsed.glossary.addressing).toEqual({});
  });

  it("từ chối status lạ", () => {
    expect(() => storyConfigSchema.parse({ ...story, autoGlossary: "lạ" })).toThrow();
  });

  it("parse session event dạng tagged", () => {
    expect(sessionEventSchema.parse({ type: "started", session_no: 1 })).toEqual({ type: "started", session_no: 1 });
    expect(sessionEventSchema.parse({ type: "agy_log", line: "x", stream: "stdout" }).type).toBe("agy_log");
    expect(sessionEventSchema.parse({ type: "stopped", kind: "agy_failed", code: 3 })).toEqual({
      type: "stopped",
      kind: "agy_failed",
      code: 3,
    });
    expect(sessionEventSchema.parse({ type: "stopped", kind: "internal", message: "x" })).toEqual({
      type: "stopped",
      kind: "internal",
      message: "x",
    });
    const progress = sessionEventSchema.parse({
      type: "progress",
      done: 1,
      queued: 2,
      translating: 0,
      error: 0,
      skipped: 0,
      warnings_count: 0,
      current: null,
    });
    expect(progress.type).toBe("progress");
  });
});
