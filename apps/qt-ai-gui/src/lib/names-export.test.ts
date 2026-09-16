import { describe, expect, it } from "vitest";

import { defaultSelection, NAMES_EXPORT_KEYS, planNamesExport, renderNames } from "@/lib/names-export";
import type { StoryFormValues } from "@/lib/story-form";

const empty = (): StoryFormValues["glossary"] => ({
  names: [],
  places: [],
  items: [],
  creatures: [],
  skills: [],
  common: [],
  signature_phrases: [],
  addressing: [],
});

describe("planNamesExport", () => {
  it("liệt kê theo thứ tự nhóm rồi thứ tự dòng, trim hai cột, bỏ addressing", () => {
    const g = empty();
    g.places = [{ source: " 京城 ", target: " Kinh Thành " }];
    g.names = [{ source: "赵静文", target: "Triệu Tĩnh Văn" }];
    g.addressing = [{ source: "甲→乙", target: "ta–ngươi" }];
    const rows = planNamesExport(g);
    expect(rows.map((r) => [r.id, r.source, r.target])).toEqual([
      ["names:0", "赵静文", "Triệu Tĩnh Văn"],
      ["places:0", "京城", "Kinh Thành"],
    ]);
    expect(rows.every((r) => r.skip === undefined)).toBe(true);
    expect(NAMES_EXPORT_KEYS).not.toContain("addressing");
  });

  it("đánh dấu bỏ: thiếu Hán/Việt, có dấu =, nhiều dòng", () => {
    const g = empty();
    g.names = [
      { source: "  ", target: "x" },
      { source: "甲", target: "" },
      { source: "a=b", target: "c" },
      { source: "d", target: "e=f" },
      { source: "多\n行", target: "g" },
    ];
    expect(planNamesExport(g).map((r) => r.skip)).toEqual([
      "thiếu Hán/Việt",
      "thiếu Hán/Việt",
      "có dấu =",
      "có dấu =",
      "nhiều dòng",
    ]);
  });

  it("trùng source: dòng đầu giữ, dòng sau bỏ kèm nhãn nhóm đã giữ (cùng nhóm hoặc khác nhóm)", () => {
    const g = empty();
    g.names = [
      { source: "赵静文", target: "A" },
      { source: " 赵静文", target: "B" },
    ];
    g.common = [{ source: "赵静文", target: "C" }];
    const rows = planNamesExport(g);
    expect(rows[0].skip).toBeUndefined();
    expect(rows[1].skip).toBe("trùng nhóm Tên nhân vật");
    expect(rows[2].skip).toBe("trùng nhóm Tên nhân vật");
  });
});

describe("defaultSelection", () => {
  it("tick mọi dòng hợp lệ trừ signature_phrases và dòng bị bỏ", () => {
    const g = empty();
    g.names = [
      { source: "甲", target: "A" },
      { source: "甲", target: "B" },
    ];
    g.signature_phrases = [{ source: "乙", target: "C" }];
    const rows = planNamesExport(g);
    expect([...defaultSelection(rows)]).toEqual(["names:0"]);
  });
});

describe("renderNames", () => {
  it("chỉ dòng đã tick và không bị bỏ, CRLF, BOM tuỳ chọn", () => {
    const g = empty();
    g.names = [
      { source: "甲", target: "A" },
      { source: "乙", target: "B" },
      { source: "甲", target: "C" },
    ];
    const rows = planNamesExport(g);
    const selected = new Set(["names:0", "names:1", "names:2"]);
    expect(renderNames(rows, selected)).toBe("甲=A\r\n乙=B\r\n");
    expect(renderNames(rows, selected, { bom: true })).toBe("﻿甲=A\r\n乙=B\r\n");
    expect(renderNames(rows, new Set(["names:1"]))).toBe("乙=B\r\n");
    expect(renderNames(rows, new Set())).toBe("");
  });
});
