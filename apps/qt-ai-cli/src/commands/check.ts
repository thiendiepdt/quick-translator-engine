import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  aiParagraphsOf, labeledAiRepairPayload, parseLabeledAiTranslation,
} from "@/lib/ai-paragraphs";
import {
  checkAiTranslationViolations,
  type TranslationViolation,
} from "@/lib/ai-translation";
import {
  loadState, loadStoryConfig, readRawChapter, saveState, storyPaths, workFile,
} from "../story-fs.ts";

export interface CheckResult {
  pass: boolean;
  missing: number[];
  violations: TranslationViolation[];
  ratio: number;
  escalatedToError: boolean;
  reviewPath?: string;
}

/**
 * `violations` chứa `line` theo văn bản đã lắp (finalText): mỗi đoạn còn lại
 * (paragraph không undefined trong `parsed`) chiếm 1 dòng, cách nhau 1 dòng
 * trống — đoạn thứ k (tính trên các đoạn CÒN LẠI, 1-based) nằm ở dòng 2k−1.
 * Muốn agent sửa đúng chỗ trong draft, phải quy dòng đó về nhãn [[n]] gốc:
 * lấy vị trí k rồi tra ngược `definedIndices` (chỉ số 0-based trong mảng
 * `parsed` gốc, đã bỏ các đoạn thiếu) để ra nhãn thật.
 */
function buildViolationsSection(
  id: string,
  parsed: Array<string | undefined>,
  violations: TranslationViolation[],
): string {
  const definedIndices = parsed
    .map((paragraph, index) => (paragraph !== undefined ? index : -1))
    .filter((index) => index >= 0);
  const list = violations
    .map((item) => {
      const position = Math.ceil(item.line / 2); // vị trí đoạn trong finalText (1-based)
      const originalIndex = definedIndices[position - 1];
      const label = originalIndex !== undefined ? originalIndex + 1 : position;
      return `- [[${label}]] (dòng ${item.line} trong bản lắp): ${item.message} — "${item.text}"`;
    })
    .join("\n");
  return [
    `# Vi phạm rule — sửa tối thiểu ngay trong work/${id}.draft.md`,
    "",
    "Sửa ĐÚNG TẠI CHỖ trong work/" + id + ".draft.md: với mỗi vi phạm dưới đây, tìm đoạn có nhãn [[n]] tương ứng, " +
      "chỉ đổi đúng từ/cụm gây lỗi, GIỮ NGUYÊN toàn bộ nhãn [[n]] hiện có và không viết lại hay chau chuốt các đoạn khác.",
    "",
    list,
  ].join("\n");
}

export function assembleDraft(root: string, id: string): {
  paragraphs: string[];
  parsed: Array<string | undefined>;
  finalText: string;
} {
  const paths = storyPaths(resolve(root));
  const draftPath = workFile(paths, id, "draft");
  if (!existsSync(draftPath)) {
    throw new Error(`Chưa có bản dịch ${draftPath} — agent phải ghi draft trước khi check.`);
  }
  const paragraphs = aiParagraphsOf(readRawChapter(paths, id));
  const parsed =
    parseLabeledAiTranslation(readFileSync(draftPath, "utf8"), paragraphs.length) ??
    Array.from({ length: paragraphs.length }, () => undefined);
  const finalText = parsed
    .filter((paragraph): paragraph is string => paragraph !== undefined)
    .join("\n\n");
  return { paragraphs, parsed, finalText };
}

export function runCheck(root: string, id: string): CheckResult {
  const paths = storyPaths(resolve(root));
  const state = loadState(paths);
  const chapter = state.chapters[id];
  if (!chapter) throw new Error(`Không có chương ${id} trong state.json.`);
  const story = loadStoryConfig(paths);
  const { paragraphs, parsed, finalText } = assembleDraft(root, id);

  const missing = parsed
    .map((paragraph, index) => (paragraph === undefined ? index + 1 : 0))
    .filter((label) => label > 0);
  const violations = checkAiTranslationViolations(finalText, story.checkRules);
  const rawLength = paragraphs.join("").replace(/\s/g, "").length;
  const translatedLength = finalText.replace(/\s/g, "").length;
  const ratio = rawLength > 0 ? translatedLength / rawLength : 1;
  const tooShort = ratio < state.settings.minLengthRatio;
  const pass = missing.length === 0 && violations.length === 0 && !tooShort;

  let escalatedToError = false;
  let reviewPath: string | undefined;

  if (!pass) {
    if (chapter.reviewRound >= state.settings.maxReviewRounds) {
      escalatedToError = true;
      state.chapters[id] = {
        ...chapter,
        status: "error",
        reason: `Quá ${state.settings.maxReviewRounds} vòng review vẫn chưa đạt (thiếu ${missing.length} đoạn, ${violations.length} vi phạm, ratio ${ratio.toFixed(2)}).`,
        updatedAt: Date.now(),
      };
    } else {
      const sections: string[] = [];
      if (missing.length > 0) {
        sections.push(
          `# Đoạn còn thiếu — dịch bổ sung các đoạn dưới đây rồi CHÈN vào work/${id}.draft.md, ` +
            "giữ đúng nhãn [[n]] cho từng đoạn, không sửa các đoạn đã có\n\n" +
            labeledAiRepairPayload(paragraphs, missing.map((label) => label - 1)),
        );
      }
      if (violations.length > 0) {
        sections.push(buildViolationsSection(id, parsed, violations));
      }
      if (tooShort) {
        sections.push(
          `# Bản dịch quá ngắn\n\nTỉ lệ ký tự dịch/raw = ${ratio.toFixed(2)} < ${state.settings.minLengthRatio}. Rà từng đoạn xem có bị tóm tắt/lược ý; dịch đủ 100% nội dung.`,
        );
      }
      reviewPath = workFile(paths, id, "review");
      writeFileSync(reviewPath, `${sections.join("\n\n---\n\n")}\n`, "utf8");
      state.chapters[id] = { ...chapter, reviewRound: chapter.reviewRound + 1, updatedAt: Date.now() };
    }
    saveState(paths, state);
  }

  writeFileSync(
    workFile(paths, id, "check"),
    `${JSON.stringify(
      {
        pass, missing, violationCount: violations.length, ratio,
        reviewRound: (state.chapters[id] ?? chapter).reviewRound,
        checkedAt: Date.now(),
      },
      null, 2,
    )}\n`,
    "utf8",
  );
  return { pass, missing, violations, ratio, escalatedToError, ...(reviewPath ? { reviewPath } : {}) };
}
