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

describe("Cloudflare Lambda gateway", () => {
  it("signs and proxies a translation request without leaking client credentials", async () => {
    const body = JSON.stringify({
      text: "很好",
      mode: "vietphrase-one",
      dictionaries: { names: "萧炎=Tiêu Viêm" },
    });
    let upstreamRequest: Request | undefined;
    const upstreamFetch = vi.fn((input: RequestInfo | URL) => {
      upstreamRequest = input instanceof Request ? input : new Request(input);
      return Response.json({ translated: "Rất tốt" });
    });
    vi.stubGlobal("fetch", upstreamFetch);

    const response = await invoke(
      new Request("https://api.example.com/translate?source=web", {
        method: "POST",
        headers: {
          accept: "application/json",
          authorization: "Bearer client-token-must-not-reach-aws",
          "content-type": "application/json; charset=utf-8",
          cookie: "session=must-not-reach-aws",
        },
        body,
      }),
    );

    expect(response.status).toBe(200);
    expect(await response.json()).toEqual({ translated: "Rất tốt" });
    expect(response.headers.get("cache-control")).toBe("no-store");
    expect(response.headers.get("x-request-id")).toBeTruthy();
    expect(upstreamFetch).toHaveBeenCalledTimes(1);
    expect(upstreamRequest).toBeDefined();

    const signed = upstreamRequest;
    if (signed === undefined) {
      throw new Error("expected a signed upstream request");
    }
    expect(signed.url).toBe(
      "https://test-function.lambda-url.ap-southeast-1.on.aws/translate?source=web",
    );
    expect(signed.method).toBe("POST");
    expect(signed.headers.get("authorization")).toMatch(
      /^AWS4-HMAC-SHA256 Credential=AKIDEXAMPLE\//,
    );
    expect(signed.headers.get("authorization")).toContain(
      "/ap-southeast-1/lambda/aws4_request",
    );
    expect(signed.headers.get("x-amz-date")).toMatch(/^\d{8}T\d{6}Z$/);
    expect(signed.headers.get("cookie")).toBeNull();
    expect(signed.headers.get("content-type")).toBe(
      "application/json; charset=utf-8",
    );
    expect(await signed.text()).toBe(body);
  });

  it("answers browser preflight and attaches CORS headers for an allowed origin", async () => {
    const upstreamFetch = vi.fn(() => new Response("unexpected"));
    vi.stubGlobal("fetch", upstreamFetch);

    const preflight = await invoke(
      new Request("https://api.example.com/translate", {
        method: "OPTIONS",
        headers: {
          origin: "http://localhost:5173",
          "access-control-request-method": "POST",
          "access-control-request-headers": "content-type",
        },
      }),
    );

    expect(preflight.status).toBe(204);
    expect(preflight.headers.get("access-control-allow-origin")).toBe(
      "http://localhost:5173",
    );
    expect(preflight.headers.get("access-control-allow-methods")).toBe("POST");
    expect(preflight.headers.get("vary")).toBe("Origin");
    expect(upstreamFetch).not.toHaveBeenCalled();

    const upstream = vi.fn(() => Response.json({ status: "ok" }));
    vi.stubGlobal("fetch", upstream);
    const actual = await invoke(
      new Request("https://api.example.com/health", {
        headers: { origin: "http://localhost:5173" },
      }),
    );
    expect(actual.headers.get("access-control-allow-origin")).toBe(
      "http://localhost:5173",
    );
    expect(actual.headers.get("access-control-expose-headers")).toBe("x-request-id");
  });

  it("rejects browser origins outside the configured allowlist", async () => {
    const upstreamFetch = vi.fn(() => new Response("unexpected"));
    vi.stubGlobal("fetch", upstreamFetch);

    const response = await invoke(
      new Request("https://api.example.com/health", {
        headers: { origin: "https://untrusted.example" },
      }),
    );

    expect(response.status).toBe(403);
    expect(await response.json()).toEqual({ error: "Origin is not allowed" });
    expect(upstreamFetch).not.toHaveBeenCalled();
  });

  it("allows only the engine routes and methods", async () => {
    const upstreamFetch = vi.fn(() => new Response("unexpected"));
    vi.stubGlobal("fetch", upstreamFetch);

    const unknown = await invoke(
      new Request("https://api.example.com/admin"),
    );
    expect(unknown.status).toBe(404);

    const wrongMethod = await invoke(
      new Request("https://api.example.com/translate", {
        method: "GET",
      }),
    );
    expect(wrongMethod.status).toBe(405);
    expect(wrongMethod.headers.get("allow")).toBe("POST");
    expect(upstreamFetch).not.toHaveBeenCalled();
  });

  it("rejects non-JSON and oversized bodies before invoking Lambda", async () => {
    const upstreamFetch = vi.fn(() => new Response("unexpected"));
    vi.stubGlobal("fetch", upstreamFetch);

    const unsupported = await invoke(
      new Request("https://api.example.com/translate", {
        method: "POST",
        headers: { "content-type": "text/plain" },
        body: "很好",
      }),
    );
    expect(unsupported.status).toBe(415);

    const oversized = await invoke(
      new Request("https://api.example.com/translate", {
        method: "POST",
        headers: {
          "content-length": String(5 * 1024 * 1024 + 1),
          "content-type": "application/json",
        },
        body: "{}",
      }),
    );
    expect(oversized.status).toBe(413);
    expect(upstreamFetch).not.toHaveBeenCalled();
  });

  it("does not retry a failed Lambda invocation", async () => {
    const upstreamFetch = vi.fn(() =>
      Response.json({ error: "temporarily unavailable" }, { status: 503 }),
    );
    vi.stubGlobal("fetch", upstreamFetch);

    const response = await invoke(
      new Request("https://api.example.com/health"),
    );

    expect(response.status).toBe(503);
    expect(upstreamFetch).toHaveBeenCalledTimes(1);
  });
});
