export const aiSettingsStorageKey = "qt-web-ai-settings";

export type AiProvider = "deepseek" | "gemini";

export interface AiProviderConfig {
  apiKey: string;
  model: string;
  /** Endpoint proxy tùy chọn; trống = endpoint chính thức của provider. */
  baseUrl: string;
}

/**
 * Cấu hình AI của chính người dùng cho tính năng lọc tên. Trình duyệt gọi
 * thẳng provider (hoặc proxy tự cấu hình) bằng key này — key không bao giờ
 * đi qua server của mình và provider tính phí trực tiếp vào tài khoản của
 * người dùng.
 *
 * Key/model/baseUrl được lưu tách riêng theo từng provider: đổi provider
 * trong dropdown chỉ đổi cấu hình đang dùng, không bao giờ mang key của công
 * ty này gửi sang endpoint của công ty kia.
 */
export interface AiSettings {
  provider: AiProvider;
  deepseek: AiProviderConfig;
  gemini: AiProviderConfig;
}

/** Model điền sẵn cho người dùng mới theo từng provider; sửa được trong Cài đặt. */
export const DEFAULT_DEEPSEEK_MODEL = "deepseek-v4-flash";
export const DEFAULT_GEMINI_MODEL = "gemini-3.1-flash-lite";

export const defaultAiSettings: AiSettings = {
  provider: "deepseek",
  deepseek: { apiKey: "", model: DEFAULT_DEEPSEEK_MODEL, baseUrl: "" },
  gemini: { apiKey: "", model: DEFAULT_GEMINI_MODEL, baseUrl: "" },
};

export function isAiProvider(value: unknown): value is AiProvider {
  return value === "deepseek" || value === "gemini";
}

/** Cấu hình của provider đang được chọn. */
export function activeAiProviderConfig(settings: AiSettings): AiProviderConfig {
  return settings[settings.provider];
}

function normalizeConfig(value: unknown, defaultModel = ""): AiProviderConfig {
  const record =
    typeof value === "object" && value !== null ? (value as Partial<AiProviderConfig>) : {};
  const model = typeof record.model === "string" ? record.model.trim() : "";
  return {
    apiKey: typeof record.apiKey === "string" ? record.apiKey.trim() : "",
    model: model || defaultModel,
    baseUrl: typeof record.baseUrl === "string" ? record.baseUrl.trim() : "",
  };
}

export function readStoredAiSettings(): AiSettings {
  try {
    const raw = window.localStorage.getItem(aiSettingsStorageKey);
    if (!raw) return defaultAiSettings;
    const parsed = JSON.parse(raw) as Partial<Record<keyof AiSettings, unknown>> | null;
    return {
      provider: isAiProvider(parsed?.provider) ? parsed.provider : "deepseek",
      deepseek: normalizeConfig(parsed?.deepseek, DEFAULT_DEEPSEEK_MODEL),
      gemini: normalizeConfig(parsed?.gemini, DEFAULT_GEMINI_MODEL),
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
