import { beforeEach, describe, expect, it } from "vitest";

import {
  activeAiProviderConfig,
  activeAiTranslationProviderConfig,
  aiSettingsStorageKey,
  DEFAULT_AI_TRANSLATION_DEEPSEEK_MODEL,
  DEFAULT_AI_TRANSLATION_GEMINI_MODEL,
  DEFAULT_DEEPSEEK_MODEL,
  DEFAULT_GEMINI_MODEL,
  defaultAiSettings,
  readStoredAiSettings,
  storeAiSettings,
} from "@/lib/ai-settings";

beforeEach(() => {
  localStorage.clear();
});

describe("AI credentials preference", () => {
  it("falls back to defaults when nothing valid is stored", () => {
    expect(readStoredAiSettings()).toEqual(defaultAiSettings);

    localStorage.setItem(aiSettingsStorageKey, "not json");
    expect(readStoredAiSettings()).toEqual(defaultAiSettings);

    localStorage.setItem(aiSettingsStorageKey, JSON.stringify({ provider: "claude" }));
    expect(readStoredAiSettings().provider).toBe("gemini");
  });

  it("defaults both name-filter and translation providers to Gemini", () => {
    expect(defaultAiSettings.provider).toBe("gemini");
    expect(defaultAiSettings.translation.provider).toBe("gemini");
    expect(defaultAiSettings.translation.models.gemini).toBe("gemini-3.7-flash");
  });

  it("keeps each provider's key, model and base URL isolated across switches", () => {
    storeAiSettings({
      ...defaultAiSettings,
      provider: "deepseek",
      deepseek: { apiKey: " sk-deepseek ", model: "", baseUrl: " https://proxy.example.com/v1 " },
      gemini: { apiKey: "AIza-google", model: "gemini-2.5-flash", baseUrl: "" },
    });

    const restored = readStoredAiSettings();
    expect(restored.deepseek).toEqual({
      apiKey: "sk-deepseek",
      model: DEFAULT_DEEPSEEK_MODEL,
      baseUrl: "https://proxy.example.com/v1",
    });
    expect(restored.gemini).toEqual({
      apiKey: "AIza-google",
      model: "gemini-2.5-flash",
      baseUrl: "",
    });

    // Đổi provider chỉ đổi con trỏ cấu hình; key DeepSeek không được trả về
    // khi provider đang là Gemini.
    expect(activeAiProviderConfig(restored).apiKey).toBe("sk-deepseek");
    expect(activeAiProviderConfig({ ...restored, provider: "gemini" }).apiKey).toBe(
      "AIza-google",
    );
  });

  it("fills each provider's model with its default when empty", () => {
    expect(defaultAiSettings.deepseek.model).toBe(DEFAULT_DEEPSEEK_MODEL);
    expect(defaultAiSettings.gemini.model).toBe(DEFAULT_GEMINI_MODEL);

    storeAiSettings({
      ...defaultAiSettings,
      provider: "deepseek",
      deepseek: { apiKey: "sk-deepseek", model: "  ", baseUrl: "" },
      gemini: { apiKey: "AIza-google", model: "", baseUrl: "" },
    });
    const restored = readStoredAiSettings();
    expect(restored.deepseek.model).toBe(DEFAULT_DEEPSEEK_MODEL);
    expect(restored.gemini.model).toBe(DEFAULT_GEMINI_MODEL);

    // Model do người dùng tự chọn không bị ghi đè.
    storeAiSettings({
      ...defaultAiSettings,
      provider: "deepseek",
      deepseek: { apiKey: "sk-deepseek", model: "deepseek-chat", baseUrl: "" },
      gemini: { apiKey: "", model: "", baseUrl: "" },
    });
    expect(readStoredAiSettings().deepseek.model).toBe("deepseek-chat");
  });

  it("shares credentials while keeping translation provider, models and thinking separate", () => {
    storeAiSettings({
      ...defaultAiSettings,
      provider: "deepseek",
      deepseek: {
        apiKey: "sk-shared",
        model: "deepseek-name-filter",
        baseUrl: "https://deepseek.example.com",
      },
      gemini: {
        apiKey: "AIza-shared",
        model: "gemini-name-filter",
        baseUrl: "https://gemini.example.com",
      },
      translation: {
        provider: "gemini",
        models: {
          grok: "grok-4.6",
          glm: "glm-5.3-flash",
          openai: "gpt-5.6-sol",
          deepseek: "deepseek-translate",
          gemini: "gemini-translate",
        },
        thinking: false,
        openaiReasoningEffort: "low",
        grokFallback: false,
        autoGlossary: true,
      },
    });

    const restored = readStoredAiSettings();
    expect(restored.translation.openaiReasoningEffort).toBe("low");
    expect(activeAiProviderConfig(restored)).toMatchObject({
      apiKey: "sk-shared",
      model: "deepseek-name-filter",
    });
    expect(activeAiTranslationProviderConfig(restored)).toEqual({
      apiKey: "AIza-shared",
      model: "gemini-translate",
      baseUrl: "https://gemini.example.com",
    });
    expect(restored.translation.thinking).toBe(false);
  });

  it("migrates old settings with translation defaults without copying a model", () => {
    localStorage.setItem(
      aiSettingsStorageKey,
      JSON.stringify({
        provider: "deepseek",
        deepseek: { apiKey: "sk-old", model: "deepseek-name", baseUrl: "" },
        gemini: { apiKey: "AIza-old", model: "gemini-name", baseUrl: "" },
      }),
    );

    const restored = readStoredAiSettings();
    expect(restored.translation).toEqual({
      provider: "gemini",
      models: {
        deepseek: DEFAULT_AI_TRANSLATION_DEEPSEEK_MODEL,
        gemini: DEFAULT_AI_TRANSLATION_GEMINI_MODEL,
        grok: "grok-4.6",
        glm: "glm-5.3-flash",
        openai: "gpt-5.6-sol",
      },
      thinking: true,
      openaiReasoningEffort: "high",
      grokFallback: true,
      autoGlossary: true,
    });
    // Provider mặc định giờ là Gemini nên key hoạt động là key Gemini cũ.
    expect(activeAiTranslationProviderConfig(restored).apiKey).toBe("AIza-old");
  });

  it("never assigns a flat legacy apiKey to any provider", () => {
    localStorage.setItem(
      aiSettingsStorageKey,
      JSON.stringify({ provider: "gemini", apiKey: "sk-deepseek", model: "deepseek-chat" }),
    );

    const restored = readStoredAiSettings();
    expect(restored.provider).toBe("gemini");
    expect(restored.deepseek.apiKey).toBe("");
    expect(restored.gemini.apiKey).toBe("");
  });
});

describe("auto glossary setting", () => {
  it("defaults to enabled, including for legacy stored settings", () => {
    expect(defaultAiSettings.translation.autoGlossary).toBe(true);
    window.localStorage.setItem(
      aiSettingsStorageKey,
      JSON.stringify({ translation: { provider: "gemini" } }),
    );
    expect(readStoredAiSettings().translation.autoGlossary).toBe(true);
  });

  it("round-trips an explicit off state", () => {
    storeAiSettings({
      ...defaultAiSettings,
      translation: { ...defaultAiSettings.translation, autoGlossary: false },
    });
    expect(readStoredAiSettings().translation.autoGlossary).toBe(false);
  });
});

describe("grok support", () => {
  it("ships an empty grok config with defaults and fallback off", () => {
    expect(defaultAiSettings.grok).toEqual({
      apiKey: "",
      model: "grok-4.6",
      baseUrl: "",
    });
    expect(defaultAiSettings.translation.models.grok).toBe("grok-4.6");
    expect(defaultAiSettings.translation.grokFallback).toBe(true);
    expect(readStoredAiSettings().grok.model).toBe("grok-4.6");
  });

  it("round-trips the grok key, translation model and fallback toggle", () => {
    storeAiSettings({
      ...defaultAiSettings,
      grok: { apiKey: " xai-key ", model: "grok-4.6", baseUrl: "" },
      translation: {
        ...defaultAiSettings.translation,
        provider: "grok",
        models: { ...defaultAiSettings.translation.models, grok: "grok-4.5" },
        grokFallback: true,
      },
    });
    const restored = readStoredAiSettings();
    expect(restored.grok.apiKey).toBe("xai-key");
    expect(restored.translation.provider).toBe("grok");
    expect(restored.translation.models.grok).toBe("grok-4.5");
    expect(restored.translation.grokFallback).toBe(true);
    expect(activeAiTranslationProviderConfig(restored)).toEqual({
      apiKey: "xai-key",
      model: "grok-4.5",
      baseUrl: "",
    });
  });

  it("defaults grokFallback to on for legacy stored settings", () => {
    localStorage.setItem(
      aiSettingsStorageKey,
      JSON.stringify({ translation: { provider: "gemini" } }),
    );
    const restored = readStoredAiSettings();
    expect(restored.translation.grokFallback).toBe(true);
    expect(restored.grok).toEqual({ apiKey: "", model: "grok-4.6", baseUrl: "" });
  });
});

describe("grok fallback opt-out", () => {
  it("keeps an explicit off through storage", () => {
    storeAiSettings({
      ...defaultAiSettings,
      translation: { ...defaultAiSettings.translation, grokFallback: false },
    });
    expect(readStoredAiSettings().translation.grokFallback).toBe(false);
  });
});

describe("openai provider settings", () => {
  it("ships an OpenAI config with the official model defaults", () => {
    expect(defaultAiSettings.openai).toEqual({ apiKey: "", model: "gpt-5.6-sol", baseUrl: "" });
    expect(defaultAiSettings.translation.models.openai).toBe("gpt-5.6-sol");
    expect(defaultAiSettings.translation.openaiReasoningEffort).toBe("high");
  });

  it("round-trips the OpenAI reasoning effort and drops unknown levels", () => {
    storeAiSettings({
      ...defaultAiSettings,
      translation: { ...defaultAiSettings.translation, openaiReasoningEffort: "xhigh" },
    });
    expect(readStoredAiSettings().translation.openaiReasoningEffort).toBe("xhigh");

    localStorage.setItem(
      aiSettingsStorageKey,
      JSON.stringify({ translation: { openaiReasoningEffort: "ultra" } }),
    );
    expect(readStoredAiSettings().translation.openaiReasoningEffort).toBe("high");
  });

  it("round-trips a custom OpenAI-compatible hub for translation", () => {
    storeAiSettings({
      ...defaultAiSettings,
      openai: { apiKey: "sk-hub-x", model: "", baseUrl: "http://192.0.2.10/v1" },
      translation: {
        ...defaultAiSettings.translation,
        provider: "openai",
        models: { ...defaultAiSettings.translation.models, openai: "gemini-3.7-flash" },
      },
    });
    const restored = readStoredAiSettings();
    expect(restored.openai.model).toBe("gpt-5.6-sol");
    expect(activeAiTranslationProviderConfig(restored)).toEqual({
      apiKey: "sk-hub-x",
      model: "gemini-3.7-flash",
      baseUrl: "http://192.0.2.10/v1",
    });
  });
});
