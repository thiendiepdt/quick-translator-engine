import { describe, expect, it } from "vitest";

import { storyConfigSchema } from "@/lib/schema";
import { diffStoryConfig, fromFormValues, toFormValues } from "@/lib/story-form";

const base = storyConfigSchema.parse({
  name: "A",
  sourceUrl: "",
  protagonist: "",
  summary: "",
  genre: { setting: "modern", names: "mixed" },
  glossary: { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {} },
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
    expect(fromFormValues({ ...values, genreNames: "foreign" }, base).genre).toEqual({
      setting: "modern",
      names: "foreign",
    });
  });

  it("diff liệt kê genre khi đổi", () => {
    const after = { ...base, genre: { setting: "ancient" as const, names: "han" as const } };
    expect(diffStoryConfig(base, after).map((d) => d.field)).toEqual(["genre"]);
  });

  it("schema bắt buộc genre hợp lệ", () => {
    expect(() => storyConfigSchema.parse({ ...base, genre: { setting: "x", names: "han" } })).toThrow();
  });
});
