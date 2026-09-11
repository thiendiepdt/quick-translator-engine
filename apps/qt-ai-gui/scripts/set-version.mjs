// Đổi version app ở 4 file (package.json, package-lock.json, tauri.conf.json, Cargo.toml) rồi cập nhật Cargo.lock.
// Dùng: node scripts/set-version.mjs 0.2.0 — sau đó commit "release(qt-ai-gui): 0.2.0", tag qt-ai-gui-v0.2.0, push.
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

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
    /("": \{\n\s+"name": "[^"]*",\n\s+"version": )"[^"]*"/,
    `$1"${version}"`,
    'version của packages[""]',
  );
  written.push(replaceOnce(join(appDir, "src-tauri", "tauri.conf.json"), topLevel, `  "version": "${version}"`, "version cấp gốc"));
  written.push(
    replaceOnce(join(appDir, "src-tauri", "Cargo.toml"), /^version = "[^"]*"/m, `version = "${version}"`, "dòng version"),
  );
  return written;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  const version = process.argv[2];
  if (!version || !isValidVersion(version)) {
    console.error("Dùng: node scripts/set-version.mjs <x.y.z>");
    process.exit(1);
  }
  const app = resolve(dirname(fileURLToPath(import.meta.url)), "..");
  for (const file of applyVersion(app, version)) console.log(`Đã ghi ${file}`);
  execFileSync("cargo", ["update", "-p", "qt-ai-gui", "--offline"], { cwd: resolve(app, "../.."), stdio: "inherit" });
  console.log(
    `Xong: ${version}. Tiếp: git commit -am "release(qt-ai-gui): ${version}" && git tag qt-ai-gui-v${version} && git push origin HEAD --tags`,
  );
}
