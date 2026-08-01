import {
  dictionaryDefaultsSchema,
  healthResponseSchema,
  nameFilterResponseSchema,
  translationResponseSchema,
} from "@/lib/schema";
import type {
  DictionaryDefaults,
  HealthResponse,
  NameFilterRequest,
  NameFilterResponse,
  TranslationRequest,
  TranslationResponse,
} from "@/lib/types";

const REQUEST_TIMEOUT_MS = 45_000;
// Lọc tên có AI chạy nhiều lượt gọi model: server tự dừng AI ở ~100 giây và
// Lambda cho phép 120 giây, nên client phải chờ lâu hơn cả hai mốc đó.
const AI_REQUEST_TIMEOUT_MS = 130_000;

export class ApiError extends Error {
  constructor(
    message: string,
    readonly status?: number,
    readonly requestId?: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

function endpointUrl(endpoint: string, pathname: string): string {
  const base = endpoint.trim().replace(/\/$/, "");
  if (base.startsWith("/")) return `${base}${pathname}`;
  return new URL(pathname.replace(/^\//, ""), `${base}/`).toString();
}

async function responseError(response: Response): Promise<ApiError> {
  const requestId = response.headers.get("x-request-id") ?? undefined;
  const contentType = response.headers.get("content-type") ?? "";
  let message = `Request thất bại với HTTP ${response.status}`;

  if (contentType.includes("application/json")) {
    const body = (await response.json().catch(() => undefined)) as { error?: unknown } | undefined;
    if (typeof body?.error === "string") message = body.error;
  } else {
    const text = await response.text().catch(() => "");
    if (text.trim()) message = text.trim();
  }

  return new ApiError(message, response.status, requestId);
}

async function requestJson<T>(
  url: string,
  init: RequestInit,
  parse: (value: unknown) => T,
  timeoutMs = REQUEST_TIMEOUT_MS,
): Promise<T> {
  const controller = new AbortController();
  const timeout = window.setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await fetch(url, {
      ...init,
      signal: controller.signal,
      headers: {
        accept: "application/json",
        ...init.headers,
      },
    });
    if (!response.ok) throw await responseError(response);
    return parse(await response.json());
  } catch (error) {
    if (error instanceof DOMException && error.name === "AbortError") {
      throw new ApiError(`Request quá ${Math.round(timeoutMs / 1000)} giây và đã bị hủy`);
    }
    if (error instanceof ApiError) throw error;
    throw new ApiError(error instanceof Error ? error.message : "Không thể gọi API");
  } finally {
    window.clearTimeout(timeout);
  }
}

export function translateChapter(
  endpoint: string,
  request: TranslationRequest,
): Promise<TranslationResponse> {
  return requestJson(
    endpointUrl(endpoint, "/translate"),
    {
      method: "POST",
      headers: { "content-type": "application/json; charset=utf-8" },
      body: JSON.stringify(request),
    },
    (value) => translationResponseSchema.parse(value),
  );
}

export function checkHealth(endpoint: string): Promise<HealthResponse> {
  return requestJson(
    endpointUrl(endpoint, "/health"),
    { method: "GET" },
    (value) => healthResponseSchema.parse(value),
  );
}

export function fetchDictionaryDefaults(endpoint: string): Promise<DictionaryDefaults> {
  return requestJson(
    endpointUrl(endpoint, "/dictionaries/defaults"),
    { method: "GET" },
    (value) => dictionaryDefaultsSchema.parse(value),
  );
}

export function filterChapterNames(
  endpoint: string,
  request: NameFilterRequest,
): Promise<NameFilterResponse> {
  const aiEnabled = request.aiExtract.enabled || request.aiFallback.enabled;
  return requestJson(
    endpointUrl(endpoint, "/names/filter"),
    {
      method: "POST",
      headers: { "content-type": "application/json; charset=utf-8" },
      body: JSON.stringify(request),
    },
    (value) => nameFilterResponseSchema.parse(value),
    aiEnabled ? AI_REQUEST_TIMEOUT_MS : REQUEST_TIMEOUT_MS,
  );
}
