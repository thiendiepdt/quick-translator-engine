import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open, save } from "@tauri-apps/plugin-dialog";
import { z } from "zod";

import {
  agyStatusSchema,
  aiFillResultSchema,
  appConfigSchema,
  baseViewSchema,
  bookInfoViewSchema,
  castSchema,
  chapterViewSchema,
  exportOutcomeSchema,
  harnessSettingsSchema,
  agyDataCleanedSchema,
  agyDataStatsSchema,
  importOutcomeSchema,
  libraryRescanSchema,
  vnLibraryProgressSchema,
  vnLibraryScanSchema,
  recentSummarySchema,
  retryRangeOutcomeSchema,
  scanCastViewSchema,
  glossaryUsageSchema,
  replaceFindSchema,
  replaceOutcomeSchema,
  replaceUndoInfoSchema,
  replaceUndoOutcomeSchema,
  paddingSettingsSchema,
  shortChaptersSchema,
  zhConvertReportSchema,
  zhStoryOutcomeSchema,
  zhStoryStatusSchema,
  zhUndoOutcomeSchema,
  glossaryBackfillSchema,
  deleteOutcomeSchema,
  progressDriftSchema,
  requeueUntrackedResultSchema,
  sessionStatusSchema,
  sourceImageSchema,
  storyConfigSchema,
  storyDefaultsSchema,
  storySnapshotSchema,
  syncProgressResultSchema,
  titleSurveySchema,
  titlesAppliedSchema,
  vnEntrySchema,
  vnHitViewSchema,
  vnSettingsSchema,
  vnSummarySchema,
  vnVerifyOutcomeSchema,
} from "@/lib/schema";
import type {
  AppConfig,
  BaseKind,
  Cast,
  CheckRule,
  GenreNames,
  GenreSetting,
  HarnessSettings,
  StoryConfig,
  StoryGenre,
  VnSettings,
  VnStatus,
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
/** Tên tiêu đề lặp của truyện + các kiểu dịch đang có trong out/. */
export const titlesSurvey = (root: string) => call("titles_survey", { root }, (v) => titleSurveySchema.parse(v));
/** Chốt bản dịch cho các tên trong `targets` rồi ghi lại dòng tiêu đề của mọi chương đã dịch cho khớp. */
export const titlesApply = (root: string, targets: Record<string, string>) =>
  call("titles_apply", { root, targets }, (v) => titlesAppliedSchema.parse(v));
export const castLoad = (root: string) => call("cast_load", { root }, (v) => castSchema.parse(v));
export const castSave = (root: string, cast: Cast) => call("cast_save", { root, cast }, (v) => castSchema.parse(v));
/** Quét bù giới tính bằng API key (một lượt JSON mỗi 40 tên) — chỉ điền tên chưa rõ giới. */
export const castScan = (root: string) => call("cast_scan", { root }, (v) => scanCastViewSchema.parse(v));
/** Xoá khỏi story.json các cặp xưng hô trái giới đã chốt; trả các key đã xoá. */
export const castCleanAddressing = (root: string) =>
  call("cast_clean_addressing", { root }, (v) => z.array(z.string()).parse(v));
/** Đoạn đã dịch có `key` trong raw mà bản dịch không chứa `target` (50 đoạn mẫu + tổng). */
export const glossaryScan = (root: string, key: string, target: string) =>
  call("glossary_scan", { root, key, target }, (v) => glossaryUsageSchema.parse(v));
/** Thay `from` → `to` trong các đoạn có `key` ở raw; `apply = false` chỉ đếm để xem trước. */
export const glossaryBackfill = (root: string, key: string, from: string, to: string, apply: boolean) =>
  call("glossary_backfill", { root, key, from, to, apply }, (v) => glossaryBackfillSchema.parse(v));
/** Truy vấn Tìm & thay (không regex). `from`/`to`: phạm vi id chương, null = không giới hạn. */
export interface ReplaceQuery {
  find: string;
  replace: string;
  caseSensitive: boolean;
  wholeWord: boolean;
  rawFilter: string;
  from: string | null;
  to: string | null;
}
export interface ReplaceHitRef {
  chapter: string;
  paragraph: number;
  occurrence: number;
}
/** Một chỗ trong hồ sơ (Rust `ProfileRef`). */
export interface ReplaceProfileRef {
  field: string;
  key: string;
  index: number;
  occurrence: number;
}
/** Tìm trong bản dịch đã chốt: tổng + số theo chương (đủ) + 500 chỗ đầu kèm ngữ cảnh; hồ sơ trả đủ mọi chỗ. */
export const replaceFind = (root: string, query: ReplaceQuery) =>
  call("replace_find", { root, query }, (v) => replaceFindSchema.parse(v));
/** Thay mọi chỗ khớp trong chương và hồ sơ trừ chương / chỗ bỏ chọn; lưu bản gốc để hoàn tác. */
export const replaceApply = (
  root: string,
  query: ReplaceQuery,
  excludedChapters: string[],
  excludedHits: ReplaceHitRef[],
  excludedProfile: ReplaceProfileRef[],
) =>
  call("replace_apply", { root, query, excludedChapters, excludedHits, excludedProfile }, (v) =>
    replaceOutcomeSchema.parse(v),
  );
export const replaceUndoInfo = (root: string) => call("replace_undo_info", { root }, (v) => replaceUndoInfoSchema.parse(v));
export const replaceUndo = (root: string) => call("replace_undo", { root }, (v) => replaceUndoOutcomeSchema.parse(v));
/** Raw phồn thể: `full = false` chỉ đọc mấy chương đầu (thanh nhắc), true đọc cả truyện. */
export const zhStoryStatus = (root: string, full: boolean) =>
  call("zh_story_status", { root, full }, (v) => zhStoryStatusSchema.parse(v));
/** Chuyển chương phồn thể / khác UTF-8 sang giản thể, bản gốc vào raw-phon-the/, đổi khoá hồ sơ theo. */
export const zhStoryConvert = (root: string) => call("zh_story_convert", { root }, (v) => zhStoryOutcomeSchema.parse(v));
export const zhStoryUndo = (root: string) => call("zh_story_undo", { root }, (v) => zhUndoOutcomeSchema.parse(v));
/** Cách ghi của công cụ chuyển: đè file gốc / ra folder khác (giữ cấu trúc) / cạnh file gốc thêm hậu tố. */
export type ZhOutputMode = { kind: "replace" } | { kind: "folder"; dir: string } | { kind: "suffix"; suffix: string };
export interface ZhConvertJob {
  /** File hoặc folder; file trong folder lọc theo `extensions` (rỗng = mọi file text). */
  inputs: string[];
  recursive: boolean;
  extensions: string[];
  mode: ZhOutputMode;
}
export type PaddingSettings = z.infer<typeof paddingSettingsSchema>;
export const paddingGet = () => call("padding_get", undefined, (v) => paddingSettingsSchema.parse(v));
export const paddingSave = (settings: PaddingSettings) =>
  call("padding_save", { settings }, (v) => paddingSettingsSchema.parse(v));
/** Về mặc định: tắt, 500 chữ, văn mẫu gốc. */
export const paddingReset = () => call("padding_reset", undefined, (v) => paddingSettingsSchema.parse(v));
/** Chương đã dịch dưới ngưỡng chữ của nền tảng đăng truyện. */
export const shortChapters = (root: string) => call("short_chapters", { root }, (v) => shortChaptersSchema.parse(v));
/** Chèn cho đủ ngưỡng một chương; false khi chương đã đủ, không đổi gì. */
export const chapterPad = (root: string, id: string) => call("chapter_pad", { root, id }, (v) => z.boolean().parse(v));
export const zhConvertText = (text: string) => call("zh_convert_text", { text }, (v) => z.string().parse(v));
export const zhConvertFiles = (job: ZhConvertJob) => call("zh_convert_files", { job }, (v) => zhConvertReportSchema.parse(v));
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
/** Reset truyện về như lúc vừa tạo: chỉ giữ tên + link; raw/, out/, export/ còn nguyên; mọi chương về hàng đợi. */
export const storyReset = (root: string) => call("story_reset", { root }, (v) => storySnapshotSchema.parse(v));
/** Chương lệch giữa state.json và out/ (Drive sync lệch) — chỉ đọc, để xem trước. */
export const progressDrift = (root: string) => call("progress_drift", { root }, (v) => progressDriftSchema.parse(v));
/** Đánh dấu đã dịch các chương có bản dịch trong out/; trả kèm snapshot mới. Bị chặn khi phiên đang chạy. */
export const syncProgress = (root: string) => call("sync_progress", { root }, (v) => syncProgressResultSchema.parse(v));
/** Báo lệch nhầm: các chương lệch về hàng đợi dịch lại, out/<id>.txt cất .bak; trả kèm snapshot mới. */
export const requeueUntracked = (root: string) =>
  call("requeue_untracked", { root }, (v) => requeueUntrackedResultSchema.parse(v));
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

export interface BookMeta {
  /** Tác giả hiển thị (Hán-Việt hoặc người dùng sửa). */
  author: string;
  /** Bút danh gốc chữ Hán — trang tiêu đề EPUB ghi kèm trong ngoặc. */
  authorOriginal: string;
  source: string;
  coverUrl: string;
  fetchedAt: number;
}
/** Trang / API thông tin sách qua Rust (chỉ host trong danh sách của core). */
export const sourceFetchText = (url: string) => call("source_fetch_text", { url }, (v) => String(v));
/** Ảnh bìa qua Rust (thử lại lỗi tạm thời); byte gốc, chưa đổi định dạng. */
export const sourceFetchImage = (url: string) =>
  call("source_fetch_image", { url }, (v) => {
    const image = sourceImageSchema.parse(v);
    return { bytes: Uint8Array.from(image.bytes), mime: image.mime };
  });
/** Bút danh chữ Hán → Hán-Việt viết hoa từng âm (bảng phiên âm QuickTranslator, ở Rust). */
export const hanVietName = (text: string) => call("han_viet_name", { text }, (v) => String(v));
export const bookInfoGet = (root: string) =>
  call("book_info_get", { root }, (v) => {
    const view = bookInfoViewSchema.parse(v);
    return { meta: view.meta, cover: view.cover ? Uint8Array.from(view.cover) : undefined };
  });
/** `cover` = JPEG mới cho `cover.jpg`; `clearCover` = xoá bìa đã lưu. */
export const bookInfoSave = (root: string, meta: BookMeta, cover?: Uint8Array, clearCover = false) =>
  call("book_info_save", { root, meta, cover: cover ? Array.from(cover) : null, clearCover }, noop);
export const exportEpub = (
  root: string,
  range: { from?: string; to?: string; out?: string },
  book: { author: string; cover?: Uint8Array },
) =>
  call(
    "export_epub",
    {
      root,
      from: range.from ?? null,
      to: range.to ?? null,
      out: range.out ?? null,
      author: book.author,
      cover: book.cover ? Array.from(book.cover) : null,
    },
    (v) => exportOutcomeSchema.parse(v),
  );
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
/** Đếm hội thoại agy do app sinh ra đang nằm trên đĩa (vài trăm cái thì mất vài giây). */
export const agyDataStats = () => call("agy_data_stats", undefined, (v) => agyDataStatsSchema.parse(v));
/** Xoá hội thoại agy do app sinh ra; truyện đang dịch và hội thoại vừa ghi được giữ lại. */
export const agyDataClean = () => call("agy_data_clean", undefined, (v) => agyDataCleanedSchema.parse(v));
export const appConfigGet = () => call("app_config_get", undefined, (v) => appConfigSchema.parse(v));
export const appConfigSet = (config: AppConfig) =>
  call("app_config_set", { config }, (v) => appConfigSchema.parse(v));
export const recentSummaries = () =>
  call("recent_summaries", undefined, (v) => z.array(recentSummarySchema).parse(v));
export const sessionStart = (root: string, model?: string) =>
  call("session_start", { root, model: model ?? null }, (v) => sessionStatusSchema.parse(v));
export const sessionStop = (root: string) => call("session_stop", { root }, (v) => sessionStatusSchema.parse(v));
/** Dừng mọi phiên đang chạy (đóng app khi đang dịch). */
export const sessionsStopAll = () => call("sessions_stop_all", undefined, (v) => sessionStatusSchema.parse(v));
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
/** Quét raw/ của mọi truyện trong thư viện: chương mới vào hàng đợi; truyện đang dịch bỏ qua. */
export const libraryRescan = () => call("library_rescan", undefined, (v) => libraryRescanSchema.parse(v));
/** Quét nhạy cảm VN mọi truyện trong thư viện, chạy nền ít luồng; tiến độ qua `onVnLibraryProgress`. */
export const vnScanLibrary = () => call("vn_scan_library", undefined, (v) => vnLibraryScanSchema.parse(v));
/** Dừng lượt quét thư viện sau truyện đang quét. */
export const vnScanLibraryCancel = () => call("vn_scan_library_cancel", undefined, noop);
/** Nghe tiến độ quét thư viện (mỗi truyện xong một lần); trả hàm huỷ nghe. */
export const onVnLibraryProgress = (handler: (progress: { done: number; total: number; root: string }) => void) =>
  listen("vn-library-progress", (event) => {
    const parsed = vnLibraryProgressSchema.safeParse(event.payload);
    if (parsed.success) handler(parsed.data);
  });
export const importChapters = (root: string, paths: string[]) =>
  call("import_chapters", { root, paths }, (v) => importOutcomeSchema.parse(v));

export async function pickFolder(title: string): Promise<string | undefined> {
  const selected = await open({ directory: true, multiple: false, title });
  return typeof selected === "string" ? selected : undefined;
}

export async function pickFiles(title: string): Promise<string[]> {
  const selected = await open({ directory: false, multiple: true, title });
  return Array.isArray(selected) ? selected : typeof selected === "string" ? [selected] : [];
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

export async function pickSaveFile(
  defaultName: string,
  title = "Lưu file gộp",
  filter: { name: string; extensions: string[] } = { name: "Văn bản UTF-8", extensions: ["txt"] },
): Promise<string | undefined> {
  const selected = await save({ title, defaultPath: defaultName, filters: [filter] });
  return selected ?? undefined;
}

/** Ghi nguyên chuỗi ra file (BOM nếu cần phải nằm sẵn trong `content`). */
export const writeTextFile = (path: string, content: string) =>
  call("write_text_file", { path, content }, () => undefined);

export const vnRefresh = (root: string) => call("vn_refresh", { root }, (v) => vnSummarySchema.parse(v));
export const vnReport = (root: string) => call("vn_report", { root }, (v) => z.array(vnHitViewSchema).parse(v));
export const vnSetStatus = (root: string, ids: string[], status: VnStatus) =>
  call("vn_set_status", { root, ids, status }, (v) => z.number().parse(v));
export const vnAiVerify = (root: string, ids: string[]) => call("vn_ai_verify", { root, ids }, (v) => vnVerifyOutcomeSchema.parse(v));
export const vnBuiltinEntries = () => call("vn_builtin_entries", undefined, (v) => z.array(vnEntrySchema).parse(v));
export const vnSettingsGet = () => call("vn_settings_get", undefined, (v) => vnSettingsSchema.parse(v));
export const vnSettingsSave = (settings: VnSettings) => call("vn_settings_save", { settings }, (v) => vnSettingsSchema.parse(v));
export interface VnReplaceRequest {
  from: string;
  to: string;
  chapters: string[];
  paragraphs?: [string, number][];
}
/** Thay nhiều cặp một lượt, chung một lần hoàn tác (cụm gốc dài chạy trước; cặp có cụm thay trống bị bỏ). */
export const vnReplace = (root: string, requests: VnReplaceRequest[]) =>
  call("vn_replace", { root, requests }, (v) => replaceOutcomeSchema.parse(v));
