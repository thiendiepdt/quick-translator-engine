import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

import { applyVersion, isValidVersion } from "./set-version.mjs";

function fixture() {
  const dir = mkdtempSync(join(tmpdir(), "set-version-"));
  mkdirSync(join(dir, "src-tauri"), { recursive: true });
  writeFileSync(
    join(dir, "package.json"),
    JSON.stringify({ name: "qt-ai-gui", version: "0.1.0", scripts: { dev: "vite" } }, null, 2) + "\n",
  );
  writeFileSync(
    join(dir, "package-lock.json"),
    JSON.stringify(
      {
        name: "qt-ai-gui",
        version: "0.1.0",
        lockfileVersion: 3,
        packages: { "": { name: "qt-ai-gui", version: "0.1.0" }, "node_modules/x": { version: "4.5.0" } },
      },
      null,
      2,
    ) + "\n",
  );
  writeFileSync(
    join(dir, "src-tauri", "tauri.conf.json"),
    '{\n  "productName": "VNCVT AI Translator",\n  "version": "0.1.0",\n  "bundle": { "targets": ["nsis", "msi"] }\n}\n',
  );
  writeFileSync(
    join(dir, "src-tauri", "Cargo.toml"),
    '[package]\nname = "qt-ai-gui"\nversion = "0.1.0"\n\n[dependencies]\ntauri = { version = "2", features = [] }\n',
  );
  return dir;
}

describe("applyVersion", () => {
  it("đổi version ở 4 file, giữ nguyên phần còn lại", () => {
    const dir = fixture();
    const written = applyVersion(dir, "0.2.0");
    expect(written).toHaveLength(4);
    expect(JSON.parse(readFileSync(join(dir, "package.json"), "utf8"))).toEqual({
      name: "qt-ai-gui",
      version: "0.2.0",
      scripts: { dev: "vite" },
    });
    const lock = JSON.parse(readFileSync(join(dir, "package-lock.json"), "utf8"));
    expect(lock.version).toBe("0.2.0");
    expect(lock.packages[""].version).toBe("0.2.0");
    expect(lock.packages["node_modules/x"].version).toBe("4.5.0");
    expect(readFileSync(join(dir, "src-tauri", "tauri.conf.json"), "utf8")).toBe(
      '{\n  "productName": "VNCVT AI Translator",\n  "version": "0.2.0",\n  "bundle": { "targets": ["nsis", "msi"] }\n}\n',
    );
    expect(readFileSync(join(dir, "src-tauri", "Cargo.toml"), "utf8")).toBe(
      '[package]\nname = "qt-ai-gui"\nversion = "0.2.0"\n\n[dependencies]\ntauri = { version = "2", features = [] }\n',
    );
  });

  it("package-lock.json xuống dòng CRLF (npm trên Windows) vẫn đổi được packages[\"\"]", () => {
    const dir = fixture();
    const lockPath = join(dir, "package-lock.json");
    writeFileSync(lockPath, readFileSync(lockPath, "utf8").replace(/\n/g, "\r\n"));
    applyVersion(dir, "0.2.0");
    const text = readFileSync(lockPath, "utf8");
    expect(text).toContain("\r\n");
    const lock = JSON.parse(text);
    expect(lock.version).toBe("0.2.0");
    expect(lock.packages[""].version).toBe("0.2.0");
  });

  it("đặt lại cùng version vẫn chạy, file không đổi", () => {
    const dir = fixture();
    const before = readFileSync(join(dir, "package-lock.json"), "utf8");
    expect(applyVersion(dir, "0.1.0")).toHaveLength(4);
    expect(readFileSync(join(dir, "package-lock.json"), "utf8")).toBe(before);
  });

  it("từ chối version sai định dạng", () => {
    expect(isValidVersion("1.2")).toBe(false);
    expect(isValidVersion("v1.2.3")).toBe(false);
    expect(isValidVersion("1.2.3")).toBe(true);
    expect(() => applyVersion(fixture(), "abc")).toThrow(/x\.y\.z/);
  });
});
