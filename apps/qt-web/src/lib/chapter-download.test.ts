import { describe, expect, it } from "vitest";

import {
  doneChapterRange,
  mergeChapterOutputs,
  selectChaptersForDownload,
} from "@/lib/chapter-download";
import type { AiTranslationChapter } from "@/lib/ai-story";

function chapter(
  filename: string,
  status: AiTranslationChapter["status"],
  output = "",
): AiTranslationChapter {
  return {
    id: filename,
    filename,
    source: "raw",
    output,
    thinking: "",
    violations: [],
    status,
    reviewRound: 0,
    updatedAt: 0,
  };
}

const CHAPTERS = [
  chapter("chuong-1.txt", "queued"),
  chapter("chuong-2.txt", "done", "Bản dịch chương hai.\n"),
  chapter("chuong-3.txt", "error"),
  chapter("chuong-4.txt", "done", "Bản dịch chương bốn.\n"),
  chapter("chuong-5.txt", "queued"),
];

describe("doneChapterRange", () => {
  it("spans from the first to the last done chapter (1-based)", () => {
    expect(doneChapterRange(CHAPTERS)).toEqual({ from: 2, to: 4 });
  });

  it("returns undefined when nothing is done", () => {
    expect(doneChapterRange([chapter("a.txt", "queued")])).toBeUndefined();
  });
});

describe("selectChaptersForDownload", () => {
  it("keeps only done chapters with output inside the range", () => {
    expect(
      selectChaptersForDownload(CHAPTERS, 2, 4).map(({ filename }) => filename),
    ).toEqual(["chuong-2.txt", "chuong-4.txt"]);
  });

  it("clamps an out-of-bounds or reversed range", () => {
    expect(
      selectChaptersForDownload(CHAPTERS, 0, 99).map(({ filename }) => filename),
    ).toEqual(["chuong-2.txt", "chuong-4.txt"]);
    expect(selectChaptersForDownload(CHAPTERS, 4, 2)).toEqual([]);
  });
});

describe("mergeChapterOutputs", () => {
  it("joins chapters with exactly one blank line and a trailing newline", () => {
    const merged = mergeChapterOutputs([
      chapter("a.txt", "done", "Chương một.\n"),
      chapter("b.txt", "done", "Chương hai.\n\n"),
    ]);
    expect(merged).toBe("Chương một.\n\nChương hai.\n");
  });
});
