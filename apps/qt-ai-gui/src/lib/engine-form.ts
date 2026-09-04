import { z } from "zod";

import { apiProviderSchema, engineSchema, reasoningEffortSchema } from "@/lib/schema";
import type { ApiSettings, AppConfig } from "@/lib/types";

/** Phần "Động cơ dịch" của form Cài đặt — phẳng để react-hook-form register từng ô. */
export const engineFormSchema = z.object({
  engine: engineSchema,
  apiProvider: apiProviderSchema,
  geminiApiKey: z.string(),
  geminiModel: z.string(),
  geminiBaseUrl: z.string(),
  openaiApiKey: z.string(),
  openaiModel: z.string(),
  openaiBaseUrl: z.string(),
  thinking: z.boolean(),
  reasoningEffort: reasoningEffortSchema,
});
export type EngineForm = z.infer<typeof engineFormSchema>;

export function engineFormFromConfig(config: Pick<AppConfig, "engine" | "api">): EngineForm {
  return {
    engine: config.engine,
    apiProvider: config.api.provider,
    geminiApiKey: config.api.gemini.apiKey,
    geminiModel: config.api.gemini.model,
    geminiBaseUrl: config.api.gemini.baseUrl,
    openaiApiKey: config.api.openai.apiKey,
    openaiModel: config.api.openai.model,
    openaiBaseUrl: config.api.openai.baseUrl,
    thinking: config.api.thinking,
    reasoningEffort: config.api.reasoningEffort,
  };
}

/** Trim mọi ô text; key/model/baseUrl của provider không chọn vẫn giữ nguyên (đổi provider không mất key). */
export function apiSettingsFromForm(values: EngineForm): ApiSettings {
  return {
    provider: values.apiProvider,
    gemini: { apiKey: values.geminiApiKey.trim(), model: values.geminiModel.trim(), baseUrl: values.geminiBaseUrl.trim() },
    openai: { apiKey: values.openaiApiKey.trim(), model: values.openaiModel.trim(), baseUrl: values.openaiBaseUrl.trim() },
    thinking: values.thinking,
    reasoningEffort: values.reasoningEffort,
  };
}
