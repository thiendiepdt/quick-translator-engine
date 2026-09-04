import { describe, expect, it } from "vitest";

import {
  emptyAiStoryConfig,
  normalizeAiStoryConfig,
  normalizeAiTranslationChapters,
  parseAiStoryConfigJson,
} from "@/lib/ai-story";

describe("AI story workspace data", () => {
  it("normalizes Novel Translator glossary/style shapes", () => {
    const story = normalizeAiStoryConfig({
      name: "Đấu Phá Thương Khung",
      glossary: { names: { 萧炎: "Tiêu Viêm" } },
      style: {
        voice: "Gọn và trực diện",
        tone_rules: ["Không tô màu"],
        signature_phrases: { 莫欺少年穷: "Chớ khinh thiếu niên nghèo" },
      },
      checkRules: [{ pattern: "convert", flags: "i", message: "Còn văn convert" }],
    });

    expect(story.glossary.names).toEqual({ 萧炎: "Tiêu Viêm" });
    expect(story.glossary.places).toEqual({});
    expect(story.style.toneRules).toEqual(["Không tô màu"]);
    expect(story.style.signaturePhrases).toEqual({
      莫欺少年穷: "Chớ khinh thiếu niên nghèo",
    });
    expect(story.checkRules).toEqual([
      { pattern: "convert", flags: "i", message: "Còn văn convert" },
    ]);
  });

  it("recovers interrupted chapters as queued without losing work", () => {
    const chapters = normalizeAiTranslationChapters([
      {
        id: "c1",
        filename: "chuong-1.txt",
        source: "原文",
        output: "Bản dịch đang dở",
        thinking: "",
        violations: [],
        status: "reviewing",
        reviewRound: 2,
        updatedAt: 10,
      },
      {
        id: "c2",
        filename: "chuong-2.txt",
        source: "原文二",
        output: "Bản dịch hoàn chỉnh",
        thinking: "",
        violations: [],
        status: "done",
        reviewRound: 1,
        updatedAt: 20,
      },
    ]);

    expect(chapters[0]).toMatchObject({
      status: "queued",
      output: "Bản dịch đang dở",
      reviewRound: 2,
    });
    expect(chapters[1].status).toBe("done");
  });

  it("provides a complete empty schema for older workspaces", () => {
    expect(normalizeAiStoryConfig(undefined)).toEqual(emptyAiStoryConfig());
  });

  it("defaults genre to ancient/han and drops unknown values", () => {
    expect(normalizeAiStoryConfig({}).genre).toEqual({ setting: "ancient", names: "han" });
    expect(normalizeAiStoryConfig({ genre: { setting: "modern", names: "foreign" } }).genre).toEqual({
      setting: "modern",
      names: "foreign",
    });
    expect(normalizeAiStoryConfig({ genre: { setting: "future", names: 3 } }).genre).toEqual({
      setting: "ancient",
      names: "han",
    });
    expect(emptyAiStoryConfig().genre).toEqual({ setting: "ancient", names: "han" });
  });
});

describe("auto glossary log", () => {
  it("starts empty and survives normalization", () => {
    expect(emptyAiStoryConfig().autoGlossaryLog).toEqual([]);
    const normalized = normalizeAiStoryConfig({
      ...emptyAiStoryConfig(),
      autoGlossaryLog: [
        { source: "震雷子", target: "Chấn Lôi Tử", category: "names", chapter: "chuong-163.txt" },
        { source: "", target: "x", category: "names", chapter: "c" },
        { source: "y", target: "Y", category: "bogus", chapter: "c" },
      ],
    });
    expect(normalized.autoGlossaryLog).toEqual([
      { source: "震雷子", target: "Chấn Lôi Tử", category: "names", chapter: "chuong-163.txt" },
    ]);
  });

  it("defaults to empty for legacy stored configs", () => {
    expect(normalizeAiStoryConfig({ name: "Cũ" }).autoGlossaryLog).toEqual([]);
  });
});

describe("per-story auto glossary setting", () => {
  it("defaults to inherit, including for legacy stored configs", () => {
    expect(emptyAiStoryConfig().autoGlossary).toBe("inherit");
    expect(normalizeAiStoryConfig({ name: "Cũ" }).autoGlossary).toBe("inherit");
    expect(normalizeAiStoryConfig({ autoGlossary: "bogus" }).autoGlossary).toBe("inherit");
  });

  it("keeps explicit on/off", () => {
    expect(normalizeAiStoryConfig({ autoGlossary: "on" }).autoGlossary).toBe("on");
    expect(normalizeAiStoryConfig({ autoGlossary: "off" }).autoGlossary).toBe("off");
  });
});

describe("parseAiStoryConfigJson", () => {
  it("parses and normalizes an exported config", () => {
    const text = JSON.stringify({ name: "Đấu Phá", autoGlossary: "off" });
    const story = parseAiStoryConfigJson(text);
    expect(story?.name).toBe("Đấu Phá");
    expect(story?.autoGlossary).toBe("off");
    expect(story?.glossary.names).toEqual({});
  });

  it("returns undefined for invalid JSON or non-object payloads", () => {
    expect(parseAiStoryConfigJson("{hỏng")).toBeUndefined();
    expect(parseAiStoryConfigJson('"chuỗi"')).toBeUndefined();
    expect(parseAiStoryConfigJson("[1,2]")).toBeUndefined();
  });
});
