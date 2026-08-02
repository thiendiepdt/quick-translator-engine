import { beforeEach, describe, expect, it } from "vitest";

import {
  activeAiProviderConfig,
  aiSettingsStorageKey,
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
    expect(readStoredAiSettings().provider).toBe("deepseek");
  });

  it("keeps each provider's key, model and base URL isolated across switches", () => {
    storeAiSettings({
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
      provider: "deepseek",
      deepseek: { apiKey: "sk-deepseek", model: "  ", baseUrl: "" },
      gemini: { apiKey: "AIza-google", model: "", baseUrl: "" },
    });
    const restored = readStoredAiSettings();
    expect(restored.deepseek.model).toBe(DEFAULT_DEEPSEEK_MODEL);
    expect(restored.gemini.model).toBe(DEFAULT_GEMINI_MODEL);

    // Model do người dùng tự chọn không bị ghi đè.
    storeAiSettings({
      provider: "deepseek",
      deepseek: { apiKey: "sk-deepseek", model: "deepseek-chat", baseUrl: "" },
      gemini: { apiKey: "", model: "", baseUrl: "" },
    });
    expect(readStoredAiSettings().deepseek.model).toBe("deepseek-chat");
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
