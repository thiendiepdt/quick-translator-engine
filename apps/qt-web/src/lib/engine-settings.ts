import type { z } from "zod";

import { translationOptionsSchema } from "@/lib/schema";

const engineSettingsSchema = translationOptionsSchema.omit({ endpoint: true });

export type EngineSettings = z.output<typeof engineSettingsSchema>;

export const defaultEngineSettings: EngineSettings = {
  pretty: true,
  wrap: false,
  prioritizedName: true,
  scanRange: 30,
  translationAlgorithm: 1,
};

export const engineSettingsStorageKey = "qt-web-engine-settings-v1";

export function readStoredEngineSettings(): EngineSettings {
  try {
    const stored = window.localStorage.getItem(engineSettingsStorageKey);
    if (!stored) return defaultEngineSettings;
    const parsed = engineSettingsSchema.safeParse(JSON.parse(stored));
    return parsed.success ? parsed.data : defaultEngineSettings;
  } catch {
    return defaultEngineSettings;
  }
}

export function storeEngineSettings(settings: unknown): void {
  const parsed = engineSettingsSchema.safeParse(settings);
  if (!parsed.success) return;

  try {
    window.localStorage.setItem(
      engineSettingsStorageKey,
      JSON.stringify(parsed.data),
    );
  } catch {
    // localStorage có thể bị chặn; setting vẫn dùng được trong phiên hiện tại.
  }
}
