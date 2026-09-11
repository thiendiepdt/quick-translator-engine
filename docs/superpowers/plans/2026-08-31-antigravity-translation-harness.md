# Antigravity Translation Harness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** CLI `qt-ai` (trong `apps/qt-ai-cli`) + template AGENTS.md/workflows để agent Antigravity dịch truyện hàng loạt theo vòng `next → dịch → check → accept`, dùng lại nguyên code lõi của qt-web.

**Architecture:** CLI Node/tsx import thẳng source qt-web qua alias `@/*` → `../qt-web/src/*` (không sửa file nào của qt-web). Mọi tiến độ nằm trong `state.json` + file trong folder truyện; agent chỉ chạy lệnh CLI và ghi file draft — mọi hàng rào (đủ đoạn, check rules, sanitize glossary) là script deterministic.

**Tech Stack:** Node ≥22, TypeScript 5.9, tsx, vitest 4. Không thêm dependency runtime nào (chỉ `node:fs`/`node:path`).

**Spec:** `docs/superpowers/specs/2026-08-31-antigravity-translation-harness-design.md`

## Global Constraints

- **Không sửa bất kỳ file nào trong `apps/qt-web/`.** Chỉ import.
- Node `>=22.0.0`, `"type": "module"`, TypeScript `^5.9.3`, vitest `^4.1.10`, tsx `^4.20.0`.
- Alias `@/*` trỏ `../qt-web/src/*` — khai báo cả trong `tsconfig.json` (paths) lẫn `vitest.config.ts` (resolve.alias).
- Mọi ghi `story.json`/`state.json`: ghi file tạm cùng thư mục rồi `renameSync` (atomic); riêng `story.json` ghi thêm `story.json.bak` (bản cũ) trước khi thay.
- Schema `story.json` = `AiStoryConfig` của web, đọc bằng `parseAiStoryConfigJson`, không thêm field lạ. Knob riêng của CLI nằm trong `state.json.settings`: `minLengthRatio: 0.75`, `maxReviewRounds: 2`, `chaptersPerSession: 10`.
- Chapter id = tên file trong `raw/` bỏ đuôi `.txt` (vd `raw/0001.txt` → `0001`), sắp bằng `naturalChapterCompare`.
- Commit message tiếng Việt, prefix `feat(qt-ai)`/`test(qt-ai)`/`docs(qt-ai)`, kèm trailer `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`. **Chỉ commit khi user đã cho phép cơ chế commit theo task; nếu chưa, dừng ở bước chạy test pass và gom lại chờ lệnh.**
- Lệnh chạy test/typecheck (cwd = repo root):
  - `npm --prefix apps/qt-ai-cli test`
  - `npm --prefix apps/qt-ai-cli run typecheck`

## File Structure (toàn cục)

```
apps/qt-ai-cli/
  package.json
  tsconfig.json
  vitest.config.ts
  src/
    main.ts                 # dispatch CLI: qt-ai <cmd> <storyRoot> [id] [flags]
    story-fs.ts             # paths, load/save story+state, list raw, work files
    commands/
      init.ts               # runInit
      next.ts               # runNext
      check.ts              # runCheck
      accept.ts             # runAccept
      skip.ts               # runSkip
      status.ts             # runStatus
  test/
    helpers.ts              # dựng folder truyện tạm cho test
    alias.test.ts
    story-fs.test.ts
    init.test.ts
    next.test.ts
    check.test.ts
    accept.test.ts
    skip-status.test.ts
    e2e.test.ts
  antigravity/
    AGENTS.md               # template, placeholder {{QT_AI}} {{STORY_ROOT}}
    workflows/
      setup-story.md
      translate.md
```

Folder truyện (runtime, ngoài repo): `story.json`, `state.json`, `raw/*.txt`, `out/*.md`, `work/*`, `AGENTS.md`, `.agent/workflows/*.md`.

---

### Task 1: Scaffold `apps/qt-ai-cli` + alias sang qt-web

**Files:**
- Create: `apps/qt-ai-cli/package.json`
- Create: `apps/qt-ai-cli/tsconfig.json`
- Create: `apps/qt-ai-cli/vitest.config.ts`
- Create: `apps/qt-ai-cli/src/main.ts` (tạm chỉ in usage)
- Test: `apps/qt-ai-cli/test/alias.test.ts`

**Interfaces:**
- Produces: project chạy được `npm test`/`typecheck`; alias `@/` dùng được ở mọi task sau.

- [ ] **Step 1: Viết test fail (chưa có project)**

`apps/qt-ai-cli/test/alias.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { aiParagraphsOf } from "@/lib/ai-paragraphs";
import { emptyAiStoryConfig } from "@/lib/ai-story";

describe("alias @/ tới qt-web/src", () => {
  it("import được module lõi của qt-web", () => {
    expect(aiParagraphsOf("a\r\n\r\n b ")).toEqual(["a", "b"]);
    expect(emptyAiStoryConfig().autoGlossary).toBe("inherit");
  });
});
```

- [ ] **Step 2: Chạy để thấy fail**

Run: `npm --prefix apps/qt-ai-cli test`
Expected: FAIL (chưa có package.json / vitest).

- [ ] **Step 3: Tạo scaffold**

`apps/qt-ai-cli/package.json`:

```json
{
  "name": "qt-ai-cli",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "engines": { "node": ">=22.0.0" },
  "scripts": {
    "qt-ai": "tsx src/main.ts",
    "typecheck": "tsc --noEmit --pretty false",
    "test": "vitest run",
    "test:watch": "vitest"
  },
  "devDependencies": {
    "@types/node": "^26.1.1",
    "tsx": "^4.20.0",
    "typescript": "^5.9.3",
    "vitest": "^4.1.10"
  }
}
```

`apps/qt-ai-cli/tsconfig.json`:

```json
{
  "compilerOptions": {
    "target": "ES2023",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "lib": ["ES2023"],
    "types": ["node"],
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "skipLibCheck": true,
    "noEmit": true,
    "allowImportingTsExtensions": true,
    "baseUrl": ".",
    "paths": { "@/*": ["../qt-web/src/*"] }
  },
  "include": ["src", "test", "vitest.config.ts"]
}
```

(KHÔNG include glob `../qt-web/src/**` — tsc tự follow các file được import qua alias; include cả lib web sẽ kéo file dính DOM như `ai-settings.ts` vào typecheck và fail vì lib chỉ có ES2023.)

`apps/qt-ai-cli/vitest.config.ts`:

```ts
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitest/config";

export default defineConfig({
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("../qt-web/src", import.meta.url)),
    },
  },
  test: { environment: "node" },
});
```

`apps/qt-ai-cli/src/main.ts`:

```ts
const USAGE = `qt-ai <lệnh> <thư-mục-truyện> [chương] [cờ]

Lệnh:
  init <root>                Dựng khung folder truyện + copy AGENTS.md/workflows
  next <root>                Phát chương kế tiếp, lắp prompt vào work/
  check <root> <id>          Kiểm tra bản dịch work/<id>.draft.md
  accept <root> <id> [--force]  Chốt chương: ghi out/, merge glossary
  skip <root> <id> --reason <lý do>  Bỏ qua chương (model từ chối...)
  status <root>              Bảng tiến độ`;

export function main(_argv: string[]): number {
  console.log(USAGE);
  return 1;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  process.exit(main(process.argv.slice(2)));
}
```

- [ ] **Step 4: Cài deps và chạy test pass**

Run: `npm --prefix apps/qt-ai-cli install && npm --prefix apps/qt-ai-cli test && npm --prefix apps/qt-ai-cli run typecheck`
Expected: test alias PASS, typecheck sạch.

- [ ] **Step 5: Commit (nếu đã được phép)**

```bash
git add apps/qt-ai-cli
git commit -m "feat(qt-ai): scaffold CLI qt-ai với alias @/ sang qt-web/src

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 2: `story-fs.ts` — state, story, raw, work files

**Files:**
- Create: `apps/qt-ai-cli/src/story-fs.ts`
- Create: `apps/qt-ai-cli/test/helpers.ts`
- Test: `apps/qt-ai-cli/test/story-fs.test.ts`

**Interfaces:**
- Consumes: `parseAiStoryConfigJson`, `emptyAiStoryConfig`, `naturalChapterCompare` từ `@/lib/ai-story`.
- Produces (mọi task sau dùng đúng các tên này):

```ts
export type ChapterStatus = "queued" | "translating" | "done" | "error" | "skipped";
export interface ChapterState { status: ChapterStatus; reviewRound: number; reason?: string; updatedAt: number; }
export interface HarnessSettings { minLengthRatio: number; maxReviewRounds: number; chaptersPerSession: number; }
export interface StoryState { version: 1; settings: HarnessSettings; chapters: Record<string, ChapterState>; }
export interface StoryPaths { root: string; storyJson: string; stateJson: string; rawDir: string; outDir: string; workDir: string; }
export function storyPaths(root: string): StoryPaths;
export function defaultSettings(): HarnessSettings;               // {0.75, 2, 10}
export function listRawChapterIds(paths: StoryPaths): string[];   // *.txt, sort naturalChapterCompare
export function readRawChapter(paths: StoryPaths, id: string): string;
export function loadStoryConfig(paths: StoryPaths): AiStoryConfig; // throw Error khi thiếu/hỏng
export function saveStoryConfig(paths: StoryPaths, config: AiStoryConfig): void; // .bak + atomic
export function loadState(paths: StoryPaths): StoryState;          // throw khi thiếu/hỏng
export function saveState(paths: StoryPaths, state: StoryState): void; // atomic
export type WorkKind = "prompt" | "draft" | "glossary" | "check" | "review";
export function workFile(paths: StoryPaths, id: string, kind: WorkKind): string;
// prompt→.prompt.md  draft→.draft.md  glossary→.glossary.json  check→.check.json  review→.review.md
```

- [ ] **Step 1: Viết helper test**

`apps/qt-ai-cli/test/helpers.ts`:

```ts
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

/** Dựng folder truyện tạm: raw/ với các chương cho trước. */
export function makeStoryDir(chapters: Record<string, string>): string {
  const root = mkdtempSync(join(tmpdir(), "qt-ai-test-"));
  mkdirSync(join(root, "raw"), { recursive: true });
  for (const [id, text] of Object.entries(chapters)) {
    writeFileSync(join(root, "raw", `${id}.txt`), text, "utf8");
  }
  return root;
}
```

- [ ] **Step 2: Viết test fail**

`apps/qt-ai-cli/test/story-fs.test.ts`:

```ts
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { emptyAiStoryConfig } from "@/lib/ai-story";
import {
  defaultSettings, listRawChapterIds, loadState, loadStoryConfig,
  saveState, saveStoryConfig, storyPaths, workFile,
} from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

describe("story-fs", () => {
  it("liệt kê chương theo thứ tự tự nhiên", () => {
    const root = makeStoryDir({ "10": "十", "2": "二", "1": "一" });
    expect(listRawChapterIds(storyPaths(root))).toEqual(["1", "2", "10"]);
  });

  it("save/load story atomic kèm .bak", () => {
    const root = makeStoryDir({});
    const paths = storyPaths(root);
    const config = emptyAiStoryConfig();
    saveStoryConfig(paths, config);                    // lần đầu chưa có .bak
    expect(existsSync(`${paths.storyJson}.bak`)).toBe(false);
    saveStoryConfig(paths, { ...config, name: "Truyện A" });
    expect(loadStoryConfig(paths).name).toBe("Truyện A");
    expect(JSON.parse(readFileSync(`${paths.storyJson}.bak`, "utf8")).name).toBe("");
  });

  it("story.json hỏng thì throw chứ không trả config rỗng", () => {
    const root = makeStoryDir({});
    const paths = storyPaths(root);
    writeFileSync(paths.storyJson, "{hỏng", "utf8");
    expect(() => loadStoryConfig(paths)).toThrow(/story\.json/);
  });

  it("state round-trip và validate", () => {
    const root = makeStoryDir({ "1": "一" });
    const paths = storyPaths(root);
    const state = {
      version: 1 as const,
      settings: defaultSettings(),
      chapters: { "1": { status: "queued" as const, reviewRound: 0, updatedAt: 1 } },
    };
    saveState(paths, state);
    expect(loadState(paths)).toEqual(state);
    writeFileSync(paths.stateJson, "[]", "utf8");
    expect(() => loadState(paths)).toThrow(/state\.json/);
  });

  it("đặt tên work files đúng quy ước", () => {
    const paths = storyPaths("/x");
    expect(workFile(paths, "0001", "prompt")).toBe("/x/work/0001.prompt.md");
    expect(workFile(paths, "0001", "glossary")).toBe("/x/work/0001.glossary.json");
  });
});
```

- [ ] **Step 3: Chạy fail** — `npm --prefix apps/qt-ai-cli test` → FAIL (module chưa có).

- [ ] **Step 4: Implement `src/story-fs.ts`**

```ts
import {
  copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync,
  renameSync, writeFileSync,
} from "node:fs";
import { join } from "node:path";
import {
  naturalChapterCompare, parseAiStoryConfigJson, type AiStoryConfig,
} from "@/lib/ai-story";

export type ChapterStatus = "queued" | "translating" | "done" | "error" | "skipped";

export interface ChapterState {
  status: ChapterStatus;
  reviewRound: number;
  reason?: string;
  updatedAt: number;
}

export interface HarnessSettings {
  minLengthRatio: number;
  maxReviewRounds: number;
  chaptersPerSession: number;
}

export interface StoryState {
  version: 1;
  settings: HarnessSettings;
  chapters: Record<string, ChapterState>;
}

export interface StoryPaths {
  root: string;
  storyJson: string;
  stateJson: string;
  rawDir: string;
  outDir: string;
  workDir: string;
}

export function storyPaths(root: string): StoryPaths {
  return {
    root,
    storyJson: join(root, "story.json"),
    stateJson: join(root, "state.json"),
    rawDir: join(root, "raw"),
    outDir: join(root, "out"),
    workDir: join(root, "work"),
  };
}

export function defaultSettings(): HarnessSettings {
  return { minLengthRatio: 0.75, maxReviewRounds: 2, chaptersPerSession: 10 };
}

export function listRawChapterIds(paths: StoryPaths): string[] {
  if (!existsSync(paths.rawDir)) return [];
  return readdirSync(paths.rawDir)
    .filter((name) => name.endsWith(".txt"))
    .map((name) => name.slice(0, -4))
    .sort(naturalChapterCompare);
}

export function readRawChapter(paths: StoryPaths, id: string): string {
  return readFileSync(join(paths.rawDir, `${id}.txt`), "utf8");
}

/** Ghi atomic: file tạm cùng thư mục rồi rename đè. */
function writeAtomic(path: string, content: string): void {
  const tmp = `${path}.tmp`;
  writeFileSync(tmp, content, "utf8");
  renameSync(tmp, path);
}

export function loadStoryConfig(paths: StoryPaths): AiStoryConfig {
  if (!existsSync(paths.storyJson)) {
    throw new Error(`Không thấy story.json trong ${paths.root} — chạy: qt-ai init`);
  }
  const config = parseAiStoryConfigJson(readFileSync(paths.storyJson, "utf8"));
  if (!config) throw new Error(`story.json hỏng (không phải JSON object): ${paths.storyJson}`);
  return config;
}

export function saveStoryConfig(paths: StoryPaths, config: AiStoryConfig): void {
  if (existsSync(paths.storyJson)) {
    copyFileSync(paths.storyJson, `${paths.storyJson}.bak`);
  }
  writeAtomic(paths.storyJson, `${JSON.stringify(config, null, 2)}\n`);
}

function isChapterState(value: unknown): value is ChapterState {
  if (typeof value !== "object" || value === null) return false;
  const record = value as Record<string, unknown>;
  return (
    ["queued", "translating", "done", "error", "skipped"].includes(String(record.status)) &&
    typeof record.reviewRound === "number" &&
    typeof record.updatedAt === "number"
  );
}

export function loadState(paths: StoryPaths): StoryState {
  if (!existsSync(paths.stateJson)) {
    throw new Error(`Không thấy state.json trong ${paths.root} — chạy: qt-ai init`);
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(readFileSync(paths.stateJson, "utf8"));
  } catch (error) {
    throw new Error(`state.json hỏng: ${error instanceof Error ? error.message : String(error)}`);
  }
  if (
    typeof parsed !== "object" || parsed === null || Array.isArray(parsed) ||
    (parsed as Record<string, unknown>).version !== 1
  ) {
    throw new Error(`state.json sai schema (cần object version 1): ${paths.stateJson}`);
  }
  const record = parsed as Record<string, unknown>;
  const settingsValue = record.settings as Record<string, unknown> | undefined;
  const fallback = defaultSettings();
  const settings: HarnessSettings = {
    minLengthRatio: typeof settingsValue?.minLengthRatio === "number" ? settingsValue.minLengthRatio : fallback.minLengthRatio,
    maxReviewRounds: typeof settingsValue?.maxReviewRounds === "number" ? settingsValue.maxReviewRounds : fallback.maxReviewRounds,
    chaptersPerSession: typeof settingsValue?.chaptersPerSession === "number" ? settingsValue.chaptersPerSession : fallback.chaptersPerSession,
  };
  const chapters: Record<string, ChapterState> = {};
  if (typeof record.chapters === "object" && record.chapters !== null) {
    for (const [id, value] of Object.entries(record.chapters as Record<string, unknown>)) {
      if (isChapterState(value)) chapters[id] = value;
    }
  }
  return { version: 1, settings, chapters };
}

export function saveState(paths: StoryPaths, state: StoryState): void {
  writeAtomic(paths.stateJson, `${JSON.stringify(state, null, 2)}\n`);
}

export type WorkKind = "prompt" | "draft" | "glossary" | "check" | "review";

const WORK_SUFFIX: Record<WorkKind, string> = {
  prompt: ".prompt.md",
  draft: ".draft.md",
  glossary: ".glossary.json",
  check: ".check.json",
  review: ".review.md",
};

export function workFile(paths: StoryPaths, id: string, kind: WorkKind): string {
  return join(paths.workDir, `${id}${WORK_SUFFIX[kind]}`);
}

export function ensureStoryDirs(paths: StoryPaths): void {
  for (const dir of [paths.rawDir, paths.outDir, paths.workDir]) {
    mkdirSync(dir, { recursive: true });
  }
}
```

- [ ] **Step 5: Chạy pass** — `npm --prefix apps/qt-ai-cli test && npm --prefix apps/qt-ai-cli run typecheck` → PASS.

- [ ] **Step 6: Commit (nếu đã được phép)**

```bash
git add apps/qt-ai-cli/src/story-fs.ts apps/qt-ai-cli/test
git commit -m "feat(qt-ai): story-fs — state/story atomic IO, liệt kê chương, work files

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 3: `qt-ai init`

**Files:**
- Create: `apps/qt-ai-cli/src/commands/init.ts`
- Create: `apps/qt-ai-cli/antigravity/AGENTS.md` (placeholder tối thiểu, Task 8 viết bản đầy đủ)
- Create: `apps/qt-ai-cli/antigravity/workflows/setup-story.md` (placeholder)
- Create: `apps/qt-ai-cli/antigravity/workflows/translate.md` (placeholder)
- Test: `apps/qt-ai-cli/test/init.test.ts`

**Interfaces:**
- Consumes: toàn bộ `story-fs.ts`.
- Produces: `export function runInit(root: string): string` — trả message tóm tắt. Idempotent: chạy lại không phá state/story sẵn có, chỉ bổ sung chương raw mới thành `queued`. Template copy vào `<root>/AGENTS.md` và `<root>/.agent/workflows/`, thay `{{QT_AI}}` bằng lệnh chạy CLI tuyệt đối và `{{STORY_ROOT}}` bằng root tuyệt đối; **không ghi đè** file đích đã tồn tại.

- [ ] **Step 1: Viết test fail**

`apps/qt-ai-cli/test/init.test.ts`:

```ts
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { runInit } from "../src/commands/init.ts";
import { loadState, loadStoryConfig, storyPaths } from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

describe("qt-ai init", () => {
  it("dựng story.json rỗng, state queued cho từng chương, copy template", () => {
    const root = makeStoryDir({ "0001": "第一章", "0002": "第二章" });
    runInit(root);
    const paths = storyPaths(root);
    expect(loadStoryConfig(paths).glossary.names).toEqual({});
    const state = loadState(paths);
    expect(Object.keys(state.chapters)).toEqual(["0001", "0002"]);
    expect(state.chapters["0001"]?.status).toBe("queued");
    expect(state.settings.maxReviewRounds).toBe(2);
    const agents = readFileSync(join(root, "AGENTS.md"), "utf8");
    expect(agents).not.toContain("{{QT_AI}}");
    expect(agents).toContain(root);
    expect(existsSync(join(root, ".agent/workflows/translate.md"))).toBe(true);
  });

  it("idempotent: giữ state cũ, thêm chương mới, không đè AGENTS.md", () => {
    const root = makeStoryDir({ "0001": "第一章" });
    runInit(root);
    const paths = storyPaths(root);
    const state = loadState(paths);
    state.chapters["0001"] = { status: "done", reviewRound: 1, updatedAt: 5 };
    writeFileSync(paths.stateJson, JSON.stringify(state), "utf8");
    writeFileSync(join(root, "AGENTS.md"), "tự sửa", "utf8");
    writeFileSync(join(paths.rawDir, "0002.txt"), "第二章", "utf8");
    runInit(root);
    const after = loadState(paths);
    expect(after.chapters["0001"]?.status).toBe("done");
    expect(after.chapters["0002"]?.status).toBe("queued");
    expect(readFileSync(join(root, "AGENTS.md"), "utf8")).toBe("tự sửa");
  });
});
```

- [ ] **Step 2: Chạy fail** — `npm --prefix apps/qt-ai-cli test` → FAIL.

- [ ] **Step 3: Tạo template placeholder** (Task 8 thay nội dung thật, giữ nguyên placeholder marker)

`apps/qt-ai-cli/antigravity/AGENTS.md`:

```markdown
# Harness dịch truyện — {{STORY_ROOT}}

Lệnh CLI: `{{QT_AI}}`
(Nội dung đầy đủ được cập nhật ở task sau; file này do qt-ai init sinh ra.)
```

`apps/qt-ai-cli/antigravity/workflows/setup-story.md` và `translate.md`: mỗi file một dòng tiêu đề tạm, ví dụ `# /translate — placeholder`.

- [ ] **Step 4: Implement `src/commands/init.ts`**

```ts
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { emptyAiStoryConfig } from "@/lib/ai-story";
import {
  defaultSettings, ensureStoryDirs, listRawChapterIds, loadState,
  saveState, saveStoryConfig, storyPaths, type StoryState,
} from "../story-fs.ts";

const CLI_DIR = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const TEMPLATE_DIR = join(CLI_DIR, "antigravity");

function renderTemplate(source: string, root: string): string {
  const qtAi = `npm --prefix ${CLI_DIR} run -s qt-ai --`;
  return source.replaceAll("{{QT_AI}}", qtAi).replaceAll("{{STORY_ROOT}}", root);
}

function copyTemplates(root: string): void {
  const agentsTarget = join(root, "AGENTS.md");
  if (!existsSync(agentsTarget)) {
    writeFileSync(agentsTarget, renderTemplate(readFileSync(join(TEMPLATE_DIR, "AGENTS.md"), "utf8"), root), "utf8");
  }
  const workflowsDir = join(root, ".agent", "workflows");
  mkdirSync(workflowsDir, { recursive: true });
  for (const name of readdirSync(join(TEMPLATE_DIR, "workflows"))) {
    const target = join(workflowsDir, name);
    if (existsSync(target)) continue;
    writeFileSync(target, renderTemplate(readFileSync(join(TEMPLATE_DIR, "workflows", name), "utf8"), root), "utf8");
  }
}

export function runInit(root: string): string {
  const paths = storyPaths(resolve(root));
  ensureStoryDirs(paths);
  if (!existsSync(paths.storyJson)) saveStoryConfig(paths, emptyAiStoryConfig());
  const state: StoryState = existsSync(paths.stateJson)
    ? loadState(paths)
    : { version: 1, settings: defaultSettings(), chapters: {} };
  let added = 0;
  for (const id of listRawChapterIds(paths)) {
    if (state.chapters[id]) continue;
    state.chapters[id] = { status: "queued", reviewRound: 0, updatedAt: Date.now() };
    added += 1;
  }
  saveState(paths, state);
  copyTemplates(paths.root);
  const total = Object.keys(state.chapters).length;
  return `Đã init ${paths.root}: ${total} chương (${added} mới thêm vào hàng đợi).`;
}
```

Lưu ý bug tiềm ẩn: `saveStoryConfig` lần đầu tạo `.bak`? Không — Task 2 đã quy định chỉ copy `.bak` khi file cũ tồn tại.

- [ ] **Step 5: Chạy pass** — `npm --prefix apps/qt-ai-cli test && npm --prefix apps/qt-ai-cli run typecheck` → PASS.

- [ ] **Step 6: Commit (nếu đã được phép)**

```bash
git add apps/qt-ai-cli/src/commands/init.ts apps/qt-ai-cli/antigravity apps/qt-ai-cli/test/init.test.ts
git commit -m "feat(qt-ai): lệnh init — dựng khung folder truyện + copy template Antigravity

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 4: `qt-ai next`

**Files:**
- Create: `apps/qt-ai-cli/src/commands/next.ts`
- Test: `apps/qt-ai-cli/test/next.test.ts`

**Interfaces:**
- Consumes: `story-fs.ts`; `buildAiTranslationSystemPrompt` từ `@/lib/ai-translation` (gọi `buildAiTranslationSystemPrompt({}, story, source)` — workspace glossary rỗng, chỉ dùng glossary truyện); `aiParagraphsOf`, `labeledAiSourcePayload` từ `@/lib/ai-paragraphs`.
- Produces: `export function runNext(root: string): { chapterId: string; promptPath: string }` — throw khi: chưa init, còn chương `translating` (message chỉ rõ id + bảo chạy check/accept/skip), hoặc hết chương `queued`.

- [ ] **Step 1: Viết test fail**

`apps/qt-ai-cli/test/next.test.ts`:

```ts
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { runInit } from "../src/commands/init.ts";
import { runNext } from "../src/commands/next.ts";
import { loadState, saveStoryConfig, loadStoryConfig, storyPaths } from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

const RAW = "赵静文抬头。\n\n方寸之间。";

function initStory(chapters: Record<string, string>): string {
  const root = makeStoryDir(chapters);
  runInit(root);
  return root;
}

describe("qt-ai next", () => {
  it("phát chương queued đầu tiên, prompt đủ 3 phần, state → translating", () => {
    const root = initStory({ "0001": RAW, "0002": "第二章" });
    const paths = storyPaths(root);
    const config = loadStoryConfig(paths);
    config.glossary.names["赵静文"] = "Triệu Tĩnh Văn";
    config.glossary.names["不出现"] = "Không Xuất Hiện";
    saveStoryConfig(paths, config);

    const result = runNext(root);
    expect(result.chapterId).toBe("0001");
    const prompt = readFileSync(result.promptPath, "utf8");
    expect(prompt).toContain("dịch giả tiểu thuyết Trung Quốc");   // base prompt
    expect(prompt).toContain("Triệu Tĩnh Văn");                     // glossary đã lọc theo chương
    expect(prompt).not.toContain("Không Xuất Hiện");                // entry không có trong chương bị lọc
    expect(prompt).toContain("[[1]] 赵静文抬头。");                  // payload gắn nhãn
    expect(prompt).toContain("0001.draft.md");                      // chỉ dẫn ghi draft
    expect(prompt).toContain("0001.glossary.json");                 // chỉ dẫn đề xuất glossary
    expect(loadState(paths).chapters["0001"]?.status).toBe("translating");
  });

  it("từ chối phát chương mới khi còn chương translating", () => {
    const root = initStory({ "0001": RAW, "0002": "第二章" });
    runNext(root);
    expect(() => runNext(root)).toThrow(/0001/);
  });

  it("hết queued thì báo", () => {
    const root = initStory({});
    expect(() => runNext(root)).toThrow(/không còn chương/i);
  });
});
```

- [ ] **Step 2: Chạy fail** — FAIL.

- [ ] **Step 3: Implement `src/commands/next.ts`**

```ts
import { writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { aiParagraphsOf, labeledAiSourcePayload } from "@/lib/ai-paragraphs";
import { buildAiTranslationSystemPrompt } from "@/lib/ai-translation";
import { naturalChapterCompare } from "@/lib/ai-story";
import {
  loadState, loadStoryConfig, readRawChapter, saveState, storyPaths, workFile,
} from "../story-fs.ts";

function agentInstructions(id: string): string {
  return [
    "# Việc của agent sau khi dịch xong",
    "",
    `1. Ghi bản dịch tiếng Việt (GIỮ NGUYÊN nhãn [[n]] đầu mỗi đoạn) vào work/${id}.draft.md.`,
    `2. Ghi đề xuất TÊN RIÊNG mới (nhân vật, địa danh, đồ vật/vũ khí, sinh vật, công pháp/kỹ năng)`,
    `   xuất hiện trong raw nhưng chưa có trong từ điển của prompt vào work/${id}.glossary.json, dạng:`,
    `   {"entries": [{"source": "chữ Hán trong raw", "target": "chép nguyên văn từ bản dịch", "category": "names|places|items|creatures|skills"}]}`,
    `   Bỏ qua từ chung, chức danh, đại từ. Không có tên mới thì ghi {"entries": []}.`,
    `3. Chạy: qt-ai check ${id} (xem AGENTS.md để biết lệnh đầy đủ).`,
  ].join("\n");
}

export function runNext(root: string): { chapterId: string; promptPath: string } {
  const paths = storyPaths(resolve(root));
  const state = loadState(paths);
  const story = loadStoryConfig(paths);

  const pending = Object.entries(state.chapters)
    .filter(([, chapter]) => chapter.status === "translating")
    .map(([id]) => id);
  if (pending.length > 0) {
    throw new Error(
      `Chương ${pending.join(", ")} đang translating chưa chốt — chạy check/accept/skip trước khi lấy chương mới.`,
    );
  }

  const nextId = Object.entries(state.chapters)
    .filter(([, chapter]) => chapter.status === "queued")
    .map(([id]) => id)
    .sort(naturalChapterCompare)[0];
  if (!nextId) throw new Error("Không còn chương nào trong hàng đợi — chạy qt-ai status để xem tổng kết.");

  const source = readRawChapter(paths, nextId);
  const system = buildAiTranslationSystemPrompt({}, story, source);
  const payload = labeledAiSourcePayload(aiParagraphsOf(source));
  const prompt = `${system}\n\n---\n\n${payload}\n\n---\n\n${agentInstructions(nextId)}\n`;

  const promptPath = workFile(paths, nextId, "prompt");
  writeFileSync(promptPath, prompt, "utf8");
  state.chapters[nextId] = { status: "translating", reviewRound: 0, updatedAt: Date.now() };
  saveState(paths, state);
  return { chapterId: nextId, promptPath };
}
```

- [ ] **Step 4: Chạy pass** — PASS.

- [ ] **Step 5: Commit (nếu đã được phép)**

```bash
git add apps/qt-ai-cli/src/commands/next.ts apps/qt-ai-cli/test/next.test.ts
git commit -m "feat(qt-ai): lệnh next — lắp prompt dịch cùng code với web, khoá 1 chương đang dịch

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 5: `qt-ai check`

**Files:**
- Create: `apps/qt-ai-cli/src/commands/check.ts`
- Test: `apps/qt-ai-cli/test/check.test.ts`

**Interfaces:**
- Consumes: `story-fs.ts`; `aiParagraphsOf`, `parseLabeledAiTranslation`, `labeledAiRepairPayload` từ `@/lib/ai-paragraphs`; `checkAiTranslationViolations`, `buildAiTranslationReviewPrompt`, `type TranslationViolation` từ `@/lib/ai-translation`.
- Produces:

```ts
export interface CheckResult {
  pass: boolean;
  missing: number[];                    // nhãn 1-based còn thiếu
  violations: TranslationViolation[];
  ratio: number;                        // ký tự dịch / ký tự raw (đều đã trim toàn văn bản)
  escalatedToError: boolean;            // true khi quá maxReviewRounds
  reviewPath?: string;                  // work/<id>.review.md khi fail còn lượt sửa
}
export function runCheck(root: string, id: string): CheckResult;
// Ghi work/<id>.check.json = {pass, missing, violationCount, ratio, reviewRound, checkedAt}.
// Fail + còn lượt: reviewRound++ trong state, ghi review.md. Fail + hết lượt: status=error, reason.
// Pass: giữ status=translating (accept mới chuyển done), reviewRound giữ nguyên.
// Bản dịch “ghép” dùng chung với accept: các đoạn parse được, mỗi đoạn một dòng, nối "\n\n".
export function assembleDraft(root: string, id: string): {
  paragraphs: string[];                 // đoạn raw
  parsed: Array<string | undefined>;    // đoạn dịch theo nhãn (undefined = thiếu)
  finalText: string;                    // các đoạn đã dịch nối "\n\n"
};
```

- [ ] **Step 1: Viết test fail**

`apps/qt-ai-cli/test/check.test.ts`:

```ts
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { runCheck } from "../src/commands/check.ts";
import { runInit } from "../src/commands/init.ts";
import { runNext } from "../src/commands/next.ts";
import { loadState, storyPaths, workFile } from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

const RAW = "赵静文抬头看向远方的高塔。\n\n她沉默了很久没有说话。";

function storyWithDraft(draft: string): string {
  const root = makeStoryDir({ "0001": RAW });
  runInit(root);
  runNext(root);
  writeFileSync(workFile(storyPaths(root), "0001", "draft"), draft, "utf8");
  return root;
}

describe("qt-ai check", () => {
  it("pass khi đủ đoạn, sạch rule, đủ dài", () => {
    const root = storyWithDraft(
      "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn về phía tòa tháp cao nơi xa.\n\n[[2]] Nàng im lặng hồi lâu không nói lời nào.",
    );
    const result = runCheck(root, "0001");
    expect(result.pass).toBe(true);
    expect(result.missing).toEqual([]);
    expect(result.violations).toEqual([]);
    expect(loadState(storyPaths(root)).chapters["0001"]?.status).toBe("translating");
    const report = JSON.parse(readFileSync(workFile(storyPaths(root), "0001", "check"), "utf8"));
    expect(report.pass).toBe(true);
  });

  it("bắt thiếu đoạn + vi phạm rule, sinh review.md, tăng reviewRound", () => {
    // [[2]] thiếu; [[1]] chứa Hán tự sót (rule bắt buộc) → 2 loại lỗi cùng lúc
    const root = storyWithDraft("[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn 高塔 nơi xa.");
    const result = runCheck(root, "0001");
    expect(result.pass).toBe(false);
    expect(result.missing).toEqual([2]);
    expect(result.violations.some((v) => v.message.includes("CJK"))).toBe(true);
    expect(result.reviewPath && existsSync(result.reviewPath)).toBe(true);
    const review = readFileSync(result.reviewPath!, "utf8");
    expect(review).toContain("[[2]] 她沉默了很久没有说话。");   // repair payload đoạn thiếu
    expect(review).toContain("CJK");                            // danh sách vi phạm
    expect(loadState(storyPaths(root)).chapters["0001"]?.reviewRound).toBe(1);
  });

  it("dịch quá ngắn thì fail theo minLengthRatio", () => {
    const root = storyWithDraft("[[1]] Nàng nhìn.\n\n[[2]] Nàng im.");
    const result = runCheck(root, "0001");
    expect(result.pass).toBe(false);
    expect(result.ratio).toBeLessThan(0.75);
  });

  it("quá maxReviewRounds thì chuyển error", () => {
    const root = storyWithDraft("[[1]] 高塔");
    runCheck(root, "0001"); // round 1
    runCheck(root, "0001"); // round 2
    const result = runCheck(root, "0001"); // hết lượt
    expect(result.escalatedToError).toBe(true);
    expect(loadState(storyPaths(root)).chapters["0001"]?.status).toBe("error");
  });

  it("draft mất sạch nhãn → coi như thiếu toàn bộ", () => {
    const root = storyWithDraft("Bản dịch không có nhãn nào cả.");
    const result = runCheck(root, "0001");
    expect(result.missing).toEqual([1, 2]);
  });
});
```

- [ ] **Step 2: Chạy fail** — FAIL.

- [ ] **Step 3: Implement `src/commands/check.ts`**

```ts
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import {
  aiParagraphsOf, labeledAiRepairPayload, parseLabeledAiTranslation,
} from "@/lib/ai-paragraphs";
import {
  buildAiTranslationReviewPrompt, checkAiTranslationViolations,
  type TranslationViolation,
} from "@/lib/ai-translation";
import {
  loadState, loadStoryConfig, readRawChapter, saveState, storyPaths, workFile,
} from "../story-fs.ts";

export interface CheckResult {
  pass: boolean;
  missing: number[];
  violations: TranslationViolation[];
  ratio: number;
  escalatedToError: boolean;
  reviewPath?: string;
}

export function assembleDraft(root: string, id: string): {
  paragraphs: string[];
  parsed: Array<string | undefined>;
  finalText: string;
} {
  const paths = storyPaths(resolve(root));
  const draftPath = workFile(paths, id, "draft");
  if (!existsSync(draftPath)) {
    throw new Error(`Chưa có bản dịch ${draftPath} — agent phải ghi draft trước khi check.`);
  }
  const paragraphs = aiParagraphsOf(readRawChapter(paths, id));
  const parsed =
    parseLabeledAiTranslation(readFileSync(draftPath, "utf8"), paragraphs.length) ??
    Array.from({ length: paragraphs.length }, () => undefined);
  const finalText = parsed
    .filter((paragraph): paragraph is string => paragraph !== undefined)
    .join("\n\n");
  return { paragraphs, parsed, finalText };
}

export function runCheck(root: string, id: string): CheckResult {
  const paths = storyPaths(resolve(root));
  const state = loadState(paths);
  const chapter = state.chapters[id];
  if (!chapter) throw new Error(`Không có chương ${id} trong state.json.`);
  const story = loadStoryConfig(paths);
  const { paragraphs, parsed, finalText } = assembleDraft(root, id);

  const missing = parsed
    .map((paragraph, index) => (paragraph === undefined ? index + 1 : 0))
    .filter((label) => label > 0);
  const violations = checkAiTranslationViolations(finalText, story.checkRules);
  const rawLength = paragraphs.join("").replace(/\s/g, "").length;
  const translatedLength = finalText.replace(/\s/g, "").length;
  const ratio = rawLength > 0 ? translatedLength / rawLength : 1;
  const tooShort = ratio < state.settings.minLengthRatio;
  const pass = missing.length === 0 && violations.length === 0 && !tooShort;

  let escalatedToError = false;
  let reviewPath: string | undefined;

  if (!pass) {
    if (chapter.reviewRound >= state.settings.maxReviewRounds) {
      escalatedToError = true;
      state.chapters[id] = {
        ...chapter,
        status: "error",
        reason: `Quá ${state.settings.maxReviewRounds} vòng review vẫn chưa đạt (thiếu ${missing.length} đoạn, ${violations.length} vi phạm, ratio ${ratio.toFixed(2)}).`,
        updatedAt: Date.now(),
      };
    } else {
      const sections: string[] = [];
      if (missing.length > 0) {
        sections.push(
          "# Đoạn còn thiếu — dịch bổ sung rồi chèn vào work draft\n\n" +
            labeledAiRepairPayload(paragraphs, missing.map((label) => label - 1)),
        );
      }
      if (violations.length > 0) {
        const review = buildAiTranslationReviewPrompt(finalText, violations);
        sections.push(`# Vi phạm rule — sửa tối thiểu\n\n${review.system}\n\n${review.user}`);
      }
      if (tooShort) {
        sections.push(
          `# Bản dịch quá ngắn\n\nTỉ lệ ký tự dịch/raw = ${ratio.toFixed(2)} < ${state.settings.minLengthRatio}. Rà từng đoạn xem có bị tóm tắt/lược ý; dịch đủ 100% nội dung.`,
        );
      }
      reviewPath = workFile(paths, id, "review");
      writeFileSync(reviewPath, `${sections.join("\n\n---\n\n")}\n`, "utf8");
      state.chapters[id] = { ...chapter, reviewRound: chapter.reviewRound + 1, updatedAt: Date.now() };
    }
    saveState(paths, state);
  }

  writeFileSync(
    workFile(paths, id, "check"),
    `${JSON.stringify(
      {
        pass, missing, violationCount: violations.length, ratio,
        reviewRound: (state.chapters[id] ?? chapter).reviewRound,
        checkedAt: Date.now(),
      },
      null, 2,
    )}\n`,
    "utf8",
  );
  return { pass, missing, violations, ratio, escalatedToError, ...(reviewPath ? { reviewPath } : {}) };
}
```

- [ ] **Step 4: Chạy pass** — PASS. Nếu test ratio fail vì bản dịch mẫu quá ngắn/đủ dài ngoài dự kiến, chỉnh **văn bản mẫu trong test**, không chỉnh ngưỡng.

- [ ] **Step 5: Commit (nếu đã được phép)**

```bash
git add apps/qt-ai-cli/src/commands/check.ts apps/qt-ai-cli/test/check.test.ts
git commit -m "feat(qt-ai): lệnh check — bắt thiếu đoạn, rule vi phạm, ratio; sinh review, leo thang error

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 6: `qt-ai accept`

**Files:**
- Create: `apps/qt-ai-cli/src/commands/accept.ts`
- Test: `apps/qt-ai-cli/test/accept.test.ts`

**Interfaces:**
- Consumes: `assembleDraft`, `runCheck` từ `./check.ts`; `formatAiTranslation` từ `@/lib/ai-translation`; `stripAiParagraphMarkers` từ `@/lib/ai-paragraphs`; `collectGlossaryKeys`, `sanitizeExtractedGlossary`, `appendAutoGlossary`, `resolveAutoGlossaryEnabled` từ `@/lib/ai-glossary`; `story-fs.ts`.
- Produces: `export function runAccept(root: string, id: string, options?: { force?: boolean }): { outPath: string; addedGlossary: number }` — chạy lại check bên trong (không tin check.json cũ); fail thì throw trừ khi `force`; ghi `out/<id>.md`; merge glossary đề xuất (mặc định bật, `story.autoGlossary === "off"` thì tắt); xoá `work/<id>.*`; state → `done`.

- [ ] **Step 1: Viết test fail**

`apps/qt-ai-cli/test/accept.test.ts`:

```ts
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { runAccept } from "../src/commands/accept.ts";
import { runInit } from "../src/commands/init.ts";
import { runNext } from "../src/commands/next.ts";
import { loadState, loadStoryConfig, saveStoryConfig, storyPaths, workFile } from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

const RAW = "赵静文抬头看向远方的高塔。\n\n她沉默了很久没有说话。";
const GOOD_DRAFT =
  "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn về phía tòa tháp cao nơi xa.\n\n[[2]] Nàng im lặng hồi lâu không nói lời nào.";

function readyStory(draft: string, glossaryJson?: string): string {
  const root = makeStoryDir({ "0001": RAW });
  runInit(root);
  runNext(root);
  const paths = storyPaths(root);
  writeFileSync(workFile(paths, "0001", "draft"), draft, "utf8");
  if (glossaryJson !== undefined) {
    writeFileSync(workFile(paths, "0001", "glossary"), glossaryJson, "utf8");
  }
  return root;
}

describe("qt-ai accept", () => {
  it("ghi out sạch nhãn, merge glossary hợp lệ, dọn work, state done", () => {
    const root = readyStory(
      GOOD_DRAFT,
      JSON.stringify({
        entries: [
          { source: "赵静文", target: "Triệu Tĩnh Văn", category: "names" },
          { source: "不在raw", target: "Bịa", category: "names" },          // bị sanitize loại
          { source: "高塔", target: "không có trong dịch", category: "places" }, // bị loại
        ],
      }),
    );
    const result = runAccept(root, "0001");
    const paths = storyPaths(root);
    const out = readFileSync(result.outPath, "utf8");
    expect(out).toContain("Triệu Tĩnh Văn ngẩng đầu");
    expect(out).not.toContain("[[");
    expect(result.addedGlossary).toBe(1);
    const story = loadStoryConfig(paths);
    expect(story.glossary.names["赵静文"]).toBe("Triệu Tĩnh Văn");
    expect(story.autoGlossaryLog).toHaveLength(1);
    expect(story.autoGlossaryLog[0]?.chapter).toBe("0001");
    expect(existsSync(workFile(paths, "0001", "draft"))).toBe(false);
    expect(loadState(paths).chapters["0001"]?.status).toBe("done");
  });

  it("check fail thì từ chối, force thì cho qua", () => {
    const root = readyStory("[[1]] Còn 高塔 sót.");
    expect(() => runAccept(root, "0001")).toThrow(/check/i);
    expect(loadState(storyPaths(root)).chapters["0001"]?.status).not.toBe("done");
    const forced = runAccept(root, "0001", { force: true });
    expect(existsSync(forced.outPath)).toBe(true);
  });

  it("autoGlossary off thì không merge nhưng vẫn accept", () => {
    const root = readyStory(
      GOOD_DRAFT,
      JSON.stringify({ entries: [{ source: "赵静文", target: "Triệu Tĩnh Văn", category: "names" }] }),
    );
    const paths = storyPaths(root);
    const config = loadStoryConfig(paths);
    config.autoGlossary = "off";
    saveStoryConfig(paths, config);
    const result = runAccept(root, "0001");
    expect(result.addedGlossary).toBe(0);
    expect(loadStoryConfig(paths).glossary.names).toEqual({});
  });
});
```

- [ ] **Step 2: Chạy fail** — FAIL.

- [ ] **Step 3: Implement `src/commands/accept.ts`**

```ts
import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { stripAiParagraphMarkers } from "@/lib/ai-paragraphs";
import { formatAiTranslation } from "@/lib/ai-translation";
import {
  appendAutoGlossary, collectGlossaryKeys, resolveAutoGlossaryEnabled,
  sanitizeExtractedGlossary,
} from "@/lib/ai-glossary";
import {
  loadState, loadStoryConfig, readRawChapter, saveState, saveStoryConfig,
  storyPaths, workFile, type WorkKind,
} from "../story-fs.ts";
import { assembleDraft, runCheck } from "./check.ts";

const WORK_KINDS: WorkKind[] = ["prompt", "draft", "glossary", "check", "review"];

export function runAccept(
  root: string,
  id: string,
  options?: { force?: boolean },
): { outPath: string; addedGlossary: number } {
  const paths = storyPaths(resolve(root));
  const check = runCheck(root, id);
  if (!check.pass && !options?.force) {
    throw new Error(
      `Chương ${id} chưa qua check (thiếu ${check.missing.length} đoạn, ${check.violations.length} vi phạm, ratio ${check.ratio.toFixed(2)}) — sửa theo work/${id}.review.md hoặc dùng --force.`,
    );
  }

  const { finalText } = assembleDraft(root, id);
  const output = formatAiTranslation(stripAiParagraphMarkers(finalText));
  const outPath = join(paths.outDir, `${id}.md`);
  writeFileSync(outPath, output, "utf8");

  let story = loadStoryConfig(paths);
  let addedGlossary = 0;
  const glossaryPath = workFile(paths, id, "glossary");
  if (existsSync(glossaryPath) && resolveAutoGlossaryEnabled(story.autoGlossary, true)) {
    let entries: unknown = [];
    try {
      const envelope: unknown = JSON.parse(readFileSync(glossaryPath, "utf8"));
      entries =
        typeof envelope === "object" && envelope !== null && "entries" in envelope
          ? (envelope as { entries: unknown }).entries
          : envelope;
    } catch {
      entries = []; // đề xuất hỏng → bỏ qua, không chặn accept
    }
    const raw = readRawChapter(paths, id);
    const pairs = sanitizeExtractedGlossary(
      entries, raw, output, collectGlossaryKeys({}, story.glossary),
    );
    if (pairs.length > 0) {
      story = appendAutoGlossary(story, pairs, id);
      addedGlossary = pairs.length;
    }
  }
  saveStoryConfig(paths, story);

  const state = loadState(paths);
  const chapter = state.chapters[id];
  if (!chapter) throw new Error(`Không có chương ${id} trong state.json.`);
  state.chapters[id] = { status: "done", reviewRound: chapter.reviewRound, updatedAt: Date.now() };
  saveState(paths, state);

  for (const kind of WORK_KINDS) {
    rmSync(workFile(paths, id, kind), { force: true });
  }
  return { outPath, addedGlossary };
}
```

Lưu ý: `runCheck` khi fail sẽ tăng reviewRound/ghi review — với accept `--force` điều đó vô hại vì work files bị dọn ngay sau. Khi check leo thang `error`, accept `--force` vẫn ghi out và chốt `done` (ghi đè trạng thái error) — đúng chủ đích của force.

- [ ] **Step 4: Chạy pass** — PASS.

- [ ] **Step 5: Commit (nếu đã được phép)**

```bash
git add apps/qt-ai-cli/src/commands/accept.ts apps/qt-ai-cli/test/accept.test.ts
git commit -m "feat(qt-ai): lệnh accept — chốt bản dịch, sanitize + merge glossary, dọn work

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 7: `qt-ai skip`, `qt-ai status`, wiring `main.ts`

**Files:**
- Create: `apps/qt-ai-cli/src/commands/skip.ts`
- Create: `apps/qt-ai-cli/src/commands/status.ts`
- Modify: `apps/qt-ai-cli/src/main.ts` (dispatch thật)
- Test: `apps/qt-ai-cli/test/skip-status.test.ts`

**Interfaces:**
- Produces:
  - `export function runSkip(root: string, id: string, reason: string): void` — chỉ skip được chương `queued`/`translating`/`error`; dọn `work/<id>.*`; reason bắt buộc không rỗng.
  - `export function runStatus(root: string): string` — chuỗi nhiều dòng: đếm theo status + liệt kê từng chương `error`/`skipped` kèm reason + nhắc `chaptersPerSession`.
  - `main(argv)` dispatch đủ 6 lệnh, in message/kết quả ra stdout, lỗi ra stderr, exit code: 0 thành công/check pass, 1 check fail còn lượt, 2 lỗi (throw).

- [ ] **Step 1: Viết test fail**

`apps/qt-ai-cli/test/skip-status.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import { runInit } from "../src/commands/init.ts";
import { runNext } from "../src/commands/next.ts";
import { runSkip } from "../src/commands/skip.ts";
import { runStatus } from "../src/commands/status.ts";
import { loadState, storyPaths } from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

describe("qt-ai skip + status", () => {
  it("skip chương translating kèm lý do, next đi tiếp chương sau", () => {
    const root = makeStoryDir({ "0001": "第一章", "0002": "第二章" });
    runInit(root);
    runNext(root);
    runSkip(root, "0001", "model từ chối nội dung");
    const state = loadState(storyPaths(root));
    expect(state.chapters["0001"]?.status).toBe("skipped");
    expect(state.chapters["0001"]?.reason).toBe("model từ chối nội dung");
    expect(runNext(root).chapterId).toBe("0002");
  });

  it("skip đòi reason", () => {
    const root = makeStoryDir({ "0001": "第一章" });
    runInit(root);
    expect(() => runSkip(root, "0001", "  ")).toThrow(/reason|lý do/i);
  });

  it("status tổng hợp đủ trạng thái", () => {
    const root = makeStoryDir({ "0001": "第一章", "0002": "第二章" });
    runInit(root);
    runNext(root);
    runSkip(root, "0001", "thử");
    const report = runStatus(root);
    expect(report).toContain("queued: 1");
    expect(report).toContain("skipped: 1");
    expect(report).toContain("0001");
    expect(report).toContain("thử");
  });
});
```

- [ ] **Step 2: Chạy fail** — FAIL.

- [ ] **Step 3: Implement**

`src/commands/skip.ts`:

```ts
import { rmSync } from "node:fs";
import { resolve } from "node:path";
import { loadState, saveState, storyPaths, workFile, type WorkKind } from "../story-fs.ts";

const WORK_KINDS: WorkKind[] = ["prompt", "draft", "glossary", "check", "review"];

export function runSkip(root: string, id: string, reason: string): void {
  if (!reason.trim()) throw new Error("skip cần --reason <lý do> không rỗng.");
  const paths = storyPaths(resolve(root));
  const state = loadState(paths);
  const chapter = state.chapters[id];
  if (!chapter) throw new Error(`Không có chương ${id} trong state.json.`);
  if (chapter.status === "done") throw new Error(`Chương ${id} đã done, không skip được.`);
  state.chapters[id] = { ...chapter, status: "skipped", reason: reason.trim(), updatedAt: Date.now() };
  saveState(paths, state);
  for (const kind of WORK_KINDS) rmSync(workFile(paths, id, kind), { force: true });
}
```

`src/commands/status.ts`:

```ts
import { resolve } from "node:path";
import { naturalChapterCompare } from "@/lib/ai-story";
import { loadState, storyPaths, type ChapterStatus } from "../story-fs.ts";

export function runStatus(root: string): string {
  const state = loadState(storyPaths(resolve(root)));
  const counts: Record<ChapterStatus, number> = {
    queued: 0, translating: 0, done: 0, error: 0, skipped: 0,
  };
  const flagged: string[] = [];
  const ids = Object.keys(state.chapters).sort(naturalChapterCompare);
  for (const id of ids) {
    const chapter = state.chapters[id]!;
    counts[chapter.status] += 1;
    if (chapter.status === "error" || chapter.status === "skipped") {
      flagged.push(`  ${id} [${chapter.status}] ${chapter.reason ?? ""}`.trimEnd());
    }
    if (chapter.status === "translating") flagged.push(`  ${id} [translating] — đang dở, check/accept/skip trước`);
  }
  const lines = [
    `Tổng ${ids.length} chương — done: ${counts.done}, queued: ${counts.queued}, translating: ${counts.translating}, error: ${counts.error}, skipped: ${counts.skipped}`,
  ];
  if (flagged.length > 0) lines.push("Cần chú ý:", ...flagged);
  lines.push(`Giới hạn phiên: dịch tối đa ${state.settings.chaptersPerSession} chương/phiên rồi nghỉ.`);
  return lines.join("\n");
}
```

`src/main.ts` (thay toàn bộ):

```ts
import { runAccept } from "./commands/accept.ts";
import { runCheck } from "./commands/check.ts";
import { runInit } from "./commands/init.ts";
import { runNext } from "./commands/next.ts";
import { runSkip } from "./commands/skip.ts";
import { runStatus } from "./commands/status.ts";

const USAGE = `qt-ai <lệnh> <thư-mục-truyện> [chương] [cờ]

Lệnh:
  init <root>                        Dựng khung folder truyện + copy AGENTS.md/workflows
  next <root>                        Phát chương kế tiếp, lắp prompt vào work/
  check <root> <id>                  Kiểm tra bản dịch work/<id>.draft.md
  accept <root> <id> [--force]       Chốt chương: ghi out/, merge glossary
  skip <root> <id> --reason <lý do>  Bỏ qua chương (model từ chối...)
  status <root>                      Bảng tiến độ`;

export function main(argv: string[]): number {
  const [command, root, ...rest] = argv;
  try {
    switch (command) {
      case "init": {
        if (!root) break;
        console.log(runInit(root));
        return 0;
      }
      case "next": {
        if (!root) break;
        const result = runNext(root);
        console.log(`Chương ${result.chapterId} → đọc prompt tại: ${result.promptPath}`);
        return 0;
      }
      case "check": {
        const id = rest[0];
        if (!root || !id) break;
        const result = runCheck(root, id);
        if (result.pass) {
          console.log(`Chương ${id} PASS (ratio ${result.ratio.toFixed(2)}) — chạy accept.`);
          return 0;
        }
        if (result.escalatedToError) {
          console.error(`Chương ${id} quá số vòng review → error. Xem qt-ai status.`);
          return 2;
        }
        console.error(
          `Chương ${id} FAIL: thiếu ${result.missing.length} đoạn, ${result.violations.length} vi phạm, ratio ${result.ratio.toFixed(2)}.\nSửa theo: ${result.reviewPath}`,
        );
        return 1;
      }
      case "accept": {
        const id = rest[0];
        if (!root || !id) break;
        const result = runAccept(root, id, { force: rest.includes("--force") });
        console.log(`Đã chốt ${result.outPath} (+${result.addedGlossary} glossary mới).`);
        return 0;
      }
      case "skip": {
        const id = rest[0];
        const reasonIndex = rest.indexOf("--reason");
        const reason = reasonIndex >= 0 ? rest.slice(reasonIndex + 1).join(" ") : "";
        if (!root || !id) break;
        runSkip(root, id, reason);
        console.log(`Đã skip chương ${id}.`);
        return 0;
      }
      case "status": {
        if (!root) break;
        console.log(runStatus(root));
        return 0;
      }
    }
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    return 2;
  }
  console.error(USAGE);
  return 2;
}

if (import.meta.url === `file://${process.argv[1]}`) {
  process.exit(main(process.argv.slice(2)));
}
```

- [ ] **Step 4: Chạy pass + smoke thủ công**

Run: `npm --prefix apps/qt-ai-cli test && npm --prefix apps/qt-ai-cli run typecheck`
Run thêm: `npm --prefix apps/qt-ai-cli run -s qt-ai -- status /tmp/không-tồn-tại`
Expected: test PASS; lệnh smoke in message lỗi tiếng Việt, exit 2.

- [ ] **Step 5: Commit (nếu đã được phép)**

```bash
git add apps/qt-ai-cli/src apps/qt-ai-cli/test/skip-status.test.ts
git commit -m "feat(qt-ai): skip/status + dispatch CLI hoàn chỉnh

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 8: Template AGENTS.md + workflows cho Antigravity

**Files:**
- Modify: `apps/qt-ai-cli/antigravity/AGENTS.md` (thay placeholder bằng bản đầy đủ)
- Modify: `apps/qt-ai-cli/antigravity/workflows/setup-story.md`
- Modify: `apps/qt-ai-cli/antigravity/workflows/translate.md`
- Test: `apps/qt-ai-cli/test/init.test.ts` (bổ sung assert nội dung)

**Interfaces:**
- Consumes: cơ chế render `{{QT_AI}}`/`{{STORY_ROOT}}` của Task 3.
- Produces: template hoàn chỉnh; không đổi API code nào.

- [ ] **Step 1: Bổ sung test fail** — thêm vào `describe` trong `init.test.ts`:

```ts
  it("template đầy đủ: có vòng lặp translate và luật vệ sinh context", () => {
    const root = makeStoryDir({ "0001": "第一章" });
    runInit(root);
    const translate = readFileSync(join(root, ".agent/workflows/translate.md"), "utf8");
    expect(translate).toContain("next");
    expect(translate).toContain("check");
    expect(translate).toContain("accept");
    expect(translate).toContain("skip");
    const agents = readFileSync(join(root, "AGENTS.md"), "utf8");
    expect(agents).toContain("không đọc lại out/");
    expect(agents).toContain("chương/phiên");
  });
```

- [ ] **Step 2: Chạy fail** — FAIL.

- [ ] **Step 3: Viết template thật**

`apps/qt-ai-cli/antigravity/AGENTS.md`:

```markdown
# Workspace dịch truyện — điều khiển bằng CLI qt-ai

Thư mục truyện: `{{STORY_ROOT}}`
Chạy CLI (dùng NGUYÊN VĂN, đừng tự chế đường dẫn khác):

    {{QT_AI}} <lệnh> {{STORY_ROOT}} [chương] [cờ]

Lệnh: `init` · `next` · `check <id>` · `accept <id> [--force]` · `skip <id> --reason <lý do>` · `status`

## Luật bắt buộc

1. **Mọi tiến độ nằm trong file, không nằm trong trí nhớ của mày.** Bắt đầu phiên bằng `status`. Không bao giờ tự sửa `state.json`, `story.json` bằng tay — chỉ qua lệnh CLI (trừ workflow /setup-story được phép điền `story.json`).
2. **Dịch đúng một chương một lúc** theo vòng lặp trong `.agent/workflows/translate.md`. Không dịch gộp, không nhảy chương.
3. **Vệ sinh context:** không đọc lại out/ của các chương đã xong; sau khi `accept`, quên nội dung chương đó đi; chỉ giữ trong đầu chương đang dịch.
4. **Giới hạn phiên:** dịch tối đa số chương/phiên ghi trong dòng cuối của `status` (mặc định 10). Đủ số thì dừng, chạy `status`, báo người dùng mở phiên mới.
5. **Bản dịch phải đủ 100% số đoạn, giữ nhãn [[n]].** `check` sẽ bắt lỗi thiếu — sửa theo `work/<id>.review.md` chứ không cãi.
6. **Nếu bị chính sách nội dung chặn không dịch được chương nào:** KHÔNG chế lại nội dung, không tóm tắt thay thế. Chạy `skip <id> --reason "model từ chối: <mô tả ngắn>"` rồi sang chương kế.
7. Không sửa file trong `raw/`. Không xoá gì trong `out/`.
```

`apps/qt-ai-cli/antigravity/workflows/setup-story.md`:

```markdown
# /setup-story — điền hồ sơ truyện trước khi dịch

1. Chạy `{{QT_AI}} init {{STORY_ROOT}}` (an toàn chạy lại).
2. Đọc 2–3 chương đầu trong `raw/` (chỉ để nắm truyện, không dịch).
3. Mở `story.json`, điền bằng tiếng Việt:
   - `name`, `protagonist`, `summary` (3–5 câu, không spoil quá chương đã đọc).
   - `style.voice` (1 câu tả giọng kể), `style.toneRules` (3–5 luật xưng hô/giọng điệu rút từ chính truyện),
     `style.avoid` (những kiểu diễn đạt cần tránh với truyện này).
   - `glossary`: seed các tên riêng gặp trong các chương đã đọc vào đúng nhóm
     (`names`, `places`, `items`, `creatures`, `skills`, `common`, `signature_phrases`) — source chữ Hán, target Hán-Việt.
   - Giữ nguyên các field khác (`customPrompt`, `checkRules`, `autoGlossaryLog`, `autoGlossary`).
4. Trình `story.json` cho người dùng duyệt trước khi chạy /translate.
```

`apps/qt-ai-cli/antigravity/workflows/translate.md`:

```markdown
# /translate — vòng lặp dịch batch

Lặp cho tới khi hết chương hoặc chạm giới hạn chương/phiên (xem AGENTS.md luật 4):

1. `{{QT_AI}} next {{STORY_ROOT}}` → nhận id chương + đường dẫn `work/<id>.prompt.md`.
   - Nếu báo còn chương translating dở: xử lý chương đó trước (bước 2–5) thay vì lấy chương mới.
   - Nếu báo hết hàng đợi: chạy `status`, tổng kết cho người dùng, dừng.
2. Đọc TOÀN BỘ `work/<id>.prompt.md` và làm đúng theo nó: dịch, ghi `work/<id>.draft.md`
   (giữ nhãn [[n]]), ghi `work/<id>.glossary.json`.
3. `{{QT_AI}} check {{STORY_ROOT}} <id>`
   - FAIL còn lượt sửa: đọc `work/<id>.review.md`, sửa đúng chỗ trong `work/<id>.draft.md`
     (dịch bổ sung đoạn thiếu / thay cụm vi phạm, KHÔNG viết lại chỗ khác), rồi chạy lại bước 3.
   - Báo "quá số vòng review → error": bỏ chương này, quay lại bước 1.
4. `{{QT_AI}} accept {{STORY_ROOT}} <id>` — không tự ý dùng `--force`; force là quyết định của người dùng.
5. Báo một dòng tiến độ (`x/y chương của phiên`) rồi quay lại bước 1.

Model từ chối dịch vì chính sách nội dung → làm theo AGENTS.md luật 6 (skip kèm lý do).
```

- [ ] **Step 4: Chạy pass** — `npm --prefix apps/qt-ai-cli test` → PASS.

- [ ] **Step 5: Commit (nếu đã được phép)**

```bash
git add apps/qt-ai-cli/antigravity apps/qt-ai-cli/test/init.test.ts
git commit -m "feat(qt-ai): template AGENTS.md + workflows setup-story/translate cho Antigravity

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 9: E2E dry-run + prompt parity

**Files:**
- Test: `apps/qt-ai-cli/test/e2e.test.ts`

**Interfaces:**
- Consumes: mọi lệnh đã có. Không tạo API mới.

- [ ] **Step 1: Viết test (fail nếu bất kỳ mắt xích nào hở)**

`apps/qt-ai-cli/test/e2e.test.ts`:

```ts
import { readFileSync, writeFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { buildAiTranslationSystemPrompt } from "@/lib/ai-translation";
import { runAccept } from "../src/commands/accept.ts";
import { runCheck } from "../src/commands/check.ts";
import { runInit } from "../src/commands/init.ts";
import { runNext } from "../src/commands/next.ts";
import { runStatus } from "../src/commands/status.ts";
import { loadStoryConfig, readRawChapter, storyPaths, workFile } from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

const CH1 = "赵静文抬头看向远方的高塔。\n\n她沉默了很久没有说话。";
const CH2 = "第二天早上他们出发了。";
const DRAFTS: Record<string, string> = {
  "0001":
    "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn về phía tòa tháp cao nơi xa.\n\n[[2]] Nàng im lặng hồi lâu không nói lời nào.",
  "0002": "[[1]] Sáng sớm hôm sau bọn họ liền lên đường xuất phát.",
};

describe("e2e: hai chương liên tiếp, glossary học từ chương 1 lọt vào prompt chương 2", () => {
  it("chạy trọn vòng next→draft→check→accept cho cả truyện", () => {
    const root = makeStoryDir({ "0001": CH1, "0002": CH2 });
    runInit(root);
    const paths = storyPaths(root);

    for (const id of ["0001", "0002"]) {
      const next = runNext(root);
      expect(next.chapterId).toBe(id);
      writeFileSync(workFile(paths, id, "draft"), DRAFTS[id]!, "utf8");
      if (id === "0001") {
        writeFileSync(
          workFile(paths, id, "glossary"),
          JSON.stringify({ entries: [{ source: "赵静文", target: "Triệu Tĩnh Văn", category: "names" }] }),
          "utf8",
        );
      }
      expect(runCheck(root, id).pass).toBe(true);
      runAccept(root, id);
    }

    expect(runStatus(root)).toContain("done: 2");
    expect(readFileSync(`${paths.outDir}/0001.md`, "utf8")).toContain("Triệu Tĩnh Văn");
    expect(loadStoryConfig(paths).glossary.names["赵静文"]).toBe("Triệu Tĩnh Văn");
    expect(() => runNext(root)).toThrow(/không còn chương/i);
  });

  it("prompt parity: prompt CLI lắp = buildAiTranslationSystemPrompt của web với cùng config", () => {
    const root = makeStoryDir({ "0001": CH1 });
    runInit(root);
    const paths = storyPaths(root);
    const next = runNext(root);
    const cliPrompt = readFileSync(next.promptPath, "utf8");
    const webPrompt = buildAiTranslationSystemPrompt(
      {}, loadStoryConfig(paths), readRawChapter(paths, "0001"),
    );
    expect(cliPrompt.startsWith(webPrompt)).toBe(true); // phần system y hệt web, CLI chỉ nối thêm payload + chỉ dẫn
  });
});
```

- [ ] **Step 2: Chạy** — `npm --prefix apps/qt-ai-cli test` → Expected: PASS thẳng nếu Task 1–8 đúng; fail chỗ nào sửa chỗ đó (đây là lưới an toàn cuối).

- [ ] **Step 3: Chạy toàn bộ verify**

Run: `npm --prefix apps/qt-ai-cli test && npm --prefix apps/qt-ai-cli run typecheck && npm --prefix apps/qt-web test`
Expected: cả ba PASS — dòng cuối chứng minh qt-web không bị ảnh hưởng.

- [ ] **Step 4: Commit (nếu đã được phép)**

```bash
git add apps/qt-ai-cli/test/e2e.test.ts
git commit -m "test(qt-ai): e2e dry-run 2 chương + prompt parity với qt-web

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

## Sau plan (thủ công, không phải task code)

1. Dry-run người thật: dựng folder truyện từ corpus `phuong-thon-dao-chu`, Claude đóng vai agent chạy 2–3 chương thật.
2. Pilot trên Antigravity: mở folder truyện, `/setup-story` → user duyệt → `/translate`; đo chương/phiên, tỉ lệ fail check, tỉ lệ refuse.
3. Chỉnh `chaptersPerSession`/`minLengthRatio` trong `state.json` và câu chữ AGENTS.md theo số đo.
