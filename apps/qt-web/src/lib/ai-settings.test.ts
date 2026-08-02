import { beforeEach, describe, expect, it } from "vitest";

import {
  activeAiProviderConfig,
  aiSettingsStorageKey,
  DEFAULT_DEEPSEEK_MODEL,
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

  it("keeps each provider's key and model isolated across switches", () => {
    storeAiSettings({
      provider: "deepseek",
      deepseek: { apiKey: " sk-deepseek ", model: "" },
      gemini: { apiKey: "AIza-google", model: "gemini-2.5-flash" },
    });

    const restored = readStoredAiSettings();
    expect(restored.deepseek).toEqual({ apiKey: "sk-deepseek", model: DEFAULT_DEEPSEEK_MODEL });
    expect(restored.gemini).toEqual({ apiKey: "AIza-google", model: "gemini-2.5-flash" });

    // Đổi provider chỉ đổi con trỏ cấu hình; key DeepSeek không được trả về
    // khi provider đang là Gemini.
    expect(activeAiProviderConfig(restored).apiKey).toBe("sk-deepseek");
    expect(activeAiProviderConfig({ ...restored, provider: "gemini" }).apiKey).toBe(
      "AIza-google",
    );
  });

  it("fills the DeepSeek model with the default when empty, keeps Gemini explicit", () => {
    expect(defaultAiSettings.deepseek.model).toBe(DEFAULT_DEEPSEEK_MODEL);

    storeAiSettings({
      provider: "deepseek",
      deepseek: { apiKey: "sk-deepseek", model: "  " },
      gemini: { apiKey: "AIza-google", model: "" },
    });
    const restored = readStoredAiSettings();
    expect(restored.deepseek.model).toBe(DEFAULT_DEEPSEEK_MODEL);
    expect(restored.gemini.model).toBe("");

    // Model do người dùng tự chọn không bị ghi đè.
    storeAiSettings({
      provider: "deepseek",
      deepseek: { apiKey: "sk-deepseek", model: "deepseek-chat" },
      gemini: { apiKey: "", model: "" },
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
