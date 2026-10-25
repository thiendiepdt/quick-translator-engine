import { create } from "zustand";

import { progressDrift } from "@/lib/api";
import { filterChapters, type ChapterFilter } from "@/lib/chapters";
import { pathKey, samePath } from "@/lib/paths";
import type { AgyStatus, AppConfig, Progress, SessionEvent, StopReason, StorySnapshot } from "@/lib/types";

export type Page = "translate" | "story" | "vn" | "replace" | "export" | "settings";

export type SessionState =
  | { status: "idle" }
  /** `waitUntil` (ms epoch): agy hết quota, phiên đang chờ tới giờ đó rồi tự chạy tiếp. */
  | { status: "running"; sessionNo: number; waitUntil?: number }
  | { status: "stopped"; reason: StopReason };

export interface LogLine {
  seq: number;
  line: string;
  stream: "stdout" | "stderr";
}

const MAX_LOGS = 2000;
/** Khớp MAX_RECENT bên Rust app_config.rs. */
const MAX_RECENT = 10;
let logSeq = 0;

const IDLE: SessionState = { status: "idle" };
const NO_LOGS: LogLine[] = [];

/** Bản sao `AppConfig::touch_recent` của Rust: root mới lên đầu, khử trùng, cắt còn 10. */
export function touchRecent(recent: string[], root: string): string[] {
  return [root, ...recent.filter((item) => item !== root)].slice(0, MAX_RECENT);
}

/** Chương chưa có bản dịch mà phiên sẽ không tự quay lại — người dùng phải xem (dịch lại, sửa, xoá). */
export interface StoryIssues {
  skipped: number;
  error: number;
}

/** Ghi số chương bỏ qua/lỗi của một truyện; không đổi thì trả đúng object cũ để selector không render lại. */
function withIssues(issues: Record<string, StoryIssues>, key: string, skipped: number, error: number) {
  const current = issues[key];
  if (current ? current.skipped === skipped && current.error === error : skipped === 0 && error === 0) return issues;
  return { ...issues, [key]: { skipped, error } };
}

/** Chỗ đang xem ở trang Dịch của một truyện: đổi truyện qua lại thì về đúng chương, bộ lọc, ô tìm cũ. */
interface StoryView {
  selectedId?: string;
  statusFilter: ChapterFilter;
  searchQuery: string;
}

/** Phiên/tiến độ/log tách theo truyện, khoá bằng `pathKey(root)`; `roots` giữ root đúng chữ để hiện. */
interface PerStory {
  sessions: Record<string, SessionState>;
  progress: Record<string, Progress>;
  logs: Record<string, LogLine[]>;
  roots: Record<string, string>;
  /** Tên truyện (story.name) ghi lúc mở, để dock/dialog gọi tên truyện đang dịch dù đã đóng nó. */
  names: Record<string, string>;
  /** Số chương bỏ qua/lỗi biết được gần nhất (snapshot lúc mở/nạp lại, event progress của phiên) — thẻ truyện
   * ở cột phải dùng để nhắc, kể cả với truyện đang không mở. */
  issues: Record<string, StoryIssues>;
}

interface StoryState extends PerStory {
  /** `converter`: công cụ chuyển phồn → giản, không gắn truyện; `toolFrom` là màn để quay lại. */
  screen: "picker" | "workbench" | "converter";
  toolFrom: "picker" | "workbench";
  page: Page;
  /** Root mọi truyện đã mở trong phiên app này theo thứ tự mở lần đầu (sidebar phải). Không lưu đĩa. */
  opened: string[];
  root?: string;
  snapshot?: StorySnapshot;
  selectedId?: string;
  statusFilter: ChapterFilter;
  searchQuery: string;
  /** Chỗ đang xem của từng truyện đã rời trong phiên app này, khoá `pathKey(root)`. Không lưu đĩa. */
  views: Record<string, StoryView>;
  agy?: AgyStatus;
  config?: AppConfig;
  /** Tăng mỗi lần dialog Bản mặc định lưu/xoá — hook defaults nạp lại prompt/rule mặc định. */
  baseVersion: number;
  bumpBaseVersion: () => void;
  openStory: (snapshot: StorySnapshot) => void;
  /** Chuyển nhanh sang truyện khác từ dock/dialog: như openStory nhưng giữ trang đang xem. */
  switchStory: (snapshot: StorySnapshot) => void;
  closeStory: () => void;
  openConverter: () => void;
  closeConverter: () => void;
  setPage: (page: Page) => void;
  setSnapshot: (snapshot: StorySnapshot) => void;
  /** Đang nhắc chương lệch tiến độ (banner) thì hỏi lại Rust cho con số mới — gọi sau phiên dừng, dịch lại, xoá, reset.
   * Không nhắc gì thì bỏ qua: các việc đó không làm phát sinh chương lệch, khỏi đọc lại cả out/. */
  refreshProgressDrift: (root: string) => Promise<void>;
  select: (id?: string) => void;
  setStatusFilter: (filter: ChapterFilter) => void;
  setSearchQuery: (query: string) => void;
  applySessionEvent: (root: string, event: SessionEvent) => void;
  clearLogs: (root: string) => void;
  setAgy: (agy: AgyStatus) => void;
  setConfig: (config: AppConfig) => void;
}

/** Reducer thuần: event của một truyện → thay đổi các map theo truyện. */
export function applySessionEventPure(state: PerStory, root: string, event: SessionEvent): Partial<PerStory> {
  const key = pathKey(root);
  const roots = state.roots[key] === root ? state.roots : { ...state.roots, [key]: root };
  switch (event.type) {
    case "started":
      return { roots, sessions: { ...state.sessions, [key]: { status: "running", sessionNo: event.session_no } } };
    case "progress": {
      const { type: _type, ...progress } = event;
      return {
        roots,
        progress: { ...state.progress, [key]: progress },
        issues: withIssues(state.issues, key, progress.skipped, progress.error),
      };
    }
    case "quota_wait": {
      const current = state.sessions[key];
      const sessionNo = current?.status === "running" ? current.sessionNo : 0;
      return { roots, sessions: { ...state.sessions, [key]: { status: "running", sessionNo, waitUntil: event.until } } };
    }
    case "agy_log": {
      logSeq += 1;
      const lines = [...(state.logs[key] ?? NO_LOGS), { seq: logSeq, line: event.line, stream: event.stream }];
      return { roots, logs: { ...state.logs, [key]: lines.length > MAX_LOGS ? lines.slice(lines.length - MAX_LOGS) : lines } };
    }
    case "stopped": {
      const { type: _type, ...reason } = event;
      return { roots, sessions: { ...state.sessions, [key]: { status: "stopped", reason } } };
    }
  }
}

/** Phiên của truyện `root` (idle nếu chưa từng chạy). */
export function sessionOf(state: PerStory, root: string | undefined): SessionState {
  return root ? (state.sessions[pathKey(root)] ?? IDLE) : IDLE;
}

export function isRunning(state: PerStory, root: string | undefined): boolean {
  return sessionOf(state, root).status === "running";
}

/** Root (đúng chữ) của mọi truyện đang chạy, sắp theo tên. Trả mảng mới — dùng trong useMemo, không làm selector. */
export function runningRoots(state: Pick<PerStory, "sessions" | "roots">): string[] {
  return Object.entries(state.sessions)
    .filter(([, session]) => session.status === "running")
    .map(([key]) => state.roots[key] ?? key)
    .sort();
}

export const selectCurrentSession = (s: StoryState): SessionState => sessionOf(s, s.root);
export const selectCurrentRunning = (s: StoryState): boolean => isRunning(s, s.root);
export const selectCurrentProgress = (s: StoryState): Progress | undefined =>
  s.root ? s.progress[pathKey(s.root)] : undefined;
export const selectCurrentLogs = (s: StoryState): LogLine[] => (s.root ? (s.logs[pathKey(s.root)] ?? NO_LOGS) : NO_LOGS);

/** Độ lệch tiến độ chỉ tính lúc mở / quét lại / đồng bộ; snapshot thường (null) của cùng truyện giữ con số đã biết để
 * banner nhắc đồng bộ không tắt chỉ vì người dùng sửa gì đó khác — bấm Dịch lúc đó là dịch lại cả trăm chương. */
function keepProgressDrift(previous: StorySnapshot | undefined, next: StorySnapshot): StorySnapshot {
  if (next.progressDrift != null || !previous?.progressDrift || !samePath(previous.root, next.root)) return next;
  return { ...next, progressDrift: previous.progressDrift };
}

/** Ghi chỗ đang xem của truyện đang mở trước khi rời nó. */
function rememberView(state: StoryState): Record<string, StoryView> {
  if (!state.root) return state.views;
  const view = { selectedId: state.selectedId, statusFilter: state.statusFilter, searchQuery: state.searchQuery };
  return { ...state.views, [pathKey(state.root)]: view };
}

/** Chỗ đã xem của truyện sắp vào; chương đã bị xoá hoặc bộ lọc giờ không còn chương nào thì về mặc định. */
function restoreView(views: Record<string, StoryView>, snapshot: StorySnapshot): StoryView {
  const saved = views[pathKey(snapshot.root)];
  if (!saved || filterChapters(snapshot.chapters, saved.statusFilter, saved.searchQuery).length === 0) {
    return { selectedId: undefined, statusFilter: "all", searchQuery: "" };
  }
  const exists = saved.selectedId !== undefined && snapshot.chapters.some((c) => c.id === saved.selectedId);
  return { ...saved, selectedId: exists ? saved.selectedId : undefined };
}

/** Vào một truyện: `page` là trang sẽ hiện (picker → translate; dock/dialog → giữ trang hiện tại). Quay lại truyện
 * đã xem trong phiên app thì về đúng chương, bộ lọc, ô tìm cũ. */
function enterStory(state: StoryState, snapshot: StorySnapshot, page: Page): Partial<StoryState> {
  const key = pathKey(snapshot.root);
  const views = rememberView(state);
  const current = state.sessions[key] ?? IDLE;
  let sessions = state.sessions;
  if (snapshot.sessionRunning && current.status !== "running") {
    sessions = { ...state.sessions, [key]: { status: "running", sessionNo: 0 } };
  } else if (!snapshot.sessionRunning && current.status === "running") {
    sessions = { ...state.sessions, [key]: IDLE };
  }
  const name = snapshot.story.name.trim();
  return {
    config: state.config && { ...state.config, recent: touchRecent(state.config.recent, snapshot.root) },
    screen: "workbench",
    page,
    root: snapshot.root,
    snapshot,
    ...restoreView(views, snapshot),
    views,
    sessions,
    roots: state.roots[key] === snapshot.root ? state.roots : { ...state.roots, [key]: snapshot.root },
    names: state.names[key] === name ? state.names : { ...state.names, [key]: name },
    issues: withIssues(state.issues, key, snapshot.counts.skipped, snapshot.counts.error),
    // Giữ thứ tự mở lần đầu: bấm qua lại giữa các truyện thì ô trong cột phải không đổi chỗ.
    opened: state.opened.some((item) => samePath(item, snapshot.root)) ? state.opened : [...state.opened, snapshot.root],
  };
}

export const useStoryStore = create<StoryState>()((set, get) => ({
  screen: "picker",
  toolFrom: "picker",
  page: "translate",
  statusFilter: "all",
  searchQuery: "",
  views: {},
  sessions: {},
  progress: {},
  logs: {},
  roots: {},
  names: {},
  issues: {},
  opened: [],
  baseVersion: 0,
  bumpBaseVersion: () => set((state) => ({ baseVersion: state.baseVersion + 1 })),
  // Rust open_story đã touch_recent và ghi đĩa; store phải làm y hệt, nếu không picker hiện danh sách cũ
  // và lần appConfigSet kế tiếp (đổi theme, settings…) đẩy recent cũ đè lên đĩa, mất truyện vừa mở.
  // Phiên/tiến độ/log của truyện khác giữ nguyên — nhiều truyện chạy song song.
  openStory: (snapshot) => set((state) => enterStory(state, snapshot, "translate")),
  switchStory: (snapshot) => set((state) => enterStory(state, snapshot, state.page)),
  closeStory: () =>
    set((state) => ({ screen: "picker", root: undefined, snapshot: undefined, selectedId: undefined, views: rememberView(state) })),
  openConverter: () => set((state) => (state.screen === "converter" ? {} : { screen: "converter", toolFrom: state.screen })),
  // Về màn trước; truyện đã đóng trong lúc đó (không còn root) thì về màn chọn truyện.
  closeConverter: () => set((state) => ({ screen: state.toolFrom === "workbench" && state.root ? "workbench" : "picker" })),
  setPage: (page) => set({ page }),
  setSnapshot: (snapshot) =>
    set((state) => ({
      snapshot: keepProgressDrift(state.snapshot, snapshot),
      issues: withIssues(state.issues, pathKey(snapshot.root), snapshot.counts.skipped, snapshot.counts.error),
    })),
  refreshProgressDrift: async (root) => {
    const current = get().snapshot;
    if (!current || !samePath(current.root, root) || !current.progressDrift?.untracked) return;
    try {
      const drift = await progressDrift(root);
      const latest = get().snapshot;
      if (!latest || !samePath(latest.root, root)) return;
      set({ snapshot: { ...latest, progressDrift: { untracked: drift.untracked.length, missing: drift.missing.length } } });
    } catch {
      // Giữ con số cũ: dialog Đồng bộ luôn đối chiếu lại khi mở.
    }
  },
  select: (id) => set({ selectedId: id }),
  setStatusFilter: (statusFilter) => set({ statusFilter }),
  setSearchQuery: (searchQuery) => set({ searchQuery }),
  applySessionEvent: (root, event) => set((state) => applySessionEventPure(state, root, event)),
  clearLogs: (root) => set((state) => ({ logs: { ...state.logs, [pathKey(root)]: NO_LOGS } })),
  setAgy: (agy) => set({ agy }),
  setConfig: (config) => set({ config }),
}));
