import { pathKey, samePath } from "@/lib/paths";
import type { Progress } from "@/lib/types";
import { runningRoots, type SessionState } from "@/store/story";

export interface DockEntry {
  root: string;
  name: string;
  current: boolean;
  running: boolean;
  /** Tiến độ live (0–100) khi đang dịch; undefined nếu chưa có event progress. */
  percent?: number;
  currentChapter?: string | null;
}

/** Hai chữ cái đầu: hai từ đầu của tên truyện, không có tên thì lấy từ tên folder cuối. */
export function initials(name: string, root: string): string {
  const source = name.trim() || root.replace(/[\\/]+$/, "").split(/[\\/]/).pop() || "?";
  const words = source.split(/[\s\-_]+/).filter(Boolean);
  const first = words[0] ?? source;
  const second = words[1];
  const letters = second ? `${first.charAt(0)}${second.charAt(0)}` : first.slice(0, 2);
  return letters.toUpperCase();
}

export function progressPercent(progress: Progress | undefined): number | undefined {
  if (!progress) return undefined;
  const total = progress.done + progress.queued + progress.translating + progress.error + progress.skipped;
  return total > 0 ? Math.round((progress.done / total) * 100) : 0;
}

/** Truyện đang mở trước, rồi các truyện đang dịch khác theo tên. */
export function dockEntries(state: {
  root?: string;
  sessions: Record<string, SessionState>;
  roots: Record<string, string>;
  names: Record<string, string>;
  progress: Record<string, Progress>;
}): DockEntry[] {
  const running = runningRoots(state);
  const entry = (root: string, current: boolean): DockEntry => {
    const key = pathKey(root);
    const isRunning = running.some((r) => samePath(r, root));
    const progress = state.progress[key];
    return {
      root,
      name: state.names[key] ?? "",
      current,
      running: isRunning,
      percent: isRunning ? progressPercent(progress) : undefined,
      currentChapter: isRunning ? (progress?.current ?? null) : undefined,
    };
  };
  const list: DockEntry[] = state.root ? [entry(state.root, true)] : [];
  const others = running
    .filter((root) => !state.root || !samePath(root, state.root))
    .map((root) => entry(root, false))
    .sort((a, b) => (a.name || a.root).localeCompare(b.name || b.root, "vi"));
  return [...list, ...others];
}
