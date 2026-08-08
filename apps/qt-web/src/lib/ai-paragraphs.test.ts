import { describe, expect, it } from "vitest";

import {
  aiParagraphRanges,
  aiParagraphsOf,
  labeledAiRepairPayload,
  labeledAiSourcePayload,
  parseLabeledAiTranslation,
  stripAiParagraphMarkers,
} from "@/lib/ai-paragraphs";

describe("aligned AI paragraphs", () => {
  it("splits non-empty lines and tracks their offsets", () => {
    const text = "第一段\n\n  第二段  \n第三段\n";
    expect(aiParagraphsOf(text)).toEqual(["第一段", "第二段", "第三段"]);

    const ranges = aiParagraphRanges(text);
    expect(ranges).toHaveLength(3);
    expect(text.slice(ranges[1].start, ranges[1].start + ranges[1].length)).toBe(
      "第二段",
    );
  });

  it("labels every paragraph and explains the format", () => {
    const payload = labeledAiSourcePayload(["第一段", "第二段"]);
    expect(payload).toContain("[[1]] 第一段");
    expect(payload).toContain("[[2]] 第二段");
    expect(payload).toContain("từ [[1]] đến [[2]]");
  });

  it("parses labeled output and reports missing paragraphs", () => {
    const parsed = parseLabeledAiTranslation(
      "[[1]] Đoạn một.\n\n[[3]] Đoạn ba.",
      3,
    );
    expect(parsed).toEqual(["Đoạn một.", undefined, "Đoạn ba."]);
  });

  it("collapses internal newlines and ignores out-of-range labels", () => {
    const parsed = parseLabeledAiTranslation(
      "[[1]] Dòng một\ntiếp theo.\n\n[[9]] lạc loài",
      2,
    );
    expect(parsed).toEqual(["Dòng một tiếp theo.", undefined]);
  });

  it("returns undefined when the model dropped every label", () => {
    expect(parseLabeledAiTranslation("Bản dịch trơn.", 2)).toBeUndefined();
  });

  it("builds a repair payload with only the missing paragraphs", () => {
    const payload = labeledAiRepairPayload(["第一段", "第二段", "第三段"], [1]);
    expect(payload).toContain("[[2]] 第二段");
    expect(payload).not.toContain("[[1]] 第一段");
    expect(payload).not.toContain("[[3]] 第三段");
  });

  it("strips labels from streamed text for display", () => {
    expect(stripAiParagraphMarkers("[[1]] Một.\n\n[[2]] Hai.")).toBe(
      "Một.\n\nHai.",
    );
  });
});
