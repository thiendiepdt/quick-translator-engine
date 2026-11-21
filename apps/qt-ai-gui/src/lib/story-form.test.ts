import { describe, expect, it } from "vitest";

import { storyConfigSchema } from "@/lib/schema";
import { diffStoryConfig, fromFormValues, toFormValues } from "@/lib/story-form";

const base = storyConfigSchema.parse({
  name: "A",
  sourceUrl: "",
  protagonist: "",
  summary: "",
  genre: { setting: "modern", names: "mixed" },
  glossary: { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {}, addressing: {} },
  style: { voice: "", toneRules: [], signaturePhrases: {}, avoid: [] },
  customPrompt: "",
  checkRules: [],
  autoGlossaryLog: [],
  autoGlossary: "inherit",
});

describe("story-form genre", () => {
  it("round-trip genre qua form", () => {
    const values = toFormValues(base);
    expect(values.genreSetting).toBe("modern");
    expect(values.genreNames).toBe("mixed");
    expect(values.genreTone).toBe("neutral"); // schema default khi story.json cũ không có tone
    expect(fromFormValues({ ...values, genreNames: "foreign", genreTone: "romance" }, base).genre).toEqual({
      setting: "modern",
      names: "foreign",
      tone: "romance",
    });
  });

  it("ngôi kể: chưa chọn thì không ghi field, chọn thì ghi vào genre.pov", () => {
    const values = toFormValues(base);
    expect(values.genrePov).toBe("unset");
    expect(fromFormValues(values, base).genre).not.toHaveProperty("pov");
    expect(fromFormValues({ ...values, genrePov: "first-toi" }, base).genre.pov).toBe("first-toi");
    const chosen = storyConfigSchema.parse({ ...base, genre: { ...base.genre, pov: "third" } });
    expect(toFormValues(chosen).genrePov).toBe("third");
    expect(() => storyConfigSchema.parse({ ...base, genre: { ...base.genre, pov: "ngôi hai" } })).toThrow();
  });

  it("diff liệt kê genre khi đổi", () => {
    const after = { ...base, genre: { setting: "ancient" as const, names: "han" as const, tone: "neutral" as const } };
    expect(diffStoryConfig(base, after).map((d) => d.field)).toEqual(["genre"]);
  });

  it("schema bắt buộc genre hợp lệ", () => {
    expect(() => storyConfigSchema.parse({ ...base, genre: { setting: "x", names: "han" } })).toThrow();
  });
});

describe("parseSettingArcs", () => {
  it("nhận số chương + bối cảnh tiếng Anh hoặc tiếng Việt, bỏ dòng sai", async () => {
    const { parseSettingArcs } = await import("@/lib/story-form");
    expect(parseSettingArcs("0843 cổ đại\n0868 modern\nrác\n0900 không rõ")).toEqual([
      { from: "0843", setting: "ancient" },
      { from: "0868", setting: "modern" },
    ]);
  });
});
