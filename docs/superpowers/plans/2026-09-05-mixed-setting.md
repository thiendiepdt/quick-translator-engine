# Bối cảnh hỗn hợp — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Thêm `genre.setting = "mixed"` cho truyện xuyên qua lại cổ đại ↔ hiện đại và đô thị tu tiên: prompt có cả hai bộ xưng hô + hai bảng thuật ngữ, rule chỉ trung lập.

**Architecture:** Mở rộng enum ở ba schema; thêm `SettingModule` thứ ba trong `ai-translation-prompt.ts` tái dùng chữ của hai module sẵn có; golden xuất 9 base; Rust/GUI chỉ thêm biến thể enum vì logic lọc rule và tra base đã theo key.

**Tech Stack:** như plan `2026-09-05-multi-genre-translation.md`.

## Global Constraints

- ancient/han vẫn bằng đúng từng byte (test hash FNV có sẵn phải xanh).
- Không thêm rule mới; `mixed` = rule trung lập.
- Không chạy `cargo fmt`. Không chạy dịch thật.
- Chỉ commit theo task; không push.

---

### Task 1: qt-web — enum `mixed`, module prompt, test

**Files:**
- Modify: `apps/qt-web/src/lib/ai-story.ts` (`GENRE_SETTINGS`, `GENRE_SETTING_LABELS`)
- Modify: `apps/qt-web/src/lib/ai-translation-prompt.ts` (module `mixedSetting`, `SETTINGS`, `PROMPT_GENRE_COMBOS`)
- Test: `apps/qt-web/src/lib/ai-translation-prompt.test.ts`, `apps/qt-web/src/lib/ai-translation.test.ts`, `apps/qt-web/src/components/ai-story-config-dialog.test.tsx`

- [ ] **Step 1: Test đỏ**

`ai-translation-prompt.test.ts`: sửa test "6 tổ hợp" thành 9 (thứ tự ancient×3, modern×3, mixed×3) và thêm:

```ts
  it("mixed có cả hai bộ xưng hô, hai bảng thuật ngữ", () => {
    const mixed = composeBasePrompt({ setting: "mixed", names: "han" });
    expect(mixed).toContain("| 我          | **ta**");
    expect(mixed).toContain("| 他          | **anh** / **anh ta** / **hắn**");
    expect(mixed).toContain("### Tu tiên / Xianxia");
    expect(mixed).toContain("### Đô thị / Hiện đại");
    expect(mixed).toContain("theo cảnh");
  });
```

`ai-translation.test.ts` thêm:

```ts
  it("mixed chỉ chạy rule trung lập: vợ, ngươi, Ừm qua; dấu câu Trung vẫn bắt", () => {
    expect(defaultAiCheckRules("mixed").map((r) => r.message)).not.toContain("Dùng vợ/chồng → thay bằng thê tử/phu quân");
    expect(defaultAiCheckRules("mixed").map((r) => r.message)).not.toContain(
      "Xưng hô cổ trang trong truyện hiện đại → anh/cô/tôi theo quan hệ",
    );
    const text = "Vợ anh nói: Ngươi dám? Ừm，được.";
    expect(checkAiTranslationViolations(text, undefined, "mixed").map((v) => v.message)).toEqual([
      "Dấu câu tiếng Trung còn sót → dùng dấu câu thường",
    ]);
  });
```

`ai-story-config-dialog.test.tsx` thêm vào describe "story genre":

```ts
  it("Hỗn hợp: tab Kiểm tra không có rule xưng hô của cả hai bối cảnh", async () => {
    renderDialog();
    const user = userEvent.setup();
    await user.click(screen.getByRole("combobox", { name: "Bối cảnh" }));
    await user.click(await screen.findByRole("option", { name: /Hỗn hợp/ }));
    await user.click(screen.getByRole("tab", { name: /Kiểm tra/ }));
    const messages = screen.getAllByLabelText(/^Mô tả rule/).map((el) => (el as HTMLInputElement).value);
    expect(messages.some((m) => m.includes("thê tử/phu quân"))).toBe(false);
    expect(messages.some((m) => m.includes("Xưng hô cổ trang"))).toBe(false);
  });
```

Run: `cd apps/qt-web && npx vitest run src/lib src/components/ai-story-config-dialog.test.tsx` — Expected: FAIL (`"mixed"` không hợp lệ / thiếu option).

- [ ] **Step 2: Enum + nhãn**

`ai-story.ts`: `GENRE_SETTINGS = ["ancient", "modern", "mixed"] as const`; thêm `mixed: { label: "Hỗn hợp / xuyên qua lại", hint: "Chọn xưng hô theo cảnh; không bắt lỗi xưng hô" }`.

- [ ] **Step 3: Module `mixedSetting`**

Trong `ai-translation-prompt.ts`, sau `const modern`, thêm (tái dùng mảng của hai module qua slice để không chép lại chữ):

```ts
/** Hai bảng đại từ của ancient/modern, bỏ heading "## 1." và dấu "---" cuối để ghép vào một mục. */
const stripSection = (lines: string[]) => lines.slice(2, lines.lastIndexOf("---"));

const mixedSetting: SettingModule = {
  constraints: [
    "Truyện đổi bối cảnh theo cảnh hoặc chương (xuyên không qua lại, đô thị có tu luyện). Trước mỗi đoạn, xác định cảnh đang ở thời nào qua tín hiệu trong raw: điện thoại, xe, công ty, trường học, mạng → hiện đại; cung điện, tu vi, đan dược, tước vị, kiệu ngựa → cổ đại. Cảnh cổ đại dùng bộ xưng hô cổ (`ta`/`ngươi`/`hắn`/`nàng`), cảnh hiện đại dùng bộ hiện đại (`tôi`/`anh`/`cô` theo quan hệ); chốt cho từng cặp nhân vật trong từng cảnh, đổi cảnh thì đổi cả bộ, tuyệt đối không lai hai bộ trong cùng một câu. Nhân vật xuyên không giữ thói quen xưng hô cũ nếu raw thể hiện vậy.",
    "Thán từ theo cảnh: cảnh cổ đại dùng `A?` / `Ân`; cảnh hiện đại dùng `Ừ` / `Ừm` / `Ơ?` / `À` / `Ôi`. Không đem thán từ hiện đại vào cảnh cổ và ngược lại.",
  ],
  pronouns: [
    "## 1. Đại từ nhân xưng — chọn bảng theo cảnh",
    "",
    "### Cảnh cổ đại",
    "",
    ...stripSection(ancient.pronouns),
    "### Cảnh hiện đại",
    "",
    ...stripSection(modern.pronouns),
    "---",
    "",
  ],
  terms: [...ancient.terms.slice(0, ancient.terms.lastIndexOf("---")), ...modern.terms],
  inversion: [
    "### Đảo ngữ cổ phong — chỉ trong cảnh cổ đại",
    "",
    ...ancient.inversion.slice(2),
    ...modern.inversion.slice(2),
  ],
  vocabulary: [
    "### Từ gia đình theo cảnh",
    "",
    "Cảnh cổ đại: thê tử, phu nhân, phu quân, lang quân, phụ thân, mẫu thân — KHÔNG dùng \"vợ\", \"chồng\". Cảnh hiện đại: vợ, chồng, bố, mẹ, bạn trai, bạn gái — KHÔNG dùng thê tử, phu quân, phụ thân, mẫu thân trừ khi nhân vật cố tình nói cổ.",
    "",
  ],
  editing: [
    "Soát lại xưng hô theo cảnh: mỗi cảnh chỉ dùng một bộ (cổ: ta/ngươi/hắn/nàng; hiện đại: tôi/anh/cô), không lẫn `ngươi` vào cảnh hiện đại hay `anh/tôi` vào cảnh cổ; không tự thêm hành động raw không có.",
  ],
};

const SETTINGS: Record<GenreSetting, SettingModule> = { ancient, modern, mixed: mixedSetting };

export const PROMPT_GENRE_COMBOS: StoryGenre[] = (["ancient", "modern", "mixed"] as const).flatMap((setting) =>
  (["han", "foreign", "mixed"] as const).map((names) => ({ setting, names })),
);
```

Kiểm bằng mắt: `ancient.pronouns` bắt đầu `["## 1. Đại từ nhân xưng", "", ...bảng..., "", "---", ""]` nên `slice(2, lastIndexOf("---"))` giữ bảng + hai ghi chú + dòng trống cuối; `ancient.terms` kết thúc `"", "---", ""`; `ancient.inversion`/`modern.inversion` bắt đầu bằng heading + dòng trống.

- [ ] **Step 4: Chạy test, lint, typecheck; commit**

Run: `cd apps/qt-web && npx vitest run && npm run typecheck && npx eslint src/lib src/components/ai-story-config-dialog.tsx`
Expected: PASS; test hash ancient/han vẫn xanh.

```bash
git add apps/qt-web/src/lib apps/qt-web/src/components/ai-story-config-dialog.test.tsx
git commit -m "feat(qt-web): bối cảnh hỗn hợp — hai bộ xưng hô theo cảnh, rule chỉ trung lập"
```

---

### Task 2: Golden + Rust

**Files:**
- Modify: `apps/qt-ai-cli/scripts/gen-golden.ts`
- Modify: `crates/qt-ai-core/src/story.rs`, `crates/qt-ai-core/src/prompt.rs` (test), `crates/qt-ai-core/tests/golden.rs`
- Regenerate: `crates/qt-ai-core/prompts/prompts.json`, `tests/fixtures/{prompt,check,story}.json`

- [ ] **Step 1: gen-golden**

`check.defaultRules` thêm `mixed: defaultAiCheckRules("mixed")`; cases thêm:

```ts
    { name: "mixed-default", text: MODERN_TEXT, rules: null, setting: "mixed", violations: checkAiTranslationViolations(MODERN_TEXT, undefined, "mixed") },
```

`story.normalize` thêm `{ input: { genre: { setting: "mixed", names: "han" } }, output: null as unknown }` + gán output. `promptCases` tự có 9 case nhờ `PROMPT_GENRE_COMBOS`.

Run: `cd apps/qt-ai-cli && npm run typecheck && npm run golden` — 4 file đổi.

- [ ] **Step 2: Rust test đỏ rồi sửa**

`story.rs`: `GenreSetting` thêm `Mixed` (`as_str` → `"mixed"`, normalize `Some("mixed") => GenreSetting::Mixed`); test `normalize_genre_thieu_hoac_sai_ve_ancient_han` thêm assert `StoryConfig::normalize(&json!({"genre": {"setting": "mixed"}})).genre.setting == GenreSetting::Mixed`.
`prompt.rs` test: vòng `for setting in [Ancient, Modern, Mixed]`, `seen.len() == 9`.
`golden.rs`: `DefaultRules` thêm `mixed: Vec<CheckRule>`; test so `default_rules_as_check_rules(GenreSetting::Mixed)`.

Run: `cargo test -p qt-ai-core` — Expected: PASS (9 base, golden khớp).

- [ ] **Step 3: Commit**

```bash
git add apps/qt-ai-cli/scripts/gen-golden.ts crates/qt-ai-core
git commit -m "feat(qt-ai-core): GenreSetting::Mixed, golden 9 base và rule hỗn hợp"
```

---

### Task 3: GUI + workflow

**Files:**
- Modify: `apps/qt-ai-gui/src/lib/schema.ts` (`GENRE_SETTINGS`), `src/lib/types.ts` (nhãn), `src-tauri/src/story_cmds.rs` (test)
- Modify: `apps/qt-ai-cli/antigravity/workflows/setup-story.md`, `apps/qt-ai-gui/README.md`
- Test: `apps/qt-ai-gui/src/components/pages/story-page.test.tsx`

- [ ] **Step 1: Test đỏ**

`story-page.test.tsx` thêm:

```ts
  it("chọn Hỗn hợp gọi defaults với setting mixed", async () => {
    const user = userEvent.setup();
    render(<StoryPage />);
    await user.click(screen.getByRole("tab", { name: "Thể loại" }));
    await user.click(screen.getByRole("combobox", { name: "Bối cảnh" }));
    await user.click(await screen.findByRole("option", { name: /Hỗn hợp/ }));
    const { storyDefaults } = await import("@/lib/api");
    expect(storyDefaults).toHaveBeenCalledWith({ setting: "mixed", names: "han" });
  });
```

Run: `cd apps/qt-ai-gui && npx vitest run src/components/pages/story-page.test.tsx` — FAIL.

- [ ] **Step 2: Sửa**

`schema.ts`: `GENRE_SETTINGS = ["ancient", "modern", "mixed"] as const`. `types.ts`: thêm `mixed: { label: "Hỗn hợp / xuyên qua lại", hint: "Chọn xưng hô theo cảnh; không bắt lỗi xưng hô" }`. `story_cmds.rs` test `defaults_theo_genre` thêm: `defaults(&StoryGenre { setting: GenreSetting::Mixed, names: GenreNames::Han }).check_rules` không chứa message "vợ/chồng" lẫn "Xưng hô cổ trang".

`setup-story.md` dòng `genre.setting`: thêm `hoặc \`mixed\` (xuyên qua lại cổ đại ↔ hiện đại, đô thị tu tiên)`. README mục Thể loại: `setting: "ancient" | "modern" | "mixed"` và một câu về mixed.

- [ ] **Step 3: Kiểm + commit**

Run: `cd apps/qt-ai-gui && npm run check && cd src-tauri && cargo test -p qt-ai-gui && cd ../../qt-ai-cli && npm run -s golden:check`
Expected: PASS, "Golden khớp."

```bash
git add apps/qt-ai-gui apps/qt-ai-cli/antigravity/workflows/setup-story.md
git commit -m "feat(qt-ai-gui): bối cảnh Hỗn hợp trong Hồ sơ truyện; workflow setup-story biết mixed"
```
