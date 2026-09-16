import type { z } from "zod";

import {
  GLOSSARY_KEYS,
  type agyStatusSchema,
  type aiFillResultSchema,
  type apiProviderSchema,
  type apiSettingsSchema,
  type appConfigSchema,
  type baseViewSchema,
  type checkRuleSchema,
  type engineSchema,
  type chapterRowSchema,
  type chapterStatusSchema,
  type chapterViewSchema,
  type ApiStep,
  type exportOutcomeSchema,
  type deleteOutcomeSchema,
  type retryRangeOutcomeSchema,
  type harnessSettingsSchema,
  type progressSchema,
  type recentSummarySchema,
  type importOutcomeSchema,
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
export type GenreTone = StoryGenre["tone"];
export type StoryDefaults = z.infer<typeof storyDefaultsSchema>;
export type CheckRule = z.infer<typeof checkRuleSchema>;
export type BaseView = z.infer<typeof baseViewSchema>;
export type BaseKind = BaseView["kind"];
export type BaseSource = BaseView["source"];

export const GENRE_SETTING_LABELS: Record<GenreSetting, { label: string; hint: string }> = {
  ancient: { label: "Cổ đại / tiên hiệp", hint: "ta/ngươi/hắn/nàng, thán từ A?/Ân, cấm vợ/chồng" },
  modern: { label: "Hiện đại", hint: "anh/cô/tôi theo quan hệ, từ đời thường, thán từ hiện đại" },
  mixed: { label: "Hỗn hợp / xuyên qua lại", hint: "Chọn xưng hô theo cảnh; không bắt lỗi xưng hô" },
};
export const GENRE_NAMES_LABELS: Record<GenreNames, { label: string; hint: string }> = {
  han: { label: "Hán-Việt", hint: "Kế Duyên, Bắc Kinh" },
  foreign: { label: "Gốc nước ngoài", hint: "Emily, New York, Naruto" },
  mixed: { label: "Hỗn hợp", hint: "Họ Hán → Hán-Việt, tên phiên âm → gốc" },
};
/** Giọng khác Trung tính: prompt được chèn thêm mục "Giọng văn: …" (không hiện ở tab Prompt), không đụng xưng hô. */
export const GENRE_TONE_LABELS: Record<GenreTone, { label: string; hint: string }> = {
  neutral: { label: "Trung tính", hint: "Mặc định: tiết chế, bám sát nguyên tác; prompt như trước" },
  romance: { label: "Ngôn tình (truyện nữ)", hint: "Giữ ngọt, hài, chớt nhả trong thoại; prompt được chèn thêm mục giọng ngôn tình" },
  witty: {
    label: "Hài hước, cợt nhả",
    hint: "Truyện nam giọng đùa, hậu cung nhật thường: giữ punchline, thán từ tự nhiên, bớt Hán-Việt sách vở; chèn thêm mục giọng",
  },
  punchy: { label: "Sảng văn, dồn dập", hint: "Chiến đấu, vô địch lưu, hệ thống: câu ngắn, khí thế, cảm thán đúng lúc; chèn thêm mục giọng" },
  lyrical: { label: "Cổ phong, trữ tình", hint: "Cổ ngôn, văn thanh: giữ hình ảnh, nhịp cân đối, cho phép Hán-Việt; chèn thêm mục giọng" },
  erotic: {
    label: "Sắc (sắc hiệp, sắc đô thị)",
    hint: "Truyện người lớn: cảnh thân mật dịch đúng độ trực diện của raw, từ vựng truyện sắc, không nói giảm; chèn thêm mục giọng",
  },
  youth: {
    label: "Thanh xuân, đời thường",
    hint: "Đô thị nhẹ nhàng, giải trí văn, học đường: thoại người trẻ tự nhiên, lời kể ấm, đoạn tả nhạc/cảm xúc giữ độ bay bổng; chèn thêm mục giọng",
  },
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
export type RetryRangeOutcome = z.infer<typeof retryRangeOutcomeSchema>;
export type DeleteOutcome = z.infer<typeof deleteOutcomeSchema>;
export type RecentSummary = z.infer<typeof recentSummarySchema>;
export type ImportOutcome = z.infer<typeof importOutcomeSchema>;

export const GLOSSARY_LABELS: Record<GlossaryKey, string> = {
  names: "Tên nhân vật",
  places: "Địa danh",
  items: "Đồ vật / vũ khí",
  creatures: "Sinh vật",
  skills: "Kỹ năng / công pháp",
  common: "Từ thông dụng",
  signature_phrases: "Cụm từ đặc trưng",
  addressing: "Xưng hô theo cặp",
};

export const ENGINE_LABELS: Record<Engine, string> = { agy: "Antigravity CLI (agy)", api: "API key" };
export const API_STEP_LABELS: Record<ApiStep, string> = {
  translate: "Dịch (kể cả dịch lại, bù đoạn)",
  review: "Soát vi phạm",
  glossary: "Trích glossary",
  fill: "AI điền hồ sơ",
};
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
      return `Gọi API lỗi — đã dừng, chương đang dịch trả về hàng đợi. ${reason.message}`;
    case "user_cancelled":
      return "Đã dừng theo yêu cầu.";
    case "max_sessions":
      return "Đã chạy đủ số phiên tối đa trong Cài đặt.";
    case "internal":
      return `Lỗi nội bộ: ${reason.message}`;
  }
}
