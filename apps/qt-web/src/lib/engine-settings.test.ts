import { beforeEach, describe, expect, it } from "vitest";

import {
  defaultEngineSettings,
  engineSettingsStorageKey,
  readStoredEngineSettings,
  storeEngineSettings,
} from "@/lib/engine-settings";

beforeEach(() => {
  localStorage.clear();
});

describe("engine settings preference", () => {
  it("uses defaults when storage is missing or invalid", () => {
    expect(readStoredEngineSettings()).toEqual(defaultEngineSettings);

    localStorage.setItem(engineSettingsStorageKey, "not-json");
    expect(readStoredEngineSettings()).toEqual(defaultEngineSettings);
  });

  it("stores and restores valid engine settings", () => {
    const settings = {
      pretty: false,
      wrap: true,
      prioritizedName: false,
      scanRange: 45,
      translationAlgorithm: 2,
    };

    storeEngineSettings(settings);

    expect(readStoredEngineSettings()).toEqual(settings);
  });

  it("does not overwrite valid settings with invalid input", () => {
    storeEngineSettings(defaultEngineSettings);
    const stored = localStorage.getItem(engineSettingsStorageKey);

    storeEngineSettings({ ...defaultEngineSettings, scanRange: 0 });

    expect(localStorage.getItem(engineSettingsStorageKey)).toBe(stored);
  });
});
