import { describe, expect, it } from "vitest";

import {
  lowercaseText,
  sentenceCaseText,
  titleCaseText,
  uppercaseText,
} from "@/lib/text-case";

describe("Vietnamese text case helpers", () => {
  it("supports all dictionary editor casing actions", () => {
    expect(titleCaseText("kIM mỹ-đÌNH")).toBe("Kim Mỹ-Đình");
    expect(lowercaseText("KIM MỸ ĐÌNH")).toBe("kim mỹ đình");
    expect(sentenceCaseText("  KIM MỸ ĐÌNH")).toBe("  Kim mỹ đình");
    expect(uppercaseText("Kim Mỹ Đình")).toBe("KIM MỸ ĐÌNH");
  });
});
