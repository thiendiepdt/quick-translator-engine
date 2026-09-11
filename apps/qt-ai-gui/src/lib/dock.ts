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

/**
 * Sidebar phải: truyện đang dịch trước (theo tên), rồi mọi truyện đã mở trong phiên app theo lần mở
 * gần nhất (`opened` đã sắp mới nhất đứng đầu). Truyện đang dịch mà chưa mở lần nào cũng hiện.
 */
export function dockEntries(state: {
  root?: string;
  opened: string[];
  sessions: Record<string, SessionState>;
  roots: Record<string, string>;
  names: Record<string, string>;
  progress: Record<string, Progress>;
}): DockEntry[] {
  const running = runningRoots(state);
  const entry = (root: string): DockEntry => {
    const key = pathKey(root);
    const isRunning = running.some((r) => samePath(r, root));
    const progress = state.progress[key];
    return {
      root,
      name: state.names[key] ?? "",
      current: state.root !== undefined && samePath(state.root, root),
      running: isRunning,
      percent: isRunning ? progressPercent(progress) : undefined,
      currentChapter: isRunning ? (progress?.current ?? null) : undefined,
    };
  };
  const top = running.map(entry).sort((a, b) => (a.name || a.root).localeCompare(b.name || b.root, "vi"));
  const rest = state.opened.filter((root) => !running.some((r) => samePath(r, root))).map(entry);
  return [...top, ...rest];
}
