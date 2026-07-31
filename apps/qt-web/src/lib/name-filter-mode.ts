import type { NameFilterMode } from "@/lib/types";

export const defaultNameFilterMode: NameFilterMode = "qt";
export const nameFilterModeStorageKey = "qt-web-name-filter-mode";
export const defaultNameApprovalThreshold = 85;
export const nameApprovalThresholdStorageKey = "qt-web-name-approval-threshold";

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

export function readStoredNameApprovalThreshold(): number {
  try {
    const raw = window.localStorage.getItem(nameApprovalThresholdStorageKey);
    if (raw === null) return defaultNameApprovalThreshold;
    const stored = Number(raw);
    return Number.isInteger(stored) && stored >= 0 && stored <= 100
      ? stored
      : defaultNameApprovalThreshold;
  } catch {
    return defaultNameApprovalThreshold;
  }
}

export function storeNameApprovalThreshold(threshold: number): void {
  try {
    window.localStorage.setItem(nameApprovalThresholdStorageKey, String(threshold));
  } catch {
    // localStorage có thể bị chặn; ngưỡng vẫn hoạt động trong phiên hiện tại.
  }
}
