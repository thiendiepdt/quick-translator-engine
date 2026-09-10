import { create } from "zustand";

import type { ChapterFilter } from "@/lib/chapters";
import { pathKey, samePath } from "@/lib/paths";
import type { AgyStatus, AppConfig, Progress, SessionEvent, StopReason, StorySnapshot } from "@/lib/types";

export type Page = "translate" | "story" | "export" | "settings";

export type SessionState =
  | { status: "idle" }
  | { status: "running"; sessionNo: number }
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

/** Phiên/tiến độ/log tách theo truyện, khoá bằng `pathKey(root)`; `roots` giữ root đúng chữ để hiện. */
interface PerStory {
  sessions: Record<string, SessionState>;
  progress: Record<string, Progress>;
  logs: Record<string, LogLine[]>;
  roots: Record<string, string>;
  /** Tên truyện (story.name) ghi lúc mở, để dock/dialog gọi tên truyện đang dịch dù đã đóng nó. */
  names: Record<string, string>;
}

interface StoryState extends PerStory {
  screen: "picker" | "workbench";
  page: Page;
  /** Root mọi truyện đã mở trong phiên app này, mới mở nhất đứng đầu (sidebar phải). Không lưu đĩa. */
  opened: string[];
  root?: string;
  snapshot?: StorySnapshot;
  selectedId?: string;
  statusFilter: ChapterFilter;
  searchQuery: string;
  agy?: AgyStatus;
  config?: AppConfig;
  /** Tăng mỗi lần dialog Bản mặc định lưu/xoá — hook defaults nạp lại prompt/rule mặc định. */
  baseVersion: number;
  bumpBaseVersion: () => void;
  openStory: (snapshot: StorySnapshot) => void;
  /** Chuyển nhanh sang truyện khác từ dock/dialog: như openStory nhưng giữ trang đang xem. */
  switchStory: (snapshot: StorySnapshot) => void;
  closeStory: () => void;
  setPage: (page: Page) => void;
  setSnapshot: (snapshot: StorySnapshot) => void;
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
      return { roots, progress: { ...state.progress, [key]: progress } };
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

/** Vào một truyện: `page` là trang sẽ hiện (picker → translate; dock/dialog → giữ trang hiện tại). */
function enterStory(state: StoryState, snapshot: StorySnapshot, page: Page): Partial<StoryState> {
  const key = pathKey(snapshot.root);
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
    selectedId: undefined,
    statusFilter: "all",
    searchQuery: "",
    sessions,
    roots: state.roots[key] === snapshot.root ? state.roots : { ...state.roots, [key]: snapshot.root },
    names: state.names[key] === name ? state.names : { ...state.names, [key]: name },
    opened: [snapshot.root, ...state.opened.filter((item) => !samePath(item, snapshot.root))],
  };
}

export const useStoryStore = create<StoryState>()((set) => ({
  screen: "picker",
  page: "translate",
  statusFilter: "all",
  searchQuery: "",
  sessions: {},
  progress: {},
  logs: {},
  roots: {},
  names: {},
  opened: [],
  baseVersion: 0,
  bumpBaseVersion: () => set((state) => ({ baseVersion: state.baseVersion + 1 })),
  // Rust open_story đã touch_recent và ghi đĩa; store phải làm y hệt, nếu không picker hiện danh sách cũ
  // và lần appConfigSet kế tiếp (đổi theme, settings…) đẩy recent cũ đè lên đĩa, mất truyện vừa mở.
  // Phiên/tiến độ/log của truyện khác giữ nguyên — nhiều truyện chạy song song.
  openStory: (snapshot) => set((state) => enterStory(state, snapshot, "translate")),
  switchStory: (snapshot) => set((state) => enterStory(state, snapshot, state.page)),
  closeStory: () => set({ screen: "picker", root: undefined, snapshot: undefined, selectedId: undefined }),
  setPage: (page) => set({ page }),
  setSnapshot: (snapshot) => set({ snapshot }),
  select: (id) => set({ selectedId: id }),
  setStatusFilter: (statusFilter) => set({ statusFilter }),
  setSearchQuery: (searchQuery) => set({ searchQuery }),
  applySessionEvent: (root, event) => set((state) => applySessionEventPure(state, root, event)),
  clearLogs: (root) => set((state) => ({ logs: { ...state.logs, [pathKey(root)]: NO_LOGS } })),
  setAgy: (agy) => set({ agy }),
  setConfig: (config) => set({ config }),
}));
