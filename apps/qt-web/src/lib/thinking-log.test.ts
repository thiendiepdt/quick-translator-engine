import { describe, expect, it } from "vitest";

import { appendThinking, parseThinkingLog, thinkingPinAfterScroll } from "@/lib/thinking-log";

describe("appendThinking", () => {
  it("adds a labeled header for the first segment", () => {
    expect(appendThinking("", "Dịch", "suy nghĩ A")).toBe("── Dịch ──\nsuy nghĩ A");
  });

  it("separates later segments with a blank line", () => {
    const log = appendThinking("", "Dịch", "suy nghĩ A");
    expect(appendThinking(log, "Soát lần 1", "suy nghĩ B")).toBe(
      "── Dịch ──\nsuy nghĩ A\n\n── Soát lần 1 ──\nsuy nghĩ B",
    );
  });

  it("returns the log unchanged when the segment is empty", () => {
    const log = appendThinking("", "Dịch", "suy nghĩ A");
    expect(appendThinking(log, "Soát lần 1", "")).toBe(log);
  });
});

describe("parseThinkingLog", () => {
  it("splits an accumulated log back into labeled segments", () => {
    const log = appendThinking(
      appendThinking("", "Dịch", "suy nghĩ A\nvẫn A"),
      "Soát lần 1",
      "suy nghĩ B",
    );
    expect(parseThinkingLog(log)).toEqual([
      { label: "Dịch", text: "suy nghĩ A\nvẫn A" },
      { label: "Soát lần 1", text: "suy nghĩ B" },
    ]);
  });

  it("wraps headerless text in a single unlabeled segment", () => {
    expect(parseThinkingLog("suy nghĩ trơn")).toEqual([
      { label: "", text: "suy nghĩ trơn" },
    ]);
  });

  it("returns no segments for an empty log", () => {
    expect(parseThinkingLog("")).toEqual([]);
  });
});

describe("thinkingPinAfterScroll", () => {
  const node = (scrollTop: number, scrollHeight: number, clientHeight: number) => ({
    scrollTop,
    scrollHeight,
    clientHeight,
  });

  it("pins when the viewport is near the bottom", () => {
    expect(thinkingPinAfterScroll(false, 0, node(180, 300, 100))).toBe(true);
  });

  it("unpins when the user scrolls upward away from the bottom", () => {
    expect(thinkingPinAfterScroll(true, 150, node(100, 300, 100))).toBe(false);
  });

  it("stays pinned when content grows without the user scrolling up", () => {
    // Chunk mới làm scrollHeight nhảy 300→500 trong khi scrollTop đứng yên;
    // scroll event chen vào giữa không được phép unpin.
    expect(thinkingPinAfterScroll(true, 200, node(200, 500, 100))).toBe(true);
  });

  it("stays unpinned while the user is parked mid-content", () => {
    expect(thinkingPinAfterScroll(false, 100, node(100, 500, 100))).toBe(false);
  });
});
