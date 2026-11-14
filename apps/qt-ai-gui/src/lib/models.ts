import type { AgyStatus } from "@/lib/types";

/** id khối "Model mặc định" ở Cài đặt — nút trong cảnh báo ở trang Dịch cuộn tới đây. */
export const MODEL_SETTING_ID = "model-setting";

/** Model agy chọn nhanh ở Cài đặt: bản High dịch tốt mà không chậm như XHigh. */
export const QUICK_MODELS = ["gemini-3.8-flash-high", "gemini-4-hydrogen-high"] as const;

const EFFORTS = ["low", "medium", "high", "xhigh", "max"] as const;
export type Effort = (typeof EFFORTS)[number];
const EFFORT_LABELS: Record<Effort, string> = { low: "Low", medium: "Medium", high: "High", xhigh: "XHigh", max: "Max" };

/** Mức suy nghĩ của model agy: đuôi id (`…-medium`) hoặc nhãn (`… (Medium)`); model không ghi mức thì `null`. */
export function modelEffort(id: string, label = ""): Effort | null {
  const found = (/-(low|medium|high|xhigh|max)$/i.exec(id.trim())?.[1] ?? /\((low|medium|high|xhigh|max)\)$/i.exec(label.trim())?.[1])?.toLowerCase();
  return EFFORTS.find((effort) => effort === found) ?? null;
}

export interface ActiveModel {
  id: string;
  label: string;
  /** `agy` = ô model để trống, agy dùng model mặc định trong cài đặt của agy. */
  fromAgy: boolean;
}

/** Model thật sự dùng khi dịch bằng agy: model chọn (trang Dịch / Cài đặt), trống thì model mặc định của agy. */
export function activeModel(chosen: string | null | undefined, agy: AgyStatus | undefined): ActiveModel | null {
  const id = chosen?.trim();
  if (id) return { id, label: agy?.models.find((m) => m.id === id)?.label ?? id, fromAgy: false };
  return agy?.defaultModel ? { ...agy.defaultModel, fromAgy: true } : null;
}

/** Cảnh báo khi model ở mức Low / Medium: chất lượng dịch giảm khá nhiều so với High. */
export function weakModelWarning(model: ActiveModel | null): string | null {
  if (!model) return null;
  const effort = modelEffort(model.id, model.label);
  if (effort !== "low" && effort !== "medium") return null;
  const where = model.fromAgy ? " (model mặc định của agy)" : "";
  return `${model.label}${where} đang ở mức suy nghĩ ${EFFORT_LABELS[effort]} — chất lượng dịch giảm khá nhiều. Nên chọn bản High, ví dụ ${QUICK_MODELS.join(" hoặc ")}.`;
}
