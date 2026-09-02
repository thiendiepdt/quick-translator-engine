import {
  copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync,
  renameSync, writeFileSync,
} from "node:fs";
import { join } from "node:path";
import {
  naturalChapterCompare, parseAiStoryConfigJson, type AiStoryConfig,
} from "@/lib/ai-story";

export type ChapterStatus = "queued" | "translating" | "done" | "error" | "skipped";

export interface ChapterState {
  status: ChapterStatus;
  reviewRound: number;
  reason?: string;
  /** Chương done nhưng hết vòng review vẫn còn vi phạm rule — người dùng xem lại sau. */
  warnings?: string[];
  updatedAt: number;
}

export interface HarnessSettings {
  minLengthRatio: number;
  maxReviewRounds: number;
  chaptersPerSession: number;
}

export interface StoryState {
  version: 1;
  settings: HarnessSettings;
  chapters: Record<string, ChapterState>;
}

export interface StoryPaths {
  root: string;
  storyJson: string;
  stateJson: string;
  rawDir: string;
  outDir: string;
  workDir: string;
}

export function storyPaths(root: string): StoryPaths {
  return {
    root,
    storyJson: join(root, "story.json"),
    stateJson: join(root, "state.json"),
    rawDir: join(root, "raw"),
    outDir: join(root, "out"),
    workDir: join(root, "work"),
  };
}

export function defaultSettings(): HarnessSettings {
  return { minLengthRatio: 0.75, maxReviewRounds: 3, chaptersPerSession: 10 };
}

export function listRawChapterIds(paths: StoryPaths): string[] {
  if (!existsSync(paths.rawDir)) return [];
  return readdirSync(paths.rawDir)
    .filter((name) => name.endsWith(".txt"))
    .map((name) => name.slice(0, -4))
    .sort(naturalChapterCompare);
}

export function readRawChapter(paths: StoryPaths, id: string): string {
  return readFileSync(join(paths.rawDir, `${id}.txt`), "utf8");
}

/** Ghi atomic: file tạm cùng thư mục rồi rename đè. */
function writeAtomic(path: string, content: string): void {
  const tmp = `${path}.tmp`;
  writeFileSync(tmp, content, "utf8");
  renameSync(tmp, path);
}

export function loadStoryConfig(paths: StoryPaths): AiStoryConfig {
  if (!existsSync(paths.storyJson)) {
    throw new Error(`Không thấy story.json trong ${paths.root} — chạy: qt-ai init`);
  }
  const config = parseAiStoryConfigJson(readFileSync(paths.storyJson, "utf8"));
  if (!config) throw new Error(`story.json hỏng (không phải JSON object): ${paths.storyJson}`);
  return config;
}

export function saveStoryConfig(paths: StoryPaths, config: AiStoryConfig): void {
  if (existsSync(paths.storyJson)) {
    copyFileSync(paths.storyJson, `${paths.storyJson}.bak`);
  }
  writeAtomic(paths.storyJson, `${JSON.stringify(config, null, 2)}\n`);
}

function isChapterState(value: unknown): value is ChapterState {
  if (typeof value !== "object" || value === null) return false;
  const record = value as Record<string, unknown>;
  return (
    ["queued", "translating", "done", "error", "skipped"].includes(String(record.status)) &&
    typeof record.reviewRound === "number" &&
    typeof record.updatedAt === "number"
  );
}

export function loadState(paths: StoryPaths): StoryState {
  if (!existsSync(paths.stateJson)) {
    throw new Error(`Không thấy state.json trong ${paths.root} — chạy: qt-ai init`);
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(readFileSync(paths.stateJson, "utf8"));
  } catch (error) {
    throw new Error(`state.json hỏng: ${error instanceof Error ? error.message : String(error)}`);
  }
  if (
    typeof parsed !== "object" || parsed === null || Array.isArray(parsed) ||
    (parsed as Record<string, unknown>).version !== 1
  ) {
    throw new Error(`state.json sai schema (cần object version 1): ${paths.stateJson}`);
  }
  const record = parsed as Record<string, unknown>;
  const settingsValue = record.settings as Record<string, unknown> | undefined;
  const fallback = defaultSettings();
  const settings: HarnessSettings = {
    minLengthRatio: typeof settingsValue?.minLengthRatio === "number" ? settingsValue.minLengthRatio : fallback.minLengthRatio,
    maxReviewRounds: typeof settingsValue?.maxReviewRounds === "number" ? settingsValue.maxReviewRounds : fallback.maxReviewRounds,
    chaptersPerSession: typeof settingsValue?.chaptersPerSession === "number" ? settingsValue.chaptersPerSession : fallback.chaptersPerSession,
  };
  const chapters: Record<string, ChapterState> = {};
  if (typeof record.chapters === "object" && record.chapters !== null) {
    for (const [id, value] of Object.entries(record.chapters as Record<string, unknown>)) {
      if (isChapterState(value)) chapters[id] = value;
    }
  }
  return { version: 1, settings, chapters };
}

export function saveState(paths: StoryPaths, state: StoryState): void {
  writeAtomic(paths.stateJson, `${JSON.stringify(state, null, 2)}\n`);
}

export type WorkKind = "prompt" | "draft" | "glossary" | "check" | "review";

const WORK_SUFFIX: Record<WorkKind, string> = {
  prompt: ".prompt.md",
  draft: ".draft.md",
  glossary: ".glossary.json",
  check: ".check.json",
  review: ".review.md",
};

export function workFile(paths: StoryPaths, id: string, kind: WorkKind): string {
  return join(paths.workDir, `${id}${WORK_SUFFIX[kind]}`);
}

export function ensureStoryDirs(paths: StoryPaths): void {
  for (const dir of [paths.rawDir, paths.outDir, paths.workDir]) {
    mkdirSync(dir, { recursive: true });
  }
}
