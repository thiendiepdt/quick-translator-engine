import { describe, expect, it } from "vitest";

import { splitStoryName, wrapText } from "@/lib/epub-cover";

describe("splitStoryName", () => {
  it("tách tên gốc chữ Hán trong ngoặc, cùng luật với core", () => {
    expect(splitStoryName("Phương Thốn Đạo Chủ (方寸道主)")).toEqual({ title: "Phương Thốn Đạo Chủ", originalTitle: "方寸道主" });
    expect(splitStoryName("Tên（长生）")).toEqual({ title: "Tên", originalTitle: "长生" });
    expect(splitStoryName("Trường Sinh Là Mèo")).toEqual({ title: "Trường Sinh Là Mèo", originalTitle: "" });
    expect(splitStoryName("Tên (bản dịch)")).toEqual({ title: "Tên (bản dịch)", originalTitle: "" });
  });
});

describe("wrapText", () => {
  // Mỗi ký tự rộng 1 đơn vị.
  const measure = (text: string) => Array.from(text).length;

  it("ngắt theo từ cho vừa bề rộng", () => {
    expect(wrapText(measure, "Phương Thốn Đạo Chủ", 12)).toEqual(["Phương Thốn", "Đạo Chủ"]);
    expect(wrapText(measure, "  Một  ", 10)).toEqual(["Một"]);
  });

  it("từ dài hơn cả dòng (tên chữ Hán không dấu cách) thì ngắt theo chữ", () => {
    expect(wrapText(measure, "快穿疯批美人不肯当炮灰", 4)).toEqual(["快穿疯批", "美人不肯", "当炮灰"]);
  });
});
