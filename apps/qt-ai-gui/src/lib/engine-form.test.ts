import { describe, expect, it } from "vitest";

import { apiSettingsFromForm, engineFormFromConfig } from "@/lib/engine-form";
import { appConfigSchema } from "@/lib/schema";
import { engineLabel, stopReasonLabel } from "@/lib/types";

const config = appConfigSchema.parse({
  engine: "api",
  api: {
    provider: "openai",
    gemini: { apiKey: "AIza", model: "gemini-3.7-flash", baseUrl: "" },
    openai: { apiKey: " sk-hub ", model: " gemini-3.8-flash ", baseUrl: " http://192.0.2.10/v1 " },
    thinking: false,
    reasoningEffort: "max",
  },
  agyPath: null,
  model: null,
  maxSessions: 50,
  recent: [],
});

describe("engine form", () => {
  it("config → form → api settings round-trip, trim ô text, giữ key provider không chọn", () => {
    const form = engineFormFromConfig(config);
    expect(form.engine).toBe("api");
    expect(form.apiProvider).toBe("openai");
    expect(form.openaiApiKey).toBe(" sk-hub ");
    const api = apiSettingsFromForm({ ...form, apiProvider: "gemini" });
    expect(api.provider).toBe("gemini");
    expect(api.gemini.apiKey).toBe("AIza");
    expect(api.openai).toEqual({ apiKey: "sk-hub", model: "gemini-3.8-flash", baseUrl: "http://192.0.2.10/v1" });
    expect(api.thinking).toBe(false);
    expect(api.reasoningEffort).toBe("max");
  });

  it("engineLabel và stopReasonLabel cho động cơ API", () => {
    expect(engineLabel(config)).toBe("API · OpenAI-compatible · gemini-3.8-flash");
    expect(engineLabel({ ...config, api: { ...config.api, provider: "gemini" } })).toBe("API · Gemini · gemini-3.7-flash");
    expect(engineLabel({ ...config, api: { ...config.api, openai: { ...config.api.openai, model: "" } } })).toBe(
      "API · OpenAI-compatible · gpt-5.6-sol",
    );
    expect(engineLabel({ ...config, engine: "agy" })).toBe("agy");
    expect(engineLabel(undefined)).toBe("agy");
    expect(stopReasonLabel({ kind: "api_failed", message: "OpenAI trả 401: bad key" })).toContain("OpenAI trả 401");
  });
});
