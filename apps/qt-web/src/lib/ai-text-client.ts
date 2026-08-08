import type { AiCallConfig } from "@/lib/ai-client";

export type AiTextChunkKind = "thinking" | "text";

export interface AiTextGenerationOptions {
  thinking: boolean;
  signal?: AbortSignal;
  onChunk?: (kind: AiTextChunkKind, chunk: string) => void;
  /** Gemini-only: bật grounding Google Search cho tác vụ tra cứu metadata. */
  googleSearch?: boolean;
  /** Gemini-only: gọi khi response có grounding metadata — model đã tra Google thật. */
  onGoogleSearchUsed?: () => void;
}

const MAX_OUTPUT_TOKENS = 65_536;

const GEMINI_SAFETY_SETTINGS = [
  "HARM_CATEGORY_HARASSMENT",
  "HARM_CATEGORY_HATE_SPEECH",
  "HARM_CATEGORY_SEXUALLY_EXPLICIT",
  "HARM_CATEGORY_DANGEROUS_CONTENT",
  "HARM_CATEGORY_CIVIC_INTEGRITY",
].map((category) => ({ category, threshold: "OFF" }));

function normalizedModel(model: string): string {
  return model.replace(/^models\//, "");
}

function geminiModelMajor(model: string): number | undefined {
  const match = /^gemini-(\d+)/.exec(normalizedModel(model));
  return match ? Number(match[1]) : undefined;
}

/** Port từ `build_generation_config` của novel-translator. */
export function buildGeminiTextGenerationConfig(
  model: string,
  thinkingEnabled: boolean,
): Record<string, unknown> {
  const major = geminiModelMajor(model);
  const config: Record<string, unknown> = { maxOutputTokens: MAX_OUTPUT_TOKENS };
  if (major === undefined || major < 3) config.temperature = 0.3;

  const thinkingConfig: Record<string, unknown> = {};
  if (major === 2 && normalizedModel(model).includes("2.5")) {
    thinkingConfig.thinkingBudget = thinkingEnabled ? -1 : 0;
  } else if (major !== undefined && major >= 3 && !thinkingEnabled) {
    // Gemini 3.x không tắt hoàn toàn thinking; minimal là mức thấp nhất.
    thinkingConfig.thinkingLevel = "minimal";
  }
  if ((major !== undefined && major >= 3) || Object.keys(thinkingConfig).length > 0) {
    thinkingConfig.includeThoughts = true;
  }
  if (Object.keys(thinkingConfig).length > 0) config.thinkingConfig = thinkingConfig;
  return config;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function truncate(value: string, maxCharacters: number): string {
  return [...value].slice(0, maxCharacters).join("");
}

async function responseProblem(response: Response, provider: string): Promise<Error> {
  const body = await response.text().catch(() => "");
  let detail = body;
  try {
    const parsed: unknown = JSON.parse(body);
    if (isRecord(parsed) && isRecord(parsed.error)) {
      const message = parsed.error.message;
      if (typeof message === "string") detail = message;
    }
  } catch {
    // Giữ response text khi body không phải JSON.
  }
  return new Error(`${provider} returned ${response.status}: ${truncate(detail, 500)}`);
}

async function readSse(
  response: Response,
  onData: (payload: unknown) => void,
): Promise<void> {
  if (!response.body) throw new Error("AI returned an empty response stream");
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";

  function processLine(line: string) {
    const data = line.trimEnd().replace(/^data:\s?/, "");
    if (data === line.trimEnd() || !data || data === "[DONE]") return;
    try {
      onData(JSON.parse(data) as unknown);
    } catch (error) {
      throw new Error(
        `Không đọc được stream AI: ${error instanceof Error ? error.message : String(error)}`,
        { cause: error },
      );
    }
  }

  for (;;) {
    const { done, value } = await reader.read();
    buffer += decoder.decode(value, { stream: !done });
    let newline = buffer.indexOf("\n");
    while (newline >= 0) {
      processLine(buffer.slice(0, newline).replace(/\r$/, ""));
      buffer = buffer.slice(newline + 1);
      newline = buffer.indexOf("\n");
    }
    if (done) break;
  }
  if (buffer.trim()) processLine(buffer.replace(/\r$/, ""));
}

/** Grounding metadata chỉ xuất hiện khi model thật sự chạy Google Search. */
function geminiSearchedGoogle(candidate: Record<string, unknown>): boolean {
  const metadata = candidate.groundingMetadata;
  if (!isRecord(metadata)) return false;
  return (
    (Array.isArray(metadata.webSearchQueries) && metadata.webSearchQueries.length > 0) ||
    (Array.isArray(metadata.groundingChunks) && metadata.groundingChunks.length > 0)
  );
}

function geminiBlockedReason(payload: Record<string, unknown>): string | undefined {
  const feedback = payload.promptFeedback;
  if (isRecord(feedback) && typeof feedback.blockReason === "string") {
    return feedback.blockReason;
  }
  const candidates = payload.candidates;
  if (!Array.isArray(candidates) || !isRecord(candidates[0])) return undefined;
  const reason = candidates[0].finishReason;
  return typeof reason === "string" && reason !== "STOP" && reason !== "MAX_TOKENS"
    ? reason
    : undefined;
}

async function generateGeminiText(
  config: AiCallConfig,
  systemPrompt: string,
  userMessage: string,
  options: AiTextGenerationOptions,
): Promise<string> {
  const model = normalizedModel(config.model);
  const response = await fetch(
    `${config.baseUrl}/v1beta/models/${encodeURIComponent(model)}:streamGenerateContent?alt=sse`,
    {
      method: "POST",
      headers: {
        "content-type": "application/json",
        "x-goog-api-key": config.apiKey,
      },
      body: JSON.stringify({
        systemInstruction: { parts: [{ text: systemPrompt }] },
        contents: [{ role: "user", parts: [{ text: userMessage }] }],
        ...(options.googleSearch ? { tools: [{ googleSearch: {} }] } : {}),
        safetySettings: GEMINI_SAFETY_SETTINGS,
        generationConfig: buildGeminiTextGenerationConfig(model, options.thinking),
      }),
      signal: options.signal,
    },
  );
  if (!response.ok) throw await responseProblem(response, "Gemini");

  let output = "";
  let blockedReason: string | undefined;
  let searchReported = false;
  await readSse(response, (payload) => {
    if (!isRecord(payload)) return;
    if (isRecord(payload.error) && typeof payload.error.message === "string") {
      throw new Error(payload.error.message);
    }
    blockedReason = geminiBlockedReason(payload) ?? blockedReason;
    const candidates = payload.candidates;
    if (!Array.isArray(candidates) || !isRecord(candidates[0])) return;
    if (!searchReported && geminiSearchedGoogle(candidates[0])) {
      searchReported = true;
      options.onGoogleSearchUsed?.();
    }
    const content = candidates[0].content;
    if (!isRecord(content) || !Array.isArray(content.parts)) return;
    for (const part of content.parts) {
      if (!isRecord(part) || typeof part.text !== "string" || !part.text) continue;
      const kind = part.thought === true ? "thinking" : "text";
      options.onChunk?.(kind, part.text);
      if (kind === "text") output += part.text;
    }
  });

  if (output) return output;
  if (blockedReason) {
    throw new Error(`Gemini chặn nội dung với finishReason/blockReason: ${blockedReason}`);
  }
  throw new Error("Gemini không trả về nội dung");
}

async function generateDeepseekText(
  config: AiCallConfig,
  systemPrompt: string,
  userMessage: string,
  options: AiTextGenerationOptions,
): Promise<string> {
  const response = await fetch(`${config.baseUrl}/chat/completions`, {
    method: "POST",
    headers: {
      accept: "text/event-stream",
      "content-type": "application/json",
      authorization: `Bearer ${config.apiKey}`,
    },
    body: JSON.stringify({
      model: normalizedModel(config.model),
      messages: [
        { role: "system", content: systemPrompt },
        { role: "user", content: userMessage },
      ],
      thinking: { type: options.thinking ? "enabled" : "disabled" },
      max_tokens: MAX_OUTPUT_TOKENS,
      stream: true,
    }),
    signal: options.signal,
  });
  if (!response.ok) throw await responseProblem(response, "DeepSeek");

  let output = "";
  await readSse(response, (payload) => {
    if (!isRecord(payload)) return;
    if (isRecord(payload.error) && typeof payload.error.message === "string") {
      throw new Error(payload.error.message);
    }
    if (!Array.isArray(payload.choices) || !isRecord(payload.choices[0])) return;
    const delta = payload.choices[0].delta;
    if (!isRecord(delta)) return;
    if (typeof delta.reasoning_content === "string" && delta.reasoning_content) {
      options.onChunk?.("thinking", delta.reasoning_content);
    }
    if (typeof delta.content === "string" && delta.content) {
      output += delta.content;
      options.onChunk?.("text", delta.content);
    }
  });
  if (!output) throw new Error("DeepSeek không trả về nội dung");
  return output;
}

/**
 * Sinh text tự do bằng cùng credential/proxy mà lọc tên đang dùng. Model và
 * thinking đã được caller lấy từ cấu hình Dịch AI riêng.
 */
export async function generateAiText(
  config: AiCallConfig,
  systemPrompt: string,
  userMessage: string,
  options: AiTextGenerationOptions,
): Promise<string> {
  try {
    if (config.provider === "gemini") {
      return await generateGeminiText(config, systemPrompt, userMessage, options);
    }
    return await generateDeepseekText(config, systemPrompt, userMessage, options);
  } catch (error) {
    if (options.signal?.aborted || !(error instanceof TypeError)) throw error;
    const provider = config.provider === "gemini" ? "Gemini" : "DeepSeek";
    throw new Error(
      `${provider} request failed: ${error.message} (endpoint phải cho phép CORS)`,
      { cause: error },
    );
  }
}
