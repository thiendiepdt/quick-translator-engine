import { describe, expect, it } from "vitest";

import { mentionsNarratorPronoun, povNotices, toiPovWarning } from "@/lib/story-pov";

const first = { chapters: 60, paragraphs: 2673, firstPerson: 1689, ratio: 0.632, isFirstPerson: true };
const third = { chapters: 60, paragraphs: 5608, firstPerson: 593, ratio: 0.106, isFirstPerson: false };
const base = { savedPov: "unset" as const, setting: "modern" as const, voice: "", toneRules: "", doneChapters: 0 };

describe("story-pov", () => {
  it("dòng style quy định người kể xưng gì (giống Rust mentions_narrator_pronoun)", () => {
    for (const text of [
      "Lời kể ngôi thứ nhất xưng 'tôi', giữ giọng văn hiện đại gãy gọn, gần gũi, không dùng 'ta' trong lời dẫn truyện.",
      "Người kể tự xưng ta.",
      "Ngôi một, nữ chính xưng tôi.",
      'Kể chuyện ở ngôi thứ nhất, xưng "tôi"',
    ]) {
      expect(mentionsNarratorPronoun(text), text).toBe(true);
    }
    for (const text of [
      "Lời kể ngôi ba dùng hắn / cô.",
      "Nữ chính xưng tôi với đồng nghiệp, gọi sếp là anh.",
      "Lời kể nhắc người ta, chúng ta, bọn tôi vẫn được.",
      "Giọng kể tưng tửng, nhiều câu cảm thán.",
    ]) {
      expect(mentionsNarratorPronoun(text), text).toBe(false);
    }
  });

  it("raw ngôi một: hiện đại chưa chọn hoặc để ngôi ba thì nhắc, gợi ý xưng tôi; cổ đại chưa chọn thì thôi (đã là ta)", () => {
    const [unset] = povNotices({ ...base, report: first, pov: "unset" });
    expect(unset?.text).toContain("Raw kể ngôi một (63,2% đoạn có 我 ở lời kể, đọc thử 60 chương");
    expect(unset?.fix).toEqual({ label: "Đặt: Ngôi một — xưng tôi", pov: "first-toi" });
    expect(povNotices({ ...base, report: first, pov: "third", setting: "ancient" })[0]?.fix?.pov).toBe("first-ta");
    expect(povNotices({ ...base, report: first, pov: "unset", setting: "ancient" })).toEqual([]);
    expect(povNotices({ ...base, report: first, pov: "first-toi" })).toEqual([]);
    expect(povNotices({ ...base, report: first, pov: "first-ta" })).toEqual([]);
  });

  it("raw ngôi ba mà để ngôi một thì nhắc đặt Ngôi ba; chưa đọc được raw thì không đoán", () => {
    const [notice] = povNotices({ ...base, report: third, pov: "first-toi" });
    expect(notice?.text).toContain("Raw có vẻ kể ngôi ba (10,6%");
    expect(notice?.fix?.pov).toBe("third");
    expect(povNotices({ ...base, report: third, pov: "unset" })).toEqual([]);
    expect(povNotices({ ...base, pov: "first-toi" })).toEqual([]);
    expect(povNotices({ ...base, report: { ...first, isFirstPerson: null }, pov: "unset" })).toEqual([]);
  });

  it("cảnh báo màn dịch khi chọn xưng tôi: kèm câu raw khớp hay lệch; chưa đọc được raw thì bỏ câu raw", () => {
    const lead = 'Ngôi kể: Ngôi một — người kể xưng tôi. Lời kể dịch "tôi" thay cho "ta".';
    expect(toiPovWarning(first)).toEqual({ lead, raw: "Raw kể ngôi một (63,2% đoạn có 我 ở lời kể).", mismatch: false });
    expect(toiPovWarning(third)).toEqual({
      lead,
      raw: "Raw có vẻ kể ngôi ba (10,6% đoạn có 我 ở lời kể) — rất có thể chọn nhầm.",
      mismatch: true,
    });
    expect(toiPovWarning(undefined)).toEqual({ lead, mismatch: false });
    expect(toiPovWarning({ ...first, isFirstPerson: null })).toEqual({ lead, mismatch: false });
  });

  it("style còn dòng người kể thì liệt kê để xoá; đổi ngôi khi đã có chương dịch thì nhắc dịch lại", () => {
    const notices = povNotices({
      ...base,
      pov: "first-toi",
      savedPov: "unset",
      voice: "Ngôi thứ nhất, người kể xưng tôi, giọng gần gũi",
      toneRules: "Gọi cha mẹ là cha/mẹ\nLời dẫn truyện không dùng ta",
      doneChapters: 12,
    });
    expect(notices.map((n) => n.lines ?? [])).toContainEqual(["Ngôi thứ nhất, người kể xưng tôi, giọng gần gũi", "Lời dẫn truyện không dùng ta"]);
    expect(notices.at(-1)?.text).toContain("12 chương đã dịch vẫn giữ cách xưng cũ");
    expect(povNotices({ ...base, pov: "first-toi", savedPov: "first-toi", doneChapters: 12 })).toEqual([]);
  });
});
