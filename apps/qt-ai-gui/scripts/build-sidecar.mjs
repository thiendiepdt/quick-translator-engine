// Build binary qt-ai (crate qt-ai-core) ở chế độ release rồi copy vào src-tauri/binaries/ theo tên
// Tauri yêu cầu cho externalBin: <name>-<target-triple>[.exe]. Chạy trước `tauri build`.
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const app = resolve(here, "..");
const workspace = resolve(app, "../..");

const hostLine = execFileSync("rustc", ["-vV"], { encoding: "utf8" })
  .split(/\r?\n/)
  .find((l) => l.startsWith("host: "));
if (!hostLine) throw new Error("Không đọc được target triple từ `rustc -vV`");
const triple = hostLine.slice("host: ".length).trim();
const ext = process.platform === "win32" ? ".exe" : "";

execFileSync("cargo", ["build", "-p", "qt-ai-core", "--release", "--bin", "qt-ai"], {
  cwd: workspace,
  stdio: "inherit",
});

const source = join(workspace, "target", "release", `qt-ai${ext}`);
const targetDir = join(app, "src-tauri", "binaries");
mkdirSync(targetDir, { recursive: true });
const target = join(targetDir, `qt-ai-${triple}${ext}`);
copyFileSync(source, target);
console.log(`Sidecar: ${target}`);
