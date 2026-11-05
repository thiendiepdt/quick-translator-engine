import { describe, expect, it } from "vitest";

import {
  countChars,
  countWords,
  hanRuns,
  hasHan,
  numberLines,
  READING_WIDTHS,
  readingWidthOf,
  replaceHan,
  splitHan,
  withLine,
} from "@/lib/reading";

describe("thay chữ Hán bằng phiên âm", () => {
  const readings = new Map([
    ["攜", "Huề"],
    ["攜帶", "Huề Đái"],
    ["得", "Đắc"],
    ["濡目染", "Nhu Mục Nhiễm"],
  ]);

  it("hanRuns đánh dấu cụm nằm trọn trong ngoặc", () => {
    expect(hanRuns("Chữ “攜” trong ‘攜帶’, chạy得 nhanh").map(({ text, quoted }) => [text, quoted])).toEqual([
      ["攜", true],
      ["攜帶", true],
      ["得", false],
    ]);
  });

  it("chữ trong ngoặc viết hoa từng âm, chữ trần viết thường", () => {
    expect(replaceHan("Trên chữ “攜” trong “攜帶”, cô dừng bút.", readings)).toBe("Trên chữ “Huề” trong “Huề Đái”, cô dừng bút.");
    expect(replaceHan("Chữ ‘攜’ có vết tẩy.", readings)).toBe("Chữ ‘Huề’ có vết tẩy.");
    expect(replaceHan("Nàng chạy 得 nhanh.", readings)).toBe("Nàng chạy đắc nhanh.");
  });

  it("chữ Hán dính liền chữ Việt thì thêm dấu cách", () => {
    expect(replaceHan("Nàng chạy得 nhanh.", readings)).toBe("Nàng chạy đắc nhanh.");
    expect(replaceHan("tai濡目染 không nói", readings)).toBe("tai nhu mục nhiễm không nói");
    expect(replaceHan("A得B", readings)).toBe("A đắc B");
  });

  it("chữ chưa có âm giữ nguyên; truyền only thì chỉ thay cụm đó, kể cả khi lặp trong dòng", () => {
    expect(replaceHan("“攜” và “龘”", readings)).toBe("“Huề” và “龘”");
    expect(replaceHan("“攜”, “得”, ‘攜’", readings, "攜")).toBe("“Huề”, “得”, ‘Huề’");
  });

  it("withLine đổi đúng dòng, giữ nguyên các dòng khác và kiểu xuống dòng", () => {
    expect(withLine("Một\r\n\r\nHai\n\nBa\n", 2, "Mới")).toBe("Một\r\n\r\nMới\n\nBa\n");
  });
});

describe("reading width", () => {
  it("nhận đúng 4 mức, giá trị lạ về normal", () => {
    expect(READING_WIDTHS).toEqual(["narrow", "normal", "wide", "full"]);
    expect(readingWidthOf("wide")).toBe("wide");
    expect(readingWidthOf("huge")).toBe("normal");
    expect(readingWidthOf(undefined)).toBe("normal");
  });

  it("numberLines đánh số dòng không rỗng (trùng nhãn [[n]] của harness), dòng trống giữ chỗ không số", () => {
    expect(numberLines("Chương 1: A\n\nĐoạn hai.\r\n   \nĐoạn ba.\n")).toEqual([
      { n: 1, text: "Chương 1: A" },
      { text: "" },
      { n: 2, text: "Đoạn hai." },
      { text: "   " },
      { n: 3, text: "Đoạn ba." },
      { text: "" },
    ]);
    // Bản gốc không có dòng trống giữa các đoạn vẫn mỗi dòng một số.
    expect(numberLines("一\n二").map((line) => line.n)).toEqual([1, 2]);
    expect(numberLines("")).toEqual([{ text: "" }]);
  });

  it("hasHan/splitHan bắt chữ Hán còn sót, không tính ngoặc 【】「」 và chữ Việt", () => {
    expect(hasHan("Nàng nói: 好的, đi thôi")).toBe(true);
    expect(hasHan("【Nơi gửi não】 「Được」 Đường Tăng… ……")).toBe(false);
    expect(splitHan("Nàng nói: 好的, đi 吧")).toEqual([
      { text: "Nàng nói: ", han: false },
      { text: "好的", han: true },
      { text: ", đi ", han: false },
      { text: "吧", han: true },
    ]);
    expect(splitHan("魔法 mở đầu")).toEqual([
      { text: "魔法", han: true },
      { text: " mở đầu", han: false },
    ]);
    expect(splitHan("Không có gì")).toEqual([{ text: "Không có gì", han: false }]);
  });

  it("countChars bỏ khoảng trắng (như harness), countWords đếm tiếng và bỏ cụm chỉ có dấu câu", () => {
    expect(countChars("第1章 甲\n\n她抬起头。\r\n")).toBe(9);
    expect(countChars("")).toBe(0);
    expect(countWords("Chương 1: Mưa đêm\n\n— Nàng ngẩng đầu… ……\n")).toBe(7);
    expect(countWords("  \n ")).toBe(0);
  });
});
