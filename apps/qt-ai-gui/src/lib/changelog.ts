// "Có gì mới" trong app: CHANGELOG.md nhúng lúc build (Vite ?raw), không đọc file lúc chạy.
// Parser cùng ngữ nghĩa với scripts/changelog.mjs (script release dùng bản đó) — sửa một bên thì sửa cả hai.
import changelogText from "../../CHANGELOG.md?raw";

export interface ChangelogEntry {
  /** null = mục "Chưa phát hành". */
  version: string | null;
  date: string | null;
  items: string[];
}

export interface ReleasedEntry extends ChangelogEntry {
  version: string;
  date: string;
}

const RELEASED_HEADING = /^## (\d+\.\d+\.\d+) — (\d{4}-\d{2}-\d{2})\s*$/;
const UNRELEASED_HEADING = /^## Chưa phát hành\s*$/;
const BULLET = /^- (.+)$/;

export function parseChangelog(text: string): ChangelogEntry[] {
  const entries: ChangelogEntry[] = [];
  let current: ChangelogEntry | null = null;
  for (const line of text.split(/\r?\n/)) {
    const released = RELEASED_HEADING.exec(line);
    if (released) {
      current = { version: released[1], date: released[2], items: [] };
      entries.push(current);
    } else if (UNRELEASED_HEADING.test(line)) {
      current = { version: null, date: null, items: [] };
      entries.push(current);
    } else if (line.startsWith("## ")) {
      current = null;
    } else if (current) {
      const bullet = BULLET.exec(line.trimEnd());
      if (bullet) current.items.push(bullet[1]);
    }
  }
  return entries;
}

export const CHANGELOG_ENTRIES: readonly ChangelogEntry[] = parseChangelog(changelogText);

/** So hai version x.y.z theo số: >0 nếu a mới hơn b. */
export function compareVersions(a: string, b: string): number {
  const pa = a.split(".").map(Number);
  const pb = b.split(".").map(Number);
  for (let i = 0; i < 3; i++) {
    const diff = (pa[i] || 0) - (pb[i] || 0);
    if (diff !== 0) return diff;
  }
  return 0;
}

function isReleased(entry: ChangelogEntry): entry is ReleasedEntry {
  return entry.version !== null && entry.date !== null;
}

/** Các bản đã phát hành tới bản đang dùng (bỏ "Chưa phát hành" và bản mới hơn), mới nhất trước. */
export function releasedEntries(currentVersion: string, entries: readonly ChangelogEntry[] = CHANGELOG_ENTRIES): ReleasedEntry[] {
  return entries
    .filter(isReleased)
    .filter((entry) => compareVersions(entry.version, currentVersion) <= 0)
    .sort((a, b) => compareVersions(b.version, a.version));
}

export function entryFor(version: string, entries: readonly ChangelogEntry[] = CHANGELOG_ENTRIES): ReleasedEntry | undefined {
  return entries.filter(isReleased).find((entry) => entry.version === version);
}

/**
 * Các bản vừa có thêm khi cập nhật từ `since` lên `current` (since < v ≤ current), mới nhất trước.
 * `since` null = không rõ bản trước (người dùng cũ chưa từng lưu version) → chỉ bản đang dùng.
 */
export function entriesSince(current: string, since: string | null, entries: readonly ChangelogEntry[] = CHANGELOG_ENTRIES): ReleasedEntry[] {
  if (since === null) {
    const entry = entryFor(current, entries);
    return entry ? [entry] : [];
  }
  return releasedEntries(current, entries).filter((entry) => compareVersions(entry.version, since) > 0);
}

export interface InlineToken {
  kind: "text" | "bold" | "code";
  text: string;
}

const INLINE = /\*\*(.+?)\*\*|`([^`]+)`/g;

/** Tách bullet thành chữ thường / **đậm** / `code` để dựng phần tử React (không dùng HTML thô). */
export function inlineTokens(text: string): InlineToken[] {
  const tokens: InlineToken[] = [];
  let last = 0;
  for (const match of text.matchAll(INLINE)) {
    if (match.index > last) tokens.push({ kind: "text", text: text.slice(last, match.index) });
    tokens.push(match[1] !== undefined ? { kind: "bold", text: match[1] } : { kind: "code", text: match[2] });
    last = match.index + match[0].length;
  }
  if (last < text.length) tokens.push({ kind: "text", text: text.slice(last) });
  return tokens;
}
