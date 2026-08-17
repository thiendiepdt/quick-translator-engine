import { describe, expect, it } from "vitest";

import {
  buildAiTranslationReviewPrompt,
  buildAiTranslationSystemPrompt,
  buildWorkspaceTranslationGlossary,
  checkAiTranslationViolations,
  formatAiTranslation,
  wordCount,
} from "@/lib/ai-translation";
import { NOVEL_TRANSLATOR_BASE_PROMPT } from "@/lib/ai-translation-prompt";
import { emptyAiStoryConfig } from "@/lib/ai-story";
import { dictionaryUpdateKeys, type LocalDictionaryEntries } from "@/lib/types";

function emptyEntries(): LocalDictionaryEntries {
  return Object.fromEntries(
    dictionaryUpdateKeys.map((key) => [key, {}]),
  ) as LocalDictionaryEntries;
}

describe("AI translation prompt", () => {
  it("stores the ported prompt with real Markdown line breaks", () => {
    expect(NOVEL_TRANSLATOR_BASE_PROMPT).toContain("\n# Ngữ cảnh tác vụ chuyển ngữ\n");
    expect(NOVEL_TRANSLATOR_BASE_PROMPT).not.toContain("\\n# Ngữ cảnh tác vụ chuyển ngữ");
  });

  it("uses only compact workspace entries and accepted names as its glossary", () => {
    const entries = emptyEntries();
    entries.names = { 药老: "Dược Lão" };
    entries.vietPhrase = { 斗气: "đấu khí" };
    entries.hauTu = { 宗: "tông" };

    const glossary = buildWorkspaceTranslationGlossary(entries, { 萧炎: "Tiêu Viêm" });
    expect(glossary).toEqual({
      names: { 药老: "Dược Lão", 萧炎: "Tiêu Viêm" },
      viet_phrase: { 斗气: "đấu khí" },
    });
    const prompt = buildAiTranslationSystemPrompt(glossary);
    expect(prompt).toContain("chuyển ngữ trung thành");
    expect(prompt).toContain("树倒猢狲散");
    expect(prompt).toContain('"萧炎": "Tiêu Viêm"');
    expect(prompt).toContain("Chỉ xuất bản dịch tiếng Việt");
  });

  it("adds story metadata, glossary, style and a custom prompt", () => {
    const story = emptyAiStoryConfig();
    story.name = "Đấu Phá Thương Khung";
    story.protagonist = "Tiêu Viêm";
    story.glossary.names = { 萧炎: "Tiêu Viêm bản truyện" };
    story.style.voice = "Gọn, không tô màu";
    story.customPrompt = "PROMPT RIÊNG CỦA TRUYỆN";

    const prompt = buildAiTranslationSystemPrompt(
      { names: { 萧炎: "Tiêu Viêm workspace" } },
      story,
    );

    expect(prompt).toContain("PROMPT RIÊNG CỦA TRUYỆN");
    expect(prompt).toContain('"name": "Đấu Phá Thương Khung"');
    expect(prompt).toContain('"萧炎": "Tiêu Viêm bản truyện"');
    expect(prompt).toContain("Gọn, không tô màu");
  });
});

describe("AI translation post-processing", () => {
  it("normalizes paragraph spacing and a trailing newline", () => {
    expect(formatAiTranslation("Đoạn một  \n\n\nĐoạn hai\n")).toBe(
      "Đoạn một\n\nĐoạn hai\n",
    );
  });

  it("finds the ported automatic violations and builds a narrow review prompt", () => {
    const text = "But hắn còn nói…\n萧炎 bước vào】";
    const violations = checkAiTranslationViolations(text);
    expect(violations.map((item) => item.message)).toEqual(
      expect.arrayContaining([
        "Từ nối tiếng Anh lọt vào bản dịch → dịch sang tiếng Việt hoặc chỉ giữ khi có căn cứ",
        "Còn ký tự … → chuẩn hóa thành dấu chấm ASCII, giữ số lượng (… → ..., …… → ......)",
        "CJK còn sót (chưa dịch hết!)",
      ]),
    );
    const review = buildAiTranslationReviewPrompt(text, violations);
    expect(review.user).toContain("chỉ thay từ hoặc cụm gây lỗi");
    expect(review.user).toContain("But hắn còn nói");
    expect(review.system).toContain("không đổi văn phong");
  });

  it("uses configured story rules in place of defaults", () => {
    const violations = checkAiTranslationViolations("Vẫn còn văn convert", [
      { pattern: "CONVERT", flags: "i", message: "Rule riêng" },
    ]);
    expect(violations).toEqual([
      { line: 1, message: "Rule riêng", text: "Vẫn còn văn convert" },
    ]);
  });

  it("always flags leftover Han characters even with custom rules", () => {
    const violations = checkAiTranslationViolations("Hắn đọc 㐀 trong bí tịch", [
      { pattern: "CONVERT", flags: "i", message: "Rule riêng" },
    ]);
    expect(violations.map((item) => item.message)).toEqual([
      "CJK còn sót (chưa dịch hết!)",
    ]);
  });

  it("flags fullwidth Chinese punctuation by default", () => {
    const violations = checkAiTranslationViolations("Hắn gật đầu，rồi rời đi。");
    expect(violations.map((item) => item.message)).toContain(
      "Dấu câu tiếng Trung còn sót → dùng dấu câu thường",
    );
  });
});

describe("wordCount", () => {
  it("counts whitespace-separated words", () => {
    expect(wordCount("hắn vô cùng cao hứng")).toBe(5);
  });

  it("ignores blank lines and extra whitespace", () => {
    expect(wordCount("một  hai\n\nba\n")).toBe(3);
  });

  it("returns zero for empty text", () => {
    expect(wordCount("")).toBe(0);
    expect(wordCount("  \n ")).toBe(0);
  });
});
