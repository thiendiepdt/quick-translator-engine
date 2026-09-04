import type { z } from "zod";

import {
  GLOSSARY_KEYS,
  type agyStatusSchema,
  type aiFillResultSchema,
  type apiProviderSchema,
  type apiSettingsSchema,
  type appConfigSchema,
  type engineSchema,
  type chapterRowSchema,
  type chapterStatusSchema,
  type chapterViewSchema,
  type exportOutcomeSchema,
  type harnessSettingsSchema,
  type progressSchema,
  type recentSummarySchema,
  type sessionEventSchema,
  type stopReasonSchema,
  type storyConfigSchema,
  type storyDefaultsSchema,
  type storyGenreSchema,
  type storySnapshotSchema,
} from "@/lib/schema";

export { GLOSSARY_KEYS };
export type GlossaryKey = (typeof GLOSSARY_KEYS)[number];
export type ChapterStatus = z.infer<typeof chapterStatusSchema>;
export type StoryConfig = z.infer<typeof storyConfigSchema>;
export type StoryGenre = z.infer<typeof storyGenreSchema>;
export type GenreSetting = StoryGenre["setting"];
export type GenreNames = StoryGenre["names"];
export type StoryDefaults = z.infer<typeof storyDefaultsSchema>;

export const GENRE_SETTING_LABELS: Record<GenreSetting, { label: string; hint: string }> = {
  ancient: { label: "Cổ đại / tiên hiệp", hint: "ta/ngươi/hắn/nàng, thán từ A?/Ân, cấm vợ/chồng" },
  modern: { label: "Hiện đại", hint: "anh/cô/tôi theo quan hệ, từ đời thường, thán từ hiện đại" },
};
export const GENRE_NAMES_LABELS: Record<GenreNames, { label: string; hint: string }> = {
  han: { label: "Hán-Việt", hint: "Kế Duyên, Bắc Kinh" },
  foreign: { label: "Gốc nước ngoài", hint: "Emily, New York, Naruto" },
  mixed: { label: "Hỗn hợp", hint: "Họ Hán → Hán-Việt, tên phiên âm → gốc" },
};
export type HarnessSettings = z.infer<typeof harnessSettingsSchema>;
export type ChapterRow = z.infer<typeof chapterRowSchema>;
export type StorySnapshot = z.infer<typeof storySnapshotSchema>;
export type ChapterView = z.infer<typeof chapterViewSchema>;
export type AgyStatus = z.infer<typeof agyStatusSchema>;
export type AppConfig = z.infer<typeof appConfigSchema>;
export type Engine = z.infer<typeof engineSchema>;
export type ApiProvider = z.infer<typeof apiProviderSchema>;
export type ApiSettings = z.infer<typeof apiSettingsSchema>;
export type Progress = z.infer<typeof progressSchema>;
export type StopReason = z.infer<typeof stopReasonSchema>;
export type SessionEvent = z.infer<typeof sessionEventSchema>;
export type AiFillResult = z.infer<typeof aiFillResultSchema>;
export type ExportOutcome = z.infer<typeof exportOutcomeSchema>;
export type RecentSummary = z.infer<typeof recentSummarySchema>;

export const GLOSSARY_LABELS: Record<GlossaryKey, string> = {
  names: "Tên nhân vật",
  places: "Địa danh",
  items: "Đồ vật / vũ khí",
  creatures: "Sinh vật",
  skills: "Kỹ năng / công pháp",
  common: "Từ thông dụng",
  signature_phrases: "Cụm từ đặc trưng",
};

export const ENGINE_LABELS: Record<Engine, string> = { agy: "Antigravity CLI (agy)", api: "API key" };
export const API_PROVIDER_LABELS: Record<ApiProvider, string> = { gemini: "Gemini", openai: "OpenAI-compatible" };

/** Nhãn ngắn cho toolbar/log: "API · Gemini · gemini-3.7-flash" hoặc "agy". */
export function engineLabel(config: Pick<AppConfig, "engine" | "api"> | undefined): string {
  if (!config || config.engine !== "api") return "agy";
  const active = config.api[config.api.provider];
  const model = active.model.trim() || (config.api.provider === "gemini" ? "gemini-3.7-flash" : "gpt-5.6-sol");
  return `API · ${API_PROVIDER_LABELS[config.api.provider]} · ${model}`;
}

export const STATUS_LABELS: Record<ChapterStatus, string> = {
  queued: "Chờ dịch",
  translating: "Đang dịch",
  done: "Xong",
  error: "Lỗi",
  skipped: "Bỏ qua",
};

export function stopReasonLabel(reason: StopReason): string {
  switch (reason.kind) {
    case "finished":
      return "Hết hàng đợi — dịch xong.";
    case "no_progress":
      return "Phiên vừa rồi không chốt thêm chương nào — đã dừng để khỏi đốt quota. Xem log và trạng thái chương.";
    case "agy_failed":
      return `agy thoát lỗi (mã ${reason.code}) hai lần liên tiếp — kiểm tra đăng nhập/quota.`;
    case "api_failed":
      return `Gọi API lỗi liên tiếp — đã dừng. ${reason.message}`;
    case "user_cancelled":
      return "Đã dừng theo yêu cầu.";
    case "max_sessions":
      return "Đã chạy đủ số phiên tối đa trong Cài đặt.";
    case "internal":
      return `Lỗi nội bộ: ${reason.message}`;
  }
}
