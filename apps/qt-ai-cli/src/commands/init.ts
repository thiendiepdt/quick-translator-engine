import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { emptyAiStoryConfig } from "@/lib/ai-story";
import {
  defaultSettings, ensureStoryDirs, listRawChapterIds, loadState,
  saveState, saveStoryConfig, storyPaths, type StoryState,
} from "../story-fs.ts";

const CLI_DIR = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const TEMPLATE_DIR = join(CLI_DIR, "antigravity");

function renderTemplate(source: string, root: string): string {
  const qtAi = `npm --prefix ${CLI_DIR} run -s qt-ai --`;
  return source.replaceAll("{{QT_AI}}", qtAi).replaceAll("{{STORY_ROOT}}", root);
}

function copyTemplates(root: string): void {
  const agentsTarget = join(root, "AGENTS.md");
  if (!existsSync(agentsTarget)) {
    writeFileSync(agentsTarget, renderTemplate(readFileSync(join(TEMPLATE_DIR, "AGENTS.md"), "utf8"), root), "utf8");
  }
  const workflowsDir = join(root, ".agent", "workflows");
  mkdirSync(workflowsDir, { recursive: true });
  for (const name of readdirSync(join(TEMPLATE_DIR, "workflows"))) {
    const target = join(workflowsDir, name);
    if (existsSync(target)) continue;
    writeFileSync(target, renderTemplate(readFileSync(join(TEMPLATE_DIR, "workflows", name), "utf8"), root), "utf8");
  }
}

export function runInit(root: string): string {
  const paths = storyPaths(resolve(root));
  ensureStoryDirs(paths);
  if (!existsSync(paths.storyJson)) saveStoryConfig(paths, emptyAiStoryConfig());
  const state: StoryState = existsSync(paths.stateJson)
    ? loadState(paths)
    : { version: 1, settings: defaultSettings(), chapters: {} };
  let added = 0;
  for (const id of listRawChapterIds(paths)) {
    if (state.chapters[id]) continue;
    state.chapters[id] = { status: "queued", reviewRound: 0, updatedAt: Date.now() };
    added += 1;
  }
  saveState(paths, state);
  copyTemplates(paths.root);
  const total = Object.keys(state.chapters).length;
  return `Đã init ${paths.root}: ${total} chương (${added} mới thêm vào hàng đợi).`;
}
