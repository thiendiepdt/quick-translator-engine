import { afterEach, describe, expect, it, vi } from "vitest";

import {
  buildGeminiTextGenerationConfig,
  GeminiBlockedError,
  generateAiText,
} from "@/lib/ai-text-client";

afterEach(() => {
  vi.restoreAllMocks();
});

function sse(...payloads: unknown[]): Response {
  return new Response(
    payloads.map((payload) => `data: ${JSON.stringify(payload)}\n\n`).join("") +
      "data: [DONE]\n\n",
    { headers: { "content-type": "text/event-stream" } },
  );
}

function requestUrlOf(input: RequestInfo | URL): string {
  if (typeof input === "string") return input;
  return input instanceof URL ? input.href : input.url;
}

describe("Gemini text generation settings", () => {
  it("ports model-specific thinking behavior from Novel Translator", () => {
    expect(buildGeminiTextGenerationConfig("gemini-2.5-flash", true)).toEqual({
      temperature: 0.3,
      maxOutputTokens: 65_536,
      thinkingConfig: { thinkingBudget: -1, includeThoughts: true },
    });
    expect(buildGeminiTextGenerationConfig("gemini-2.5-flash", false)).toEqual({
      temperature: 0.3,
      maxOutputTokens: 65_536,
      thinkingConfig: { thinkingBudget: 0, includeThoughts: true },
    });
    // Bật thinking trên 3.x: ép "high" tường minh — default của API để model
    // tự co mức nghĩ, chương "dễ" sẽ bị nghĩ nông.
    expect(buildGeminiTextGenerationConfig("gemini-3.5-flash", true)).toEqual({
      maxOutputTokens: 65_536,
      thinkingConfig: { thinkingLevel: "high", includeThoughts: true },
    });
    expect(buildGeminiTextGenerationConfig("gemini-3.5-flash", false)).toEqual({
      maxOutputTokens: 65_536,
      thinkingConfig: { thinkingLevel: "minimal", includeThoughts: true },
    });
  });
});

describe("generateAiText", () => {
  it("streams Gemini thought and output parts separately", async () => {
    const chunks: Array<[string, string]> = [];
    const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      sse({
        candidates: [
          {
            content: {
              parts: [
                { thought: true, text: "Đối chiếu raw" },
                { text: "Tiêu Viêm bước vào." },
              ],
            },
          },
        ],
      }),
    );

    await expect(
      generateAiText(
        {
          provider: "gemini",
          apiKey: "AIza-test",
          model: "gemini-3.5-flash",
          baseUrl: "https://generativelanguage.googleapis.com",
        },
        "system",
        "萧炎走来。",
        { thinking: true, onChunk: (kind, text) => chunks.push([kind, text]) },
      ),
    ).resolves.toBe("Tiêu Viêm bước vào.");

    const [url, init] = fetchSpy.mock.calls[0];
    expect(requestUrlOf(url)).toContain(
      "/v1beta/models/gemini-3.5-flash:streamGenerateContent?alt=sse",
    );
    expect((init?.headers as Record<string, string>)["x-goog-api-key"]).toBe(
      "AIza-test",
    );
    expect(chunks).toEqual([
      ["thinking", "Đối chiếu raw"],
      ["text", "Tiêu Viêm bước vào."],
    ]);
  });

  it("enables Google Search only for metadata calls that request it", async () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      sse({ candidates: [{ content: { parts: [{ text: "{}" }] } }] }),
    );

    await generateAiText(
      {
        provider: "gemini",
        apiKey: "AIza-test",
        model: "gemini-3.5-flash",
        baseUrl: "https://generativelanguage.googleapis.com",
      },
      "system",
      "metadata",
      { thinking: false, googleSearch: true },
    );

    const [, init] = fetchSpy.mock.calls[0];
    expect(JSON.parse(init?.body as string)).toMatchObject({
      tools: [{ googleSearch: {} }],
    });
  });

  it("reports Google Search grounding once when metadata appears in the stream", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      sse(
        {
          candidates: [
            {
              content: { parts: [{ text: "{" }] },
              groundingMetadata: { webSearchQueries: ["方寸道主"] },
            },
          ],
        },
        {
          candidates: [
            {
              content: { parts: [{ text: "}" }] },
              groundingMetadata: { webSearchQueries: ["方寸道主"] },
            },
          ],
        },
      ),
    );

    const onGoogleSearchUsed = vi.fn();
    await generateAiText(
      {
        provider: "gemini",
        apiKey: "AIza-test",
        model: "gemini-3.5-flash",
        baseUrl: "https://generativelanguage.googleapis.com",
      },
      "system",
      "metadata",
      { thinking: false, googleSearch: true, onGoogleSearchUsed },
    );
    expect(onGoogleSearchUsed).toHaveBeenCalledTimes(1);
  });

  it("does not report grounding when Gemini answers without searching", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      sse({ candidates: [{ content: { parts: [{ text: "{}" }] }, groundingMetadata: {} }] }),
    );

    const onGoogleSearchUsed = vi.fn();
    await generateAiText(
      {
        provider: "gemini",
        apiKey: "AIza-test",
        model: "gemini-3.5-flash",
        baseUrl: "https://generativelanguage.googleapis.com",
      },
      "system",
      "metadata",
      { thinking: false, googleSearch: true, onGoogleSearchUsed },
    );
    expect(onGoogleSearchUsed).not.toHaveBeenCalled();
  });

  it("uses the separate DeepSeek model/thinking shape and parses reasoning", async () => {
    const chunks: Array<[string, string]> = [];
    const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      sse(
        { choices: [{ delta: { reasoning_content: "Suy luận" } }] },
        { choices: [{ delta: { content: "Bản dịch" } }] },
      ),
    );

    await expect(
      generateAiText(
        {
          provider: "deepseek",
          apiKey: "sk-test",
          model: "deepseek-translate",
          baseUrl: "https://api.deepseek.com",
        },
        "system",
        "raw",
        { thinking: false, onChunk: (kind, text) => chunks.push([kind, text]) },
      ),
    ).resolves.toBe("Bản dịch");

    const [, init] = fetchSpy.mock.calls[0];
    const body = JSON.parse(init?.body as string) as {
      model: string;
      thinking: { type: string };
      stream: boolean;
    };
    expect(body).toMatchObject({
      model: "deepseek-translate",
      thinking: { type: "disabled" },
      stream: true,
    });
    expect(chunks).toEqual([
      ["thinking", "Suy luận"],
      ["text", "Bản dịch"],
    ]);
  });
});

describe("grok text generation", () => {
  it("streams via chat/completions without the DeepSeek thinking field", async () => {
    const fetchSpy = vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        `data: ${JSON.stringify({ choices: [{ delta: { content: "Bản dịch Grok." } }] })}\n\ndata: [DONE]\n\n`,
      ),
    );
    const result = await generateAiText(
      { provider: "grok", apiKey: "xai-key", model: "grok-4.6", baseUrl: "https://api.x.ai/v1" },
      "system",
      "user",
      { thinking: true },
    );
    expect(result).toBe("Bản dịch Grok.");
    expect(requestUrlOf(fetchSpy.mock.calls[0][0])).toBe(
      "https://api.x.ai/v1/chat/completions",
    );
    const body = JSON.parse((fetchSpy.mock.calls[0][1] as RequestInit).body as string) as Record<string, unknown>;
    expect(body.model).toBe("grok-4.6");
    expect("thinking" in body).toBe(false);
    expect(
      new Headers((fetchSpy.mock.calls[0][1] as RequestInit).headers).get("authorization"),
    ).toBe("Bearer xai-key");
  });
});

describe("glm text generation", () => {
  const glmConfig = {
    provider: "glm" as const,
    apiKey: "zai-key",
    model: "glm-5.3-flash",
    baseUrl: "https://api.z.ai/api/paas/v4",
  };

  function mockGlmStream() {
    return vi.spyOn(globalThis, "fetch").mockResolvedValue(
      sse(
        { choices: [{ delta: { reasoning_content: "Suy luận GLM" } }] },
        { choices: [{ delta: { content: "Bản dịch GLM." } }] },
      ),
    );
  }

  it("streams via chat/completions with the Z.ai recommended parameters", async () => {
    const fetchSpy = mockGlmStream();
    const chunks: Array<[string, string]> = [];
    const result = await generateAiText(glmConfig, "system", "user", {
      thinking: true,
      onChunk: (kind, text) => chunks.push([kind, text]),
    });

    expect(result).toBe("Bản dịch GLM.");
    expect(requestUrlOf(fetchSpy.mock.calls[0][0])).toBe(
      "https://api.z.ai/api/paas/v4/chat/completions",
    );
    const init = fetchSpy.mock.calls[0][1] as RequestInit;
    expect(new Headers(init.headers).get("authorization")).toBe("Bearer zai-key");
    expect(JSON.parse(init.body as string) as Record<string, unknown>).toMatchObject({
      model: "glm-5.3-flash",
      thinking: { type: "enabled", clear_thinking: false },
      reasoning_effort: "high",
      temperature: 1,
      top_p: 0.95,
      stream: true,
    });
    expect(chunks).toEqual([
      ["thinking", "Suy luận GLM"],
      ["text", "Bản dịch GLM."],
    ]);
  });

  // Bỏ trống reasoning_effort là rơi về "max" của Z.ai, chương nào cũng nghĩ
  // hàng chục phút — công tắc Thinking phải gửi mức thấp tường minh.
  it("drops to the lowest supported reasoning effort when thinking is off", async () => {
    const fetchSpy = mockGlmStream();
    await generateAiText(glmConfig, "system", "user", { thinking: false });

    const body = JSON.parse(
      (fetchSpy.mock.calls[0][1] as RequestInit).body as string,
    ) as Record<string, unknown>;
    expect(body.thinking).toEqual({ type: "enabled", clear_thinking: false });
    expect(body.reasoning_effort).toBe("low");
  });
});

describe("Gemini content blocking", () => {
  it("throws a typed error carrying the block reason", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(
        `data: ${JSON.stringify({ candidates: [{ finishReason: "PROHIBITED_CONTENT" }] })}\n\ndata: [DONE]\n\n`,
      ),
    );
    const call = generateAiText(
      { provider: "gemini", apiKey: "AIza", model: "gemini-3.7-flash", baseUrl: "https://generativelanguage.googleapis.com" },
      "system",
      "user",
      { thinking: false },
    );
    await expect(call).rejects.toBeInstanceOf(GeminiBlockedError);
    await expect(call).rejects.toMatchObject({ reason: "PROHIBITED_CONTENT" });
  });
});
