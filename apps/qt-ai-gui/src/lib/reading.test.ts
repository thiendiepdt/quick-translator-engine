import { describe, expect, it } from "vitest";

import { countChars, countWords, hasHan, numberLines, READING_WIDTHS, readingWidthOf, splitHan } from "@/lib/reading";

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
