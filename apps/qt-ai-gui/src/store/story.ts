import { create } from "zustand";

import type {
  AgyStatus,
  AppConfig,
  ChapterStatus,
  Progress,
  SessionEvent,
  StopReason,
  StorySnapshot,
} from "@/lib/types";

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
let logSeq = 0;

interface StoryState {
  screen: "picker" | "workbench";
  root?: string;
  snapshot?: StorySnapshot;
  selectedId?: string;
  statusFilter: ChapterStatus | "all";
  session: SessionState;
  progress?: Progress;
  logs: LogLine[];
  agy?: AgyStatus;
  config?: AppConfig;
  openStory: (snapshot: StorySnapshot) => void;
  closeStory: () => void;
  setSnapshot: (snapshot: StorySnapshot) => void;
  select: (id?: string) => void;
  setStatusFilter: (filter: ChapterStatus | "all") => void;
  applySessionEvent: (event: SessionEvent) => void;
  clearLogs: () => void;
  setAgy: (agy: AgyStatus) => void;
  setConfig: (config: AppConfig) => void;
}

/** Reducer thuần: event từ runner → thay đổi state. */
export function applySessionEventPure(
  state: Pick<StoryState, "session" | "progress" | "logs">,
  event: SessionEvent,
): Partial<Pick<StoryState, "session" | "progress" | "logs">> {
  switch (event.type) {
    case "started":
      return { session: { status: "running", sessionNo: event.session_no } };
    case "progress": {
      const { type: _type, ...progress } = event;
      return { progress };
    }
    case "agy_log": {
      logSeq += 1;
      const logs = [...state.logs, { seq: logSeq, line: event.line, stream: event.stream }];
      return { logs: logs.length > MAX_LOGS ? logs.slice(logs.length - MAX_LOGS) : logs };
    }
    case "stopped": {
      const { type: _type, ...reason } = event;
      return { session: { status: "stopped", reason } };
    }
  }
}

export const useStoryStore = create<StoryState>()((set) => ({
  screen: "picker",
  statusFilter: "all",
  session: { status: "idle" },
  logs: [],
  openStory: (snapshot) =>
    set({
      screen: "workbench",
      root: snapshot.root,
      snapshot,
      selectedId: undefined,
      statusFilter: "all",
      progress: undefined,
      logs: [],
      session: snapshot.sessionRunning ? { status: "running", sessionNo: 0 } : { status: "idle" },
    }),
  closeStory: () =>
    set({ screen: "picker", root: undefined, snapshot: undefined, selectedId: undefined, progress: undefined }),
  setSnapshot: (snapshot) => set({ snapshot }),
  select: (id) => set({ selectedId: id }),
  setStatusFilter: (statusFilter) => set({ statusFilter }),
  applySessionEvent: (event) => set((state) => applySessionEventPure(state, event)),
  clearLogs: () => set({ logs: [] }),
  setAgy: (agy) => set({ agy }),
  setConfig: (config) => set({ config }),
}));
