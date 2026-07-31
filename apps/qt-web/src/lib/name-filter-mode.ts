import type { NameFilterMode } from "@/lib/types";

export const defaultNameFilterMode: NameFilterMode = "qt";
export const nameFilterModeStorageKey = "qt-web-name-filter-mode";

export function isNameFilterMode(value: unknown): value is NameFilterMode {
  return value === "qt" || value === "hybrid";
}

export function readStoredNameFilterMode(): NameFilterMode {
  try {
    const stored = window.localStorage.getItem(nameFilterModeStorageKey);
    return isNameFilterMode(stored) ? stored : defaultNameFilterMode;
  } catch {
    return defaultNameFilterMode;
  }
}

export function storeNameFilterMode(mode: NameFilterMode): void {
  try {
    window.localStorage.setItem(nameFilterModeStorageKey, mode);
  } catch {
    // localStorage có thể bị chặn; mode vẫn hoạt động trong phiên hiện tại.
  }
}
