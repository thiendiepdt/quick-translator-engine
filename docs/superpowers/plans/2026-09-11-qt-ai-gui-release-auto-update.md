# qt-ai-gui Release + Auto Update Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Tag `qt-ai-gui-v<x.y.z>` sinh GitHub Release Windows (NSIS, MSI, `.sig`, `latest.json`, zip portable); app đã cài tự hỏi và cài bản mới khi mở.

**Architecture:** Bê nguyên mẫu novelkit: `tauri-apps/tauri-action` trong workflow riêng cho app, `tauri-plugin-updater` + `tauri-plugin-process` phía Rust, hook React gọi `check()` → `ask()` → `downloadAndInstall()` → `relaunch()`. Logic hook tách thành hàm thuần nhận deps để test không cần Tauri.

**Tech Stack:** Tauri 2.11, tauri-plugin-updater 2, tauri-plugin-process 2, `@tauri-apps/plugin-updater` ^2.11, `@tauri-apps/plugin-process` ^2.3, GitHub Actions, vitest.

## Global Constraints

- Spec: `docs/superpowers/specs/2026-09-11-qt-ai-gui-release-auto-update-design.md`.
- Chỉ build Windows; tag `qt-ai-gui-v*`; hộp thoại native; portable vẫn lên release và cập nhật bằng NSIS.
- Không ghi IP thật / API key vào repo. Không commit cho tới khi người dùng bảo "commit đi"; khi commit thì tách theo từng task, kết thúc bằng `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>`.
- Private key ký nằm ở `~/.tauri/qt-ai-gui.key`, không bao giờ nằm trong repo.
- Chạy lệnh npm từ `apps/qt-ai-gui`, cargo từ gốc repo.

---

### Task 1: Hàm thuần `runUpdateCheck` + hook `useUpdateCheck`

**Files:**
- Create: `apps/qt-ai-gui/src/hooks/use-update-check.ts`
- Test: `apps/qt-ai-gui/src/hooks/use-update-check.test.ts`
- Modify: `apps/qt-ai-gui/src/app.tsx` (thêm import + gọi hook cạnh `useUndoFallback()`)
- Modify: `apps/qt-ai-gui/package.json` (deps mới)

**Interfaces:**
- Produces: `runUpdateCheck(deps: UpdateCheckDeps): Promise<UpdateCheckResult>` với `UpdateCheckResult = "none" | "declined" | "installed" | "failed"`; `useUpdateCheck(enabled = !import.meta.env.DEV): void`.

- [ ] **Step 1: Cài deps npm**

```bash
cd apps/qt-ai-gui && npm install @tauri-apps/plugin-updater@^2.11.0 @tauri-apps/plugin-process@^2.3.1
```

- [ ] **Step 2: Viết test thất bại**

```ts
// apps/qt-ai-gui/src/hooks/use-update-check.test.ts
import { describe, expect, it, vi } from "vitest";

import { runUpdateCheck, type UpdateCheckDeps } from "./use-update-check";

function deps(overrides: Partial<UpdateCheckDeps> = {}): UpdateCheckDeps & { calls: string[] } {
  const calls: string[] = [];
  const update = {
    version: "0.2.0",
    downloadAndInstall: vi.fn(async () => {
      calls.push("install");
    }),
  };
  return {
    calls,
    check: vi.fn(async () => update),
    ask: vi.fn(async () => {
      calls.push("ask");
      return true;
    }),
    message: vi.fn(async () => {
      calls.push("message");
    }),
    relaunch: vi.fn(async () => {
      calls.push("relaunch");
    }),
    warn: vi.fn(),
    ...overrides,
  };
}

describe("runUpdateCheck", () => {
  it("có bản mới và đồng ý: hỏi → tải → báo → khởi động lại", async () => {
    const d = deps();
    await expect(runUpdateCheck(d)).resolves.toBe("installed");
    expect(d.calls).toEqual(["ask", "install", "message", "relaunch"]);
    expect(d.ask).toHaveBeenCalledWith(expect.stringContaining("0.2.0"), expect.objectContaining({ title: "Cập nhật VNCVT AI Translator" }));
  });

  it("từ chối thì không tải", async () => {
    const d = deps({ ask: vi.fn(async () => false) });
    await expect(runUpdateCheck(d)).resolves.toBe("declined");
    expect(d.calls).toEqual([]);
    expect(d.relaunch).not.toHaveBeenCalled();
  });

  it("không có bản mới thì không hỏi", async () => {
    const d = deps({ check: vi.fn(async () => null) });
    await expect(runUpdateCheck(d)).resolves.toBe("none");
    expect(d.ask).not.toHaveBeenCalled();
  });

  it("check ném lỗi: chỉ warn, không ném ra ngoài", async () => {
    const error = new Error("offline");
    const d = deps({ check: vi.fn(async () => { throw error; }) });
    await expect(runUpdateCheck(d)).resolves.toBe("failed");
    expect(d.warn).toHaveBeenCalledWith("Kiểm tra cập nhật thất bại:", error);
    expect(d.ask).not.toHaveBeenCalled();
  });
});
```

- [ ] **Step 3: Chạy test, phải fail**

Run: `cd apps/qt-ai-gui && npx vitest run src/hooks/use-update-check.test.ts`
Expected: FAIL, không resolve được module `./use-update-check`.

- [ ] **Step 4: Viết hook**

```ts
// apps/qt-ai-gui/src/hooks/use-update-check.ts
import { useEffect } from "react";

/** Phần của Update (plugin-updater) mà luồng cập nhật cần — thu hẹp để test giả được. */
export interface UpdateHandle {
  version: string;
  downloadAndInstall(): Promise<void>;
}

export interface UpdateCheckDeps {
  check(): Promise<UpdateHandle | null>;
  ask(text: string, options: { title: string; kind: "info" }): Promise<boolean>;
  message(text: string, options: { title: string }): Promise<void>;
  relaunch(): Promise<void>;
  warn(...args: unknown[]): void;
}

export type UpdateCheckResult = "none" | "declined" | "installed" | "failed";

export const UPDATE_TITLE = "Cập nhật VNCVT AI Translator";

/**
 * Luồng cập nhật y hệt novelkit: dò → hỏi hộp thoại native → tải & cài → báo → khởi động lại.
 * Lỗi ở bất kỳ bước nào (offline, endpoint chưa có release, chữ ký sai) chỉ warn, không chặn app.
 */
export async function runUpdateCheck(deps: UpdateCheckDeps): Promise<UpdateCheckResult> {
  try {
    const update = await deps.check();
    if (!update) return "none";
    const yes = await deps.ask(`Phiên bản mới ${update.version} đã có! Bạn có muốn cập nhật ngay không?`, {
      title: UPDATE_TITLE,
      kind: "info",
    });
    if (!yes) return "declined";
    await update.downloadAndInstall();
    await deps.message("Cập nhật hoàn tất. Ứng dụng sẽ khởi động lại.", { title: "Cập nhật thành công" });
    await deps.relaunch();
    return "installed";
  } catch (error) {
    deps.warn("Kiểm tra cập nhật thất bại:", error);
    return "failed";
  }
}

/** Deps thật, import động để vitest/jsdom không kéo `@tauri-apps/*` khi hook bị tắt. */
async function tauriDeps(): Promise<UpdateCheckDeps> {
  const [{ check }, { ask, message }, { relaunch }] = await Promise.all([
    import("@tauri-apps/plugin-updater"),
    import("@tauri-apps/plugin-dialog"),
    import("@tauri-apps/plugin-process"),
  ]);
  return { check, ask, message, relaunch, warn: console.warn };
}

/** Dò bản mới một lần khi app mở. Tắt ở dev server: không có bản cài nào để cập nhật. */
export function useUpdateCheck(enabled: boolean = !import.meta.env.DEV): void {
  useEffect(() => {
    if (!enabled) return;
    void tauriDeps().then(runUpdateCheck);
  }, [enabled]);
}
```

- [ ] **Step 5: Gọi hook trong `app.tsx`**

Thêm `import { useUpdateCheck } from "@/hooks/use-update-check";` cạnh import `useUndoFallback`, và `useUpdateCheck();` ngay sau dòng `useUndoFallback();`.

- [ ] **Step 6: Chạy test, typecheck, lint**

Run: `cd apps/qt-ai-gui && npx vitest run src/hooks/use-update-check.test.ts && npm run typecheck && npm run lint`
Expected: 4 PASS, không lỗi.

### Task 2: Phía Rust và cấu hình Tauri

**Files:**
- Modify: `apps/qt-ai-gui/src-tauri/Cargo.toml` (deps)
- Modify: `apps/qt-ai-gui/src-tauri/src/lib.rs:66-82` (builder + setup)
- Modify: `apps/qt-ai-gui/src-tauri/capabilities/main.json`
- Modify: `apps/qt-ai-gui/src-tauri/tauri.conf.json`

- [ ] **Step 1: Sinh khoá ký (một lần)**

```bash
mkdir -p ~/.tauri && cd apps/qt-ai-gui && npx tauri signer generate -w ~/.tauri/qt-ai-gui.key --ci
```

Lấy nội dung `~/.tauri/qt-ai-gui.key.pub` làm `pubkey`. `--ci` sinh key không mật khẩu; `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` để rỗng.

- [ ] **Step 2: Cargo.toml**

Thêm dưới `tauri-plugin-dialog = "2"`:

```toml
tauri-plugin-process = "2"
tauri-plugin-updater = "2"
```

- [ ] **Step 3: lib.rs**

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_process::init())
    .setup(|app| {
        // Updater chỉ có trên desktop; đăng ký trong setup như novelkit nên mock_builder trong test không cần.
        #[cfg(desktop)]
        app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;
        let config_path = …
```

- [ ] **Step 4: capabilities/main.json**

```json
"permissions": ["core:default", "core:event:default", "dialog:allow-open", "dialog:allow-save", "dialog:allow-ask", "dialog:allow-message", "updater:default", "process:allow-restart"]
```

- [ ] **Step 5: tauri.conf.json**

Thêm `"createUpdaterArtifacts": true` vào `bundle` và khối:

```json
"plugins": {
  "updater": {
    "endpoints": ["https://github.com/thiendiepdt/quick-translator-engine/releases/latest/download/latest.json"],
    "pubkey": "<nội dung ~/.tauri/qt-ai-gui.key.pub>"
  }
}
```

- [ ] **Step 6: Kiểm tra**

Run: `cargo check -p qt-ai-gui && cargo test -p qt-ai-gui`
Expected: build qua, 30 test qua (lib.rs test `lenh_async_tra_loi_qua_ipc` vẫn xanh).

### Task 3: Script bump version + portable bỏ qua build

**Files:**
- Create: `apps/qt-ai-gui/scripts/set-version.mjs`
- Test: `apps/qt-ai-gui/scripts/set-version.test.mjs`
- Modify: `apps/qt-ai-gui/scripts/build-portable.mjs`
- Modify: `apps/qt-ai-gui/package.json` (script `"set-version": "node scripts/set-version.mjs"`)

**Interfaces:**
- Produces: `applyVersion(appDir: string, version: string): string[]` (trả danh sách file đã ghi, ném `Error` nếu version sai định dạng); `isValidVersion(text): boolean`.

- [ ] **Step 1: Test thất bại**

```js
// apps/qt-ai-gui/scripts/set-version.test.mjs
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import { applyVersion, isValidVersion } from "./set-version.mjs";

function fixture() {
  const dir = mkdtempSync(join(tmpdir(), "set-version-"));
  mkdirSync(join(dir, "src-tauri"), { recursive: true });
  writeFileSync(join(dir, "package.json"), JSON.stringify({ name: "qt-ai-gui", version: "0.1.0", scripts: { dev: "vite" } }, null, 2) + "\n");
  writeFileSync(
    join(dir, "package-lock.json"),
    JSON.stringify({ name: "qt-ai-gui", version: "0.1.0", lockfileVersion: 3, packages: { "": { name: "qt-ai-gui", version: "0.1.0" }, "node_modules/x": { version: "4.5.0" } } }, null, 2) + "\n",
  );
  writeFileSync(join(dir, "src-tauri", "tauri.conf.json"), JSON.stringify({ productName: "VNCVT AI Translator", version: "0.1.0" }, null, 2) + "\n");
  writeFileSync(join(dir, "src-tauri", "Cargo.toml"), '[package]\nname = "qt-ai-gui"\nversion = "0.1.0"\n\n[dependencies]\ntauri = { version = "2", features = [] }\n');
  return dir;
}

describe("applyVersion", () => {
  it("đổi version ở 4 file, giữ nguyên phần còn lại", () => {
    const dir = fixture();
    const written = applyVersion(dir, "0.2.0");
    expect(written).toHaveLength(4);
    expect(JSON.parse(readFileSync(join(dir, "package.json"), "utf8"))).toEqual({ name: "qt-ai-gui", version: "0.2.0", scripts: { dev: "vite" } });
    const lock = JSON.parse(readFileSync(join(dir, "package-lock.json"), "utf8"));
    expect(lock.version).toBe("0.2.0");
    expect(lock.packages[""].version).toBe("0.2.0");
    expect(lock.packages["node_modules/x"].version).toBe("4.5.0");
    expect(JSON.parse(readFileSync(join(dir, "src-tauri", "tauri.conf.json"), "utf8")).version).toBe("0.2.0");
    expect(readFileSync(join(dir, "src-tauri", "Cargo.toml"), "utf8")).toBe('[package]\nname = "qt-ai-gui"\nversion = "0.2.0"\n\n[dependencies]\ntauri = { version = "2", features = [] }\n');
  });

  it("từ chối version sai định dạng", () => {
    expect(isValidVersion("1.2")).toBe(false);
    expect(isValidVersion("v1.2.3")).toBe(false);
    expect(isValidVersion("1.2.3")).toBe(true);
    expect(() => applyVersion(fixture(), "abc")).toThrow(/x\.y\.z/);
  });
});
```

- [ ] **Step 2: Chạy, phải fail**

Run: `cd apps/qt-ai-gui && npx vitest run scripts/set-version.test.mjs`
Expected: FAIL, module không tồn tại.

- [ ] **Step 3: Viết script**

```js
// apps/qt-ai-gui/scripts/set-version.mjs
// Đổi version app ở 4 file (package.json, package-lock.json, tauri.conf.json, Cargo.toml) rồi cập nhật Cargo.lock.
// Dùng: node scripts/set-version.mjs 0.2.0 — sau đó commit "release(qt-ai-gui): 0.2.0", tag qt-ai-gui-v0.2.0, push.
import { execFileSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

export function isValidVersion(text) {
  return /^\d+\.\d+\.\d+$/.test(text);
}

function editJson(path, edit) {
  const data = JSON.parse(readFileSync(path, "utf8"));
  edit(data);
  writeFileSync(path, JSON.stringify(data, null, 2) + "\n");
  return path;
}

/** Ghi version vào 4 file dưới appDir, trả danh sách file đã ghi. */
export function applyVersion(appDir, version) {
  if (!isValidVersion(version)) throw new Error(`Version phải dạng x.y.z, nhận "${version}"`);
  const written = [];
  written.push(editJson(join(appDir, "package.json"), (d) => { d.version = version; }));
  written.push(editJson(join(appDir, "package-lock.json"), (d) => {
    d.version = version;
    if (d.packages?.[""]) d.packages[""].version = version;
  }));
  written.push(editJson(join(appDir, "src-tauri", "tauri.conf.json"), (d) => { d.version = version; }));
  const cargoPath = join(appDir, "src-tauri", "Cargo.toml");
  const cargo = readFileSync(cargoPath, "utf8");
  const next = cargo.replace(/^version = "[^"]*"/m, `version = "${version}"`);
  if (next === cargo) throw new Error(`Không thấy dòng version trong ${cargoPath}`);
  writeFileSync(cargoPath, next);
  written.push(cargoPath);
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
  console.log(`Xong: ${version}. Tiếp: git commit -am "release(qt-ai-gui): ${version}" && git tag qt-ai-gui-v${version} && git push origin HEAD --tags`);
}
```

- [ ] **Step 4: build-portable.mjs bỏ qua build khi `PORTABLE_SKIP_BUILD`**

Thay dòng `execFileSync("npx", ["tauri", "build", "--no-bundle"], …)` bằng:

```js
// CI đã chạy tauri-action nên chỉ gom binary sẵn có; local vẫn build.
if (process.env.PORTABLE_SKIP_BUILD) {
  console.log("PORTABLE_SKIP_BUILD: dùng binary có sẵn trong target/release");
} else {
  execFileSync("npx", ["tauri", "build", "--no-bundle"], { cwd: app, stdio: "inherit", shell: process.platform === "win32" });
}
```

Thêm vào README.txt của portable, sau dòng qt-ai.exe:

```
"Cập nhật tự động: khi có bản mới, app hỏi rồi chạy trình cài đặt — máy sẽ thành bản cài đặt (Start Menu, gỡ ở Settings).",
"Muốn giữ portable thì chọn Không và tự tải zip mới ở https://github.com/thiendiepdt/quick-translator-engine/releases.",
```

- [ ] **Step 5: package.json** thêm `"set-version": "node scripts/set-version.mjs"` vào scripts.

- [ ] **Step 6: Chạy test + lint**

Run: `cd apps/qt-ai-gui && npx vitest run scripts/set-version.test.mjs && npm run lint`
Expected: 2 PASS, lint sạch (nếu eslint không cover `.mjs` thì bỏ qua).

### Task 4: Workflow GitHub Actions + README

**Files:**
- Create: `.github/workflows/release-qt-ai-gui.yml`
- Modify: `apps/qt-ai-gui/README.md` (mục Build → thêm mục "Phát hành và cập nhật")

- [ ] **Step 1: Workflow**

```yaml
name: Release qt-ai-gui

on:
  push:
    tags: ['qt-ai-gui-v*']

jobs:
  release:
    runs-on: windows-latest
    permissions:
      contents: write
    defaults:
      run:
        working-directory: apps/qt-ai-gui
    steps:
      - uses: actions/checkout@v6

      - name: Setup Node.js
        uses: actions/setup-node@v6
        with:
          node-version: 20
          cache: npm
          cache-dependency-path: apps/qt-ai-gui/package-lock.json

      - name: Install Rust stable
        uses: dtolnay/rust-toolchain@stable

      - name: Rust cache
        uses: swatinem/rust-cache@v2
        with:
          workspaces: '. -> target'

      - name: Install frontend dependencies
        run: npm ci

      # beforeBuildCommand của app đã build sidecar qt-ai.exe + vite; tauri-action tạo release + latest.json.
      - name: Build and release Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          projectPath: apps/qt-ai-gui
          tagName: qt-ai-gui-v__VERSION__
          releaseName: 'VNCVT AI Translator v__VERSION__'
          releaseBody: 'Xem changelog bên dưới.'
          releaseDraft: false
          prerelease: false
          updaterJsonPreferNsis: true

      - name: Create portable zip
        shell: pwsh
        env:
          PORTABLE_SKIP_BUILD: '1'
        run: npm run build:portable

      - name: Upload portable zip to release
        shell: pwsh
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: |
          $version = (Get-Content src-tauri/tauri.conf.json | ConvertFrom-Json).version
          gh release upload "qt-ai-gui-v${version}" "dist-portable/VNCVT-AI-Translator-${version}-portable.zip" --clobber
```

- [ ] **Step 2: README** — thêm sau mục `npm run build:portable`:

```markdown
## Phát hành và cập nhật

Tag `qt-ai-gui-v<x.y.z>` kích hoạt `.github/workflows/release-qt-ai-gui.yml` (Windows): tauri-action build NSIS + MSI, ký bằng
`TAURI_SIGNING_PRIVATE_KEY` / `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (secret của repo), tạo GitHub Release kèm `latest.json`; bước sau
gom zip portable và upload thêm. App đã cài dò `releases/latest/download/latest.json` mỗi lần mở (tắt ở dev), có bản mới thì hỏi
bằng hộp thoại native, tải, cài rồi khởi động lại (`src/hooks/use-update-check.ts`). Bản portable cũng cập nhật bằng NSIS nên sau đó
thành bản cài đặt.

```
npm run set-version 0.2.0     # package.json, package-lock.json, tauri.conf.json, Cargo.toml, Cargo.lock
git commit -am "release(qt-ai-gui): 0.2.0"
git tag qt-ai-gui-v0.2.0 && git push origin HEAD --tags
```

Khoá ký sinh bằng `npx tauri signer generate -w ~/.tauri/qt-ai-gui.key`; public key nằm ở `plugins.updater.pubkey` trong
`tauri.conf.json`. Mất private key thì bản đã phát hành không nhận bản mới nữa.
```

- [ ] **Step 3: Kiểm tra** — `actionlint .github/workflows/release-qt-ai-gui.yml` nếu có; `cd apps/qt-ai-gui && npm run check` toàn bộ.

## Self-review

- Spec §1 → Task 4; §2 → Task 2 bước 1 + README; §3 → Task 2; §4 → Task 1; §5 → Task 3; §6 → test trong từng task. Không còn TBD. Tên `runUpdateCheck`, `applyVersion`, `isValidVersion`, `PORTABLE_SKIP_BUILD` dùng nhất quán.
