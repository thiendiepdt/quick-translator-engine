import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  aiParagraphsOf, labeledAiRepairPayload, parseLabeledAiTranslation,
} from "@/lib/ai-paragraphs";
import {
  buildAiTranslationReviewPrompt, checkAiTranslationViolations,
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
          "# Đoạn còn thiếu — dịch bổ sung rồi chèn vào work draft\n\n" +
            labeledAiRepairPayload(paragraphs, missing.map((label) => label - 1)),
        );
      }
      if (violations.length > 0) {
        const review = buildAiTranslationReviewPrompt(finalText, violations);
        sections.push(`# Vi phạm rule — sửa tối thiểu\n\n${review.system}\n\n${review.user}`);
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
