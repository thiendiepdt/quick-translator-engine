import { env } from "cloudflare:workers";
import { createExecutionContext } from "cloudflare:test";
import { afterEach, describe, expect, it, vi } from "vitest";

import worker from "../src/index";

afterEach(() => {
  vi.unstubAllGlobals();
});

function invoke(request: Request): Promise<Response> {
  return worker.fetch(request, env, createExecutionContext());
}

function stubUpstream(respond: (request: Request) => Response) {
  const calls: Request[] = [];
  const upstreamFetch = vi.fn((input: RequestInfo | URL, init?: RequestInit) => {
    const request = input instanceof Request ? input : new Request(input, init);
    calls.push(request);
    return Promise.resolve(respond(request));
  });
  vi.stubGlobal("fetch", upstreamFetch);
  return { upstreamFetch, calls };
}

function chatRequest(init: RequestInit = {}, url = "https://proxy.example.com/chat/completions") {
  return new Request(url, {
    method: "POST",
    headers: {
      accept: "text/event-stream",
      authorization: "Bearer sk-hub-client-key",
      "content-type": "application/json",
      cookie: "session=must-not-reach-hub",
      origin: "https://dich.example.com",
    },
    body: JSON.stringify({ model: "gemini-3.7-flash", messages: [], stream: true }),
    ...init,
  });
}

async function errorMessage(response: Response): Promise<string> {
  const payload: unknown = await response.json();
  if (typeof payload === "object" && payload !== null && "error" in payload) {
    const error = (payload as { error: { message?: unknown } }).error;
    if (typeof error.message === "string") return error.message;
  }
  throw new Error("expected an OpenAI-style error payload");
}

describe("OpenAI-compatible AI proxy", () => {
  it("forwards chat/completions to the hub with the client's key and streams SSE back", async () => {
    const sse = "data: {\"choices\":[{\"delta\":{\"content\":\"Xin chào\"}}]}\n\ndata: [DONE]\n\n";
    const { upstreamFetch, calls } = stubUpstream(
      () => new Response(sse, { headers: { "content-type": "text/event-stream" } }),
    );

    const response = await invoke(chatRequest());

    expect(response.status).toBe(200);
    expect(response.headers.get("content-type")).toBe("text/event-stream");
    expect(response.headers.get("cache-control")).toBe("no-store");
    expect(response.headers.get("access-control-allow-origin")).toBe("https://dich.example.com");
    expect(response.headers.get("x-request-id")).toBeTruthy();
    expect(await response.text()).toBe(sse);

    expect(upstreamFetch).toHaveBeenCalledTimes(1);
    const upstream = calls[0];
    if (upstream === undefined) throw new Error("expected an upstream request");
    expect(upstream.url).toBe("http://hub.test/v1/chat/completions");
    expect(upstream.method).toBe("POST");
    expect(upstream.headers.get("authorization")).toBe("Bearer sk-hub-client-key");
    expect(upstream.headers.get("content-type")).toBe("application/json");
    expect(upstream.headers.get("accept")).toBe("text/event-stream");
    expect(upstream.headers.get("cookie")).toBeNull();
    expect(upstream.headers.get("origin")).toBeNull();
    expect(await upstream.json()).toEqual({ model: "gemini-3.7-flash", messages: [], stream: true });
  });

  it("forwards GET /models and keeps the query string", async () => {
    const { calls } = stubUpstream(() => Response.json({ data: [] }));
    const response = await invoke(
      new Request("https://proxy.example.com/models?limit=5", {
        headers: { authorization: "Bearer sk-hub-client-key" },
      }),
    );
    expect(response.status).toBe(200);
    expect(await response.json()).toEqual({ data: [] });
    expect(calls[0]?.url).toBe("http://hub.test/v1/models?limit=5");
  });

  it("passes upstream error statuses through untouched", async () => {
    stubUpstream(
      () => Response.json({ error: { message: "model not found" } }, { status: 404 }),
    );
    const response = await invoke(chatRequest());
    expect(response.status).toBe(404);
    expect(await errorMessage(response)).toBe("model not found");
  });

  it("answers 502 in the OpenAI error shape when the hub is unreachable", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(() => Promise.reject(new Error("connect ECONNREFUSED"))),
    );
    const response = await invoke(chatRequest());
    expect(response.status).toBe(502);
    expect(await errorMessage(response)).toBe("Upstream request failed");
  });

  it("rejects requests without an Authorization header before touching the hub", async () => {
    const { upstreamFetch } = stubUpstream(() => Response.json({}));
    const response = await invoke(
      chatRequest({
        headers: { "content-type": "application/json", origin: "https://dich.example.com" },
      }),
    );
    expect(response.status).toBe(401);
    expect(upstreamFetch).not.toHaveBeenCalled();
  });

  it("only exposes the two routes qt-web needs", async () => {
    const { upstreamFetch } = stubUpstream(() => Response.json({}));
    const notFound = await invoke(
      new Request("https://proxy.example.com/embeddings", {
        method: "POST",
        headers: { authorization: "Bearer x", "content-type": "application/json" },
        body: "{}",
      }),
    );
    expect(notFound.status).toBe(404);
    const wrongMethod = await invoke(
      new Request("https://proxy.example.com/chat/completions", {
        headers: { authorization: "Bearer x" },
      }),
    );
    expect(wrongMethod.status).toBe(405);
    expect(wrongMethod.headers.get("allow")).toBe("POST");
    expect(upstreamFetch).not.toHaveBeenCalled();
  });

  it("refuses browsers from origins outside the allowlist", async () => {
    const { upstreamFetch } = stubUpstream(() => Response.json({}));
    const response = await invoke(
      chatRequest({
        headers: {
          authorization: "Bearer x",
          "content-type": "application/json",
          origin: "https://evil.example.com",
        },
      }),
    );
    expect(response.status).toBe(403);
    expect(response.headers.get("access-control-allow-origin")).toBeNull();
    expect(upstreamFetch).not.toHaveBeenCalled();
  });

  it("answers the preflight qt-web sends, allowing the Authorization header", async () => {
    const response = await invoke(
      new Request("https://proxy.example.com/chat/completions", {
        method: "OPTIONS",
        headers: {
          origin: "http://localhost:5173",
          "access-control-request-method": "POST",
          "access-control-request-headers": "authorization, content-type",
        },
      }),
    );
    expect(response.status).toBe(204);
    expect(response.headers.get("access-control-allow-origin")).toBe("http://localhost:5173");
    expect(response.headers.get("access-control-allow-headers")).toBe(
      "accept, authorization, content-type",
    );
    expect(response.headers.get("access-control-allow-methods")).toBe("POST");
  });

  it("rejects oversized bodies", async () => {
    const { upstreamFetch } = stubUpstream(() => Response.json({}));
    const response = await invoke(
      chatRequest({ body: "x".repeat(5 * 1024 * 1024 + 1) }),
    );
    expect(response.status).toBe(413);
    expect(upstreamFetch).not.toHaveBeenCalled();
  });
});
