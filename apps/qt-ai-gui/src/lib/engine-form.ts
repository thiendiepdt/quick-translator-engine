import { z } from "zod";

import { apiProviderSchema, engineSchema, geminiStepEffortsSchema, openaiStepEffortsSchema } from "@/lib/schema";
import type { ApiSettings, AppConfig } from "@/lib/types";

/** Phần "Động cơ dịch" của form Cài đặt — phẳng để react-hook-form register từng ô. */
export const engineFormSchema = z.object({
  engine: engineSchema,
  apiProvider: apiProviderSchema,
  geminiApiKey: z.string(),
  geminiModel: z.string(),
  geminiBaseUrl: z.string(),
  /** Mức nghĩ theo bước của Gemini (object 4 khoá, sửa qua setValue("geminiEffort.<bước>")). */
  geminiEffort: geminiStepEffortsSchema,
  openaiApiKey: z.string(),
  openaiModel: z.string(),
  openaiBaseUrl: z.string(),
  openaiEffort: openaiStepEffortsSchema,
});
export type EngineForm = z.infer<typeof engineFormSchema>;

export function engineFormFromConfig(config: Pick<AppConfig, "engine" | "api">): EngineForm {
  return {
    engine: config.engine,
    apiProvider: config.api.provider,
    geminiApiKey: config.api.gemini.apiKey,
    geminiModel: config.api.gemini.model,
    geminiBaseUrl: config.api.gemini.baseUrl,
    geminiEffort: { ...config.api.gemini.effort },
    openaiApiKey: config.api.openai.apiKey,
    openaiModel: config.api.openai.model,
    openaiBaseUrl: config.api.openai.baseUrl,
    openaiEffort: { ...config.api.openai.effort },
  };
}

/** Trim mọi ô text; key/model/baseUrl/effort của provider không chọn vẫn giữ nguyên (đổi provider không mất key). */
export function apiSettingsFromForm(values: EngineForm): ApiSettings {
  return {
    provider: values.apiProvider,
    gemini: {
      apiKey: values.geminiApiKey.trim(),
      model: values.geminiModel.trim(),
      baseUrl: values.geminiBaseUrl.trim(),
      effort: { ...values.geminiEffort },
    },
    openai: {
      apiKey: values.openaiApiKey.trim(),
      model: values.openaiModel.trim(),
      baseUrl: values.openaiBaseUrl.trim(),
      effort: { ...values.openaiEffort },
    },
  };
}
