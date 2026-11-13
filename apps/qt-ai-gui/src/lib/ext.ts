import type { ExtConfig, ExtSetting, ExtValues } from "@/lib/api";

/** Giá trị trong ô nhập: boolean cho công tắc, chuỗi cho mọi ô còn lại. Ô trống = dùng mặc định. */
export type FieldDisplay = string | boolean;

/** Trường auth (text/secret) dưới dạng trường cài đặt để dùng chung ô nhập. */
export function authFieldAsSetting(field: { key: string; label: string; kind: "text" | "secret" }, value: unknown, filled: boolean): ExtSetting {
  return { key: field.key, label: field.label, kind: field.kind, choices: [], default: null, scope: "global", value, filled };
}

/** Giá trị đã lưu (chuỗi, số, boolean) → chữ trong ô nhập. */
function asText(value: unknown): string {
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean") return String(value);
  return value == null ? "" : JSON.stringify(value);
}

export function initialDisplay(field: ExtSetting): FieldDisplay {
  if (field.kind === "secret") return "";
  if (field.kind === "boolean") return typeof field.value === "boolean" ? field.value : field.default === true;
  return asText(field.value);
}

/** Ô → giá trị gửi Rust: ô trống = `null` (mặc định), trừ secret (chuỗi rỗng = giữ giá trị cũ). */
export function toValue(field: ExtSetting, display: FieldDisplay): unknown {
  if (field.kind === "boolean") return display === true;
  const text = String(display);
  if (field.kind === "secret") return text;
  return text.trim() === "" ? null : text;
}

/** Cài đặt chung: chỉ gửi khoá đã đổi (Rust giữ nguyên khoá không gửi). */
export function changedValues(fields: ExtSetting[], displays: Record<string, FieldDisplay>): ExtValues {
  const out: ExtValues = {};
  for (const field of fields) {
    const display = displays[field.key] ?? initialDisplay(field);
    if (display !== initialDisplay(field)) out[field.key] = toValue(field, display);
  }
  return out;
}

/** Cài đặt của truyện: đủ bộ trường; trường không đổi gửi lại giá trị đã lưu. */
export function storyValues(fields: ExtSetting[], displays: Record<string, FieldDisplay>): ExtValues {
  const out: ExtValues = {};
  for (const field of fields) {
    const display = displays[field.key] ?? initialDisplay(field);
    out[field.key] = display !== initialDisplay(field) ? toValue(field, display) : (field.value ?? null);
  }
  return out;
}

export function defaultHint(field: ExtSetting): string {
  if (field.default == null || field.default === "" || field.kind === "boolean") return "";
  return `Mặc định: ${asText(field.default)}`;
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function formatTime(ms: number): string {
  const d = new Date(ms);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(d.getHours())}:${pad(d.getMinutes())} ${pad(d.getDate())}/${pad(d.getMonth() + 1)}`;
}

/** Dòng trạng thái auth trên trang Tải raw. */
export function authStatus(config: ExtConfig): string {
  const auth = config.auth;
  const scheme = auth?.schemes.find((s) => s.id === auth.scheme)?.label;
  if (config.session.loggedIn) {
    const until = config.session.expiresAt ? `, hết hạn ${formatTime(config.session.expiresAt)}` : "";
    return `Đã đăng nhập${scheme ? ` (${scheme})` : ""}${until}`;
  }
  if (!auth) return "Extension không cần auth";
  if (!scheme) return auth.required ? "Chưa nhập auth — extension này bắt buộc" : "Chưa nhập auth";
  return `Đã nhập auth: ${scheme}`;
}
