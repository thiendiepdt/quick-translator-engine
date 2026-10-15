// Đổi version app ở 4 file (package.json, package-lock.json, tauri.conf.json, Cargo.toml), cập nhật Cargo.lock,
// rồi chốt CHANGELOG.md cho bản đó ("## Chưa phát hành" → "## x.y.z — ngày", hoặc nháp từ commit).
// Dùng: node scripts/set-version.mjs 0.2.0 — sửa câu chữ CHANGELOG.md, commit "release(qt-ai-gui): 0.2.0", tag qt-ai-gui-v0.2.0, push.
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { draftFromCommits, promoteUnreleased, sectionBody } from "./changelog.mjs";

export function isValidVersion(text) {
  return /^\d+\.\d+\.\d+$/.test(text);
}

/** Thay đúng một chỗ khớp `pattern` bằng `replacement`, giữ nguyên phần còn lại của file (không parse/serialize lại để không phá format). */
function replaceOnce(path, pattern, replacement, what) {
  const text = readFileSync(path, "utf8");
  if (!pattern.test(text)) throw new Error(`Không thấy ${what} trong ${path}`);
  writeFileSync(path, text.replace(pattern, replacement));
  return path;
}

/** Ghi version vào 4 file dưới appDir, trả danh sách file đã ghi. */
export function applyVersion(appDir, version) {
  if (!isValidVersion(version)) throw new Error(`Version phải dạng x.y.z, nhận "${version}"`);
  const topLevel = /^  "version": "[^"]*"/m; // JSON thụt 2 khoảng: chỉ khớp khoá version cấp gốc
  const written = [];
  written.push(replaceOnce(join(appDir, "package.json"), topLevel, `  "version": "${version}"`, "version cấp gốc"));
  const lockPath = join(appDir, "package-lock.json");
  written.push(replaceOnce(lockPath, topLevel, `  "version": "${version}"`, "version cấp gốc"));
  replaceOnce(
    lockPath,
    // npm trên Windows có thể ghi lock CRLF → chấp nhận cả \r\n.
    /("": \{\r?\n\s+"name": "[^"]*",\r?\n\s+"version": )"[^"]*"/,
    `$1"${version}"`,
    'version của packages[""]',
  );
  written.push(replaceOnce(join(appDir, "src-tauri", "tauri.conf.json"), topLevel, `  "version": "${version}"`, "version cấp gốc"));
  written.push(
    replaceOnce(join(appDir, "src-tauri", "Cargo.toml"), /^version = "[^"]*"/m, `version = "${version}"`, "dòng version"),
  );
  return written;
}

/** Ngày theo giờ máy, dạng YYYY-MM-DD (toISOString là UTC — sáng sớm ở VN sẽ ra ngày hôm trước). */
export function localDate(now = new Date()) {
  const pad = (n) => String(n).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}`;
}

/**
 * Dựng CHANGELOG.md đã chốt cho `version` (chưa ghi): đọc file, đổi "Chưa phát hành"/chèn nháp từ `subjects`.
 * Thiếu file hoặc mục của bản đó rỗng thì ném lỗi — gọi trước khi ghi file version để không để lại nửa vời.
 */
export function planChangelog(appDir, version, subjects, now = new Date()) {
  const path = join(appDir, "CHANGELOG.md");
  const text = readFileSync(path, "utf8");
  const next = promoteUnreleased(text, version, localDate(now), draftFromCommits(subjects));
  if (!sectionBody(next, version)) throw new Error(`CHANGELOG.md có mục ${version} nhưng rỗng — thêm ghi chú rồi chạy lại`);
  return { path, text: next, changed: next !== text };
}

/** Đổi version ở 4 file và chốt CHANGELOG.md; changelog được kiểm tra trước khi ghi bất kỳ file nào. */
export function releaseVersion(appDir, version, subjects, now = new Date()) {
  if (!isValidVersion(version)) throw new Error(`Version phải dạng x.y.z, nhận "${version}"`);
  const changelog = planChangelog(appDir, version, subjects, now);
  const written = applyVersion(appDir, version);
  if (changelog.changed) {
    writeFileSync(changelog.path, changelog.text);
    written.push(changelog.path);
  }
  return { written, changelogChanged: changelog.changed };
}

/**
 * Subject commit từ tag qt-ai-gui-v* gần nhất tới HEAD, chỉ commit đụng app hoặc qt-ai-core (sidecar dịch);
 * không có tag hoặc git lỗi thì rỗng (nháp dùng câu mặc định). `repoRoot` là gốc repo: pathspec tính từ cwd.
 */
function commitSubjectsSinceLastTag(repoRoot) {
  try {
    const git = (args) => execFileSync("git", args, { cwd: repoRoot, encoding: "utf8" });
    const tag = git(["describe", "--tags", "--abbrev=0", "--match", "qt-ai-gui-v*"]).trim();
    const log = git(["log", "--format=%s", `${tag}..HEAD`, "--", "apps/qt-ai-gui", "crates/qt-ai-core"]);
    return log.split(/\r?\n/).filter(Boolean);
  } catch {
    return [];
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const version = process.argv[2];
  if (!version || !isValidVersion(version)) {
    console.error("Dùng: node scripts/set-version.mjs <x.y.z>");
    process.exit(1);
  }
  const app = resolve(dirname(fileURLToPath(import.meta.url)), "..");
  const repoRoot = resolve(app, "../..");
  const { written, changelogChanged } = releaseVersion(app, version, commitSubjectsSinceLastTag(repoRoot));
  for (const file of written) console.log(`Đã ghi ${file}`);
  execFileSync("cargo", ["update", "-p", "qt-ai-gui", "--offline"], { cwd: repoRoot, stdio: "inherit" });
  console.log(
    changelogChanged ? "Đã ghi CHANGELOG.md — kiểm tra lại câu chữ trước khi commit" : `CHANGELOG.md đã có mục ${version}, giữ nguyên`,
  );
  console.log(
    `Xong: ${version}. Tiếp: sửa CHANGELOG.md nếu cần, rồi git commit -am "release(qt-ai-gui): ${version}" (gồm CHANGELOG.md) && git tag qt-ai-gui-v${version} && git push origin HEAD --tags`,
  );
}
