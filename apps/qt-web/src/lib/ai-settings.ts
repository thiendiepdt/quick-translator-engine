export const aiSettingsStorageKey = "qt-web-ai-settings";

export type AiProvider = "deepseek" | "gemini";

export interface AiProviderConfig {
  apiKey: string;
  model: string;
}

/**
 * Cấu hình AI của chính người dùng cho tính năng lọc tên. Server không giữ
 * API key nào — key ở đây được gửi kèm từng request lọc tên có bật AI và
 * provider tính phí trực tiếp vào tài khoản của người dùng.
 *
 * Key/model được lưu tách riêng theo từng provider: đổi provider trong
 * dropdown chỉ đổi cấu hình đang dùng, không bao giờ mang key của công ty
 * này gửi sang endpoint của công ty kia.
 */
export interface AiSettings {
  provider: AiProvider;
  deepseek: AiProviderConfig;
  gemini: AiProviderConfig;
}

export const defaultAiSettings: AiSettings = {
  provider: "deepseek",
  deepseek: { apiKey: "", model: "" },
  gemini: { apiKey: "", model: "" },
};

export function isAiProvider(value: unknown): value is AiProvider {
  return value === "deepseek" || value === "gemini";
}

/** Cấu hình của provider đang được chọn. */
export function activeAiProviderConfig(settings: AiSettings): AiProviderConfig {
  return settings[settings.provider];
}

function normalizeConfig(value: unknown): AiProviderConfig {
  const record =
    typeof value === "object" && value !== null ? (value as Partial<AiProviderConfig>) : {};
  return {
    apiKey: typeof record.apiKey === "string" ? record.apiKey.trim() : "",
    model: typeof record.model === "string" ? record.model.trim() : "",
  };
}

export function readStoredAiSettings(): AiSettings {
  try {
    const raw = window.localStorage.getItem(aiSettingsStorageKey);
    if (!raw) return defaultAiSettings;
    const parsed = JSON.parse(raw) as Partial<Record<keyof AiSettings, unknown>> | null;
    return {
      provider: isAiProvider(parsed?.provider) ? parsed.provider : "deepseek",
      deepseek: normalizeConfig(parsed?.deepseek),
      gemini: normalizeConfig(parsed?.gemini),
    };
  } catch {
    return defaultAiSettings;
  }
}

export function storeAiSettings(settings: AiSettings): void {
  try {
    window.localStorage.setItem(
      aiSettingsStorageKey,
      JSON.stringify({
        provider: settings.provider,
        deepseek: normalizeConfig(settings.deepseek),
        gemini: normalizeConfig(settings.gemini),
      }),
    );
  } catch {
    // localStorage có thể bị chặn; cấu hình vẫn dùng được trong phiên hiện tại.
  }
}
