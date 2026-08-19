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

    localStorage.setItem(aiSettingsStorageKey, JSON.stringify({ provider: "openai" }));
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
          deepseek: "deepseek-translate",
          gemini: "gemini-translate",
        },
        thinking: false,
        autoGlossary: true,
      },
    });

    const restored = readStoredAiSettings();
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
      },
      thinking: true,
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
