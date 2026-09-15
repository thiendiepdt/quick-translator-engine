import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { z } from "zod";

import {
  agyStatusSchema,
  aiFillResultSchema,
  appConfigSchema,
  baseViewSchema,
  chapterViewSchema,
  exportOutcomeSchema,
  harnessSettingsSchema,
  importOutcomeSchema,
  recentSummarySchema,
  retryRangeOutcomeSchema,
  deleteOutcomeSchema,
  sessionStatusSchema,
  storyConfigSchema,
  storyDefaultsSchema,
  storySnapshotSchema,
} from "@/lib/schema";
import type {
  AppConfig,
  BaseKind,
  CheckRule,
  GenreNames,
  GenreSetting,
  HarnessSettings,
  StoryConfig,
  StoryGenre,
} from "@/lib/types";

export class ApiError extends Error {
  kind: string;
  constructor(kind: string, message: string) {
    super(message);
    this.name = "ApiError";
    this.kind = kind;
  }
}

function toApiError(error: unknown): ApiError {
  if (error instanceof ApiError) return error;
  if (typeof error === "object" && error !== null && "message" in error) {
    const record = error as { kind?: unknown; message?: unknown };
    return new ApiError(typeof record.kind === "string" ? record.kind : "unknown", String(record.message));
  }
  return new ApiError("unknown", typeof error === "string" ? error : "Lệnh thất bại");
}

async function call<T>(
  command: string,
  args: Record<string, unknown> | undefined,
  parse: (value: unknown) => T,
): Promise<T> {
  try {
    return parse(await invoke(command, args));
  } catch (error) {
    throw toApiError(error);
  }
}

const noop = () => undefined;

export const openStory = (root: string) => call("open_story", { root }, (v) => storySnapshotSchema.parse(v));
export const initStory = (root: string) => call("init_story", { root }, (v) => storySnapshotSchema.parse(v));
export const storySnapshot = (root: string) =>
  call("story_snapshot", { root }, (v) => storySnapshotSchema.parse(v));
export const readChapter = (root: string, id: string) =>
  call("read_chapter", { root, id }, (v) => chapterViewSchema.parse(v));
export const saveStory = (root: string, story: StoryConfig) =>
  call("save_story", { root, story }, (v) => storyConfigSchema.parse(v));
export const saveSettings = (root: string, settings: HarnessSettings) =>
  call("save_settings", { root, settings }, (v) => harnessSettingsSchema.parse(v));
export const chapterRetry = (root: string, id: string) => call("chapter_retry", { root, id }, noop);
export const saveChapterOutput = (root: string, id: string, text: string) =>
  call("save_chapter_output", { root, id, text }, noop);
export const chaptersRetry = (root: string, range: { from?: string; to?: string }) =>
  call("chapters_retry", { root, from: range.from ?? null, to: range.to ?? null }, (v) => retryRangeOutcomeSchema.parse(v));
/** Dịch lại đúng các chương `ids` (chương hổng xen kẽ chương done): queued bỏ qua, done giữ .bak. */
export const chaptersRetryIds = (root: string, ids: string[]) =>
  call("chapters_retry_ids", { root, ids }, (v) => retryRangeOutcomeSchema.parse(v));
/** Xoá hẳn chương: gỡ state, xoá raw/ + work/; out/<id>.txt giữ nguyên (keptOutputs). */
export const chaptersDelete = (root: string, ids: string[]) =>
  call("chapters_delete", { root, ids }, (v) => deleteOutcomeSchema.parse(v));
export const chapterSkip = (root: string, id: string, reason: string) =>
  call("chapter_skip", { root, id, reason }, noop);
export const chapterForceAccept = (root: string, id: string) =>
  call("chapter_force_accept", { root, id }, (v) => (Array.isArray(v) ? v.map(String) : []));
export const exportChapters = (root: string, range: { from?: string; to?: string; out?: string }) =>
  call("export_chapters", { root, from: range.from ?? null, to: range.to ?? null, out: range.out ?? null }, (v) =>
    exportOutcomeSchema.parse(v),
  );
export const revealFolder = (path: string) => call("reveal_folder", { path }, noop);
export const storyDefaults = (genre: StoryGenre) =>
  call("story_defaults", { genre }, (v) => storyDefaultsSchema.parse(v));

export interface BasePayload {
  text?: string;
  rules?: CheckRule[];
  glossary?: StoryConfig["glossary"];
}
/** Bản mặc định của app (Cài đặt → Bản mặc định). `names` chỉ dùng cho kind "prompt". */
export const baseGet = (kind: BaseKind, setting: GenreSetting, names?: GenreNames) =>
  call("base_get", { kind, setting, names }, (v) => baseViewSchema.parse(v));
export const baseSave = (kind: BaseKind, setting: GenreSetting, names: GenreNames | undefined, payload: BasePayload) =>
  call("base_save", { kind, setting, names, payload }, (v) => baseViewSchema.parse(v));
export const baseReset = (kind: BaseKind, setting: GenreSetting, names?: GenreNames) =>
  call("base_reset", { kind, setting, names }, (v) => baseViewSchema.parse(v));
export const agyStatus = (configured?: string) =>
  call("agy_status", { configured: configured ?? null }, (v) => agyStatusSchema.parse(v));
export const appConfigGet = () => call("app_config_get", undefined, (v) => appConfigSchema.parse(v));
export const appConfigSet = (config: AppConfig) =>
  call("app_config_set", { config }, (v) => appConfigSchema.parse(v));
export const recentSummaries = () =>
  call("recent_summaries", undefined, (v) => z.array(recentSummarySchema).parse(v));
export const sessionStart = (root: string, model?: string) =>
  call("session_start", { root, model: model ?? null }, (v) => sessionStatusSchema.parse(v));
export const sessionStop = (root: string) => call("session_stop", { root }, (v) => sessionStatusSchema.parse(v));
export const sessionState = () => call("session_state", undefined, (v) => sessionStatusSchema.parse(v));
export const aiFillStory = (root: string, name: string, sourceUrl: string) =>
  call("ai_fill_story", { root, name, sourceUrl }, (v) => aiFillResultSchema.parse(v));
export const slugifyName = (name: string) => call("slugify_name", { name }, (v) => z.string().parse(v));
/** Không truyền root → thư viện trong config; truyền root → liệt kê folder bất kỳ (dò "có phải thư viện?"). */
export const libraryList = (root?: string) =>
  call("library_list", { root: root ?? null }, (v) => z.array(recentSummarySchema).parse(v));
export const createStory = (name: string, slug: string, sourceUrl: string) =>
  call("create_story", { name, slug, sourceUrl }, (v) => storySnapshotSchema.parse(v));
export const rescanStory = (root: string) => call("rescan_story", { root }, (v) => storySnapshotSchema.parse(v));
export const importChapters = (root: string, paths: string[]) =>
  call("import_chapters", { root, paths }, (v) => importOutcomeSchema.parse(v));

export async function pickFolder(title: string): Promise<string | undefined> {
  const selected = await open({ directory: true, multiple: false, title });
  return typeof selected === "string" ? selected : undefined;
}

export async function pickAgyFile(): Promise<string | undefined> {
  const selected = await open({
    directory: false,
    multiple: false,
    title: "Chọn file agy",
    filters: [{ name: "agy", extensions: ["exe", "cmd", "bat", "*"] }],
  });
  return typeof selected === "string" ? selected : undefined;
}

export async function pickSaveFile(defaultName: string): Promise<string | undefined> {
  const selected = await save({
    title: "Lưu file gộp",
    defaultPath: defaultName,
    filters: [{ name: "Văn bản UTF-8", extensions: ["txt"] }],
  });
  return selected ?? undefined;
}
