import { describe, expect, it } from "vitest";

import { diffStoryConfig, fromFormValues, storyFormSchema, toFormValues } from "@/lib/story-form";
import type { StoryConfig } from "@/lib/types";

const base: StoryConfig = {
  name: "A",
  sourceUrl: "https://x",
  protagonist: "B",
  summary: "S",
  glossary: {
    names: { 赵静文: "Triệu Tĩnh Văn" },
    places: {},
    items: {},
    creatures: {},
    skills: {},
    common: {},
    signature_phrases: { 哼: "Hừ" },
  },
  style: {
    voice: "lạnh",
    toneRules: ["ta/ngươi", "không mình/tôi"],
    signaturePhrases: { 方寸: "Phương Thốn" },
    avoid: ["anh ấy"],
  },
  customPrompt: "",
  checkRules: [{ pattern: "x", flags: "i", message: "m" }],
  autoGlossaryLog: [{ source: "赵静文", target: "Triệu Tĩnh Văn", category: "names", chapter: "0001" }],
  autoGlossary: "on",
};

describe("story-form", () => {
  it("round-trip config → form → config giữ nguyên, kể cả autoGlossaryLog", () => {
    const values = toFormValues(base);
    expect(values.toneRules).toBe("ta/ngươi\nkhông mình/tôi");
    expect(values.glossary.names).toEqual([{ source: "赵静文", target: "Triệu Tĩnh Văn" }]);
    expect(values.checkRules[0]?.flags).toBe("i");
    expect(storyFormSchema.safeParse(values).success).toBe(true);
    expect(fromFormValues(values, base)).toEqual(base);
  });

  it("dòng trống / cặp rỗng / rule thiếu pattern bị bỏ; flags rỗng thành undefined", () => {
    const values = toFormValues(base);
    values.toneRules = "a\n\n  \nb";
    values.glossary.places = [
      { source: " ", target: "x" },
      { source: "高塔", target: "Cao Tháp" },
    ];
    values.checkRules = [
      { pattern: "", flags: "", message: "m" },
      { pattern: "y", flags: "", message: "n" },
    ];
    const config = fromFormValues(values, base);
    expect(config.style.toneRules).toEqual(["a", "b"]);
    expect(config.glossary.places).toEqual({ 高塔: "Cao Tháp" });
    expect(config.checkRules).toEqual([{ pattern: "y", message: "n" }]);
  });

  it("diff chỉ liệt kê field đổi", () => {
    const after = {
      ...base,
      name: "A2",
      glossary: { ...base.glossary, names: { ...base.glossary.names, 慕容: "Mộ Dung" } },
    };
    const diff = diffStoryConfig(base, after);
    expect(diff.map((d) => d.field)).toEqual(["name", "glossary.names"]);
    expect(diff[1]?.after).toContain("慕容");
  });
});
