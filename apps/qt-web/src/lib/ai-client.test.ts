import { afterEach, describe, expect, it, vi } from "vitest";

import {
  applyAiDecisions,
  baseUrlProblem,
  candidateContext,
  chunkByLines,
  extractEntities,
  resolveAiCall,
  reviewCandidates,
  sanitizeEntitiesForRequest,
  selectReviewCandidates,
} from "@/lib/ai-client";
import type { NameCandidate } from "@/lib/types";

afterEach(() => {
  vi.restoreAllMocks();
});

function requestUrlOf(input: RequestInfo | URL): string {
  if (typeof input === "string") return input;
  return input instanceof URL ? input.href : input.url;
}

function requestBodyOf(init?: RequestInit): unknown {
  return JSON.parse(init?.body as string) as unknown;
}

function candidate(overrides: Partial<NameCandidate> = {}): NameCandidate {
  return {
    text: "萧炎",
    suggested: "Tiêu Viêm",
    entityType: "unknown",
    score: 0.5,
    occurrences: 2,
    ranges: [],
    contexts: [],
    reasons: [],
    sources: [],
    known: false,
    ...overrides,
  };
}

describe("chunkByLines", () => {
  // Ba case port nguyên từ test Rust của chunk_by_lines.
  it("respects line boundaries", () => {
    expect(chunkByLines("aaaa\nbbbb\ncccc\n", 10)).toEqual(["aaaa\nbbbb\n", "cccc\n"]);
  });

  it("splits an oversized line into bounded chunks", () => {
    const text = `short\n${"x".repeat(25)}\nshort2`;
    const chunks = chunkByLines(text, 10);
    expect(chunks).toEqual(["short\n", "x".repeat(10), "x".repeat(10), "x".repeat(5), "short2\n"]);
    expect(chunks.every((chunk) => [...chunk].length <= 10)).toBe(true);
  });

  it("splits multibyte text on character boundaries", () => {
    const chunks = chunkByLines("汉".repeat(12), 5);
    expect(chunks).toHaveLength(3);
    expect(chunks[0]).toBe("汉".repeat(5));
    expect(chunks[2]).toBe("汉".repeat(2));
  });
});

describe("resolveAiCall", () => {
  it("fills provider defaults and trims the base URL", () => {
    expect(
      resolveAiCall("deepseek", { apiKey: " sk-test ", model: "", baseUrl: "" }),
    ).toEqual({
      provider: "deepseek",
      apiKey: "sk-test",
      model: "deepseek-chat",
      baseUrl: "https://api.deepseek.com",
    });
    expect(
      resolveAiCall("gemini", {
        apiKey: "AIza",
        model: "gemini-3.1-flash-lite",
        baseUrl: "https://relay.example.com/gemini///",
      }),
    ).toEqual({
      provider: "gemini",
      apiKey: "AIza",
      model: "gemini-3.1-flash-lite",
      baseUrl: "https://relay.example.com/gemini",
    });
  });
});

describe("baseUrlProblem", () => {
  it("accepts https, http on localhost, and empty values", () => {
    expect(baseUrlProblem("")).toBeNull();
    expect(baseUrlProblem("https://proxy.example.com/v1")).toBeNull();
    expect(baseUrlProblem("http://localhost:1234")).toBeNull();
    expect(baseUrlProblem("http://127.0.0.1:8080")).toBeNull();
  });

  it("rejects http on remote hosts and malformed values", () => {
    expect(baseUrlProblem("http://proxy.example.com")).toContain("https");
    expect(baseUrlProblem("ftp://example.com")).toContain("https");
    expect(baseUrlProblem("not a url")).toContain("không hợp lệ");
  });
});

describe("sanitizeEntitiesForRequest", () => {
  it("trims, dedupes, clamps and drops oversized entities", () => {
    const sanitized = sanitizeEntitiesForRequest([
      { text: " 萧炎 ", entityType: "person", suggested: " Tiêu Viêm ", confidence: 1.7 },
      { text: "萧炎", confidence: 0.5 },
      { text: "   ", confidence: 0.9 },
      { text: "名".repeat(101), confidence: 0.9 },
    ]);
    expect(sanitized).toEqual([
      { text: "萧炎", entityType: "person", suggested: "Tiêu Viêm", confidence: 1 },
    ]);
  });

  it("caps the list at the server limit", () => {
    const entities = Array.from({ length: 600 }, (_, index) => ({
      text: `名字${index}`,
      confidence: 0.9,
    }));
    expect(sanitizeEntitiesForRequest(entities)).toHaveLength(500);
  });
});

describe("selectReviewCandidates", () => {
  it("keeps only unknown candidates inside the ambiguity window", () => {
    const picked = selectReviewCandidates([
      candidate({ text: "chắc chắn", score: 0.95 }),
      candidate({ text: "mơ hồ", score: 0.5 }),
      candidate({ text: "đã biết", score: 0.5, known: true }),
      candidate({ text: "quá thấp", score: 0.1 }),
    ]);
    expect(picked.map((item) => item.text)).toEqual(["mơ hồ"]);
  });
});

describe("applyAiDecisions", () => {
  // Port từ test Rust ai_decisions_are_bounded_to_candidates_and_threshold.
  it("drops rejected candidates and applies confident decisions", () => {
    const result = applyAiDecisions(
      [candidate({ text: "萧炎", suggested: "Cũ" }), candidate({ text: "看向" })],
      [
        {
          text: "萧炎",
          keep: true,
          confidence: 0.9,
          entityType: "person",
          suggested: "Tiêu Viêm",
        },
        { text: "看向", keep: false, confidence: 0.95, entityType: "unknown" },
      ],
    );
    expect(result).toHaveLength(1);
    expect(result[0].text).toBe("萧炎");
    expect(result[0].suggested).toBe("Tiêu Viêm");
    expect(result[0].entityType).toBe("person");
    expect(result[0].score).toBeCloseTo(1 - (1 - 0.5) * (1 - 0.9));
    expect(result[0].sources).toContain("ai-fallback");
  });

  it("ignores decisions below the confidence threshold", () => {
    const result = applyAiDecisions(
      [candidate({ text: "看向" })],
      [{ text: "看向", keep: false, confidence: 0.3 }],
    );
    expect(result).toHaveLength(1);
    expect(result[0].sources).not.toContain("ai-fallback");
  });
});

describe("candidateContext", () => {
  it("marks the first occurrence with surrounding characters", () => {
    expect(candidateContext("abc萧炎def", "萧炎", 2)).toBe("bc【萧炎】de");
    expect(candidateContext("abcdef", "萧炎", 2)).toBe("");
  });
});

describe("extractEntities", () => {
  it("calls DeepSeek per chunk and normalizes the entities", async () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      Response.json({
        choices: [
          {
            message: {
              content: JSON.stringify({
                entities: [
                  { text: "萧炎", entityType: "person", suggested: "Tiêu Viêm", confidence: 5 },
                  { text: "药老" },
                ],
              }),
            },
          },
        ],
      }),
    );
    const result = await extractEntities("萧炎和药老。", {
      provider: "deepseek",
      apiKey: "sk-test",
      model: "deepseek-v4-flash",
      baseUrl: "https://api.deepseek.com",
    });
    expect(fetchSpy).toHaveBeenCalledTimes(1);
    const [url, init] = fetchSpy.mock.calls[0];
    expect(requestUrlOf(url)).toBe("https://api.deepseek.com/chat/completions");
    const body = requestBodyOf(init) as { model: string; response_format: unknown };
    expect(body.model).toBe("deepseek-v4-flash");
    expect(body.response_format).toEqual({ type: "json_object" });
    expect((init?.headers as Record<string, string>).authorization).toBe("Bearer sk-test");
    expect(result.warnings).toEqual([]);
    // Confidence bị kẹp về 0..1; thiếu confidence thì mặc định 0.75.
    expect(result.entities).toEqual([
      { text: "萧炎", entityType: "person", suggested: "Tiêu Viêm", confidence: 1 },
      { text: "药老", entityType: undefined, suggested: undefined, confidence: 0.75 },
    ]);
  });

  it("targets the Gemini wire format when configured", async () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      Response.json({
        candidates: [
          {
            content: {
              parts: [{ text: JSON.stringify({ entities: [] }) }],
            },
          },
        ],
      }),
    );
    const result = await extractEntities("萧炎。", {
      provider: "gemini",
      apiKey: "AIza-test",
      model: "gemini-3.1-flash-lite",
      baseUrl: "https://relay.example.com",
    });
    const [url, init] = fetchSpy.mock.calls[0];
    expect(requestUrlOf(url)).toBe(
      "https://relay.example.com/v1beta/models/gemini-3.1-flash-lite:generateContent",
    );
    const headers = init?.headers as Record<string, string>;
    expect(headers["x-goog-api-key"]).toBe("AIza-test");
    expect(headers.authorization).toBeUndefined();
    const body = requestBodyOf(init) as {
      generationConfig: { responseMimeType: string };
    };
    expect(body.generationConfig.responseMimeType).toBe("application/json");
    expect(result.entities).toEqual([]);
  });

  it("keeps finished chunks and reports failed ones as warnings", async () => {
    let call = 0;
    vi.spyOn(globalThis, "fetch").mockImplementation(() => {
      call += 1;
      if (call === 1) {
        return Promise.resolve(new Response("boom", { status: 500 }));
      }
      return Promise.resolve(
        Response.json({
          choices: [
            {
              message: {
                content: JSON.stringify({ entities: [{ text: "药老", confidence: 0.8 }] }),
              },
            },
          ],
        }),
      );
    });
    // Hai dòng, mỗi dòng quá nửa giới hạn 15k → hai khúc riêng.
    const chapter = `${"甲".repeat(9_000)}\n${"乙".repeat(9_000)}`;
    const result = await extractEntities(chapter, {
      provider: "deepseek",
      apiKey: "sk-test",
      model: "deepseek-chat",
      baseUrl: "https://api.deepseek.com",
    });
    expect(result.entities).toEqual([{ text: "药老", entityType: undefined, suggested: undefined, confidence: 0.8 }]);
    expect(result.warnings).toHaveLength(1);
    expect(result.warnings[0]).toContain("DeepSeek returned 500");
  });
});

describe("reviewCandidates", () => {
  it("keeps only decisions about known candidates", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      Response.json({
        choices: [
          {
            message: {
              content: JSON.stringify({
                decisions: [
                  { text: "萧炎", keep: true, confidence: 0.9, entityType: "person" },
                  { text: "bịa ra", keep: false, confidence: 0.9, entityType: "unknown" },
                ],
              }),
            },
          },
        ],
      }),
    );
    const result = await reviewCandidates("萧炎走来。", [candidate({ text: "萧炎" })], {
      provider: "deepseek",
      apiKey: "sk-test",
      model: "deepseek-chat",
      baseUrl: "https://api.deepseek.com",
    });
    expect(result.warnings).toEqual([]);
    expect(result.decisions).toEqual([
      { text: "萧炎", keep: true, confidence: 0.9, entityType: "person", suggested: undefined },
    ]);
  });

  it("turns provider failures into warnings instead of throwing", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(new Response("quota", { status: 429 }));
    const result = await reviewCandidates("萧炎走来。", [candidate({ text: "萧炎" })], {
      provider: "deepseek",
      apiKey: "sk-test",
      model: "deepseek-chat",
      baseUrl: "https://api.deepseek.com",
    });
    expect(result.decisions).toEqual([]);
    expect(result.warnings[0]).toContain("DeepSeek returned 429");
  });
});
