# Export glossary ra Names.txt Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Tab Glossary của Hồ sơ truyện có nút mở dialog chọn mục glossary rồi chép/lưu ra file `Hán=Việt` (Names.txt hoặc Names2.txt) cho bản convert QT.

**Architecture:** Hàm thuần trong `lib/names-export.ts` lập danh sách dòng (kèm lý do bỏ) và render text; dialog React giữ tập id đã tick, xem trước live; lưu file qua lệnh Tauri nhỏ `write_text_file` (không thêm plugin fs). Spec: `docs/superpowers/specs/2026-09-16-qt-ai-gui-glossary-names-export-design.md`.

**Tech Stack:** React 19 + react-hook-form + zod + vitest/testing-library (apps/qt-ai-gui), Tauri 2 Rust command (apps/qt-ai-gui/src-tauri), tailwind + shadcn primitives có sẵn.

## Global Constraints

- Không ghi IP thật hay API key vào code/test/doc. Không commit `deploy/aws-lambda/DEPLOY_COMMANDS.md`.
- Chỉ commit khi người dùng bảo ("commit đi"); commit message tiếng Việt, kết bằng `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>`.
- Giao diện tiếng Việt, xưng hô như các dialog hiện có. Màu qua theme token (test `no-hardcoded-colors` sẽ bắt class màu cứng).
- Nhóm `addressing` không hiện, không export. Mặc định không tick `signature_phrases`.
- File: CRLF, UTF-8 có BOM; clipboard không BOM. Bỏ dòng rỗng / có `=` / nhiều dòng / trùng source.
- Lệnh chạy trong `apps/qt-ai-gui`: `npx vitest run <file>`, `npm run typecheck`, `npm run lint`; Rust: `cargo test -p qt-ai-gui` trong `apps/qt-ai-gui/src-tauri`.

---

### Task 1: Hàm thuần `names-export.ts`

**Files:**
- Create: `apps/qt-ai-gui/src/lib/names-export.ts`
- Test: `apps/qt-ai-gui/src/lib/names-export.test.ts`

**Interfaces:**
- Consumes: `Pair { source: string; target: string }` và `StoryFormValues["glossary"]` từ `@/lib/story-form`; `GLOSSARY_LABELS`, `GlossaryKey` từ `@/lib/types`.
- Produces:
  - `NAMES_EXPORT_KEYS: readonly NamesExportKey[]` (7 nhóm, không có addressing).
  - `type NamesExportKey = Exclude<GlossaryKey, "addressing">`.
  - `interface NamesExportRow { id: string; group: NamesExportKey; source: string; target: string; skip?: string }`.
  - `planNamesExport(glossary: StoryFormValues["glossary"]): NamesExportRow[]`.
  - `defaultSelection(rows: NamesExportRow[]): Set<string>`.
  - `renderNames(rows: NamesExportRow[], selected: ReadonlySet<string>, options?: { bom?: boolean }): string`.
  - `NAMES_FILE_NAMES = ["Names.txt", "Names2.txt"] as const; type NamesFileName`.

- [x] **Step 1: Viết test thất bại**

```ts
// apps/qt-ai-gui/src/lib/names-export.test.ts
import { describe, expect, it } from "vitest";

import { defaultSelection, NAMES_EXPORT_KEYS, planNamesExport, renderNames } from "@/lib/names-export";
import type { StoryFormValues } from "@/lib/story-form";

const empty = (): StoryFormValues["glossary"] => ({
  names: [], places: [], items: [], creatures: [], skills: [], common: [], signature_phrases: [], addressing: [],
});

describe("planNamesExport", () => {
  it("liệt kê theo thứ tự nhóm rồi thứ tự dòng, trim hai cột, bỏ addressing", () => {
    const g = empty();
    g.places = [{ source: " 京城 ", target: " Kinh Thành " }];
    g.names = [{ source: "赵静文", target: "Triệu Tĩnh Văn" }];
    g.addressing = [{ source: "甲→乙", target: "ta–ngươi" }];
    const rows = planNamesExport(g);
    expect(rows.map((r) => [r.id, r.source, r.target])).toEqual([
      ["names:0", "赵静文", "Triệu Tĩnh Văn"],
      ["places:0", "京城", "Kinh Thành"],
    ]);
    expect(rows.every((r) => r.skip === undefined)).toBe(true);
    expect(NAMES_EXPORT_KEYS).not.toContain("addressing");
  });

  it("đánh dấu bỏ: thiếu Hán/Việt, có dấu =, nhiều dòng", () => {
    const g = empty();
    g.names = [
      { source: "  ", target: "x" },
      { source: "甲", target: "" },
      { source: "a=b", target: "c" },
      { source: "d", target: "e=f" },
      { source: "多\n行", target: "g" },
    ];
    expect(planNamesExport(g).map((r) => r.skip)).toEqual(["thiếu Hán/Việt", "thiếu Hán/Việt", "có dấu =", "có dấu =", "nhiều dòng"]);
  });

  it("trùng source: dòng đầu giữ, dòng sau bỏ kèm nhãn nhóm đã giữ (cùng nhóm hoặc khác nhóm)", () => {
    const g = empty();
    g.names = [{ source: "赵静文", target: "A" }, { source: " 赵静文", target: "B" }];
    g.common = [{ source: "赵静文", target: "C" }];
    const rows = planNamesExport(g);
    expect(rows[0].skip).toBeUndefined();
    expect(rows[1].skip).toBe("trùng nhóm Tên nhân vật");
    expect(rows[2].skip).toBe("trùng nhóm Tên nhân vật");
  });
});

describe("defaultSelection", () => {
  it("tick mọi dòng hợp lệ trừ signature_phrases và dòng bị bỏ", () => {
    const g = empty();
    g.names = [{ source: "甲", target: "A" }, { source: "甲", target: "B" }];
    g.signature_phrases = [{ source: "乙", target: "C" }];
    const rows = planNamesExport(g);
    expect([...defaultSelection(rows)]).toEqual(["names:0"]);
  });
});

describe("renderNames", () => {
  it("chỉ dòng đã tick và không bị bỏ, CRLF, BOM tuỳ chọn", () => {
    const g = empty();
    g.names = [{ source: "甲", target: "A" }, { source: "乙", target: "B" }, { source: "甲", target: "C" }];
    const rows = planNamesExport(g);
    const selected = new Set(["names:0", "names:1", "names:2"]);
    expect(renderNames(rows, selected)).toBe("甲=A\r\n乙=B\r\n");
    expect(renderNames(rows, selected, { bom: true })).toBe("\ufeff甲=A\r\n乙=B\r\n");
    expect(renderNames(rows, new Set(["names:1"]))).toBe("乙=B\r\n");
    expect(renderNames(rows, new Set())).toBe("");
  });
});
```

- [x] **Step 2: Chạy test, phải fail vì thiếu module**

Run: `cd apps/qt-ai-gui && npx vitest run src/lib/names-export.test.ts`
Expected: FAIL "Failed to resolve import "@/lib/names-export""

- [x] **Step 3: Viết module**

```ts
// apps/qt-ai-gui/src/lib/names-export.ts
import type { StoryFormValues } from "@/lib/story-form";
import { GLOSSARY_LABELS, type GlossaryKey } from "@/lib/types";

/** Nhóm được export, theo thứ tự ghi file. `addressing` (khoá `甲→乙`) không phải cụm Hán nên không có mặt. */
export type NamesExportKey = Exclude<GlossaryKey, "addressing">;
export const NAMES_EXPORT_KEYS: readonly NamesExportKey[] = [
  "names",
  "places",
  "items",
  "creatures",
  "skills",
  "common",
  "signature_phrases",
];
/** Nhóm không tick sẵn: cụm đặc trưng là câu/thành ngữ, hiếm khi muốn thành tên trong bản convert. */
const DEFAULT_UNCHECKED: readonly NamesExportKey[] = ["signature_phrases"];

export const NAMES_FILE_NAMES = ["Names.txt", "Names2.txt"] as const;
export type NamesFileName = (typeof NAMES_FILE_NAMES)[number];

export interface NamesExportRow {
  /** `${group}:${index}` — ổn định trong một lần mở dialog. */
  id: string;
  group: NamesExportKey;
  source: string;
  target: string;
  /** Có giá trị = dòng không export được, kèm lý do ngắn để hiện. */
  skip?: string;
}

const BOM = "\ufeff";
const EOL = "\r\n";

/**
 * Lập danh sách dòng cho dialog: trim hai cột; đánh dấu bỏ theo cách QT đọc Names.txt
 * (tách mọi `=`, mỗi dòng một mục, khoá trùng chỉ giữ dòng đầu — xem docs/engine/dictionaries.md).
 */
export function planNamesExport(glossary: StoryFormValues["glossary"]): NamesExportRow[] {
  const rows: NamesExportRow[] = [];
  const seen = new Map<string, NamesExportKey>();
  for (const group of NAMES_EXPORT_KEYS) {
    glossary[group].forEach((pair, index) => {
      const source = pair.source.trim();
      const target = pair.target.trim();
      const row: NamesExportRow = { id: `${group}:${index}`, group, source, target };
      if (!source || !target) row.skip = "thiếu Hán/Việt";
      else if (source.includes("=") || target.includes("=")) row.skip = "có dấu =";
      else if (/[\r\n]/.test(source) || /[\r\n]/.test(target)) row.skip = "nhiều dòng";
      else {
        const kept = seen.get(source);
        if (kept) row.skip = `trùng nhóm ${GLOSSARY_LABELS[kept]}`;
        else seen.set(source, group);
      }
      rows.push(row);
    });
  }
  return rows;
}

/** Tick sẵn mọi dòng hợp lệ của các nhóm không nằm trong DEFAULT_UNCHECKED. */
export function defaultSelection(rows: NamesExportRow[]): Set<string> {
  return new Set(rows.filter((r) => !r.skip && !DEFAULT_UNCHECKED.includes(r.group)).map((r) => r.id));
}

/** Nội dung file: mỗi dòng `Hán=Việt` CRLF; `bom` cho file ghi ra đĩa (QT ghi có BOM), clipboard thì không. */
export function renderNames(
  rows: NamesExportRow[],
  selected: ReadonlySet<string>,
  options: { bom?: boolean } = {},
): string {
  const body = rows
    .filter((r) => !r.skip && selected.has(r.id))
    .map((r) => `${r.source}=${r.target}${EOL}`)
    .join("");
  if (!body) return "";
  return options.bom ? BOM + body : body;
}
```

- [x] **Step 4: Chạy test, phải pass**

Run: `cd apps/qt-ai-gui && npx vitest run src/lib/names-export.test.ts`
Expected: 5 passed

- [x] **Step 5: Không commit (đợi người dùng). Ghi nhớ file để commit chung cuối.**

---

### Task 2: Lệnh Rust `write_text_file` + API TS

**Files:**
- Modify: `apps/qt-ai-gui/src-tauri/src/story_cmds.rs` (thêm command sau `reveal_folder`, test trong `mod tests`)
- Modify: `apps/qt-ai-gui/src-tauri/src/lib.rs:108` (đăng ký sau `story_cmds::reveal_folder`)
- Modify: `apps/qt-ai-gui/src/lib/api.ts` (thêm `writeTextFile`, `pickSaveFile` nhận title)

**Interfaces:**
- Produces: Rust `write_text_file(path: String, content: String) -> CmdResult<()>`; TS `writeTextFile(path: string, content: string): Promise<void>`; `pickSaveFile(defaultName: string, title = "Lưu file gộp"): Promise<string | undefined>`.

- [x] **Step 1: Viết test Rust thất bại**

Thêm vào cuối `mod tests` trong `story_cmds.rs`:

```rust
    #[test]
    fn write_text_file_ghi_nguyen_byte_va_tao_folder_cha() {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("sub").join("Names.txt");
        write_text_file(out.display().to_string(), "\u{feff}甲=A\r\n".to_string()).unwrap();
        assert_eq!(fs::read(&out).unwrap(), "\u{feff}甲=A\r\n".as_bytes());
    }

    #[test]
    fn write_text_file_bao_loi_io_khi_path_la_folder() {
        let dir = tempfile::tempdir().unwrap();
        let err = write_text_file(dir.path().display().to_string(), "x".to_string()).unwrap_err();
        assert_eq!(err.kind, "io");
    }
```

- [x] **Step 2: Chạy, phải fail vì chưa có hàm**

Run: `cd apps/qt-ai-gui/src-tauri && cargo test -p qt-ai-gui write_text_file`
Expected: error[E0425]: cannot find function `write_text_file`

- [x] **Step 3: Thêm command (sync, ghi file nhỏ nên không cần `blocking`) ngay sau `reveal_folder`**

```rust
/// Ghi nguyên `content` (BOM nếu có đã nằm trong chuỗi) ra `path`, tạo folder cha nếu thiếu.
/// Dùng cho export glossary → Names.txt; file nhỏ nên ghi thẳng, không qua spawn_blocking.
#[tauri::command]
pub fn write_text_file(path: String, content: String) -> CmdResult<()> {
    let target = Path::new(&path);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| CommandError::new("io", format!("Không tạo được folder {}: {e}", parent.display())))?;
    }
    std::fs::write(target, content.as_bytes())
        .map_err(|e| CommandError::new("io", format!("Không ghi được {path}: {e}")))
}
```

Đăng ký trong `lib.rs` sau `story_cmds::reveal_folder,`:

```rust
            story_cmds::write_text_file,
```

- [x] **Step 4: Chạy test, phải pass**

Run: `cd apps/qt-ai-gui/src-tauri && cargo test -p qt-ai-gui write_text_file`
Expected: 2 passed

- [x] **Step 5: TS API**

Trong `apps/qt-ai-gui/src/lib/api.ts`, sửa `pickSaveFile` và thêm `writeTextFile` ngay sau:

```ts
export async function pickSaveFile(defaultName: string, title = "Lưu file gộp"): Promise<string | undefined> {
  const selected = await save({
    title,
    defaultPath: defaultName,
    filters: [{ name: "Văn bản UTF-8", extensions: ["txt"] }],
  });
  return selected ?? undefined;
}

/** Ghi nguyên chuỗi ra file (BOM nếu cần phải nằm sẵn trong `content`). */
export const writeTextFile = (path: string, content: string) =>
  call("write_text_file", { path, content }, () => undefined);
```

- [x] **Step 6: Typecheck**

Run: `cd apps/qt-ai-gui && npm run typecheck`
Expected: không lỗi

---

### Task 3: `GlossaryExportDialog`

**Files:**
- Create: `apps/qt-ai-gui/src/components/glossary-export-dialog.tsx`
- Test: `apps/qt-ai-gui/src/components/glossary-export-dialog.test.tsx`

**Interfaces:**
- Consumes: Task 1 (`planNamesExport`, `defaultSelection`, `renderNames`, `NAMES_EXPORT_KEYS`, `NAMES_FILE_NAMES`, `NamesFileName`, `NamesExportRow`), Task 2 (`pickSaveFile`, `writeTextFile`), `copyText` từ `@/lib/clipboard`, `Choice`, Dialog primitives, `GLOSSARY_LABELS`.
- Produces: `GlossaryExportDialog({ open, onOpenChange, glossary }: { open: boolean; onOpenChange: (open: boolean) => void; glossary: StoryFormValues["glossary"] })`.

- [x] **Step 1: Viết test thất bại**

```tsx
// apps/qt-ai-gui/src/components/glossary-export-dialog.test.tsx
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { GlossaryExportDialog } from "@/components/glossary-export-dialog";
import { pickSaveFile, writeTextFile } from "@/lib/api";
import { copyText } from "@/lib/clipboard";
import type { StoryFormValues } from "@/lib/story-form";

vi.mock("@/lib/api", () => ({ pickSaveFile: vi.fn(), writeTextFile: vi.fn() }));
vi.mock("@/lib/clipboard", () => ({ copyText: vi.fn() }));

const glossary = (): StoryFormValues["glossary"] => ({
  names: [{ source: "赵静文", target: "Triệu Tĩnh Văn" }, { source: "a=b", target: "x" }],
  places: [{ source: "京城", target: "Kinh Thành" }],
  items: [],
  creatures: [],
  skills: [],
  common: [],
  signature_phrases: [{ source: "天道酬勤", target: "Thiên đạo thù cần" }],
  addressing: [{ source: "甲→乙", target: "ta–ngươi" }],
});

function preview(): HTMLTextAreaElement {
  return screen.getByLabelText("Xem trước") as HTMLTextAreaElement;
}

describe("GlossaryExportDialog", () => {
  beforeEach(() => {
    vi.mocked(pickSaveFile).mockReset();
    vi.mocked(writeTextFile).mockReset().mockResolvedValue(undefined);
    vi.mocked(copyText).mockReset().mockResolvedValue(undefined);
  });

  it("mặc định tick hết trừ Cụm từ đặc trưng; addressing không hiện; dòng có = bị khoá kèm lý do", () => {
    render(<GlossaryExportDialog open onOpenChange={vi.fn()} glossary={glossary()} />);
    expect(preview().value).toBe("赵静文=Triệu Tĩnh Văn\r\n京城=Kinh Thành\r\n");
    expect(screen.getByText(/Sẽ ghi/)).toHaveTextContent("Sẽ ghi 2 dòng · bỏ 1 dòng");
    expect(screen.queryByText("Xưng hô theo cặp")).not.toBeInTheDocument();
    expect(screen.queryByText("甲→乙")).not.toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: "Cụm từ đặc trưng" })).not.toBeChecked();
    const bad = screen.getByRole("checkbox", { name: "a=b = x" });
    expect(bad).toBeDisabled();
    expect(screen.getByText("có dấu =")).toBeInTheDocument();
  });

  it("tick nhóm và tick dòng đổi xem trước; bỏ hết thì khoá nút", async () => {
    const user = userEvent.setup();
    render(<GlossaryExportDialog open onOpenChange={vi.fn()} glossary={glossary()} />);
    await user.click(screen.getByRole("checkbox", { name: "Cụm từ đặc trưng" }));
    expect(preview().value).toContain("天道酬勤=Thiên đạo thù cần\r\n");
    await user.click(screen.getByRole("checkbox", { name: "赵静文 = Triệu Tĩnh Văn" }));
    expect(preview().value).not.toContain("赵静文");
    const names = screen.getByRole("checkbox", { name: "Tên nhân vật" }) as HTMLInputElement;
    expect(names.indeterminate).toBe(false);
    expect(names).not.toBeChecked();
    await user.click(screen.getByRole("checkbox", { name: "Địa danh" }));
    await user.click(screen.getByRole("checkbox", { name: "Cụm từ đặc trưng" }));
    expect(preview().value).toBe("");
    expect(screen.getByRole("button", { name: /Chép/ })).toBeDisabled();
    expect(screen.getByRole("button", { name: /Lưu/ })).toBeDisabled();
  });

  it("tìm nhanh chỉ lọc hiển thị, không đổi tick", async () => {
    const user = userEvent.setup();
    render(<GlossaryExportDialog open onOpenChange={vi.fn()} glossary={glossary()} />);
    await user.type(screen.getByPlaceholderText("Tìm Hán hoặc Việt…"), "Kinh");
    expect(screen.queryByText("赵静文")).not.toBeInTheDocument();
    expect(screen.getByText("京城")).toBeInTheDocument();
    expect(preview().value).toContain("赵静文=Triệu Tĩnh Văn");
  });

  it("Chép đưa nội dung không BOM vào clipboard", async () => {
    const user = userEvent.setup();
    render(<GlossaryExportDialog open onOpenChange={vi.fn()} glossary={glossary()} />);
    await user.click(screen.getByRole("button", { name: /Chép/ }));
    await waitFor(() => expect(copyText).toHaveBeenCalledWith("赵静文=Triệu Tĩnh Văn\r\n京城=Kinh Thành\r\n"));
  });

  it("Lưu… mở hộp thoại với tên đã chọn rồi ghi file có BOM; huỷ hộp thoại thì không ghi", async () => {
    const user = userEvent.setup();
    vi.mocked(pickSaveFile).mockResolvedValueOnce(undefined).mockResolvedValueOnce("D:\\qt\\Names2.txt");
    render(<GlossaryExportDialog open onOpenChange={vi.fn()} glossary={glossary()} />);
    await user.click(screen.getByRole("button", { name: /Lưu/ }));
    await waitFor(() => expect(pickSaveFile).toHaveBeenCalledWith("Names.txt", "Lưu Names.txt"));
    expect(writeTextFile).not.toHaveBeenCalled();
    await user.click(within(screen.getByRole("radiogroup", { name: "Tên file" })).getByRole("radio", { name: "Names2.txt" }));
    await user.click(screen.getByRole("button", { name: /Lưu/ }));
    await waitFor(() => expect(pickSaveFile).toHaveBeenLastCalledWith("Names2.txt", "Lưu Names2.txt"));
    await waitFor(() =>
      expect(writeTextFile).toHaveBeenCalledWith("D:\\qt\\Names2.txt", "\ufeff赵静文=Triệu Tĩnh Văn\r\n京城=Kinh Thành\r\n"),
    );
    expect(await screen.findByText(/Đã ghi/)).toHaveTextContent("D:\\qt\\Names2.txt");
  });
});
```

- [x] **Step 2: Chạy, phải fail vì thiếu component**

Run: `cd apps/qt-ai-gui && npx vitest run src/components/glossary-export-dialog.test.tsx`
Expected: FAIL "Failed to resolve import "@/components/glossary-export-dialog""

- [x] **Step 3: Viết component**

```tsx
// apps/qt-ai-gui/src/components/glossary-export-dialog.tsx
import { ChevronDown, ChevronRight, Copy, Save, Search } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";

import { Choice } from "@/components/choice";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { pickSaveFile, writeTextFile } from "@/lib/api";
import { copyText } from "@/lib/clipboard";
import {
  defaultSelection,
  NAMES_EXPORT_KEYS,
  NAMES_FILE_NAMES,
  planNamesExport,
  renderNames,
  type NamesExportKey,
  type NamesExportRow,
  type NamesFileName,
} from "@/lib/names-export";
import type { StoryFormValues } from "@/lib/story-form";
import { GLOSSARY_LABELS } from "@/lib/types";
import { cn } from "@/lib/utils";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Glossary đang hiển thị ở form (kể cả sửa chưa lưu). */
  glossary: StoryFormValues["glossary"];
}

const FILE_LABELS: Record<NamesFileName, string> = { "Names.txt": "Names.txt", "Names2.txt": "Names2.txt" };

/** Checkbox tổng của nhóm: checked/indeterminate theo số dòng hợp lệ đã tick. */
function GroupCheckbox({ label, total, picked, onChange }: { label: string; total: number; picked: number; onChange: (checked: boolean) => void }) {
  const ref = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (ref.current) ref.current.indeterminate = picked > 0 && picked < total;
  }, [picked, total]);
  return (
    <input
      ref={ref}
      type="checkbox"
      aria-label={label}
      className="size-4 accent-primary"
      disabled={total === 0}
      checked={total > 0 && picked === total}
      onChange={(e) => onChange(e.target.checked)}
    />
  );
}

/**
 * Chọn mục glossary rồi chép/lưu ra `Hán=Việt` mỗi dòng cho Names.txt / Names2.txt của bản convert.
 * Quy tắc bỏ dòng và mặc định tick nằm ở lib/names-export.ts.
 */
export function GlossaryExportDialog({ open, onOpenChange, glossary }: Props) {
  const rows = useMemo(() => planNamesExport(glossary), [glossary]);
  const [selected, setSelected] = useState<Set<string>>(() => defaultSelection(rows));
  const [query, setQuery] = useState("");
  const [fileName, setFileName] = useState<NamesFileName>("Names.txt");
  const [collapsed, setCollapsed] = useState<Set<NamesExportKey>>(new Set());
  const [busy, setBusy] = useState(false);
  const [written, setWritten] = useState<string | undefined>();

  // Mở lại với glossary khác → tick lại mặc định.
  useEffect(() => {
    if (open) {
      setSelected(defaultSelection(rows));
      setWritten(undefined);
    }
  }, [open, rows]);

  const text = renderNames(rows, selected);
  const picked = rows.filter((r) => !r.skip && selected.has(r.id)).length;
  const skipped = rows.filter((r) => Boolean(r.skip)).length;
  const needle = query.trim().toLowerCase();
  const byGroup = useMemo(() => {
    const map = new Map<NamesExportKey, NamesExportRow[]>();
    for (const key of NAMES_EXPORT_KEYS) map.set(key, []);
    for (const row of rows) map.get(row.group)?.push(row);
    return map;
  }, [rows]);

  function toggleRow(id: string, checked: boolean) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (checked) next.add(id);
      else next.delete(id);
      return next;
    });
  }

  function toggleGroup(key: NamesExportKey, checked: boolean) {
    setSelected((prev) => {
      const next = new Set(prev);
      for (const row of byGroup.get(key) ?? []) {
        if (row.skip) continue;
        if (checked) next.add(row.id);
        else next.delete(row.id);
      }
      return next;
    });
  }

  async function copy() {
    try {
      await copyText(text);
      toast.success(`Đã chép ${picked} dòng`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không chép được");
    }
  }

  async function saveFile() {
    setBusy(true);
    try {
      const out = await pickSaveFile(fileName, `Lưu ${fileName}`);
      if (!out) return;
      await writeTextFile(out, renderNames(rows, selected, { bom: true }));
      setWritten(out);
      toast.success(`Đã ghi ${picked} dòng`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Ghi file thất bại");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex max-h-[85vh] w-[min(96vw,64rem)] max-w-none flex-col">
        <DialogHeader>
          <DialogTitle>Export glossary ra Names.txt</DialogTitle>
          <DialogDescription>
            Mỗi dòng <code className="font-mono">Hán=Việt</code> cho từ điển tên của bản convert. Xưng hô theo cặp không
            export; dòng có dấu <code className="font-mono">=</code>, thiếu một cột hay trùng Hán bị bỏ (QT chỉ giữ dòng đầu).
          </DialogDescription>
        </DialogHeader>
        <div className="grid min-h-0 flex-1 grid-cols-[1fr_1fr] gap-4">
          <div className="flex min-h-0 flex-col gap-2">
            <div className="relative">
              <Search className="pointer-events-none absolute top-1/2 left-2 size-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                className="pl-8"
                placeholder="Tìm Hán hoặc Việt…"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
              />
            </div>
            <div className="fine-scrollbar min-h-0 flex-1 overflow-y-auto rounded-md border">
              {NAMES_EXPORT_KEYS.map((key) => {
                const all = byGroup.get(key) ?? [];
                const valid = all.filter((r) => !r.skip);
                const pickedHere = valid.filter((r) => selected.has(r.id)).length;
                const visible = needle
                  ? all.filter((r) => r.source.toLowerCase().includes(needle) || r.target.toLowerCase().includes(needle))
                  : all;
                const isOpen = !collapsed.has(key);
                return (
                  <section key={key} className="border-b last:border-b-0">
                    <div className={cn("flex items-center gap-2 px-2 py-1.5 text-sm", all.length === 0 && "text-muted-foreground")}>
                      <GroupCheckbox
                        label={GLOSSARY_LABELS[key]}
                        total={valid.length}
                        picked={pickedHere}
                        onChange={(checked) => toggleGroup(key, checked)}
                      />
                      <button
                        type="button"
                        className="flex flex-1 items-center gap-1 text-left font-medium"
                        aria-expanded={isOpen}
                        onClick={() =>
                          setCollapsed((prev) => {
                            const next = new Set(prev);
                            if (next.has(key)) next.delete(key);
                            else next.add(key);
                            return next;
                          })
                        }
                      >
                        {isOpen ? <ChevronDown className="size-4" /> : <ChevronRight className="size-4" />}
                        {GLOSSARY_LABELS[key]}
                      </button>
                      <span className="tabular-nums text-xs text-muted-foreground">
                        {pickedHere}/{valid.length}
                      </span>
                    </div>
                    {isOpen &&
                      visible.map((row) => (
                        <label
                          key={row.id}
                          className={cn("flex items-center gap-2 px-2 py-1 pl-8 text-sm", row.skip && "text-muted-foreground")}
                        >
                          <input
                            type="checkbox"
                            aria-label={`${row.source} = ${row.target}`}
                            className="size-4 accent-primary"
                            disabled={Boolean(row.skip)}
                            checked={!row.skip && selected.has(row.id)}
                            onChange={(e) => toggleRow(row.id, e.target.checked)}
                          />
                          <span className={cn("min-w-0 flex-1 truncate", row.skip && "line-through")}>
                            <span>{row.source}</span>
                            <span className="text-muted-foreground"> = </span>
                            <span>{row.target}</span>
                          </span>
                          {row.skip && <span className="shrink-0 text-xs">{row.skip}</span>}
                        </label>
                      ))}
                  </section>
                );
              })}
            </div>
          </div>
          <div className="flex min-h-0 flex-col gap-2">
            <p className="text-sm">
              Sẽ ghi <strong className="tabular-nums">{picked}</strong> dòng · bỏ{" "}
              <strong className="tabular-nums">{skipped}</strong> dòng
            </p>
            <Textarea aria-label="Xem trước" readOnly value={text} className="min-h-0 flex-1 resize-none font-mono text-xs" />
            {written && (
              <p className="text-xs text-muted-foreground">
                Đã ghi <code className="font-mono break-all">{written}</code>
              </p>
            )}
          </div>
        </div>
        <DialogFooter className="items-center sm:justify-between">
          <Choice label="Tên file" value={fileName} options={NAMES_FILE_NAMES} labels={FILE_LABELS} onChange={setFileName} />
          <div className="flex gap-2">
            <Button variant="outline" disabled={picked === 0 || busy} onClick={() => void copy()}>
              <Copy /> Chép
            </Button>
            <Button disabled={picked === 0 || busy} onClick={() => void saveFile()}>
              <Save /> Lưu…
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
```

- [x] **Step 4: Chạy test, phải pass**

Run: `cd apps/qt-ai-gui && npx vitest run src/components/glossary-export-dialog.test.tsx`
Expected: 5 passed.

- [x] **Step 5: Lint + no-hardcoded-colors**

Run: `cd apps/qt-ai-gui && npm run lint && npx vitest run src/lib/no-hardcoded-colors.test.ts`
Expected: sạch. `accent-primary` dùng token nên qua được test màu.

---

### Task 4: Nút mở dialog ở tab Glossary + README

**Files:**
- Modify: `apps/qt-ai-gui/src/components/pages/story-page.tsx:293-300` (Section glossary) và import
- Modify: `apps/qt-ai-gui/src/components/pages/story-page.test.tsx` (thêm test)
- Modify: `apps/qt-ai-gui/README.md` (mục "Trang đọc và hồ sơ truyện", dòng 121–133)

**Interfaces:**
- Consumes: `GlossaryExportDialog` (Task 3); `useWatch({ control: form.control, name: "glossary" })`.

- [x] **Step 1: Test thất bại — nút mở dialog**

Thêm vào `story-page.test.tsx` (trong `describe` hiện có, dùng `snapshot` sẵn có với glossary names `赵静文`):

```tsx
  it("tab Glossary có nút Export Names.txt… mở dialog với glossary đang hiển thị", async () => {
    const user = userEvent.setup();
    useStoryStore.setState({ root: snapshot.root, snapshot } as never);
    render(<StoryPage />);
    await user.click(screen.getByRole("button", { name: "Glossary" }));
    await user.click(screen.getByRole("button", { name: /Export Names\.txt/ }));
    expect(await screen.findByRole("dialog", { name: "Export glossary ra Names.txt" })).toBeInTheDocument();
    expect(screen.getByLabelText("Xem trước")).toHaveValue("赵静文=Triệu Tĩnh Văn\r\n");
  });
```

(Trước khi thêm, xem các test hiện có trong file dùng cách nào để vào tab Glossary và set store — sao chép đúng cách đó; `vi.mock("@/lib/api")` ở đầu file cần thêm `pickSaveFile: vi.fn(), writeTextFile: vi.fn()`.)

- [x] **Step 2: Chạy, phải fail vì chưa có nút**

Run: `cd apps/qt-ai-gui && npx vitest run src/components/pages/story-page.test.tsx -t "Export Names"`
Expected: FAIL "Unable to find role button /Export Names\.txt/"

- [x] **Step 3: Nối dialog vào story-page**

Import thêm:

```tsx
import { GlossaryExportDialog } from "@/components/glossary-export-dialog";
```

Trong component `StoryPage`, cạnh các state hiện có:

```tsx
  const [exportOpen, setExportOpen] = useState(false);
  const glossaryValues = useWatch({ control: form.control, name: "glossary" });
```

Thay Section glossary:

```tsx
              <Section id="glossary" active={active} title="Glossary">
                <div className="flex flex-wrap items-center justify-between gap-2">
                  <p className="text-xs text-muted-foreground">
                    Kho chung theo bối cảnh (Cài đặt → Bản mặc định → Glossary chung) làm nền; mục ở đây đè khi trùng.
                  </p>
                  <Button type="button" size="sm" variant="outline" onClick={() => setExportOpen(true)}>
                    <Download /> Export Names.txt…
                  </Button>
                </div>
                {GLOSSARY_KEYS.map((key) => (
                  <GlossaryEditor key={key} name={`glossary.${key}`} label={GLOSSARY_LABELS[key]} />
                ))}
                <GlossaryExportDialog open={exportOpen} onOpenChange={setExportOpen} glossary={glossaryValues} />
              </Section>
```

`Download` đã được import ở đầu file. Nếu `useWatch` trả `undefined` lúc chưa reset form, truyền `glossaryValues ?? glossaryToPairs(undefined)` (import `glossaryToPairs` từ `@/lib/story-form`).

- [x] **Step 4: Chạy test trang, phải pass**

Run: `cd apps/qt-ai-gui && npx vitest run src/components/pages/story-page.test.tsx`
Expected: tất cả pass

- [x] **Step 5: README**

Thêm vào cuối mục "## Trang đọc và hồ sơ truyện" (trước "## Thể loại"):

```markdown
Tab Glossary có **Export Names.txt…**: chọn nhóm/dòng rồi **Chép** (clipboard) hoặc **Lưu…** thành `Names.txt` /
`Names2.txt` cho bản convert QT, mỗi dòng `Hán=Việt` (CRLF, UTF-8 có BOM khi lưu file). Mặc định tick hết trừ Cụm từ
đặc trưng; Xưng hô theo cặp không export; dòng có dấu `=`, thiếu một cột hay trùng Hán bị bỏ vì QT vứt/chỉ giữ dòng đầu.
```

- [x] **Step 6: Kiểm tra toàn bộ**

Run: `cd apps/qt-ai-gui && npm run typecheck && npm run lint && npm test`
Expected: sạch, mọi test pass. Rust: `cd apps/qt-ai-gui/src-tauri && cargo test -p qt-ai-gui` pass.

- [x] **Step 7: Không commit; báo người dùng thử `npm run tauri dev` hoặc bản đang chạy để nhìn dialog.**
