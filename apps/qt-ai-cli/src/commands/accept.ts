import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { stripAiParagraphMarkers } from "@/lib/ai-paragraphs";
import { formatAiTranslation } from "@/lib/ai-translation";
import {
  appendAutoGlossary, collectGlossaryKeys, resolveAutoGlossaryEnabled,
  sanitizeExtractedGlossary,
} from "@/lib/ai-glossary";
import {
  loadState, loadStoryConfig, readRawChapter, saveState, saveStoryConfig,
  storyPaths, workFile, type WorkKind,
} from "../story-fs.ts";
import { assembleDraft, runCheck } from "./check.ts";

const WORK_KINDS: WorkKind[] = ["prompt", "draft", "glossary", "check", "review"];

export function runAccept(
  root: string,
  id: string,
  options?: { force?: boolean },
): { outPath: string; addedGlossary: number } {
  const paths = storyPaths(resolve(root));
  const check = runCheck(root, id);
  if (!check.pass && !options?.force) {
    throw new Error(
      `Chương ${id} chưa qua check (thiếu ${check.missing.length} đoạn, ${check.violations.length} vi phạm, ratio ${check.ratio.toFixed(2)}) — sửa theo work/${id}.review.md hoặc dùng --force.`,
    );
  }

  const { finalText } = assembleDraft(root, id);
  const output = formatAiTranslation(stripAiParagraphMarkers(finalText));
  const outPath = join(paths.outDir, `${id}.md`);
  writeFileSync(outPath, output, "utf8");

  let story = loadStoryConfig(paths);
  let addedGlossary = 0;
  const glossaryPath = workFile(paths, id, "glossary");
  if (existsSync(glossaryPath) && resolveAutoGlossaryEnabled(story.autoGlossary, true)) {
    let entries: unknown = [];
    try {
      const envelope: unknown = JSON.parse(readFileSync(glossaryPath, "utf8"));
      entries =
        typeof envelope === "object" && envelope !== null && "entries" in envelope
          ? (envelope as { entries: unknown }).entries
          : envelope;
    } catch {
      entries = []; // đề xuất hỏng → bỏ qua, không chặn accept
    }
    const raw = readRawChapter(paths, id);
    const pairs = sanitizeExtractedGlossary(
      entries, raw, output, collectGlossaryKeys({}, story.glossary),
    );
    if (pairs.length > 0) {
      story = appendAutoGlossary(story, pairs, id);
      addedGlossary = pairs.length;
    }
  }
  saveStoryConfig(paths, story);

  const state = loadState(paths);
  const chapter = state.chapters[id];
  if (!chapter) throw new Error(`Không có chương ${id} trong state.json.`);
  state.chapters[id] = { status: "done", reviewRound: chapter.reviewRound, updatedAt: Date.now() };
  saveState(paths, state);

  for (const kind of WORK_KINDS) {
    rmSync(workFile(paths, id, kind), { force: true });
  }
  return { outPath, addedGlossary };
}
