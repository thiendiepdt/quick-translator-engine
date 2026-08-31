import { describe, expect, it } from "vitest";
import { aiParagraphsOf } from "@/lib/ai-paragraphs";
import { emptyAiStoryConfig } from "@/lib/ai-story";

describe("alias @/ tới qt-web/src", () => {
  it("import được module lõi của qt-web", () => {
    expect(aiParagraphsOf("a\r\n\r\n b ")).toEqual(["a", "b"]);
    expect(emptyAiStoryConfig().autoGlossary).toBe("inherit");
  });
});
