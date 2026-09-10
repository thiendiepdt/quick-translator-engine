// Bản portable: build release không installer rồi gom app exe + sidecar qt-ai vào dist-portable/<tên>/
// và nén zip (Windows dùng Compress-Archive). Config vẫn ở %APPDATA%\com.vn-converter.qt-ai-gui như bản
// cài đặt — hai bản dùng chung cấu hình. Máy đích vẫn cần WebView2 (Windows 10/11 cập nhật có sẵn).
import { execFileSync } from "node:child_process";
import { copyFileSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const app = resolve(here, "..");
const workspace = resolve(app, "../..");
const ext = process.platform === "win32" ? ".exe" : "";
const version = JSON.parse(readFileSync(join(app, "package.json"), "utf8")).version;

// `tauri build --no-bundle` chạy beforeBuildCommand (build:sidecar + vite build) rồi cargo release, bỏ NSIS/MSI.
execFileSync("npx", ["tauri", "build", "--no-bundle"], { cwd: app, stdio: "inherit", shell: process.platform === "win32" });

const release = join(workspace, "target", "release");
const files = [`VNCVT-AI-Translator${ext}`, `qt-ai${ext}`];
for (const file of files) {
  if (!existsSync(join(release, file))) throw new Error(`Thiếu ${join(release, file)} — build chưa xong?`);
}

const name = `VNCVT-AI-Translator-${version}-portable`;
const outRoot = join(app, "dist-portable");
const outDir = join(outRoot, name);
rmSync(outDir, { recursive: true, force: true });
mkdirSync(outDir, { recursive: true });
for (const file of files) copyFileSync(join(release, file), join(outDir, file));
writeFileSync(
  join(outDir, "README.txt"),
  [
    "VNCVT AI Translator — bản portable",
    "",
    "Chạy VNCVT-AI-Translator.exe. Không cần cài đặt; cấu hình lưu ở %APPDATA%\\com.vn-converter.qt-ai-gui\\config.json.",
    "Cần WebView2 (Windows 10/11 cập nhật đã có sẵn; thiếu thì tải Evergreen Runtime của Microsoft).",
    "qt-ai.exe là công cụ dòng lệnh app dùng kèm — giữ cạnh VNCVT-AI-Translator.exe.",
    "",
  ].join("\r\n"),
);
console.log(`Portable: ${outDir}`);

if (process.platform === "win32") {
  const zip = join(outRoot, `${name}.zip`);
  rmSync(zip, { force: true });
  execFileSync(
    "powershell",
    ["-NoProfile", "-Command", `Compress-Archive -Path '${outDir}' -DestinationPath '${zip}'`],
    { stdio: "inherit" },
  );
  console.log(`Zip: ${zip}`);
}
