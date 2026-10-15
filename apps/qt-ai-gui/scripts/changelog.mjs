// Đọc/ghi CHANGELOG.md ("Có gì mới") của qt-ai-gui. Định dạng:
//   # Có gì mới
//   ## Chưa phát hành          ← tuỳ chọn, luôn ở đầu
//   - bullet một dòng
//   ## 1.0.21 — 2026-10-13     ← mới nhất trước
// Bản TS cùng ngữ nghĩa ở src/lib/changelog.ts (app đọc file qua ?raw) — sửa parser thì sửa cả hai.
// CLI: node scripts/changelog.mjs section <x.y.z>        → in phần bullet của bản đó
//      node scripts/changelog.mjs github-output <x.y.z>  → ghi `body` vào $GITHUB_OUTPUT (workflow release)
import { appendFileSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export const UNRELEASED = "Chưa phát hành";
export const FALLBACK_ITEM = "Sửa lỗi nhỏ và cải thiện độ ổn định.";

const RELEASED_HEADING = /^## (\d+\.\d+\.\d+) — (\d{4}-\d{2}-\d{2})\s*$/;
const UNRELEASED_HEADING = new RegExp(`^## ${UNRELEASED}\\s*$`);
const BULLET = /^- (.+)$/;

/** @returns {{ version: string | null, date: string | null, items: string[] }[]} theo thứ tự trong file; version null = Chưa phát hành. */
export function parseChangelog(text) {
  const entries = [];
  let current = null;
  for (const line of text.split(/\r?\n/)) {
    const released = RELEASED_HEADING.exec(line);
    if (released) {
      current = { version: released[1], date: released[2], items: [] };
      entries.push(current);
    } else if (UNRELEASED_HEADING.test(line)) {
      current = { version: null, date: null, items: [] };
      entries.push(current);
    } else if (line.startsWith("## ")) {
      current = null; // tiêu đề lạ: không gộp bullet của nó vào bản trước
    } else if (current) {
      const bullet = BULLET.exec(line.trimEnd());
      if (bullet) current.items.push(bullet[1]);
    }
  }
  return entries;
}

/** Các dòng bullet ("- …") của bản `version`, nối bằng "\n"; không có bản đó thì null. */
export function sectionBody(text, version) {
  const entry = parseChangelog(text).find((e) => e.version === version);
  return entry ? entry.items.map((item) => `- ${item}`).join("\n") : null;
}

/**
 * Chốt changelog cho bản `version` ngày `date`: có "## Chưa phát hành" không rỗng thì đổi tiêu đề của nó; không thì
 * chèn mục mới (bullet nháp, rỗng thì một câu mặc định) ngay sau tiêu đề file. Đã có mục của bản đó thì giữ nguyên.
 */
export function promoteUnreleased(text, version, date, draftItems) {
  if (parseChangelog(text).some((e) => e.version === version)) return text;
  const eol = text.includes("\r\n") ? "\r\n" : "\n";
  const lines = text.split(/\r?\n/);
  const heading = `## ${version} — ${date}`;
  const nextHeading = (from) => {
    const at = lines.findIndex((line, i) => i >= from && line.startsWith("## "));
    return at < 0 ? lines.length : at;
  };

  const unreleased = lines.findIndex((line) => UNRELEASED_HEADING.test(line));
  if (unreleased >= 0) {
    const end = nextHeading(unreleased + 1);
    if (lines.slice(unreleased + 1, end).some((line) => BULLET.test(line))) {
      lines[unreleased] = heading;
      return lines.join(eol);
    }
    lines.splice(unreleased, end - unreleased); // mục rỗng: bỏ hẳn rồi chèn mục mới như không có
  }

  const items = draftItems.length > 0 ? draftItems : [FALLBACK_ITEM];
  const block = [heading, ...items.map((item) => `- ${item}`)];
  const first = nextHeading(0);
  if (first < lines.length) {
    lines.splice(first, 0, ...block, "");
    return lines.join(eol);
  }
  while (lines.length > 0 && lines[lines.length - 1].trim() === "") lines.pop();
  if (lines.length === 0) lines.push("# Có gì mới");
  return [...lines, "", ...block, ""].join(eol);
}

const COMMIT = /^(feat|fix|perf)(\([^)]*\))?!?:\s*(.+)$/;
const ORDER = { feat: 0, perf: 1, fix: 2 };

/** Nháp bullet từ subject commit: chỉ feat/perf/fix, bỏ tiền tố `type(scope): `, viết hoa chữ đầu, kết thúc bằng dấu chấm. */
export function draftFromCommits(subjects) {
  const drafts = [];
  for (const subject of subjects) {
    const match = COMMIT.exec(subject.trim());
    if (!match) continue;
    const body = match[3].trim();
    const capital = body.charAt(0).toLocaleUpperCase("vi") + body.slice(1);
    drafts.push({ order: ORDER[match[1]], text: /[.!?…]$/.test(capital) ? capital : `${capital}.` });
  }
  return drafts.sort((a, b) => a.order - b.order).map((d) => d.text); // sort ổn định: giữ thứ tự commit trong nhóm
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const [command, version] = process.argv.slice(2);
  if (!["section", "github-output"].includes(command) || !version) {
    console.error("Dùng: node scripts/changelog.mjs section|github-output <x.y.z>");
    process.exit(1);
  }
  const path = join(dirname(fileURLToPath(import.meta.url)), "..", "CHANGELOG.md");
  const body = sectionBody(readFileSync(path, "utf8"), version);
  if (!body) {
    console.error(`CHANGELOG.md chưa có mục cho bản ${version} (hoặc mục rỗng) — viết "## ${version} — <ngày>" trước khi phát hành.`);
    process.exit(1);
  }
  if (command === "section") {
    console.log(body);
  } else {
    const output = process.env.GITHUB_OUTPUT;
    if (!output) {
      console.error("Thiếu biến môi trường GITHUB_OUTPUT");
      process.exit(1);
    }
    appendFileSync(output, `body<<__NOTES__\n${body}\n__NOTES__\n`);
    console.log(`Đã ghi release notes của ${version} (${body.split("\n").length} dòng)`);
  }
}
