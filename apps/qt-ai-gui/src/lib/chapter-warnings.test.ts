import { describe, expect, it } from "vitest";

import { groupWarnings } from "@/lib/chapter-warnings";

const NARRATOR = "Người kể ngôi một xưng tôi → ta (truyện kể ngôi ba thì bỏ qua)";

describe("groupWarnings", () => {
  it("gom theo luật: cùng luật khác câu trích là một nhóm, giữ số đoạn và câu trích; thứ tự nhóm theo lần đầu gặp", () => {
    const groups = groupWarnings([
      `[[3]] ${NARRATOR} — "Tôi là cô dâu của ngày hôm nay."`,
      `[[7]] Thành ngữ trong bảng: 一石二鸟 → một mũi tên trúng hai đích — "Một hòn đá hai con chim."`,
      `[[4]] ${NARRATOR} — "Trước tiên, tôi muốn gửi lời cảm ơn."`,
      `[[5]] ${NARRATOR} — "Cảm ơn mọi người."`,
    ]);
    expect(groups.map((group) => group.rule)).toEqual([NARRATOR, "Thành ngữ trong bảng: 一石二鸟 → một mũi tên trúng hai đích"]);
    expect(groups[0]?.hits).toEqual([
      { n: 3, excerpt: "Tôi là cô dâu của ngày hôm nay." },
      { n: 4, excerpt: "Trước tiên, tôi muốn gửi lời cảm ơn." },
      { n: 5, excerpt: "Cảm ơn mọi người." },
    ]);
    expect(groups[1]?.hits).toEqual([{ n: 7, excerpt: "Một hòn đá hai con chim." }]);
  });

  it("cảnh báo không có số đoạn thì là nhóm riêng không số; luật có dấu — mà không có câu trích thì giữ nguyên", () => {
    expect(groupWarnings(["Chương quá ngắn so với raw", "Chương quá ngắn so với raw"])).toEqual([{ rule: "Chương quá ngắn so với raw", hits: [] }]);
    expect(groupWarnings(["[[2]] Gọi bề trên — tự xưng ta → tiểu nhân"])).toEqual([
      { rule: "Gọi bề trên — tự xưng ta → tiểu nhân", hits: [{ n: 2 }] },
    ]);
  });
});
