import { beforeEach, describe, expect, it } from "vitest";

import {
  endpointStorageKey,
  readStoredEndpoint,
  storeEndpoint,
} from "@/lib/endpoint-setting";

beforeEach(() => {
  localStorage.clear();
});

describe("API endpoint preference", () => {
  it("uses the configured fallback when no valid setting exists", () => {
    expect(readStoredEndpoint("/api")).toBe("/api");

    localStorage.setItem(endpointStorageKey, "not a valid endpoint");
    expect(readStoredEndpoint("/api")).toBe("/api");
  });

  it("stores a normalized endpoint and restores it", () => {
    storeEndpoint("  https://api.example.com/translate  ");

    expect(localStorage.getItem(endpointStorageKey)).toBe(
      "https://api.example.com/translate",
    );
    expect(readStoredEndpoint("/api")).toBe(
      "https://api.example.com/translate",
    );
  });

  it("does not replace the saved endpoint with an invalid value", () => {
    storeEndpoint("/api");
    storeEndpoint("");

    expect(localStorage.getItem(endpointStorageKey)).toBe("/api");
  });
});
