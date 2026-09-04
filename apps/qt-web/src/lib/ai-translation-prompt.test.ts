import { describe, expect, it } from "vitest";

import { defaultStoryGenre } from "@/lib/ai-story";
import {
  composeBasePrompt,
  genreKey,
  LEGACY_BASE_PROMPT_FNV1A64,
  PROMPT_GENRE_COMBOS,
} from "@/lib/ai-translation-prompt";

/** FNV-1a 64-bit trên UTF-8 — không cần crypto của môi trường, đủ để chốt "không đổi một byte". */
function fnv1a64(text: string): string {
  let hash = 0xcbf29ce484222325n;
  for (const byte of new TextEncoder().encode(text)) {
    hash ^= BigInt(byte);
    hash = (hash * 0x100000001b3n) & 0xffffffffffffffffn;
  }
  return hash.toString(16).padStart(16, "0");
}

describe("composeBasePrompt", () => {
  it("ancient/han bằng đúng từng byte prompt cũ", () => {
    // Hash của NOVEL_TRANSLATOR_BASE_PROMPT trước khi tách — chốt không đổi hành vi truyện cũ.
    expect(fnv1a64(composeBasePrompt(defaultStoryGenre()))).toBe(LEGACY_BASE_PROMPT_FNV1A64);
  });

  it("đánh số liền mạch hai danh sách ở mọi tổ hợp", () => {
    for (const genre of PROMPT_GENRE_COMBOS) {
      const prompt = composeBasePrompt(genre);
      const constraints = prompt
        .split("# Quy tắc dịch thuật")[0]
        .match(/^\d+\. /gm)!
        .map((m) => Number.parseInt(m, 10));
      expect(constraints, genreKey(genre)).toEqual(constraints.map((_, i) => i + 1));
      const editing = prompt
        .split("## 8. Biên tập")[1]
        .match(/^\d+\. /gm)!
        .map((m) => Number.parseInt(m, 10));
      expect(editing, genreKey(genre)).toEqual(editing.map((_, i) => i + 1));
    }
  });

  it("modern bỏ xưng hô cổ, cho vợ/chồng; foreign trả tên về gốc", () => {
    const modern = composeBasePrompt({ setting: "modern", names: "han" });
    expect(modern).toContain("| 他          | **anh** / **anh ta** / **hắn**");
    expect(modern).not.toContain('KHÔNG dùng "vợ", "chồng"');
    expect(modern).not.toContain("### Tu tiên / Xianxia");
    expect(modern).toContain("Kế Duyên");
    const foreign = composeBasePrompt({ setting: "ancient", names: "foreign" });
    expect(foreign).toContain("艾米丽");
    expect(foreign).not.toContain("| 计缘   | Kế Duyên");
    expect(foreign).toContain("### Tu tiên / Xianxia");
    const mixed = composeBasePrompt({ setting: "modern", names: "mixed" });
    expect(mixed).toContain("Bách gia tính");
  });

  it("mixed có cả hai bộ xưng hô, hai bảng thuật ngữ", () => {
    const mixed = composeBasePrompt({ setting: "mixed", names: "han" });
    expect(mixed).toContain("| 我          | **ta**");
    expect(mixed).toContain("| 他          | **anh** / **anh ta** / **hắn**");
    expect(mixed).toContain("### Tu tiên / Xianxia");
    expect(mixed).toContain("### Đô thị / Hiện đại");
    expect(mixed).toContain("theo cảnh");
  });

  it("9 tổ hợp cho 9 chuỗi khác nhau", () => {
    const set = new Set(PROMPT_GENRE_COMBOS.map((g) => composeBasePrompt(g)));
    expect(set.size).toBe(9);
    expect(PROMPT_GENRE_COMBOS.map(genreKey)).toEqual([
      "ancient/han",
      "ancient/foreign",
      "ancient/mixed",
      "modern/han",
      "modern/foreign",
      "modern/mixed",
      "mixed/han",
      "mixed/foreign",
      "mixed/mixed",
    ]);
  });
});
