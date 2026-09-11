import { describe, expect, it } from "vitest";

import {
  appendAutoGlossary,
  collectGlossaryKeys,
  resolveAutoGlossaryEnabled,
  sanitizeExtractedGlossary,
  glossaryKeyTouchesSource,
} from "@/lib/ai-glossary";
import { emptyAiStoryConfig } from "@/lib/ai-story";

const RAW = "震雷子看向太清山。赵静文微微一笑。";
const TRANSLATION = "Chấn Lôi Tử nhìn về Thái Thanh Sơn. Triệu Tĩnh Văn khẽ mỉm cười.\n";

describe("collectGlossaryKeys", () => {
  it("unions workspace glossary and story glossary keys", () => {
    const story = emptyAiStoryConfig();
    story.glossary.names["赵静文"] = "Triệu Tĩnh Văn";
    const keys = collectGlossaryKeys({ names: { "萧炎": "Tiêu Viêm" } }, story.glossary);
    expect(keys.has("萧炎")).toBe(true);
    expect(keys.has("赵静文")).toBe(true);
    expect(keys.has("震雷子")).toBe(false);
  });
});

describe("sanitizeExtractedGlossary", () => {
  it("keeps only new Han-source pairs that appear in both texts", () => {
    const pairs = sanitizeExtractedGlossary(
      [
        { source: "震雷子", target: "Chấn Lôi Tử", category: "names" },
        { source: "太清山", target: "Thái Thanh Sơn", category: "places" },
        { source: "赵静文", target: "Triệu Tĩnh Văn", category: "names" }, // đã có trong glossary
        { source: "许攸", target: "Hứa Du", category: "names" },           // không có trong raw
        { source: "震雷子", target: "Trấn Lôi Tử", category: "names" },    // trùng source
        { source: "太清山", target: "Núi Thái Thanh", category: "places" }, // target không có trong bản dịch
        { source: "abc", target: "abc", category: "names" },               // source không phải Hán
        "rác",
      ],
      RAW,
      TRANSLATION,
      new Set(["赵静文"]),
    );
    expect(pairs).toEqual([
      { source: "震雷子", target: "Chấn Lôi Tử", category: "names" },
      { source: "太清山", target: "Thái Thanh Sơn", category: "places" },
    ]);
  });

  it("drops single-character sources — too ambiguous to pin", () => {
    const pairs = sanitizeExtractedGlossary(
      [{ source: "雷", target: "Lôi", category: "names" }],
      "雷声轰鸣。",
      "Lôi thanh vang rền.\n",
      new Set(),
    );
    expect(pairs).toEqual([]);
  });

  it("falls back to names for unknown categories", () => {
    const pairs = sanitizeExtractedGlossary(
      [{ source: "震雷子", target: "Chấn Lôi Tử", category: "bogus" }],
      RAW,
      TRANSLATION,
      new Set(),
    );
    expect(pairs).toEqual([{ source: "震雷子", target: "Chấn Lôi Tử", category: "names" }]);
  });
});

describe("appendAutoGlossary", () => {
  it("appends new keys per category and logs provenance without overwriting", () => {
    const story = emptyAiStoryConfig();
    story.glossary.names["震雷子"] = "Chấn Lôi Tử cũ";
    const next = appendAutoGlossary(
      story,
      [
        { source: "震雷子", target: "Chấn Lôi Tử mới", category: "names" },
        { source: "太清山", target: "Thái Thanh Sơn", category: "places" },
      ],
      "chuong-163.txt",
    );
    // Key đã tồn tại thì giữ nguyên và không log.
    expect(next.glossary.names["震雷子"]).toBe("Chấn Lôi Tử cũ");
    expect(next.glossary.places["太清山"]).toBe("Thái Thanh Sơn");
    expect(next.autoGlossaryLog).toEqual([
      { source: "太清山", target: "Thái Thanh Sơn", category: "places", chapter: "chuong-163.txt" },
    ]);
    // Không đụng vào object gốc.
    expect(story.glossary.places["太清山"]).toBeUndefined();
    expect(story.autoGlossaryLog).toEqual([]);
  });
});

describe("resolveAutoGlossaryEnabled", () => {
  it("prioritizes the story setting over the AI settings default", () => {
    expect(resolveAutoGlossaryEnabled("on", false)).toBe(true);
    expect(resolveAutoGlossaryEnabled("off", true)).toBe(false);
    expect(resolveAutoGlossaryEnabled("inherit", true)).toBe(true);
    expect(resolveAutoGlossaryEnabled("inherit", false)).toBe(false);
  });

  it("addressing: nhận cặp 甲→乙 có cả hai tên trong raw, chuẩn hoá target về X–Y, không đòi target nằm trong bản dịch", () => {
    const raw = "林枫看着苏雨：“你别怕。”苏雨点头。";
    const pairs = sanitizeExtractedGlossary(
      [
        { source: "林枫→苏雨", target: "anh - em", category: "addressing" },
        { source: "苏雨 -> 林枫", target: "em/anh", category: "addressing" },
        { source: "林枫→王五", target: "anh–em", category: "addressing" }, // 王五 không có trong raw
        { source: "林枫", target: "anh–em", category: "addressing" }, // thiếu mũi tên
        { source: "林枫→苏雨", target: "anh", category: "addressing" }, // thiếu vế
        { source: "林枫→苏雨", target: "tôi–cậu", category: "addressing" }, // trùng key
      ],
      raw,
      "Lâm Phong nhìn Tô Vũ: “Em đừng sợ.” Tô Vũ gật đầu.",
      new Set(),
    );
    expect(pairs).toEqual([
      { source: "林枫→苏雨", target: "anh–em", category: "addressing" },
      { source: "苏雨→林枫", target: "em–anh", category: "addressing" },
    ]);
    expect(glossaryKeyTouchesSource("林枫→苏雨", raw)).toBe(true);
    expect(glossaryKeyTouchesSource("林枫→王五", raw)).toBe(false);
    expect(glossaryKeyTouchesSource("林枫", raw)).toBe(true);
  });
});
