import { endpointSchema } from "@/lib/schema";

export const endpointStorageKey = "qt-web-api-endpoint";

export function readStoredEndpoint(fallback: string): string {
  try {
    const stored = window.localStorage.getItem(endpointStorageKey);
    const parsed = endpointSchema.safeParse(stored);
    return parsed.success ? parsed.data : fallback;
  } catch {
    return fallback;
  }
}

export function storeEndpoint(endpoint: string): void {
  const parsed = endpointSchema.safeParse(endpoint);
  if (!parsed.success) return;

  try {
    window.localStorage.setItem(endpointStorageKey, parsed.data);
  } catch {
    // localStorage có thể bị chặn; endpoint vẫn dùng được trong phiên hiện tại.
  }
}
