import { describe, expect, it } from "vitest";

import { appConfigSchema, sessionEventSchema, storyConfigSchema, storySnapshotSchema } from "@/lib/schema";

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

  it("appConfig cũ thiếu engine/api ra agy và api mặc định; engine api giữ key theo provider", () => {
    const parsed = appConfigSchema.parse({ agyPath: null, model: null, maxSessions: 50, recent: [] });
    expect(parsed.engine).toBe("agy");
    expect(parsed.api).toEqual({
      provider: "gemini",
      gemini: { apiKey: "", model: "gemini-3.7-flash", baseUrl: "" },
      openai: { apiKey: "", model: "gpt-5.6-sol", baseUrl: "" },
      thinking: true,
      reasoningEffort: "high",
    });
    const api = appConfigSchema.parse({
      engine: "api",
      api: { provider: "openai", openai: { apiKey: "sk-hub", baseUrl: "http://192.0.2.10/v1" }, reasoningEffort: "xhigh" },
      agyPath: null,
      model: null,
      maxSessions: 50,
      recent: [],
    });
    expect(api.engine).toBe("api");
    expect(api.api.openai).toEqual({ apiKey: "sk-hub", model: "", baseUrl: "http://192.0.2.10/v1" });
    expect(api.api.gemini.model).toBe("gemini-3.7-flash");
    expect(() => appConfigSchema.parse({ engine: "cloud", agyPath: null, model: null, maxSessions: 1, recent: [] })).toThrow();
    expect(() =>
      appConfigSchema.parse({ api: { reasoningEffort: "ultra" }, agyPath: null, model: null, maxSessions: 1, recent: [] }),
    ).toThrow();
  });

  it("parse stopped api_failed", () => {
    expect(sessionEventSchema.parse({ type: "stopped", kind: "api_failed", message: "401" })).toEqual({
      type: "stopped",
      kind: "api_failed",
      message: "401",
    });
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
