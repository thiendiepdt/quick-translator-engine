import { describe, expect, it } from "vitest";

import { renderName, stvStoryId, TEMPLATE_PRESETS, toggleSelection, validateTemplate } from "@/lib/raw";

describe("mẫu tên file raw", () => {
  it("kiểm mẫu cùng luật Rust", () => {
    expect(validateTemplate("{chuong:4}_{chapter-id}")).toBeNull();
    expect(validateTemplate("chuong-{chuong}")).toBeNull();
    expect(validateTemplate("{chapterTitle}")).toMatch(/\{chuong\}/);
    expect(validateTemplate("{chuong}_{foo}")).toMatch(/\{foo\}/);
    expect(validateTemplate("{chuong:0}")).not.toBeNull();
    expect(validateTemplate(" ")).not.toBeNull();
  });

  it("render: đệm số, bỏ ký tự cấm, cắt 120 ký tự", () => {
    expect(renderName("{chuong:4}_{chapter-id}", { index: 5, id: "7445", title: "" })).toBe("0005_7445");
    expect(renderName("{chuong:2}", { index: 123, id: "", title: "" })).toBe("123");
    expect(renderName("{chuong:4} {chapterTitle}", { index: 1, id: "", title: "第1章 何以?:*  为家 " })).toBe("0001 第1章 何以 为家");
    expect(renderName("{chuong:4}_{chapterTitle}...", { index: 1, id: "", title: "a" })).toBe("0001_a");
    expect(Array.from(renderName("{chuong:4}_{chapterTitle}", { index: 1, id: "", title: "长".repeat(300) }))).toHaveLength(120);
  });
});

describe("mẫu có sẵn của các tool tải truyện", () => {
  const parts = { index: 1, id: "7445", title: "第1章 起" };
  const names = (url: string) =>
    Object.fromEntries(
      TEMPLATE_PRESETS.map((preset) => {
        const template = preset.template(url);
        if (template !== null) expect(validateTemplate(template)).toBeNull();
        return [preset.label, template && renderName(template, parts)];
      }),
    );

  it("ra đúng tên file như tool đó đặt", () => {
    expect(names("https://fanqienovel.com/page/7445623259162542654")).toEqual({
      "Chinese Web Downloader": "0001_7445",
      "STV Web Downloader": null,
      "Novel Reader": "0001 第1章 起",
      "DTH": "0001 - 第1章 起",
    });
  });

  it("STV Web Downloader: <mã truyện>_0001_<tiêu đề>, mã truyện lấy từ link STV", () => {
    expect(names("https://sangtacviet.vip/truyen/qidian/1/730611/")["STV Web Downloader"]).toBe("730611_0001_第1章 起");
    expect(stvStoryId("http://14.225.254.182/truyen/fanqie/1/7445/")).toBe("7445");
    expect(stvStoryId("https://www.qimao.com/shuku/142631/")).toBeNull();
  });
});

describe("toggleSelection", () => {
  const order = ["1", "2", "3", "4", "5"];
  const pick = (selected: string[], id: string, last: string | undefined, shift: boolean) => [...toggleSelection(new Set(selected), order, id, last, shift)].sort();

  it("bấm thường bật / tắt một chương", () => {
    expect(pick([], "2", undefined, false)).toEqual(["2"]);
    expect(pick(["2"], "2", "2", false)).toEqual([]);
  });

  it("Shift chọn hoặc bỏ cả dải theo trạng thái chương vừa bấm", () => {
    expect(pick(["1"], "4", "1", true)).toEqual(["1", "2", "3", "4"]);
    // Dải ngược chiều.
    expect(pick(["5"], "2", "5", true)).toEqual(["2", "3", "4", "5"]);
    // Chương vừa bấm đang chọn → bỏ cả dải.
    expect(pick(["1", "2", "3", "4"], "2", "4", true)).toEqual(["1"]);
    // Chưa bấm chương nào thì Shift như bấm thường.
    expect(pick([], "3", undefined, true)).toEqual(["3"]);
  });
});
