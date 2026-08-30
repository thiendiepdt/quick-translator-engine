export const aiSettingsStorageKey = "qt-web-ai-settings";

export type AiProvider = "deepseek" | "gemini" | "grok" | "glm";

export interface AiProviderConfig {
  apiKey: string;
  model: string;
  /** Endpoint proxy tùy chọn; trống = endpoint chính thức của provider. */
  baseUrl: string;
}

export interface AiTranslationSettings {
  provider: AiProvider;
  models: Record<AiProvider, string>;
  thinking: boolean;
  /** Gemini chặn nội dung (PROHIBITED_CONTENT/SAFETY) thì tự thử lại bằng Grok. */
  grokFallback: boolean;
  /** Sau mỗi chương, tự trích tên riêng mới từ bản dịch nạp vào glossary truyện. */
  autoGlossary: boolean;
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
  /** Provider/model dùng cho lọc tên và trợ lý từ điển. */
  provider: AiProvider;
  deepseek: AiProviderConfig;
  gemini: AiProviderConfig;
  grok: AiProviderConfig;
  glm: AiProviderConfig;
  /** Dịch AI dùng chung key/base URL, nhưng chọn provider/model/thinking riêng. */
  translation: AiTranslationSettings;
}

/** Model điền sẵn cho người dùng mới theo từng provider; sửa được trong Cài đặt. */
export const DEFAULT_DEEPSEEK_MODEL = "deepseek-v4-flash";
export const DEFAULT_GEMINI_MODEL = "gemini-3.1-flash-lite";
export const DEFAULT_GROK_MODEL = "grok-4.6";
export const DEFAULT_GLM_MODEL = "glm-5.3-flash";
export const DEFAULT_AI_TRANSLATION_DEEPSEEK_MODEL = "deepseek-v4-flash";
export const DEFAULT_AI_TRANSLATION_GEMINI_MODEL = "gemini-3.7-flash";
export const DEFAULT_AI_TRANSLATION_GROK_MODEL = "grok-4.6";
export const DEFAULT_AI_TRANSLATION_GLM_MODEL = "glm-5.3-flash";

export const defaultAiSettings: AiSettings = {
  provider: "gemini",
  deepseek: { apiKey: "", model: DEFAULT_DEEPSEEK_MODEL, baseUrl: "" },
  gemini: { apiKey: "", model: DEFAULT_GEMINI_MODEL, baseUrl: "" },
  grok: { apiKey: "", model: DEFAULT_GROK_MODEL, baseUrl: "" },
  glm: { apiKey: "", model: DEFAULT_GLM_MODEL, baseUrl: "" },
  translation: {
    provider: "gemini",
    models: {
      deepseek: DEFAULT_AI_TRANSLATION_DEEPSEEK_MODEL,
      gemini: DEFAULT_AI_TRANSLATION_GEMINI_MODEL,
      grok: DEFAULT_AI_TRANSLATION_GROK_MODEL,
      glm: DEFAULT_AI_TRANSLATION_GLM_MODEL,
    },
    thinking: true,
    grokFallback: true,
    autoGlossary: true,
  },
};

export function isAiProvider(value: unknown): value is AiProvider {
  return (
    value === "deepseek" || value === "gemini" || value === "grok" || value === "glm"
  );
}

/** Cấu hình của provider đang được chọn. */
export function activeAiProviderConfig(settings: AiSettings): AiProviderConfig {
  return settings[settings.provider];
}

export function activeAiTranslationProviderConfig(settings: AiSettings): AiProviderConfig {
  const provider = settings.translation.provider;
  return {
    ...settings[provider],
    model: settings.translation.models[provider],
  };
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

function normalizeTranslationSettings(value: unknown): AiTranslationSettings {
  const record =
    typeof value === "object" && value !== null
      ? (value as Partial<AiTranslationSettings>)
      : {};
  const models =
    typeof record.models === "object" && record.models !== null
      ? (record.models as Partial<Record<AiProvider, unknown>>)
      : {};
  const deepseekModel =
    typeof models.deepseek === "string" ? models.deepseek.trim() : "";
  const geminiModel =
    typeof models.gemini === "string" ? models.gemini.trim() : "";
  const grokModel = typeof models.grok === "string" ? models.grok.trim() : "";
  const glmModel = typeof models.glm === "string" ? models.glm.trim() : "";
  return {
    provider: isAiProvider(record.provider) ? record.provider : "gemini",
    models: {
      deepseek: deepseekModel || DEFAULT_AI_TRANSLATION_DEEPSEEK_MODEL,
      gemini: geminiModel || DEFAULT_AI_TRANSLATION_GEMINI_MODEL,
      grok: grokModel || DEFAULT_AI_TRANSLATION_GROK_MODEL,
      glm: glmModel || DEFAULT_AI_TRANSLATION_GLM_MODEL,
    },
    thinking: typeof record.thinking === "boolean" ? record.thinking : true,
    grokFallback:
      typeof record.grokFallback === "boolean" ? record.grokFallback : true,
    autoGlossary: typeof record.autoGlossary === "boolean" ? record.autoGlossary : true,
  };
}

export function readStoredAiSettings(): AiSettings {
  try {
    const raw = window.localStorage.getItem(aiSettingsStorageKey);
    if (!raw) return defaultAiSettings;
    const parsed = JSON.parse(raw) as Partial<Record<keyof AiSettings, unknown>> | null;
    return {
      provider: isAiProvider(parsed?.provider) ? parsed.provider : "gemini",
      deepseek: normalizeConfig(parsed?.deepseek, DEFAULT_DEEPSEEK_MODEL),
      gemini: normalizeConfig(parsed?.gemini, DEFAULT_GEMINI_MODEL),
      grok: normalizeConfig(parsed?.grok, DEFAULT_GROK_MODEL),
      glm: normalizeConfig(parsed?.glm, DEFAULT_GLM_MODEL),
      translation: normalizeTranslationSettings(parsed?.translation),
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
        grok: normalizeConfig(settings.grok, DEFAULT_GROK_MODEL),
        glm: normalizeConfig(settings.glm, DEFAULT_GLM_MODEL),
        translation: normalizeTranslationSettings(settings.translation),
      }),
    );
  } catch {
    // localStorage có thể bị chặn; cấu hình vẫn dùng được trong phiên hiện tại.
  }
}

export const aiProviderLabels: Record<AiProvider, string> = {
  deepseek: "DeepSeek",
  gemini: "Gemini",
  grok: "Grok",
  glm: "GLM",
};
