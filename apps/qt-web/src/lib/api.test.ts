import { afterEach, describe, expect, it, vi } from "vitest";

import { translateChapter } from "@/lib/api";
import type { TranslationRequest } from "@/lib/types";

const request: TranslationRequest = {
  text: "很好",
  mode: "vietphrase-one",
  wrap: false,
  pretty: true,
  ranges: true,
  scanRange: 30,
  translationAlgorithm: 1,
  prioritizedName: true,
  dictionaries: { names: "很好=Rất tốt" },
};

afterEach(() => vi.unstubAllGlobals());

describe("translation API client", () => {
  it("posts the exact VietPhrase One request and validates ranges", async () => {
    const fetchMock = vi.fn((_url: string, init?: RequestInit) => {
      if (typeof init?.body !== "string") throw new Error("expected a JSON string body");
      expect(JSON.parse(init.body)).toEqual(request);
      return Promise.resolve(
        Response.json({
          translated: "Rất tốt",
          sourceRanges: [{ start: 0, length: 2 }],
          targetRanges: [{ start: 0, length: 7 }],
        }),
      );
    });
    vi.stubGlobal("fetch", fetchMock);

    await expect(translateChapter("https://translate.example.com", request)).resolves.toMatchObject({
      translated: "Rất tốt",
    });
    expect(fetchMock).toHaveBeenCalledWith(
      "https://translate.example.com/translate",
      expect.objectContaining({ method: "POST" }),
    );
  });

  it("surfaces JSON API errors with the request id", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(() =>
        Promise.resolve(Response.json(
          { error: "invalid dictionary" },
          { status: 400, headers: { "x-request-id": "request-123" } },
        )),
      ),
    );

    await expect(translateChapter("/api", request)).rejects.toEqual(
      expect.objectContaining({
        name: "ApiError",
        message: "invalid dictionary",
        status: 400,
        requestId: "request-123",
      }),
    );
  });
});
