import { describe, expect, it } from "vitest";

import { defaultStoryGenre } from "@/lib/ai-story";
import {
  composeBasePrompt,
  composeTonePrompt,
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

  it("tone romance chèn mục Giọng văn ngôn tình ngay trước Đại từ nhân xưng; neutral không đổi", () => {
    const romance = composeBasePrompt({ ...defaultStoryGenre(), tone: "romance" });
    const neutral = composeBasePrompt(defaultStoryGenre());
    expect(neutral).not.toContain("## Giọng văn: ngôn tình");
    const at = romance.indexOf("## Giọng văn: ngôn tình (truyện nữ)");
    expect(at).toBeGreaterThan(romance.indexOf("## 0. Ràng buộc trung thành"));
    expect(at).toBeLessThan(romance.indexOf("## 1. Đại từ nhân xưng"));
    // Chèn đúng một khối, phần còn lại y hệt neutral.
    expect(romance.replace(`${composeTonePrompt("romance")}\n`, "")).toBe(neutral);
    expect(romance).toContain("| 这么香艳，这么刺激的么？ |");
    expect(composeTonePrompt("neutral")).toBe("");
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
    const modern = composeBasePrompt({ setting: "modern", names: "han", tone: "neutral" });
    expect(modern).toContain("| 他          | **hắn** (lời kể ngôi ba, mọi nhân vật)");
    expect(modern).toContain("lời kể ngôi ba dùng `hắn` cho nhân vật nam và `cô` cho nhân vật nữ");
    expect(modern).not.toContain("hắn chỉ cho nhân vật lạnh");
    // Người kể ngôi một tự xưng `ta` như convert; `tôi` chỉ trong thoại theo quan hệ.
    expect(modern).toContain("| 我          | **ta** (lời kể ngôi một)");
    expect(modern).not.toContain("| 我          | **tôi**");
    expect(modern).toContain("`我` trong lời kể ngôi thứ nhất dùng `ta`");
    expect(modern).not.toContain("| 男人 / 男子 / 男的 | nam nhân / nam tử");
    const ancientHan = composeBasePrompt({ setting: "ancient", names: "han", tone: "neutral" });
    expect(ancientHan).toContain("| 男人 / 男子 / 男的 | nam nhân / nam tử");
    expect(ancientHan).toContain("`Nam nhân khoác hắc bào`");
    expect(modern).not.toContain('KHÔNG dùng "vợ", "chồng"');
    expect(modern).not.toContain("### Tu tiên / Xianxia");
    expect(modern).toContain("Kế Duyên");
    const foreign = composeBasePrompt({ setting: "ancient", names: "foreign", tone: "neutral" });
    expect(foreign).toContain("艾米丽");
    expect(foreign).not.toContain("| 计缘   | Kế Duyên");
    expect(foreign).toContain("### Tu tiên / Xianxia");
    const mixed = composeBasePrompt({ setting: "modern", names: "mixed", tone: "neutral" });
    expect(mixed).toContain("Bách gia tính");
  });

  it("không còn dấu vết truyện riêng; có kính ngữ, lóng mạng, văn bản ngoài truyện, tượng thanh", () => {
    for (const genre of PROMPT_GENRE_COMBOS) {
      const prompt = composeBasePrompt(genre);
      expect(prompt, genreKey(genre)).not.toMatch(/方寸|Phương Thốn|猫影无踪|周寻真/);
      expect(prompt, genreKey(genre)).toContain("### Kính ngữ và hậu tố tên");
      expect(prompt, genreKey(genre)).toContain("**Văn bản ngoài truyện:**");
      expect(prompt, genreKey(genre)).toContain("### Từ tượng thanh");
      expect(prompt, genreKey(genre)).toContain("Ngoặc thoại `「」`");
      // Ý cấm từ nối tiếng Anh chỉ còn hai chỗ: ràng buộc hệ thống và biên tập cuối.
      expect(prompt.match(/`But`/g)?.length, genreKey(genre)).toBe(2);
    }
    const ancient = composeBasePrompt({ setting: "ancient", names: "han", tone: "neutral" });
    expect(ancient).toContain("| X哥 / X姐 | X ca / X tỷ |");
    expect(ancient).not.toContain("### Tiếng lóng mạng");
    const modern = composeBasePrompt({ setting: "modern", names: "han", tone: "neutral" });
    expect(modern).toContain("| X哥 / X姐 | anh X / chị X |");
    expect(modern).toContain("| 吃瓜 | hóng chuyện / hóng drama |");
    expect(modern).not.toContain("| X兄 / X弟 / X妹 |");
    const mixed = composeBasePrompt({ setting: "mixed", names: "han", tone: "neutral" });
    expect(mixed).toContain("| X哥 / X姐 | X ca / X tỷ |");
    expect(mixed).toContain("| 吃瓜 |");
  });

  it("mixed có cả hai bộ xưng hô, hai bảng thuật ngữ", () => {
    const mixed = composeBasePrompt({ setting: "mixed", names: "han", tone: "neutral" });
    expect(mixed).toContain("| 我          | **ta**");
    expect(mixed).toContain("| 他          | **hắn** (lời kể ngôi ba, mọi nhân vật)");
    expect(mixed).toContain("lời kể `hắn`/`cô`, người kể ngôi một `ta`, trong thoại `tôi`/`anh`/`em`/`cậu`");
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

  it("mọi genre: tiêu đề chương theo định dạng `Chương N: Tiêu đề`, chương gộp `Chương N-M:`", () => {
    for (const genre of PROMPT_GENRE_COMBOS) {
      const prompt = composeBasePrompt(genre);
      expect(prompt, genreKey(genre)).toContain("`Chương N: Tiêu đề`");
      expect(prompt, genreKey(genre)).toContain("`Chương 467-468: ");
    }
  });

  it("mọi genre: bảng chống convert có mẫu động từ Hán-Việt dán trợ từ Việt", () => {
    for (const genre of PROMPT_GENRE_COMBOS) {
      const prompt = composeBasePrompt(genre);
      expect(prompt, genreKey(genre)).toContain("| 男女有别 | nam nữ hữu biệt / nam nữ khác biệt | nam nữ có biệt |");
      expect(prompt, genreKey(genre)).toContain("| 得到 / 深得……真传 | được / học được chân truyền | đắc được chân truyền |");
      expect(prompt, genreKey(genre)).toContain("| 得手 | thành công / ra tay trót lọt | đắc thủ |");
    }
  });

  it("ancient: 他们 chấp nhận bọn họ, không còn cấm họ", () => {
    const ancientHan = composeBasePrompt({ setting: "ancient", names: "han", tone: "neutral" });
    expect(ancientHan).toContain("| 他们        | **bọn họ** / **bọn hắn** / **chúng**");
    expect(ancientHan).not.toContain("| 他们        | **bọn hắn** / **chúng**          | họ");
    expect(ancientHan).not.toContain("| 她们        | **các nàng**                     | họ");
  });

  it("han: một chữ Hán một âm Hán-Việt, tên mới theo âm entry glossary sẵn có", () => {
    for (const setting of ["ancient", "modern", "mixed"] as const) {
      const han = composeBasePrompt({ setting, names: "han", tone: "neutral" });
      expect(han, setting).toContain("Đoàn Diên Khánh");
      expect(han, setting).toContain("Hy Tông");
      const foreign = composeBasePrompt({ setting, names: "foreign", tone: "neutral" });
      expect(foreign, setting).not.toContain("Đoàn Diên Khánh");
    }
  });

  it("ancient + mixed: viết hoa tước vị và hậu tố địa danh đi sau tên riêng; modern không có", () => {
    for (const setting of ["ancient", "mixed"] as const) {
      const prompt = composeBasePrompt({ setting, names: "han", tone: "neutral" });
      expect(prompt, setting).toContain("`Nhữ Dương Vương`");
      expect(prompt, setting).toContain("`chùa Thiếu Lâm`");
    }
    expect(composeBasePrompt({ setting: "modern", names: "han", tone: "neutral" })).not.toContain("`Nhữ Dương Vương`");
  });
});
