import { describe, expect, it } from "vitest";

import {
  emptyAiStoryConfig,
  normalizeAiStoryConfig,
  normalizeAiTranslationChapters,
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
});
