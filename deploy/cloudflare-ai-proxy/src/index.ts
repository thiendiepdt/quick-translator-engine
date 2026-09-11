/**
 * Reverse proxy https cho một hub OpenAI-compatible.
 *
 * qt-web deploy trên https không gọi được hub http://IP (trình duyệt chặn
 * mixed-content). Worker này đứng trước hub với URL https của Cloudflare:
 *
 *   Browser (qt-web) -> Worker (https) -> UPSTREAM_BASE_URL (http/https)
 *
 * Worker không giữ API key nào: header Authorization của client được chuyển
 * nguyên sang hub, key vẫn nằm trên trình duyệt người dùng. Chỉ hai route
 * qt-web cần được mở; body và stream SSE đi qua nguyên vẹn, không log nội dung.
 */

const MAX_REQUEST_BODY_BYTES = 5 * 1024 * 1024;
const ALLOWED_CORS_REQUEST_HEADERS = new Set(["accept", "authorization", "content-type"]);

const ALLOWED_ROUTES = new Map<string, ReadonlySet<string>>([
  ["/chat/completions", new Set(["POST"])],
  ["/models", new Set(["GET"])],
]);

class HttpError extends Error {
  constructor(
    readonly status: number,
    message: string,
    readonly headers?: HeadersInit,
  ) {
    super(message);
  }
}

/** Cùng dạng `{error:{message}}` của OpenAI để qt-web hiện được thông báo. */
function errorResponse(status: number, message: string, requestId: string): Response {
  return Response.json(
    { error: { message, type: "proxy_error" } },
    {
      status,
      headers: {
        "cache-control": "no-store",
        "x-content-type-options": "nosniff",
        "x-request-id": requestId,
      },
    },
  );
}

function validateRoute(method: string, pathname: string): void {
  const methods = ALLOWED_ROUTES.get(pathname);
  if (!methods) {
    throw new HttpError(404, "Not found");
  }
  if (!methods.has(method)) {
    throw new HttpError(405, "Method not allowed", { allow: [...methods].join(", ") });
  }
}

function allowedCorsOrigin(request: Request, env: Env): string | undefined {
  const origin = request.headers.get("origin");
  if (origin === null) {
    return undefined;
  }
  const configured = env.CORS_ALLOWED_ORIGINS.split(",")
    .map((value) => value.trim())
    .filter(Boolean);
  if (configured.includes("*")) {
    return "*";
  }
  return configured.includes(origin) ? origin : undefined;
}

function applyCors(response: Response, origin: string | undefined): Response {
  if (origin === undefined) {
    return response;
  }
  response.headers.set("access-control-allow-origin", origin);
  response.headers.set("access-control-expose-headers", "x-request-id");
  const vary = response.headers.get("vary");
  if (
    origin !== "*" &&
    !vary
      ?.toLowerCase()
      .split(",")
      .map((value) => value.trim())
      .includes("origin")
  ) {
    response.headers.set("vary", vary ? `${vary}, Origin` : "Origin");
  }
  return response;
}

function preflightResponse(request: Request, origin: string): Response {
  const pathname = new URL(request.url).pathname;
  const requestedMethod = request.headers.get("access-control-request-method")?.toUpperCase();
  if (!requestedMethod) {
    throw new HttpError(400, "Missing Access-Control-Request-Method");
  }
  validateRoute(requestedMethod, pathname);

  const requestedHeaders = (request.headers.get("access-control-request-headers") ?? "")
    .split(",")
    .map((value) => value.trim().toLowerCase())
    .filter(Boolean);
  if (requestedHeaders.some((header) => !ALLOWED_CORS_REQUEST_HEADERS.has(header))) {
    throw new HttpError(403, "CORS request header is not allowed");
  }

  return applyCors(
    new Response(null, {
      status: 204,
      headers: {
        "access-control-allow-headers": "accept, authorization, content-type",
        "access-control-allow-methods": requestedMethod,
        "access-control-max-age": "86400",
        "cache-control": "no-store",
        "x-content-type-options": "nosniff",
      },
    }),
    origin,
  );
}

/** Base URL của hub; worker chỉ nối tới đúng origin/prefix này. */
function upstreamBase(env: Env): URL {
  let base: URL;
  try {
    base = new URL(env.UPSTREAM_BASE_URL);
  } catch {
    throw new Error("UPSTREAM_BASE_URL must be a valid URL");
  }
  if (
    (base.protocol !== "http:" && base.protocol !== "https:") ||
    base.search !== "" ||
    base.hash !== "" ||
    base.username !== "" ||
    base.password !== ""
  ) {
    throw new Error("UPSTREAM_BASE_URL must be a plain http(s) URL without query or credentials");
  }
  base.pathname = base.pathname.replace(/\/+$/, "");
  return base;
}

function parseContentLength(request: Request): number | undefined {
  const value = request.headers.get("content-length");
  if (value === null) {
    return undefined;
  }
  if (!/^\d+$/.test(value)) {
    throw new HttpError(400, "Invalid Content-Length");
  }
  return Number(value);
}

/**
 * Prompt dịch một chương chỉ vài trăm KB; đọc hết vào bộ nhớ (có trần) để
 * gửi lên hub với Content-Length rõ ràng thay vì chunked upload.
 */
async function readRequestBody(request: Request): Promise<ArrayBuffer | undefined> {
  if (request.method === "GET" || request.method === "HEAD") {
    return undefined;
  }
  const contentLength = parseContentLength(request);
  if (contentLength !== undefined && contentLength > MAX_REQUEST_BODY_BYTES) {
    throw new HttpError(413, "Request body exceeds 5 MiB");
  }
  if (request.body === null) {
    return new ArrayBuffer(0);
  }

  const reader = request.body.getReader();
  const chunks: Uint8Array[] = [];
  let total = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) {
        break;
      }
      total += value.byteLength;
      if (total > MAX_REQUEST_BODY_BYTES) {
        await reader.cancel("request body too large");
        throw new HttpError(413, "Request body exceeds 5 MiB");
      }
      chunks.push(value);
    }
  } finally {
    reader.releaseLock();
  }

  const body = new ArrayBuffer(total);
  const view = new Uint8Array(body);
  let offset = 0;
  for (const chunk of chunks) {
    view.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return body;
}

/** Chỉ chuyển ba header cần cho API; cookie và mọi thứ khác dừng ở đây. */
function upstreamHeaders(request: Request, requestId: string): Headers {
  const authorization = request.headers.get("authorization");
  if (authorization === null || authorization.trim() === "") {
    throw new HttpError(401, "Missing Authorization header");
  }
  const headers = new Headers({
    authorization,
    accept: request.headers.get("accept") ?? "application/json",
    "x-request-id": requestId,
  });
  const contentType = request.headers.get("content-type");
  if (contentType !== null) {
    headers.set("content-type", contentType);
  }
  return headers;
}

async function proxyToUpstream(request: Request, env: Env, requestId: string): Promise<Response> {
  const incoming = new URL(request.url);
  validateRoute(request.method, incoming.pathname);

  if (
    request.method === "POST" &&
    request.headers.get("content-type")?.split(";", 1)[0]?.trim().toLowerCase() !==
      "application/json"
  ) {
    throw new HttpError(415, "Content-Type must be application/json");
  }

  const target = upstreamBase(env);
  target.pathname = `${target.pathname}${incoming.pathname}`;
  target.search = incoming.search;

  const headers = upstreamHeaders(request, requestId);
  const body = await readRequestBody(request);
  const upstream = await fetch(target.toString(), {
    method: request.method,
    headers,
    body,
    redirect: "manual",
  });

  const responseHeaders = new Headers();
  const contentType = upstream.headers.get("content-type");
  if (contentType !== null) {
    responseHeaders.set("content-type", contentType);
  }
  responseHeaders.set("cache-control", "no-store");
  responseHeaders.set("x-content-type-options", "nosniff");
  responseHeaders.set("x-request-id", requestId);
  // Body (kể cả SSE) stream thẳng về client, không đọc vào bộ nhớ.
  return new Response(upstream.body, {
    status: upstream.status,
    statusText: upstream.statusText,
    headers: responseHeaders,
  });
}

export default {
  async fetch(request: Request, env: Env, _ctx: ExecutionContext): Promise<Response> {
    const requestId = crypto.randomUUID();
    const startedAt = performance.now();
    const requestOrigin = request.headers.get("origin");
    const corsOrigin = allowedCorsOrigin(request, env);
    if (requestOrigin !== null && corsOrigin === undefined) {
      return errorResponse(403, "Origin is not allowed", requestId);
    }
    try {
      if (request.method === "OPTIONS") {
        if (corsOrigin === undefined) {
          throw new HttpError(400, "Missing Origin");
        }
        return preflightResponse(request, corsOrigin);
      }
      const response = applyCors(await proxyToUpstream(request, env, requestId), corsOrigin);
      console.log(
        JSON.stringify({
          event: "ai_proxy",
          requestId,
          method: request.method,
          path: new URL(request.url).pathname,
          status: response.status,
          durationMs: Math.round(performance.now() - startedAt),
        }),
      );
      return response;
    } catch (error) {
      if (error instanceof HttpError) {
        const response = errorResponse(error.status, error.message, requestId);
        if (error.headers !== undefined) {
          for (const [name, value] of new Headers(error.headers)) {
            response.headers.set(name, value);
          }
        }
        return applyCors(response, corsOrigin);
      }
      console.error(
        JSON.stringify({
          event: "ai_proxy_error",
          requestId,
          method: request.method,
          path: new URL(request.url).pathname,
          error: error instanceof Error ? error.message : "Unknown error",
        }),
      );
      return applyCors(errorResponse(502, "Upstream request failed", requestId), corsOrigin);
    }
  },
} satisfies ExportedHandler<Env>;
