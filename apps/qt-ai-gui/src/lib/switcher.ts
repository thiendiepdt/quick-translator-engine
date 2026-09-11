import { pathKey, samePath } from "@/lib/paths";
import type { Progress, RecentSummary } from "@/lib/types";

export const PAGE_SIZE = 12;

export interface SwitcherCard {
  root: string;
  name: string | null;
  done: number | null;
  total: number | null;
  running: boolean;
  current: boolean;
  /** Folder chưa init → hiện mờ, không chuyển được từ đây (picker lo khởi tạo). */
  openable: boolean;
}

/** Gộp đang dịch → thư viện → gần đây, khử trùng theo đường dẫn; đang dịch và đang mở lên đầu. */
export function mergeCandidates(input: {
  running: string[];
  names: Record<string, string>;
  progress: Record<string, Progress>;
  currentRoot?: string;
  library: RecentSummary[];
  recent: RecentSummary[];
}): SwitcherCard[] {
  const seen = new Set<string>();
  const cards: SwitcherCard[] = [];
  const push = (summary: RecentSummary) => {
    const key = pathKey(summary.root);
    if (seen.has(key)) return;
    seen.add(key);
    const running = input.running.some((r) => samePath(r, summary.root));
    const live = running ? input.progress[key] : undefined;
    cards.push({
      root: summary.root,
      name: summary.name ?? input.names[key] ?? null,
      done: live?.done ?? summary.done,
      total: summary.total,
      running,
      current: input.currentRoot !== undefined && samePath(input.currentRoot, summary.root),
      openable: summary.total !== null || running,
    });
  };
  const known = [...input.library, ...input.recent];
  for (const root of input.running) {
    push(known.find((s) => samePath(s.root, root)) ?? { root, name: null, done: null, total: null });
  }
  for (const summary of known) push(summary);
  cards.sort((a, b) => Number(b.running) - Number(a.running) || Number(b.current) - Number(a.current));
  return cards;
}

/** Lọc theo tên hoặc đường dẫn, không phân biệt hoa thường; trống = giữ nguyên. */
export function filterByQuery<T extends { root: string; name: string | null }>(items: T[], query: string): T[] {
  const q = query.trim().toLowerCase();
  if (!q) return items;
  return items.filter((item) => (item.name ?? "").toLowerCase().includes(q) || item.root.toLowerCase().includes(q));
}

export function filterCards(cards: SwitcherCard[], query: string): SwitcherCard[] {
  return filterByQuery(cards, query);
}
