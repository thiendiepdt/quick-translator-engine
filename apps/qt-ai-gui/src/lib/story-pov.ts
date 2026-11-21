import type { PovReport } from "@/lib/api";
import type { GenreSetting, StoryPov } from "@/lib/types";

/** Ô Ngôi kể trên form: "unset" = chưa chọn. */
export type PovChoice = StoryPov | "unset";

const NARRATOR_WORD = /ngôi (?:thứ )?(?:nhất|một|1)|người kể|lời kể|lời dẫn|dẫn truyện|kể chuyện|tự sự/iu;
const GROUP_PRONOUN = /(?<!\p{L})(?:người|chúng|bọn) (?:ta|tôi)(?!\p{L})/giu;
const SELF_PRONOUN = /(?<!\p{L})(?:ta|tôi)(?!\p{L})/iu;

/** Dòng style quy định người kể tự xưng `ta` / `tôi` — port `pov::mentions_narrator_pronoun` (AI điền bỏ dòng này). */
export function mentionsNarratorPronoun(text: string): boolean {
  return NARRATOR_WORD.test(text) && SELF_PRONOUN.test(text.replace(GROUP_PRONOUN, ""));
}

/** Ngôi kể gợi ý khi raw kể ngôi một: hiện đại xưng `tôi`, bối cảnh khác `ta`. */
export function suggestedFirstPov(setting: GenreSetting): StoryPov {
  return setting === "modern" ? "first-toi" : "first-ta";
}

export interface PovNotice {
  text: string;
  /** Nút sửa nhanh: đặt ô Ngôi kể. */
  fix?: { label: string; pov: StoryPov };
  /** Dòng style cần xoá (hiện kèm để người dùng tìm). */
  lines?: string[];
}

const percent = (ratio: number) => `${Math.round(ratio * 1000) / 10}%`.replace(".", ",");

/**
 * Cảnh báo ngôi kể ở Hồ sơ truyện: ô lệch raw, style còn dòng nói người kể xưng gì, đổi ngôi khi đã có chương dịch.
 * Chưa chọn + raw ngôi một chỉ nhắc ở truyện hiện đại: bối cảnh khác "chưa chọn" đã là người kể xưng `ta`.
 */
export function povNotices(input: {
  report?: PovReport;
  pov: PovChoice;
  savedPov: PovChoice;
  setting: GenreSetting;
  voice: string;
  toneRules: string;
  doneChapters: number;
}): PovNotice[] {
  const { report, pov, setting } = input;
  const notices: PovNotice[] = [];
  const sample = report
    ? `${percent(report.ratio)} đoạn có 我 ở lời kể, đọc thử ${report.chapters} chương; truyện kể ngôi ba thường dưới 11%`
    : "";
  if (report?.isFirstPerson === true && (pov === "third" || (pov === "unset" && setting === "modern"))) {
    const target = suggestedFirstPov(setting);
    notices.push({
      text: `Raw kể ngôi một (${sample}) mà ô Ngôi kể đang để "${pov === "third" ? "Ngôi ba" : "Chưa chọn"}".`,
      fix: { label: target === "first-toi" ? "Đặt: Ngôi một — xưng tôi" : "Đặt: Ngôi một — xưng ta", pov: target },
    });
  }
  if (report?.isFirstPerson === false && (pov === "first-ta" || pov === "first-toi")) {
    notices.push({
      text: `Raw có vẻ kể ngôi ba (${sample}) mà ô Ngôi kể đang để ngôi một.`,
      fix: { label: "Đặt: Ngôi ba", pov: "third" },
    });
  }
  const lines = [input.voice, ...input.toneRules.split(/\r?\n/)].map((line) => line.trim()).filter(mentionsNarratorPronoun);
  if (lines.length > 0) {
    notices.push({
      text: "Style còn dòng quy định người kể xưng gì. Ô Ngôi kể đã lo phần này, để cả hai thì prompt dễ nói hai ý ngược nhau: xoá các dòng dưới đây (tab Style) rồi chọn ô Ngôi kể.",
      lines,
    });
  }
  if (pov !== input.savedPov && input.doneChapters > 0) {
    notices.push({
      text: `Đổi ngôi kể chỉ áp cho chương dịch sau khi lưu: ${input.doneChapters} chương đã dịch vẫn giữ cách xưng cũ, muốn đồng bộ thì dịch lại các chương đó.`,
    });
  }
  return notices;
}

/** Dòng cảnh báo ở màn dịch khi truyện chọn "Ngôi một — người kể xưng tôi", kèm kết quả đọc thử raw nếu có. */
export interface ToiPovWarning {
  lead: string;
  /** Raw khớp hay lệch; thiếu khi chưa đọc được raw (đang đọc, hoặc truyện quá ít đoạn để đoán). */
  raw?: string;
  /** Raw có vẻ kể ngôi ba — dễ là chọn nhầm. */
  mismatch: boolean;
}

export function toiPovWarning(report?: PovReport): ToiPovWarning {
  const lead = 'Ngôi kể: Ngôi một — người kể xưng tôi. Lời kể dịch "tôi" thay cho "ta".';
  if (report?.isFirstPerson === true) {
    return { lead, raw: `Raw kể ngôi một (${percent(report.ratio)} đoạn có 我 ở lời kể).`, mismatch: false };
  }
  if (report?.isFirstPerson === false) {
    return { lead, raw: `Raw có vẻ kể ngôi ba (${percent(report.ratio)} đoạn có 我 ở lời kể) — rất có thể chọn nhầm.`, mismatch: true };
  }
  return { lead, mismatch: false };
}
