import { AwsClient } from "aws4fetch";

const MAX_REQUEST_BODY_BYTES = 5 * 1024 * 1024;

const ALLOWED_ROUTES = new Map<string, ReadonlySet<string>>([
  ["/health", new Set(["GET"])],
  ["/modes", new Set(["GET"])],
  ["/translate", new Set(["POST"])],
  ["/translate/batch", new Set(["POST"])],
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

function errorResponse(status: number, message: string, requestId: string): Response {
  return Response.json(
    { error: message },
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
    throw new HttpError(405, "Method not allowed", {
      allow: [...methods].join(", "),
    });
  }
}

function lambdaOrigin(env: Env): URL {
  let origin: URL;
  try {
    origin = new URL(env.LAMBDA_FUNCTION_URL);
  } catch {
    throw new Error("LAMBDA_FUNCTION_URL must be a valid URL");
  }

  const expectedSuffix = `.lambda-url.${env.AWS_REGION}.on.aws`;
  if (
    origin.protocol !== "https:" ||
    !origin.hostname.endsWith(expectedSuffix) ||
    origin.hostname === expectedSuffix.slice(1) ||
    origin.pathname !== "/" ||
    origin.search !== "" ||
    origin.hash !== "" ||
    origin.username !== "" ||
    origin.password !== ""
  ) {
    throw new Error(
      "LAMBDA_FUNCTION_URL must be an HTTPS Lambda Function URL in AWS_REGION",
    );
  }
  return origin;
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
  const bodyView = new Uint8Array(body);
  let offset = 0;
  for (const chunk of chunks) {
    bodyView.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return body;
}

function upstreamHeaders(request: Request, requestId: string): Headers {
  const headers = new Headers({
    accept: request.headers.get("accept") ?? "application/json",
    "x-request-id": requestId,
  });
  const contentType = request.headers.get("content-type");
  if (contentType !== null) {
    headers.set("content-type", contentType);
  }
  return headers;
}

async function proxyToLambda(
  request: Request,
  env: Env,
  requestId: string,
): Promise<Response> {
  const incoming = new URL(request.url);
  validateRoute(request.method, incoming.pathname);

  if (
    request.method === "POST" &&
    request.headers.get("content-type")?.split(";", 1)[0]?.trim().toLowerCase() !==
      "application/json"
  ) {
    throw new HttpError(415, "Content-Type must be application/json");
  }

  const target = lambdaOrigin(env);
  target.pathname = incoming.pathname;
  target.search = incoming.search;

  const body = await readRequestBody(request);
  const aws = new AwsClient({
    accessKeyId: env.AWS_ACCESS_KEY_ID,
    secretAccessKey: env.AWS_SECRET_ACCESS_KEY,
    service: "lambda",
    region: env.AWS_REGION,
    retries: 0,
  });

  const upstream = await aws.fetch(target.toString(), {
    method: request.method,
    headers: upstreamHeaders(request, requestId),
    body,
    redirect: "manual",
  });

  const headers = new Headers(upstream.headers);
  headers.set("cache-control", "no-store");
  headers.set("x-content-type-options", "nosniff");
  headers.set("x-request-id", requestId);
  return new Response(upstream.body, {
    status: upstream.status,
    statusText: upstream.statusText,
    headers,
  });
}

export default {
  async fetch(request: Request, env: Env, _ctx: ExecutionContext): Promise<Response> {
    const requestId = crypto.randomUUID();
    const startedAt = performance.now();
    try {
      const response = await proxyToLambda(request, env, requestId);
      console.log(
        JSON.stringify({
          event: "lambda_proxy",
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
          const headers = new Headers(error.headers);
          for (const [name, value] of headers) {
            response.headers.set(name, value);
          }
        }
        return response;
      }

      console.error(
        JSON.stringify({
          event: "lambda_proxy_error",
          requestId,
          method: request.method,
          path: new URL(request.url).pathname,
          error: error instanceof Error ? error.message : "Unknown error",
        }),
      );
      return errorResponse(502, "Upstream request failed", requestId);
    }
  },
} satisfies ExportedHandler<Env>;
