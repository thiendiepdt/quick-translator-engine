# Dịch nhiều thể loại — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Thêm `genre: { setting, names }` vào story.json; prompt và check rule mặc định tự ghép theo lựa chọn ở cả qt-web, qt-ai-cli, qt-ai-core (agy + API) và qt-ai-gui, truyện cũ ra đúng từng byte như trước.

**Architecture:** qt-web là nguồn chữ duy nhất: `ai-translation-prompt.ts` giữ core + module theo slot, `composeBasePrompt(genre)` ghép; `CHECK_RULES` gắn tag `setting`. `gen-golden.ts` xuất **6 base đã ghép** vào `prompts.json` (Rust tra bảng, không port logic ghép — không thể drift) và `defaultRules` theo setting vào `check.json`. Rust `StoryConfig.genre`, `base_prompt(&genre)`, `check_violations(text, rules, setting)`. GUI thêm mục Thể loại, `story_defaults(genre)`.

**Tech Stack:** TypeScript/React 19/vitest (qt-web, qt-ai-gui), tsx (gen-golden), Rust 2021 + serde + fancy-regex (qt-ai-core), Tauri 2, zod 4, react-hook-form.

## Global Constraints

- Tổ hợp `ancient/han` phải bằng đúng từng byte `NOVEL_TRANSLATOR_BASE_PROMPT` cũ; bộ rule ancient bằng đúng `CHECK_RULES` cũ (cùng thứ tự).
- Story thiếu/sai `genre` → `{ setting: "ancient", names: "han" }` ở cả ba schema (qt-web, Rust, GUI).
- `customPrompt` không rỗng thay toàn bộ base đã ghép; `checkRules` không rỗng thay toàn bộ rule mặc định; rule CJK sót luôn chạy. Trục `names` không có rule regex.
- Không chạy phiên dịch thật với agy hay API (đốt quota người dùng). Kiểm bằng test + golden.
- Chỉ commit theo từng task như plan ghi; không push.
- Chạy lệnh từ đúng thư mục package (`apps/qt-web`, `apps/qt-ai-cli`, `apps/qt-ai-gui`, `crates/qt-ai-core`); shell là Git Bash trên Windows, đường dẫn dự án `/d/Work/projects/quick-translator-engine`.
- Trước khi kết thúc: `npm --prefix apps/qt-ai-cli run -s golden:check` xanh, `cargo test -p qt-ai-core -p qt-ai-gui` xanh, `npm run check` xanh ở qt-web và qt-ai-gui.

---

## Bản đồ file

| File | Việc |
| --- | --- |
| `apps/qt-web/src/lib/ai-story.ts` | `GenreSetting`, `GenreNames`, `StoryGenre`, `defaultStoryGenre()`, `normalizeStoryGenre()`, field `genre` |
| `apps/qt-web/src/lib/ai-translation-prompt.ts` | Tách base thành `PROMPT_CORE_*` + `PROMPT_SETTING` + `PROMPT_NAMES`; `composeBasePrompt(genre)`; `PROMPT_GENRE_COMBOS` |
| `apps/qt-web/src/lib/ai-translation.ts` | `CHECK_RULES` gắn setting; `defaultAiCheckRules(setting)`; `checkAiTranslationViolations(text, rules?, setting?)`; `buildAiTranslationSystemPrompt` dùng `story.genre` |
| `apps/qt-web/src/lib/ai-story-fill.ts` | AI fill trả `genre` |
| `apps/qt-web/src/components/ai-story-config-dialog.tsx` | Hai Select thể loại; prompt/rule mặc định theo genre |
| `apps/qt-web/src/components/ai-translation-workspace.tsx` | Truyền `story.genre.setting` vào check |
| `apps/qt-ai-cli/src/commands/check.ts` | Truyền setting |
| `apps/qt-ai-cli/scripts/gen-golden.ts` | `prompts.json = { bases, suffix }`, `check.json.defaultRules = { ancient, modern }`, case có `setting` |
| `apps/qt-ai-cli/antigravity/workflows/setup-story.md` | Bước điền `genre` |
| `crates/qt-ai-core/src/story.rs` | `GenreSetting`, `GenreNames`, `StoryGenre`, field `genre`, normalize |
| `crates/qt-ai-core/src/prompt.rs` | `base_prompt(&StoryGenre)`, `Prompts { bases, suffix }` |
| `crates/qt-ai-core/src/check.rs` | `DEFAULT_RULES` 4 phần tử, `default_rules_as_check_rules(setting)`, `check_violations(text, rules, setting)` |
| `crates/qt-ai-core/src/commands/check.rs`, `src/api_session.rs` | Truyền `story.genre.setting` |
| `crates/qt-ai-core/tests/golden.rs` | Fixture struct mới |
| `apps/qt-ai-gui/src/lib/schema.ts`, `types.ts`, `story-form.ts`, `api.ts`, `hooks/use-story-defaults.ts` | genre + `storyDefaults(genre)` |
| `apps/qt-ai-gui/src/components/pages/story-page.tsx` (+test) | Mục "Thể loại" |
| `apps/qt-ai-gui/src-tauri/src/story_cmds.rs` | `story_defaults(genre)` |
| `apps/qt-ai-gui/README.md` | Mục Thể loại |

---

### Task 1: Kiểu `genre` và normalize ở qt-web

**Files:**
- Modify: `apps/qt-web/src/lib/ai-story.ts`
- Test: `apps/qt-web/src/lib/ai-story.test.ts`

**Interfaces:**
- Produces: `GENRE_SETTINGS`, `GENRE_NAMES`, `type GenreSetting = "ancient" | "modern"`, `type GenreNames = "han" | "foreign" | "mixed"`, `interface StoryGenre { setting: GenreSetting; names: GenreNames }`, `defaultStoryGenre(): StoryGenre`, `normalizeStoryGenre(value: unknown): StoryGenre`, `AiStoryConfig.genre: StoryGenre`, `GENRE_SETTING_LABELS`, `GENRE_NAMES_LABELS`.

- [ ] **Step 1: Viết test đỏ**

Thêm vào cuối `describe("AI story workspace data", ...)` trong `apps/qt-web/src/lib/ai-story.test.ts`:

```ts
  it("defaults genre to ancient/han and drops unknown values", () => {
    expect(normalizeAiStoryConfig({}).genre).toEqual({ setting: "ancient", names: "han" });
    expect(normalizeAiStoryConfig({ genre: { setting: "modern", names: "foreign" } }).genre).toEqual({
      setting: "modern",
      names: "foreign",
    });
    expect(normalizeAiStoryConfig({ genre: { setting: "future", names: 3 } }).genre).toEqual({
      setting: "ancient",
      names: "han",
    });
    expect(emptyAiStoryConfig().genre).toEqual({ setting: "ancient", names: "han" });
  });
```

- [ ] **Step 2: Chạy test, xác nhận đỏ**

Run: `cd apps/qt-web && npx vitest run src/lib/ai-story.test.ts`
Expected: FAIL — `genre` là `undefined`.

- [ ] **Step 3: Thêm kiểu, default, normalize**

Trong `apps/qt-web/src/lib/ai-story.ts`, ngay sau `export interface StoryStyle {...}`:

```ts
export const GENRE_SETTINGS = ["ancient", "modern"] as const;
export const GENRE_NAMES = ["han", "foreign", "mixed"] as const;
export type GenreSetting = (typeof GENRE_SETTINGS)[number];
export type GenreNames = (typeof GENRE_NAMES)[number];

/** Hai trục độc lập: bối cảnh quyết xưng hô/thán từ/thuật ngữ; tên riêng quyết cách phiên. */
export interface StoryGenre {
  setting: GenreSetting;
  names: GenreNames;
}

export const GENRE_SETTING_LABELS: Record<GenreSetting, { label: string; hint: string }> = {
  ancient: { label: "Cổ đại / tiên hiệp", hint: "ta/ngươi/hắn/nàng, thán từ A?/Ân, cấm vợ/chồng" },
  modern: { label: "Hiện đại", hint: "anh/cô/tôi theo quan hệ, từ đời thường, thán từ hiện đại" },
};
export const GENRE_NAMES_LABELS: Record<GenreNames, { label: string; hint: string }> = {
  han: { label: "Hán-Việt", hint: "Kế Duyên, Bắc Kinh" },
  foreign: { label: "Gốc nước ngoài", hint: "Emily, New York, Naruto" },
  mixed: { label: "Hỗn hợp", hint: "Họ Hán → Hán-Việt, tên phiên âm → gốc" },
};

export function defaultStoryGenre(): StoryGenre {
  return { setting: "ancient", names: "han" };
}

function isGenreSetting(value: unknown): value is GenreSetting {
  return typeof value === "string" && (GENRE_SETTINGS as readonly string[]).includes(value);
}
function isGenreNames(value: unknown): value is GenreNames {
  return typeof value === "string" && (GENRE_NAMES as readonly string[]).includes(value);
}

/** Thiếu hoặc sai → mặc định cổ đại/Hán-Việt: truyện đang dịch không đổi hành vi. */
export function normalizeStoryGenre(value: unknown): StoryGenre {
  const record = typeof value === "object" && value !== null ? (value as Record<string, unknown>) : {};
  return {
    setting: isGenreSetting(record.setting) ? record.setting : "ancient",
    names: isGenreNames(record.names) ? record.names : "han",
  };
}
```

Trong `interface AiStoryConfig`, thêm sau `summary: string;`:

```ts
  genre: StoryGenre;
```

Trong `emptyAiStoryConfig()`, thêm sau `summary: "",`:

```ts
    genre: defaultStoryGenre(),
```

Trong `normalizeAiStoryConfig`, ở object trả về thêm sau `summary: stringValue(source.summary),`:

```ts
    genre: normalizeStoryGenre(source.genre),
```

- [ ] **Step 4: Chạy test, xác nhận xanh; typecheck**

Run: `cd apps/qt-web && npx vitest run src/lib/ai-story.test.ts && npm run typecheck`
Expected: PASS. Typecheck có thể đỏ ở chỗ dựng `AiStoryConfig` literal thiếu `genre` (ví dụ test hoặc fixture) — sửa bằng `emptyAiStoryConfig()` spread, không nới kiểu.

- [ ] **Step 5: Commit**

```bash
git add apps/qt-web/src/lib/ai-story.ts apps/qt-web/src/lib/ai-story.test.ts
git commit -m "feat(qt-web): story.json có genre {setting, names}, mặc định ancient/han"
```

---

### Task 2: Tách prompt thành module và `composeBasePrompt`

**Files:**
- Modify: `apps/qt-web/src/lib/ai-translation-prompt.ts` (viết lại toàn bộ)
- Modify: `apps/qt-web/src/lib/ai-translation.ts:236`
- Test: `apps/qt-web/src/lib/ai-translation-prompt.test.ts` (mới)

**Interfaces:**
- Consumes: `StoryGenre`, `GenreSetting`, `GenreNames` từ Task 1.
- Produces: `composeBasePrompt(genre: StoryGenre): string`, `PROMPT_GENRE_COMBOS: StoryGenre[]` (6 tổ hợp, thứ tự ancient/han, ancient/foreign, ancient/mixed, modern/han, modern/foreign, modern/mixed), `genreKey(genre): string` → `"ancient/han"`. `NOVEL_TRANSLATOR_BASE_PROMPT` **giữ** như alias `composeBasePrompt(defaultStoryGenre())` để test cũ và dialog còn chạy tới Task 4.

**Cách tách.** Base cũ là một mảng dòng join `"\n"`. Ghép theo **slot**: core là các mảng dòng cố định, giữa chúng chèn mảng dòng của module. Hai danh sách đánh số (ràng buộc hệ thống, mục 8 biên tập) được lưu **không số**, `composeBasePrompt` tự đánh `1.`, `2.`… lúc ghép, nên ancient giữ nguyên số cũ còn modern không bị hổng số. Mọi dòng cũ phải copy **nguyên văn** từ file hiện tại vào đúng mảng — test byte-identity ở Step 1 sẽ bắt sai sót.

Thứ tự slot (theo file cũ):

1. `CORE_HEAD`: từ dòng đầu tới hết `"# Ràng buộc hệ thống — không quy tắc nào bên dưới được ghi đè"` + dòng trống.
2. Danh sách ràng buộc = `CORE_CONSTRAINTS_A` (1–10 cũ, bỏ số) + `SETTING.constraints` (ancient: 11, 12, 13 cũ bỏ số) + `CORE_CONSTRAINTS_B` (14, 15, 16 cũ bỏ số).
3. `CORE_PHILOSOPHY`: từ dòng trống sau ràng buộc, `# Quy tắc dịch thuật` … hết `## 0. Ràng buộc trung thành` tới `"---"` và dòng trống trước `## 1.`.
4. `SETTING.pronouns`: ancient = `## 1. Đại từ nhân xưng` … hai ghi chú `> **Phu thê…**`, `> **Lời kể gián tiếp…**`, dòng trống, `---`, dòng trống.
5. `CORE_TERMS`: `## 2. Thuật ngữ đặc thù theo thể loại` … hết bảng "Dấu hiệu câu vẫn còn là convert" và dòng `**Phép thử bắt buộc:**…` + dòng trống.
6. `SETTING.terms`: ancient = `### Tu tiên / Xianxia` … hết `#### Triều Thanh (Qing dynasty)` bảng, dòng trống, `---`, dòng trống.
7. `NAMES.section`: han = `## 3. Nhân danh & Địa danh — Phiên âm Hán-Việt` … bảng, dòng trống, `---`, dòng trống.
8. `CORE_SENTENCES`: `## 4. Dịch câu — Nguyên tắc thực hành` … hết mục `### Từ chỉ thị — không lặp “này” máy móc` (tới dòng `- Trong chuỗi …không lặp \`này\`.`) + dòng trống.
9. `SETTING.inversion`: ancient = `### Đảo ngữ cổ phong — [tông/pháp bảo] + danh từ` … `Không đảo chỉ để câu nghe “giống truyện dịch”. Địa điểm thuần Việt: giữ "của" bình thường.` + dòng trống.
10. `CORE_PUNCT`: `### Dấu câu` … `---`, dòng trống, `## 5. Từ vựng`, dòng trống.
11. `SETTING.vocabulary`: ancient = `### TUYỆT ĐỐI CẤM`, dòng trống, `KHÔNG dùng "vợ", "chồng"…`, dòng trống.
12. `CORE_VOCAB`: `### Tránh dùng` … hết mục 7 Hệ thống (`- TUYỆT ĐỐI không tự hạ toàn bộ nội dung trong \`【】\` thành chữ thường.`), dòng trống, `---`, dòng trống, `## 8. Biên tập`, dòng trống. (Dòng `| 蓝星 | lam tinh | …` giữ trong core — truyện đô thị cũng dùng "lam tinh".)
13. Danh sách biên tập = `CORE_EDIT_A` (1–13 cũ bỏ số) + `SETTING.editing` (ancient: dòng 14 cũ bỏ số) + `CORE_EDIT_B` (15, 16 cũ bỏ số).

- [ ] **Step 1: Viết test đỏ**

Tạo `apps/qt-web/src/lib/ai-translation-prompt.test.ts`:

```ts
import { describe, expect, it } from "vitest";

import { defaultStoryGenre } from "@/lib/ai-story";
import {
  composeBasePrompt,
  genreKey,
  LEGACY_BASE_PROMPT_FNV1A64,
  PROMPT_GENRE_COMBOS,
} from "@/lib/ai-translation-prompt";

/** FNV-1a 64-bit trên UTF-8 — không cần crypto của môi trường, đủ để chốt "không đổi một byte". */
function fnv1a64(text: string): string {
  let hash = 0xcbf29ce484222325n;
  for (const byte of new TextEncoder().encode(text)) {
    hash ^= BigInt(byte);
    hash = (hash * 0x100000001b3n) & 0xffffffffffffffffn;
  }
  return hash.toString(16).padStart(16, "0");
}

describe("composeBasePrompt", () => {
  it("ancient/han bằng đúng từng byte prompt cũ", () => {
    // Hash của NOVEL_TRANSLATOR_BASE_PROMPT trước khi tách — chốt không đổi hành vi truyện cũ.
    expect(fnv1a64(composeBasePrompt(defaultStoryGenre()))).toBe(LEGACY_BASE_PROMPT_FNV1A64);
  });

  it("đánh số liền mạch hai danh sách ở mọi tổ hợp", () => {
    for (const genre of PROMPT_GENRE_COMBOS) {
      const prompt = composeBasePrompt(genre);
      const constraints = prompt.split("# Quy tắc dịch thuật")[0]!.match(/^\d+\. /gm)!.map((m) => Number.parseInt(m, 10));
      expect(constraints, genreKey(genre)).toEqual(constraints.map((_, i) => i + 1));
      const editing = prompt.split("## 8. Biên tập")[1]!.match(/^\d+\. /gm)!.map((m) => Number.parseInt(m, 10));
      expect(editing, genreKey(genre)).toEqual(editing.map((_, i) => i + 1));
    }
  });

  it("modern bỏ xưng hô cổ, cho vợ/chồng; foreign trả tên về gốc", () => {
    const modern = composeBasePrompt({ setting: "modern", names: "han" });
    expect(modern).toContain("| 他          | **anh** / **anh ta** / **hắn**");
    expect(modern).not.toContain("KHÔNG dùng \"vợ\", \"chồng\"");
    expect(modern).not.toContain("### Tu tiên / Xianxia");
    expect(modern).toContain("Kế Duyên");
    const foreign = composeBasePrompt({ setting: "ancient", names: "foreign" });
    expect(foreign).toContain("艾米丽");
    expect(foreign).not.toContain("| 计缘   | Kế Duyên");
    expect(foreign).toContain("### Tu tiên / Xianxia");
    const mixed = composeBasePrompt({ setting: "modern", names: "mixed" });
    expect(mixed).toContain("Bách gia tính");
  });

  it("6 tổ hợp cho 6 chuỗi khác nhau", () => {
    const set = new Set(PROMPT_GENRE_COMBOS.map((g) => composeBasePrompt(g)));
    expect(set.size).toBe(6);
    expect(PROMPT_GENRE_COMBOS.map(genreKey)).toEqual([
      "ancient/han", "ancient/foreign", "ancient/mixed", "modern/han", "modern/foreign", "modern/mixed",
    ]);
  });
});
```

- [ ] **Step 2: Lấy hash prompt cũ trước khi sửa file**

Run (từ `apps/qt-ai-cli`, nơi có sẵn `tsx`; file cũ không import alias nên chạy thẳng):

```bash
npx tsx -e "import { NOVEL_TRANSLATOR_BASE_PROMPT as p } from '../qt-web/src/lib/ai-translation-prompt.ts'; let h = 0xcbf29ce484222325n; for (const b of new TextEncoder().encode(p)) { h ^= BigInt(b); h = (h * 0x100000001b3n) & 0xffffffffffffffffn; } console.log(h.toString(16).padStart(16, '0'))"
```

Ghi lại hash; đó là `LEGACY_BASE_PROMPT_FNV1A64`.

Run: `npx vitest run src/lib/ai-translation-prompt.test.ts`
Expected: FAIL — `composeBasePrompt` không tồn tại.

- [ ] **Step 3: Viết lại `ai-translation-prompt.ts`**

Khung file (các mảng `CORE_*` và `ancient` chứa **nguyên văn** dòng cũ theo bản đồ slot ở trên; ở đây chỉ in khung và phần chữ mới):

```ts
import type { GenreNames, GenreSetting, StoryGenre } from "@/lib/ai-story";

/** Đồng bộ từ novel-translator/src-tauri/src/translator/translate.rs, tách theo slot để ghép theo thể loại. */

// FNV-1a 64 của NOVEL_TRANSLATOR_BASE_PROMPT trước khi tách (xem ai-translation-prompt.test.ts).
export const LEGACY_BASE_PROMPT_FNV1A64 = "<dán hash từ Step 2>";

const CORE_HEAD = [
  "Bạn là dịch giả tiểu thuyết Trung Quốc sang tiếng Việt. Nhiệm vụ của bạn là chuyển ngữ trung thành, không phải sáng tác lại hay biên tập nâng giọng.",
  "",
  // … nguyên văn tới:
  "# Ràng buộc hệ thống — không quy tắc nào bên dưới được ghi đè",
  "",
];

/** Ràng buộc 1–10 cũ, bỏ số đầu dòng. */
const CORE_CONSTRAINTS_A = [
  "Chỉ dịch những gì raw text thực sự có. Mọi thông tin, hình ảnh, cảm xúc, hành động, liên kết logic và mức độ trong bản dịch phải truy ngược được về nguyên tác.",
  // … tới ràng buộc 10 cũ
];
/** Ràng buộc 14–16 cũ, bỏ số. */
const CORE_CONSTRAINTS_B = [
  "Phải dịch đủ 100% câu và đoạn của raw text theo đúng thứ tự. Tuyệt đối không bỏ sót, gộp tắt hay dừng giữa chừng một đoạn; thà dịch dài còn hơn thiếu ý.",
  // … 15, 16 cũ
];

const CORE_PHILOSOPHY = [ "", "# Quy tắc dịch thuật", "", /* … tới */ "---", "" ];
const CORE_TERMS = [ "## 2. Thuật ngữ đặc thù theo thể loại", "", /* … tới */ "**Phép thử bắt buộc:** Che câu raw đi và đọc riêng câu Việt. Nếu người Việt hiểu được nhưng gần như không ai diễn đạt như vậy, câu đó vẫn chưa dịch xong; viết lại toàn câu từ phần nghĩa đã hiểu, không vá từng từ.", "" ];
const CORE_SENTENCES = [ "## 4. Dịch câu — Nguyên tắc thực hành", "", /* … tới dòng "- Trong chuỗi …" */ "" ];
const CORE_PUNCT = [ "### Dấu câu", "", /* … */ "---", "", "## 5. Từ vựng", "" ];
const CORE_VOCAB = [ "### Tránh dùng", "", /* … hết mục 7 */ "", "---", "", "## 8. Biên tập", "" ];
/** Biên tập 1–13 cũ, bỏ số. */
const CORE_EDIT_A = [ /* … */ ];
/** Biên tập 15–16 cũ, bỏ số. */
const CORE_EDIT_B = [ /* … */ ];

interface SettingModule {
  constraints: string[];
  pronouns: string[];
  terms: string[];
  inversion: string[];
  vocabulary: string[];
  editing: string[];
}

const ancient: SettingModule = {
  constraints: [
    "Trong style mặc định, `我` trong lời kể/lời thoại dùng `ta`, không dùng `mình` hoặc `tôi`; chỉ đổi quy tắc này khi glossary/style của truyện quy định rõ cách xưng hô khác.",
    "Khi `方寸` chỉ không gian riêng của nhân vật, bắt buộc dùng `Phương Thốn` hoặc `không gian Phương Thốn`, tuyệt đối không dịch thành `một tấc vuông`.",
    "Thán từ trong bối cảnh cổ đại/huyền huyễn/tiên hiệp: khi ngạc nhiên (`嗯?`, `咦?`) dùng `A?`, không dùng `Ơ?`; khi trầm ngâm/đáp nhẹ dùng `Ân`, không dùng `Ừm/Hừm`. Không dùng thán từ hiện đại (`Okay`, `Ừ nhỉ`, `Wow`).",
  ],
  pronouns: [ "## 1. Đại từ nhân xưng", "", /* … bảng + hai ghi chú nguyên văn */ "", "---", "" ],
  terms: [ "### Tu tiên / Xianxia", "", /* … hết bảng Triều Thanh */ "", "---", "" ],
  inversion: [ "### Đảo ngữ cổ phong — [tông/pháp bảo] + danh từ", "", /* … */ "Không đảo chỉ để câu nghe “giống truyện dịch”. Địa điểm thuần Việt: giữ \"của\" bình thường.", "" ],
  vocabulary: [ "### TUYỆT ĐỐI CẤM", "", "KHÔNG dùng \"vợ\", \"chồng\", \"người vợ\", \"người chồng\". Thay bằng: thê tử, phu nhân, phu quân, lang quân, phu thê.", "" ],
  editing: [
    "Soát lại đại từ và hành động: trong style mặc định, `我` dùng `ta`, không dùng `mình/tôi`; không tự thêm hành động như `quỳ`, `cúi`, `ngồi`, `khóc`, `chắp tay` nếu raw không có.",
  ],
};

const modern: SettingModule = {
  constraints: [
    "Trong style mặc định của truyện hiện đại, `我` trong lời kể ngôi thứ nhất dùng `tôi`; trong lời thoại chọn cặp xưng hô theo quan hệ và tuổi (anh–em, tớ–cậu, tao–mày, cháu–bác, con–bố/mẹ), chốt cho từng cặp nhân vật trong phần suy nghĩ rồi giữ nhất quán cả chương. Không dùng `ta`, `ngươi`, `nàng`, `bọn ta` trừ khi nhân vật cố tình nói giọng cổ hoặc style của truyện quy định.",
    "Thán từ theo khẩu ngữ hiện đại: `嗯` → `Ừ` / `Ừm`, `咦?` → `Ơ?`, `哦` → `À` / `Ồ`, `哎呀` → `Ôi` / `Trời ơi`, `卧槽` → `vãi` / `chết tiệt`, `妈的` → `mẹ kiếp` / `mẹ nó`. Không dùng `A?` / `Ân` kiểu cổ trang.",
  ],
  pronouns: [
    "## 1. Đại từ nhân xưng",
    "",
    "| Tiếng Trung | Dùng                             | KHÔNG dùng     |",
    "| ----------- | -------------------------------- | -------------- |",
    "| 他          | **anh** / **anh ta** / **hắn** (hắn chỉ cho nhân vật lạnh, phản diện hoặc khi style quy định) | y, gã (trừ khi giọng kể mỉa) |",
    "| 她          | **cô** / **cô ta** / **chị** (theo tuổi và quan hệ) | nàng |",
    "| 我          | **tôi** (lời kể ngôi một); trong thoại theo quan hệ | ta |",
    "| 你          | **anh / em / cậu / ông / mày / bác** theo quan hệ và tuổi | ngươi |",
    "| 我们        | **chúng tôi** / **chúng ta** / **bọn tôi** | bọn ta |",
    "| 他们        | **họ** / **bọn họ**              | bọn hắn, chúng |",
    "| 她们        | **các cô** / **họ**              | các nàng |",
    "| 你们        | **các cậu** / **mọi người** / **các anh** | các ngươi |",
    "",
    "> **Chốt xưng hô theo cặp:** Với mỗi cặp nhân vật đối thoại, chọn một cặp xưng hô (ví dụ Lâm Phong–Tô Vũ: anh–em) ngay trong phần suy nghĩ và dùng nhất quán cả chương. Chỉ đổi khi raw cho thấy quan hệ thay đổi (thân lên, cãi nhau, xưng tên) và đổi cho cả hai chiều.",
    "",
    "> **Lời kể gián tiếp:** Ngoài ngoặc kép, `自己` chỉ nhân vật đang được kể phải theo ngôi ba (`anh`, `chính anh`, `bản thân cô`), kể cả khi cả câu là ý nghĩ của nhân vật đó. Chỉ dùng `tôi` bên trong ngoặc kép hoặc khi truyện kể ở ngôi thứ nhất suốt.",
    "",
    "---",
    "",
  ],
  terms: [
    "### Đô thị / Hiện đại",
    "",
    "Từ đời thường dùng tiếng Việt đời thường; không Hán-Việt hoá quan hệ gia đình, nghề nghiệp, đồ vật. Thương hiệu, app, mã chứng khoán, tên game giữ nguyên chữ Latin. Địa danh Trung Quốc vẫn phiên Hán-Việt (Bắc Kinh, Thượng Hải, Thâm Quyến, Hàng Châu).",
    "",
    "| Raw | Dùng | KHÔNG dùng |",
    "| --- | --- | --- |",
    "| 老公 / 老婆 | chồng / vợ | phu quân / thê tử |",
    "| 爸 / 妈 / 爸妈 | bố / mẹ / bố mẹ | phụ thân / mẫu thân |",
    "| 老师 | thầy / cô | lão sư |",
    "| 学长 / 学姐 | anh khoá trên / chị khoá trên | học trưởng / học tỷ |",
    "| 同事 | đồng nghiệp | đồng sự |",
    "| 总裁 | tổng giám đốc | tổng tài (trừ khi glossary yêu cầu) |",
    "| 董事长 | chủ tịch hội đồng quản trị | đổng sự trưởng |",
    "| 手机 | điện thoại | thủ cơ |",
    "| 微信 / 淘宝 / 抖音 | WeChat / Taobao / Douyin | Vi Tín / Đào Bảo |",
    "| 元 / 块 | tệ / nghìn tệ (theo số) | nguyên |",
    "| 高考 | thi đại học | cao khảo |",
    "| 公司 / 小区 / 地铁 | công ty / khu chung cư / tàu điện ngầm | công ty (giữ) / tiểu khu / địa thiết |",
    "| 打车 / 外卖 | bắt taxi / đồ ăn giao tận nơi | đả xa / ngoại mại |",
    "| 警察 / 局长 | cảnh sát / cục trưởng | công an (trừ khi raw là 公安) |",
    "",
    "Chức danh, cấp bậc công ty và cơ quan dịch theo cách gọi tiếng Việt hiện hành; chỉ giữ Hán-Việt cho tên riêng và cụm đã có trong glossary.",
    "",
    "---",
    "",
  ],
  inversion: [
    "### Không cổ phong hoá",
    "",
    "Không dùng đảo ngữ kiểu `Thiên Long Tập Đoàn công pháp` hay cụm Hán-Việt bốn chữ cho lời kể đô thị. `X 的 Y` dịch xuôi `Y của X`. Chỉ cho phép giọng cổ trong lời thoại khi raw cho thấy nhân vật cố tình nói vậy (xuyên không, cosplay, trích thơ cổ).",
    "",
  ],
  vocabulary: [
    "### Ưu tiên từ đời thường",
    "",
    "Dùng \"vợ\", \"chồng\", \"bố\", \"mẹ\", \"bạn trai\", \"bạn gái\", \"sếp\". KHÔNG dùng thê tử, phu quân, lang quân, phu nhân, phụ thân, mẫu thân, công tử, tiểu thư trừ khi raw dùng từ cổ có chủ ý hoặc glossary quy định.",
    "",
  ],
  editing: [
    "Soát lại xưng hô và hành động: không dùng `ta/ngươi/nàng/bọn ta`, không lọt phụ thân/mẫu thân/thê tử/phu quân; mỗi cặp nhân vật giữ đúng cặp xưng hô đã chốt; không tự thêm hành động như `gật đầu`, `thở dài`, `mỉm cười` nếu raw không có.",
  ],
};

const SETTINGS: Record<GenreSetting, SettingModule> = { ancient, modern };

const han: string[] = [
  "## 3. Nhân danh & Địa danh — Phiên âm Hán-Việt",
  "",
  "Tra glossary trước. Tên mới chưa có → tự phiên âm, dùng nhất quán.",
  "",
  "| Trung  | Hán-Việt       | SAI                 |",
  "| ------ | -------------- | ------------------- |",
  "| 计缘   | Kế Duyên       | Ji Yuan             |",
  "| 李长笑 | Lý Trường Tiếu | Li Changxiao        |",
  "| 水龙宗 | Thủy Long Tông | tông phái Rồng Nước |",
  "| 曾头市 | Tằng Đầu Thị   | thành phố Tăng Đầu  |",
  "",
  "---",
  "",
];

const FOREIGN_TABLE = [
  "| Raw | Dùng | KHÔNG dùng |",
  "| --- | --- | --- |",
  "| 艾米丽 | Emily | Ngải Mễ Lệ |",
  "| 迈克尔 | Michael | Mại Khắc Nhĩ |",
  "| 纽约 | New York | Nữu Ước |",
  "| 霍格沃茨 | Hogwarts | Hoắc Cách Ốc Tì |",
  "| 巴黎 / 莫斯科 | Paris / Moscow | Ba Lê / Mạc Tư Khoa |",
  "| 美国 / 英国 / 法国 | Mỹ / Anh / Pháp | Mỹ quốc / Anh quốc |",
  "| 鸣人 / 佐藤 | Naruto / Sato | Minh Nhân / Tá Đằng |",
  "| 金秀贤 | Kim Soo-hyun | Kim Tú Hiền |",
  "",
  "- Có tên tiếng Việt quen thuộc (Paris, Mỹ, Anh, Đức, Nhật) thì dùng tên đó. Nhật / Hàn theo dạng phổ biến ở Việt Nam; glossary thắng mọi bảng.",
  "- Tên hư cấu không tra được: phiên Latin gần nhất theo âm chữ Hán (`阿尔泰` → Altai, `塞琳娜` → Selina), không Hán-Việt hoá, chốt trong phần suy nghĩ và dùng nhất quán; tên tự thêm vào glossary ghi đúng dạng đã chốt.",
  "",
  "#### Thuật ngữ phương Tây",
  "",
  "| Trung | Việt |",
  "| ----- | ---- |",
  "| 公爵 / 侯爵 / 伯爵 / 子爵 / 男爵 | công tước / hầu tước / bá tước / tử tước / nam tước |",
  "| 骑士 / 领主 / 国王 / 王子 / 公主 | hiệp sĩ / lãnh chúa / quốc vương / hoàng tử / công chúa |",
  "| 法师 / 魔法 / 魔法师 | pháp sư / ma pháp / ma pháp sư |",
  "| 精灵 / 矮人 / 兽人 / 巨龙 | tinh linh / người lùn / thú nhân / rồng |",
  "| 教皇 / 神父 / 修女 / 圣殿 | giáo hoàng / cha xứ / nữ tu / thánh điện |",
  "| 先生 / 小姐 / 夫人 | ngài / cô / phu nhân **hoặc** Mr. / Miss / Mrs. — chọn một hệ cho cả truyện |",
  "",
];

const foreign: string[] = [
  "## 3. Nhân danh & Địa danh — Trả về dạng gốc",
  "",
  "Tên người, địa danh, tổ chức được phiên âm bằng chữ Hán phải trả về dạng gốc (Latin, romaji, romanized), tra glossary trước. KHÔNG phiên Hán-Việt tên nước ngoài.",
  "",
  ...FOREIGN_TABLE,
  "---",
  "",
];

const mixed: string[] = [
  "## 3. Nhân danh & Địa danh — Hỗn hợp",
  "",
  "Tra glossary trước. Quyết định theo mặt chữ:",
  "",
  "- Họ Hán thuần (họ trong Bách gia tính: 李 王 张 刘 陈 赵 …) + tên 1–2 chữ → phiên âm Hán-Việt như truyện Trung (`赵静文` → Triệu Tĩnh Văn, `北京` → Bắc Kinh).",
  "- Chuỗi từ 3 chữ trở lên có ký tự phiên âm đặc trưng (`尔 斯 克 姆 特 娜 丽 德 洛 布 罗 伊 森 卡 蒂`) hoặc địa danh ngoài Trung Quốc → dạng gốc Latin / romaji (`艾米丽` → Emily, `纽约` → New York).",
  "- Không chắc → ưu tiên dạng gốc nếu bối cảnh câu là nước ngoài, Hán-Việt nếu là Trung Quốc; chốt một lần trong phần suy nghĩ, tên tự thêm vào glossary ghi đúng dạng đã chốn để chương sau theo.",
  "",
  ...FOREIGN_TABLE,
  "---",
  "",
];

const NAMES: Record<GenreNames, string[]> = { han, foreign, mixed };

const numbered = (lines: string[]) => lines.map((line, index) => `${index + 1}. ${line}`);

export function genreKey(genre: StoryGenre): string {
  return `${genre.setting}/${genre.names}`;
}

export const PROMPT_GENRE_COMBOS: StoryGenre[] = (["ancient", "modern"] as const).flatMap((setting) =>
  (["han", "foreign", "mixed"] as const).map((names) => ({ setting, names })),
);

/** Base prompt ghép theo thể loại; ancient/han bằng đúng từng byte prompt cũ. */
export function composeBasePrompt(genre: StoryGenre): string {
  const setting = SETTINGS[genre.setting];
  return [
    ...CORE_HEAD,
    ...numbered([...CORE_CONSTRAINTS_A, ...setting.constraints, ...CORE_CONSTRAINTS_B]),
    ...CORE_PHILOSOPHY,
    ...setting.pronouns,
    ...CORE_TERMS,
    ...setting.terms,
    ...NAMES[genre.names],
    ...CORE_SENTENCES,
    ...setting.inversion,
    ...CORE_PUNCT,
    ...setting.vocabulary,
    ...CORE_VOCAB,
    ...numbered([...CORE_EDIT_A, ...setting.editing, ...CORE_EDIT_B]),
  ].join("\n");
}

/** @deprecated prompt cổ đại/Hán-Việt — còn dùng tới khi mọi nơi nhận genre. */
export const NOVEL_TRANSLATOR_BASE_PROMPT = composeBasePrompt({ setting: "ancient", names: "han" });
```

Sửa lỗi gõ `đã chốn` → `đã chốt` trong `mixed` khi chép. Chú ý: dòng cuối biên tập 16 cũ không có `"\n"` kết thúc — mảng join `"\n"` không thêm dòng trống cuối, y như cũ.

Trong `apps/qt-web/src/lib/ai-translation.ts` dòng 236 đổi thành:

```ts
  const basePrompt = story?.customPrompt.trim() || composeBasePrompt(story?.genre ?? defaultStoryGenre());
```

và sửa import đầu file:

```ts
import { composeBasePrompt } from "@/lib/ai-translation-prompt";
import { defaultStoryGenre, type AiCheckRule, type AiStoryConfig } from "@/lib/ai-story";
```

- [ ] **Step 4: Chạy test byte-identity và toàn bộ test web**

Run: `cd apps/qt-web && npx vitest run src/lib/ai-translation-prompt.test.ts src/lib/ai-translation.test.ts`
Expected: PASS. Nếu test hash đỏ: in `composeBasePrompt(defaultStoryGenre())` ra file scratch, `diff` với bản cũ (`git show HEAD:apps/qt-web/src/lib/ai-translation-prompt.ts` + join) để tìm dòng chép sai.

- [ ] **Step 5: Typecheck + lint, commit**

Run: `cd apps/qt-web && npm run typecheck && npm run lint`

```bash
git add apps/qt-web/src/lib/ai-translation-prompt.ts apps/qt-web/src/lib/ai-translation-prompt.test.ts apps/qt-web/src/lib/ai-translation.ts
git commit -m "feat(qt-web): tách prompt dịch thành core + module bối cảnh/tên riêng, composeBasePrompt theo genre"
```

---

### Task 3: Check rules gắn tag setting

**Files:**
- Modify: `apps/qt-web/src/lib/ai-translation.ts:37-115, 266-296`
- Modify: `apps/qt-web/src/components/ai-translation-workspace.tsx:467-497, 715`
- Modify: `apps/qt-ai-cli/src/commands/check.ts:96`
- Test: `apps/qt-web/src/lib/ai-translation.test.ts`

**Interfaces:**
- Produces: `defaultAiCheckRules(setting: GenreSetting): AiCheckRule[]`; `checkAiTranslationViolations(text, configuredRules?, setting: GenreSetting = "ancient")`; `DEFAULT_AI_CHECK_RULES` **xoá** (thay bằng hàm).

- [ ] **Step 1: Viết test đỏ**

Thêm vào `describe("AI translation prompt", ...)` trong `ai-translation.test.ts`:

```ts
  it("bộ rule ancient giữ nguyên thứ tự cũ; modern bỏ rule cổ trang và thêm rule xưng hô", () => {
    const ancient = defaultAiCheckRules("ancient");
    const modern = defaultAiCheckRules("modern");
    expect(ancient[0]?.message).toBe("Dấu câu tiếng Trung còn sót → dùng dấu câu thường");
    expect(ancient.map((r) => r.message)).toContain("Dùng vợ/chồng → thay bằng thê tử/phu quân");
    expect(modern.map((r) => r.message)).not.toContain("Dùng vợ/chồng → thay bằng thê tử/phu quân");
    expect(modern.map((r) => r.message)).toContain("Xưng hô cổ trang trong truyện hiện đại → anh/cô/tôi theo quan hệ");
    expect(ancient.map((r) => r.message)).not.toContain("Xưng hô cổ trang trong truyện hiện đại → anh/cô/tôi theo quan hệ");
  });

  it("checks theo setting: modern cho vợ/chồng qua, bắt ngươi/nàng/thê tử/tổng tài", () => {
    const text = [
      "Vợ anh đang đợi ở công ty.",
      "Ngươi dám nói vậy sao?",
      "Nàng im lặng.",
      "Thê tử của tổng tài Lâm.",
      "Bố mẹ tôi ở Bắc Kinh, ừm.",
    ].join("\n");
    const modern = checkAiTranslationViolations(text, undefined, "modern");
    expect(modern.map((v) => `${v.line}:${v.message}`)).toEqual([
      "2:Xưng hô cổ trang trong truyện hiện đại → anh/cô/tôi theo quan hệ",
      "3:Xưng hô cổ trang trong truyện hiện đại → anh/cô/tôi theo quan hệ",
      "4:Từ gia đình cổ trang → vợ/chồng/bố/mẹ",
      "4:tổng tài → tổng giám đốc",
    ]);
    const ancient = checkAiTranslationViolations(text);
    expect(ancient.map((v) => v.message)).toContain("Dùng vợ/chồng → thay bằng thê tử/phu quân");
    expect(ancient.map((v) => v.message)).toContain("Hừm/Ừm → Ân");
  });
```

Sửa import test: bỏ gì không dùng, thêm `defaultAiCheckRules`.

- [ ] **Step 2: Chạy test, xác nhận đỏ**

Run: `cd apps/qt-web && npx vitest run src/lib/ai-translation.test.ts`
Expected: FAIL — `defaultAiCheckRules` không tồn tại.

- [ ] **Step 3: Gắn tag và lọc theo setting**

Trong `ai-translation.ts`, đổi kiểu `CHECK_RULES` và gắn tag. Thêm phần tử thứ ba `"ancient"` cho đúng các rule sau (giữ nguyên vị trí, không đổi regex hay message): `vợ|chồng`, `anh ấy|anh ta|cô ấy|chị ấy`, `mình|tôi`, `Miêu Ảnh Vô Tông`, `một tấc vuông`, `Hừm|Ừm`, `Ơ\s*[?!,.…]`, `kho tàng|kho báu`, `thê tử danh nghĩa`, `vỏ dao|rút dao|thanh dao`, `bom khói`. Các rule còn lại không tag (trung lập). Thêm **cuối mảng** các rule `"modern"`:

```ts
type Setting = GenreSetting;
const CHECK_RULES: Array<[RegExp, string, Setting?]> = [
  [/[，。、；：？！]/, "Dấu câu tiếng Trung còn sót → dùng dấu câu thường"],
  // … như cũ, với "ancient" ở các rule đã liệt kê, ví dụ:
  [/(?<!\p{L})(?:vợ|chồng)(?!\p{L})/iu, "Dùng vợ/chồng → thay bằng thê tử/phu quân", "ancient"],
  // …
  [/…/, "Còn ký tự … → chuẩn hóa thành dấu chấm ASCII, giữ số lượng (… → ..., …… → ......)"],
  // Rule riêng cho truyện hiện đại: bắt giọng cổ trang lọt vào đô thị.
  [/(?<!\p{L})(?:ngươi|nàng|bọn ta|các ngươi)(?!\p{L})/iu, "Xưng hô cổ trang trong truyện hiện đại → anh/cô/tôi theo quan hệ", "modern"],
  [/(?<!\p{L})(?:thê tử|phu quân|lang quân|phụ thân|mẫu thân)(?!\p{L})/iu, "Từ gia đình cổ trang → vợ/chồng/bố/mẹ", "modern"],
  [/(?<!\p{L})tổng tài(?!\p{L})/iu, "tổng tài → tổng giám đốc", "modern"],
  [/nói đạo/i, "说道 → nói / đáp, không \"nói đạo\"", "modern"],
];

/** Rule mặc định cho một bối cảnh: rule trung lập + rule gắn đúng setting, giữ thứ tự khai báo. */
export function defaultAiCheckRules(setting: GenreSetting): AiCheckRule[] {
  return CHECK_RULES.filter(([, , tag]) => tag === undefined || tag === setting).map(
    ([pattern, message]) => ({
      pattern: pattern.source,
      ...(pattern.flags ? { flags: pattern.flags } : {}),
      message,
    }),
  );
}
```

Xoá `export const DEFAULT_AI_CHECK_RULES`. Import `GenreSetting` từ `@/lib/ai-story`.

Sửa `checkAiTranslationViolations`:

```ts
export function checkAiTranslationViolations(
  text: string,
  configuredRules?: AiCheckRule[],
  setting: GenreSetting = "ancient",
): TranslationViolation[] {
  const violations: TranslationViolation[] = [];
  const baseRules: Array<[RegExp, string]> =
    configuredRules && configuredRules.length > 0
      ? configuredRules.flatMap(({ pattern, flags, message }) => {
          try {
            return [[new RegExp(pattern, flags), message] as [RegExp, string]];
          } catch {
            return [];
          }
        })
      : CHECK_RULES.filter(([, , tag]) => tag === undefined || tag === setting).map(
          ([pattern, message]) => [pattern, message] as [RegExp, string],
        );
  // … phần còn lại giữ nguyên
```

Người gọi:
- `ai-translation-workspace.tsx` dòng 467, 494: `checkAiTranslationViolations(translated, freshStory.checkRules, freshStory.genre.setting)` và `(reviewed, freshStory.checkRules, freshStory.genre.setting)`; dòng 715: `checkAiTranslationViolations(value, story.checkRules, story.genre.setting)`.
- `apps/qt-ai-cli/src/commands/check.ts:96`: `checkAiTranslationViolations(finalText, story.checkRules, story.genre.setting)`.
- `ai-story-config-dialog.tsx:47, 214`: tạm đổi `DEFAULT_AI_CHECK_RULES` → `defaultAiCheckRules(draft.genre.setting)` (Task 4 sửa tiếp UI).
- `gen-golden.ts:11, 133`: tạm đổi `DEFAULT_AI_CHECK_RULES` → `defaultAiCheckRules("ancient")` để typecheck qua (Task 5 viết lại).

- [ ] **Step 4: Chạy test + typecheck web và cli**

Run: `cd apps/qt-web && npx vitest run && npm run typecheck && cd ../qt-ai-cli && npm run typecheck && npx vitest run`
Expected: PASS toàn bộ.

- [ ] **Step 5: Commit**

```bash
git add apps/qt-web/src/lib/ai-translation.ts apps/qt-web/src/lib/ai-translation.test.ts apps/qt-web/src/components/ai-translation-workspace.tsx apps/qt-web/src/components/ai-story-config-dialog.tsx apps/qt-ai-cli/src/commands/check.ts apps/qt-ai-cli/scripts/gen-golden.ts
git commit -m "feat(qt-web): check rule gắn tag bối cảnh, defaultAiCheckRules(setting), rule hiện đại bắt xưng hô cổ trang"
```

---

### Task 4: UI qt-web — chọn thể loại, prompt/rule theo genre, AI fill

**Files:**
- Modify: `apps/qt-web/src/components/ai-story-config-dialog.tsx`
- Modify: `apps/qt-web/src/lib/ai-story-fill.ts`
- Test: `apps/qt-web/src/components/ai-story-config-dialog.test.tsx`

- [ ] **Step 1: Viết test đỏ**

Thêm vào `ai-story-config-dialog.test.tsx` một `describe` mới:

```ts
describe("story genre", () => {
  it("lưu bối cảnh và tên riêng đã chọn", async () => {
    const onSave = renderDialog();
    const user = userEvent.setup();

    await user.click(screen.getByRole("combobox", { name: "Bối cảnh" }));
    await user.click(await screen.findByRole("option", { name: /Hiện đại/ }));
    await user.click(screen.getByRole("combobox", { name: "Tên riêng" }));
    await user.click(await screen.findByRole("option", { name: /Gốc nước ngoài/ }));
    await user.click(screen.getByRole("button", { name: "Lưu cấu hình" }));

    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({ genre: { setting: "modern", names: "foreign" } }),
    );
  });

  it("tab Kiểm tra hiện bộ rule theo bối cảnh khi chưa có rule riêng", async () => {
    renderDialog();
    const user = userEvent.setup();
    await user.click(screen.getByRole("tab", { name: /Kiểm tra/ }));
    expect(screen.getAllByLabelText(/^Mô tả rule/).some((el) => (el as HTMLInputElement).value.includes("vợ/chồng"))).toBe(true);

    await user.click(screen.getByRole("tab", { name: /Thông tin/ }));
    await user.click(screen.getByRole("combobox", { name: "Bối cảnh" }));
    await user.click(await screen.findByRole("option", { name: /Hiện đại/ }));
    await user.click(screen.getByRole("tab", { name: /Kiểm tra/ }));
    expect(screen.getAllByLabelText(/^Mô tả rule/).some((el) => (el as HTMLInputElement).value.includes("vợ/chồng"))).toBe(false);
  });
});
```

Radix Select trong jsdom cần `hasPointerCapture`/`scrollIntoView` stub — kiểm `src/test/setup.ts` của qt-web đã có chưa (grep `hasPointerCapture`); nếu chưa, thêm vào setup:

```ts
Element.prototype.hasPointerCapture ??= () => false;
Element.prototype.scrollIntoView ??= () => {};
```

- [ ] **Step 2: Chạy test, xác nhận đỏ**

Run: `cd apps/qt-web && npx vitest run src/components/ai-story-config-dialog.test.tsx`
Expected: FAIL — không có combobox "Bối cảnh".

- [ ] **Step 3: Thêm hai Select vào tab Thông tin**

Trong `ai-story-config-dialog.tsx`, import thêm:

```ts
import { GENRE_NAMES, GENRE_NAMES_LABELS, GENRE_SETTINGS, GENRE_SETTING_LABELS, normalizeAiStoryConfig, type AiStoryConfig, type GenreNames, type GenreSetting } from "@/lib/ai-story";
import { composeBasePrompt } from "@/lib/ai-translation-prompt";
import { defaultAiCheckRules } from "@/lib/ai-translation";
```

(gộp với import `@/lib/ai-story` sẵn có; bỏ import `NOVEL_TRANSLATOR_BASE_PROMPT`.)

Trong cột trái tab `info`, sau khối `story-protagonist`, thêm:

```tsx
                <div className="grid grid-cols-2 gap-3">
                  <div className="grid gap-2">
                    <Label htmlFor="story-genre-setting">Bối cảnh</Label>
                    <Select
                      value={draft.genre.setting}
                      onValueChange={(setting) =>
                        patch({ genre: { ...draft.genre, setting: setting as GenreSetting } })
                      }
                    >
                      <SelectTrigger id="story-genre-setting" aria-label="Bối cảnh" className="bg-card">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {GENRE_SETTINGS.map((id) => (
                          <SelectItem key={id} value={id}>{GENRE_SETTING_LABELS[id].label}</SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <p className="text-xs text-muted-foreground">{GENRE_SETTING_LABELS[draft.genre.setting].hint}</p>
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="story-genre-names">Tên riêng</Label>
                    <Select
                      value={draft.genre.names}
                      onValueChange={(names) => patch({ genre: { ...draft.genre, names: names as GenreNames } })}
                    >
                      <SelectTrigger id="story-genre-names" aria-label="Tên riêng" className="bg-card">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {GENRE_NAMES.map((id) => (
                          <SelectItem key={id} value={id}>{GENRE_NAMES_LABELS[id].label}</SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <p className="text-xs text-muted-foreground">{GENRE_NAMES_LABELS[draft.genre.names].hint}</p>
                  </div>
                </div>
```

Prompt tab: `initialValue={draft.customPrompt || composeBasePrompt(draft.genre)}` và `key={`${promptEditorVersion}-${draft.genre.setting}-${draft.genre.names}`}` để editor nạp lại bản mặc định mới khi đổi genre (chỉ khi đang dùng mặc định — `customPrompt` rỗng; khi có customPrompt, key đổi vẫn nạp lại chính customPrompt nên vô hại). Mô tả tab Prompt sửa thành: `Bản mặc định ghép theo bối cảnh và tên riêng đã chọn; từ điển, style và thông tin truyện vẫn được nối vào phía sau.`

Rules tab: `effectiveRules` đã dùng `defaultAiCheckRules(draft.genre.setting)` từ Task 3.

- [ ] **Step 4: AI fill trả genre**

Trong `ai-story-fill.ts`, JSON mẫu trong prompt thêm sau `"summary"`:

```
  "genre": {
    "setting": "ancient | modern — ancient nếu cổ đại/tiên hiệp/huyền huyễn/cung đấu, modern nếu đô thị/hiện đại/vô hạn lưu",
    "names": "han | foreign | mixed — han nếu nhân vật Trung Quốc, foreign nếu bối cảnh phương Tây/Nhật/Hàn, mixed nếu lẫn"
  },
```

Trong `normalizeAiStoryConfig({...})` của hàm, thêm `genre: isRecord(json.genre) ? json.genre : current.genre,` và trong `values` trả về thêm `genre: normalized.genre,`.

- [ ] **Step 5: Chạy test, lint, typecheck; commit**

Run: `cd apps/qt-web && npx vitest run src/components/ai-story-config-dialog.test.tsx && npm run typecheck && npm run lint`
Expected: PASS.

```bash
git add apps/qt-web/src/components/ai-story-config-dialog.tsx apps/qt-web/src/components/ai-story-config-dialog.test.tsx apps/qt-web/src/lib/ai-story-fill.ts apps/qt-web/src/test/setup.ts
git commit -m "feat(qt-web): chọn bối cảnh và tên riêng trong Cấu hình truyện, prompt/rule mặc định theo genre, AI fill điền genre"
```

---

### Task 5: Golden mới — 6 base, rule theo setting, case genre

**Files:**
- Modify: `apps/qt-ai-cli/scripts/gen-golden.ts`
- Regenerate: `crates/qt-ai-core/prompts/prompts.json`, `crates/qt-ai-core/tests/fixtures/{story,prompt,check}.json`

**Interfaces:**
- Produces: `prompts.json = { bases: Record<"ancient/han" | …, string>, suffix: string }`; `check.json = { defaultRules: { ancient: AiCheckRule[]; modern: AiCheckRule[] }, cases: Array<{ name, text, rules, setting: "ancient" | "modern", violations }> }`; `prompt.json` thêm 6 case `genre-<key>` và `genre-custom-prompt`.

Sau task này Rust đỏ tới hết Task 6 — đó là luồng golden bình thường (`golden` → sửa Rust cho xanh).

- [ ] **Step 1: Sửa gen-golden.ts**

Import:

```ts
import { composeBasePrompt, genreKey, PROMPT_GENRE_COMBOS } from "@/lib/ai-translation-prompt";
import {
  defaultAiCheckRules, buildAiTranslationSystemPrompt, checkAiTranslationViolations,
  filterTranslationGlossaryForSource, formatAiTranslation, glossaryEntryMatchesSource,
} from "@/lib/ai-translation";
import { emptyAiStoryConfig, naturalChapterCompare, normalizeAiStoryConfig, type StoryGenre } from "@/lib/ai-story";
```

Thay khối `const prompts = ...`:

```ts
const prompts = (() => {
  const noStory = buildAiTranslationSystemPrompt({});
  const legacy = composeBasePrompt({ setting: "ancient", names: "han" });
  return {
    bases: Object.fromEntries(PROMPT_GENRE_COMBOS.map((g) => [genreKey(g), composeBasePrompt(g)])),
    suffix: noStory.slice(legacy.length),
  };
})();
```

Thêm helper và case genre:

```ts
function storyGenre(genre: StoryGenre) {
  const story = emptyAiStoryConfig();
  story.name = "Truyện thử genre";
  story.summary = "Một câu.";
  story.genre = genre;
  return story;
}
function storyGenreCustomPrompt() {
  const story = storyGenre({ setting: "modern", names: "foreign" });
  story.customPrompt = "Prompt riêng thắng genre.";
  return story;
}
```

`promptCases` thêm sau `{ name: "rules", ... }`:

```ts
  ...PROMPT_GENRE_COMBOS.map((g) => ({ name: `genre-${genreKey(g)}`, story: storyGenre(g), source: CH1 })),
  { name: "genre-custom-prompt", story: storyGenreCustomPrompt(), source: CH1 },
```

Thay khối `const check = {...}`:

```ts
const MODERN_TEXT = "Vợ anh đang đợi ở công ty.\nNgươi dám nói vậy sao?\nThê tử của tổng tài Lâm, ừm.";
const check = {
  defaultRules: { ancient: defaultAiCheckRules("ancient"), modern: defaultAiCheckRules("modern") },
  cases: [
    { name: "default", text: CHECK_TEXT, rules: null, setting: "ancient", violations: checkAiTranslationViolations(CHECK_TEXT) },
    { name: "crlf", text: "a…\r\nb，", rules: null, setting: "ancient", violations: checkAiTranslationViolations("a…\r\nb，") },
    { name: "story-rules", text: "có từ cấm và 漢 và Vợ", rules: storyRules().checkRules, setting: "ancient", violations: checkAiTranslationViolations("có từ cấm và 漢 và Vợ", storyRules().checkRules) },
    { name: "story-rules-no-han-override", text: "漢 sót", rules: SIMPLE_RULES, setting: "ancient", violations: checkAiTranslationViolations("漢 sót", SIMPLE_RULES) },
    { name: "long-line-cut-120", text: "não hải ".repeat(30), rules: null, setting: "ancient", violations: checkAiTranslationViolations("não hải ".repeat(30)) },
    { name: "modern-default", text: MODERN_TEXT, rules: null, setting: "modern", violations: checkAiTranslationViolations(MODERN_TEXT, undefined, "modern") },
    { name: "modern-story-rules-ignore-setting", text: MODERN_TEXT, rules: SIMPLE_RULES, setting: "modern", violations: checkAiTranslationViolations(MODERN_TEXT, SIMPLE_RULES, "modern") },
    { name: "ancient-on-modern-text", text: MODERN_TEXT, rules: null, setting: "ancient", violations: checkAiTranslationViolations(MODERN_TEXT) },
  ],
};
```

`story.normalize` thêm case thứ ba:

```ts
    { input: { genre: { setting: "modern", names: "sai" } }, output: null as unknown },
```

và sau vòng gán output hiện có thêm: `story.normalize[2]!.output = normalizeAiStoryConfig(story.normalize[2]!.input);`

- [ ] **Step 2: Sinh fixture, xem diff**

Run: `cd apps/qt-ai-cli && npm run typecheck && npm run golden && git status --short ../../crates/qt-ai-core`
Expected: 4 file đổi (`prompts.json`, `story.json`, `prompt.json`, `check.json`); `prompts.json` có 6 key trong `bases`; `git diff --stat` cho `prompt.json` chỉ thêm case mới, 7 case cũ không đổi (kiểm `git diff crates/qt-ai-core/tests/fixtures/prompt.json | grep '^-' | grep -v '^---'` không ra dòng nào ngoài `"cases"` cấu trúc).

- [ ] **Step 3: Commit**

```bash
git add apps/qt-ai-cli/scripts/gen-golden.ts crates/qt-ai-core/prompts/prompts.json crates/qt-ai-core/tests/fixtures
git commit -m "test(golden): prompts.json 6 base theo genre, check.json rule theo setting, case genre và hiện đại"
```

---

### Task 6: Port Rust qt-ai-core

**Files:**
- Modify: `crates/qt-ai-core/src/story.rs`
- Modify: `crates/qt-ai-core/src/prompt.rs`
- Modify: `crates/qt-ai-core/src/check.rs`
- Modify: `crates/qt-ai-core/src/commands/check.rs:96`, `crates/qt-ai-core/src/api_session.rs:206-207`
- Modify: `crates/qt-ai-core/tests/golden.rs`

**Interfaces:**
- Produces: `story::GenreSetting { Ancient, Modern }` (serde lowercase, `Default = Ancient`), `story::GenreNames { Han, Foreign, Mixed }` (`Default = Han`), `story::StoryGenre { setting, names }` (`Default`, `key(&self) -> String` → `"ancient/han"`), `StoryConfig.genre` (sau `summary`), `prompt::base_prompt(genre: &StoryGenre) -> &'static str`, `check::default_rules_as_check_rules(setting: GenreSetting) -> Vec<CheckRule>`, `check::check_violations(text, configured, setting: GenreSetting) -> Vec<Violation>`.

- [ ] **Step 1: Viết test đỏ trong story.rs**

Thêm vào `mod tests` của `story.rs`:

```rust
    #[test]
    fn normalize_genre_thieu_hoac_sai_ve_ancient_han() {
        let none = StoryConfig::normalize(&json!({}));
        assert_eq!(none.genre, StoryGenre::default());
        assert_eq!(none.genre.key(), "ancient/han");
        let ok = StoryConfig::normalize(&json!({ "genre": { "setting": "modern", "names": "foreign" } }));
        assert_eq!(ok.genre, StoryGenre { setting: GenreSetting::Modern, names: GenreNames::Foreign });
        let bad = StoryConfig::normalize(&json!({ "genre": { "setting": "future", "names": 3 } }));
        assert_eq!(bad.genre, StoryGenre::default());
        let json = ok.to_json_pretty();
        assert!(json.contains("\"summary\": \"\",\n  \"genre\": {\n    \"setting\": \"modern\",\n    \"names\": \"foreign\"\n  },\n  \"glossary\""));
    }
```

Run: `cd crates/qt-ai-core && cargo test -p qt-ai-core story::` — Expected: lỗi biên dịch (chưa có `StoryGenre`).

- [ ] **Step 2: Thêm genre vào story.rs**

Sau `pub struct StoryStyle {...}`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenreSetting {
    #[default]
    Ancient,
    Modern,
}

impl GenreSetting {
    pub fn as_str(self) -> &'static str {
        match self {
            GenreSetting::Ancient => "ancient",
            GenreSetting::Modern => "modern",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GenreNames {
    #[default]
    Han,
    Foreign,
    Mixed,
}

impl GenreNames {
    pub fn as_str(self) -> &'static str {
        match self {
            GenreNames::Han => "han",
            GenreNames::Foreign => "foreign",
            GenreNames::Mixed => "mixed",
        }
    }
}

/// Hai trục độc lập (port `StoryGenre` của qt-web). Thiếu/sai → ancient/han để truyện cũ không đổi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StoryGenre {
    pub setting: GenreSetting,
    pub names: GenreNames,
}

impl StoryGenre {
    /// Key tra `prompts.json` — cùng định dạng `genreKey` của web.
    pub fn key(&self) -> String {
        format!("{}/{}", self.setting.as_str(), self.names.as_str())
    }

    fn normalize(value: Option<&Value>) -> Self {
        let record = value.and_then(Value::as_object);
        let get = |key: &str| record.and_then(|r| r.get(key)).and_then(Value::as_str);
        StoryGenre {
            setting: match get("setting") {
                Some("modern") => GenreSetting::Modern,
                _ => GenreSetting::Ancient,
            },
            names: match get("names") {
                Some("foreign") => GenreNames::Foreign,
                Some("mixed") => GenreNames::Mixed,
                _ => GenreNames::Han,
            },
        }
    }
}
```

`StoryConfig`: thêm `pub genre: StoryGenre,` ngay sau `pub summary: String,`. `empty()`: thêm `genre: StoryGenre::default(),` sau `summary`. `normalize()`: thêm `genre: StoryGenre::normalize(get("genre")),` sau `summary`. Test `empty_round_trip_giu_thu_tu_key_nhu_web` sửa chuỗi `starts_with` thành:

```rust
            "{\n  \"name\": \"\",\n  \"sourceUrl\": \"\",\n  \"protagonist\": \"\",\n  \"summary\": \"\",\n  \"genre\": {\n    \"setting\": \"ancient\",\n    \"names\": \"han\"\n  },\n  \"glossary\": {\n    \"names\": {},"
```

Run: `cargo test -p qt-ai-core story::` — Expected: PASS.

- [ ] **Step 3: prompt.rs tra base theo genre**

```rust
use crate::story::{Glossary, StoryConfig, StoryGenre, StringMap};
use indexmap::IndexMap;

#[derive(Deserialize)]
struct Prompts {
    /// 6 base đã ghép sẵn ở web theo key `setting/names` — Rust không port logic ghép.
    bases: IndexMap<String, String>,
    suffix: String,
}

pub fn base_prompt(genre: &StoryGenre) -> &'static str {
    PROMPTS
        .bases
        .get(&genre.key())
        .map(String::as_str)
        .unwrap_or_else(|| panic!("prompts.json thiếu base cho genre {}", genre.key()))
}
```

Trong `build_system_prompt`:

```rust
    let default_genre = StoryGenre::default();
    let base = story
        .map(|s| s.custom_prompt.trim())
        .filter(|custom| !custom.is_empty())
        .unwrap_or_else(|| base_prompt(story.map(|s| &s.genre).unwrap_or(&default_genre)));
```

Thêm test trong `prompt.rs` (tạo `#[cfg(test)] mod tests` nếu chưa có):

```rust
    #[test]
    fn base_prompt_co_du_6_genre_va_khac_nhau() {
        use crate::story::{GenreNames, GenreSetting};
        let mut seen = std::collections::HashSet::new();
        for setting in [GenreSetting::Ancient, GenreSetting::Modern] {
            for names in [GenreNames::Han, GenreNames::Foreign, GenreNames::Mixed] {
                let base = base_prompt(&StoryGenre { setting, names });
                assert!(base.len() > 5000, "{setting:?}/{names:?}");
                assert!(seen.insert(base), "trùng base {setting:?}/{names:?}");
            }
        }
        assert!(base_prompt(&StoryGenre::default()).contains("| 我          | **ta**"));
        assert!(!base_prompt(&StoryGenre { setting: GenreSetting::Modern, names: GenreNames::Han }).contains("| 我          | **ta**"));
    }
```

- [ ] **Step 4: check.rs theo setting**

Đổi `DEFAULT_RULES` thành 4 phần tử `(&str, &str, &str, Option<&str>)`; gắn `Some("ancient")` cho đúng 11 rule liệt kê ở Task 3, `None` cho phần còn lại, thêm 4 rule modern cuối mảng (copy regex/message nguyên văn từ web):

```rust
pub const DEFAULT_RULES: &[(&str, &str, &str, Option<&str>)] = &[
    (r"[，。、；：？！]", "", "Dấu câu tiếng Trung còn sót → dùng dấu câu thường", None),
    // …
    (r"(?<!\p{L})(?:vợ|chồng)(?!\p{L})", "iu", "Dùng vợ/chồng → thay bằng thê tử/phu quân", Some("ancient")),
    // …
    (r"…", "", "Còn ký tự … → chuẩn hóa thành dấu chấm ASCII, giữ số lượng (… → ..., …… → ......)", None),
    (r"(?<!\p{L})(?:ngươi|nàng|bọn ta|các ngươi)(?!\p{L})", "iu", "Xưng hô cổ trang trong truyện hiện đại → anh/cô/tôi theo quan hệ", Some("modern")),
    (r"(?<!\p{L})(?:thê tử|phu quân|lang quân|phụ thân|mẫu thân)(?!\p{L})", "iu", "Từ gia đình cổ trang → vợ/chồng/bố/mẹ", Some("modern")),
    (r"(?<!\p{L})tổng tài(?!\p{L})", "iu", "tổng tài → tổng giám đốc", Some("modern")),
    (r"nói đạo", "i", "说道 → nói / đáp, không \"nói đạo\"", Some("modern")),
];

fn rules_for(setting: GenreSetting) -> impl Iterator<Item = &'static (&'static str, &'static str, &'static str, Option<&'static str>)> {
    DEFAULT_RULES.iter().filter(move |(_, _, _, tag)| tag.is_none_or(|t| t == setting.as_str()))
}

pub fn default_rules_as_check_rules(setting: GenreSetting) -> Vec<CheckRule> {
    rules_for(setting)
        .map(|(pattern, flags, message, _)| CheckRule {
            pattern: pattern.to_string(),
            flags: (!flags.is_empty()).then(|| flags.to_string()),
            message: message.to_string(),
        })
        .collect()
}

pub fn check_violations(text: &str, configured: &[CheckRule], setting: GenreSetting) -> Vec<Violation> {
    let mut rules: Vec<CompiledRule> = if configured.is_empty() {
        rules_for(setting).filter_map(|(p, f, m, _)| compile(p, f, m)).collect()
    } else {
        // … như cũ
```

(`Option::is_none_or` cần Rust 1.82+; nếu toolchain cũ hơn dùng `tag.map_or(true, |t| t == setting.as_str())`.) Import `use crate::story::{CheckRule, GenreSetting};`. Test trong `check.rs` (nếu có `mod tests` dùng `check_violations(text, &[])`) thêm tham số `GenreSetting::Ancient`.

Người gọi: `commands/check.rs:96` → `check_violations(&final_text, &story.check_rules, story.genre.setting)`; `api_session.rs:206-207` → thêm `story.genre.setting` (biến `story` đã có trong scope — kiểm chữ ký hàm chứa hai dòng đó, thêm tham số nếu `story` chưa được truyền vào).

- [ ] **Step 5: golden.rs theo fixture mới**

```rust
use qt_ai_core::story::{natural_chapter_compare, AutoGlossarySetting, CheckRule, GenreSetting, Glossary, StoryConfig};

#[derive(Deserialize)]
struct CheckFixture {
    #[serde(rename = "defaultRules")]
    default_rules: DefaultRules,
    cases: Vec<CheckCase>,
}
#[derive(Deserialize)]
struct DefaultRules {
    ancient: Vec<CheckRule>,
    modern: Vec<CheckRule>,
}
#[derive(Deserialize)]
struct CheckCase {
    name: String,
    text: String,
    rules: Option<Vec<CheckRule>>,
    setting: GenreSetting,
    violations: Vec<Violation>,
}

#[test]
fn check_rules_mac_dinh_theo_setting_nhu_web() {
    let f: CheckFixture = fixture(include_str!("fixtures/check.json"));
    assert_eq!(default_rules_as_check_rules(GenreSetting::Ancient), f.default_rules.ancient);
    assert_eq!(default_rules_as_check_rules(GenreSetting::Modern), f.default_rules.modern);
}

#[test]
fn check_violations_khop_web() {
    let f: CheckFixture = fixture(include_str!("fixtures/check.json"));
    for case in f.cases {
        let got = check_violations(&case.text, case.rules.as_deref().unwrap_or(&[]), case.setting);
        assert_eq!(got, case.violations, "case {}", case.name);
    }
}
```

Xoá test cũ `check_rules_mac_dinh_dung_63_rule_nhu_web`. `prompt_khop_tung_byte_voi_web` và `story_normalize_va_sort_khop_ts` không cần sửa (case mới tự chạy qua).

- [ ] **Step 6: Chạy toàn bộ test core + GUI Rust**

Run: `cd crates/qt-ai-core && cargo test -p qt-ai-core`
Expected: PASS toàn bộ, gồm `prompt_khop_tung_byte_voi_web` với 7 case genre mới.

`cargo test -p qt-ai-gui` lúc này đỏ biên dịch ở `story_cmds.rs` (`base_prompt()` và `default_rules_as_check_rules()` đổi chữ ký) — Task 7 sửa.

- [ ] **Step 7: Commit**

```bash
git add crates/qt-ai-core
git commit -m "feat(qt-ai-core): StoryConfig.genre, base prompt tra theo genre, check rule theo setting"
```

---

### Task 7: qt-ai-gui — mục Thể loại, defaults theo genre

**Files:**
- Modify: `apps/qt-ai-gui/src-tauri/src/story_cmds.rs:108-128, 312-321`
- Modify: `apps/qt-ai-gui/src/lib/schema.ts`, `src/lib/api.ts:72`, `src/lib/story-form.ts`, `src/hooks/use-story-defaults.ts`
- Modify: `apps/qt-ai-gui/src/components/pages/story-page.tsx`
- Test: `apps/qt-ai-gui/src/lib/story-form.test.ts` (thêm nếu chưa có), `src/components/pages/story-page.test.tsx`

**Interfaces:**
- Produces: Tauri command `story_defaults(genre: StoryGenre) -> StoryDefaults`; TS `storyDefaults(genre: StoryGenre)`; `useStoryDefaults(genre: StoryGenre)`; `StoryFormValues.genreSetting`, `genreNames`; `storyConfigSchema.genre`.

- [ ] **Step 1: Rust — `story_defaults(genre)`**

```rust
use qt_ai_core::story::{natural_chapter_compare, CheckRule, StoryConfig, StoryGenre};

pub fn defaults(genre: &StoryGenre) -> StoryDefaults {
    StoryDefaults {
        base_prompt: base_prompt(genre).to_string(),
        prompt_suffix: prompt_suffix().to_string(),
        check_rules: default_rules_as_check_rules(genre.setting),
    }
}

#[tauri::command]
pub fn story_defaults(genre: StoryGenre) -> CmdResult<StoryDefaults> {
    Ok(defaults(&genre))
}
```

Test `defaults_co_prompt_goc_duoi_va_bo_rule_mac_dinh` sửa thành:

```rust
    #[test]
    fn defaults_theo_genre() {
        use qt_ai_core::story::{GenreNames, GenreSetting};
        let d = defaults(&StoryGenre::default());
        assert!(d.base_prompt.len() > 200 && !d.base_prompt.contains("Dịch raw text tiếng Trung"));
        assert!(d.prompt_suffix.contains("Dịch raw text tiếng Trung"));
        assert!(d.check_rules.iter().any(|r| r.message.contains("vợ/chồng")));
        let m = defaults(&StoryGenre { setting: GenreSetting::Modern, names: GenreNames::Foreign });
        assert!(m.base_prompt.contains("Emily"));
        assert!(!m.check_rules.iter().any(|r| r.message.contains("vợ/chồng")));
        let json = serde_json::to_value(&d).unwrap();
        assert!(json["checkRules"][0]["pattern"].is_string());
    }
```

Run: `cd apps/qt-ai-gui/src-tauri && cargo test -p qt-ai-gui` — Expected: PASS.

- [ ] **Step 2: TS schema, api, form (test đỏ trước)**

Tạo/bổ sung `apps/qt-ai-gui/src/lib/story-form.test.ts`:

```ts
import { describe, expect, it } from "vitest";

import { storyConfigSchema } from "@/lib/schema";
import { diffStoryConfig, fromFormValues, toFormValues } from "@/lib/story-form";

const base = storyConfigSchema.parse({
  name: "A", sourceUrl: "", protagonist: "", summary: "",
  genre: { setting: "modern", names: "mixed" },
  glossary: { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {} },
  style: { voice: "", toneRules: [], signaturePhrases: {}, avoid: [] },
  customPrompt: "", checkRules: [], autoGlossaryLog: [], autoGlossary: "inherit",
});

describe("story-form genre", () => {
  it("round-trip genre qua form", () => {
    const values = toFormValues(base);
    expect(values.genreSetting).toBe("modern");
    expect(values.genreNames).toBe("mixed");
    expect(fromFormValues({ ...values, genreNames: "foreign" }, base).genre).toEqual({ setting: "modern", names: "foreign" });
  });
  it("diff liệt kê genre khi đổi", () => {
    const after = { ...base, genre: { setting: "ancient" as const, names: "han" as const } };
    expect(diffStoryConfig(base, after).map((d) => d.field)).toEqual(["genre"]);
  });
  it("schema bắt buộc genre hợp lệ", () => {
    expect(() => storyConfigSchema.parse({ ...base, genre: { setting: "x", names: "han" } })).toThrow();
  });
});
```

Run: `cd apps/qt-ai-gui && npx vitest run src/lib/story-form.test.ts` — Expected: FAIL.

`schema.ts` thêm sau `checkRuleSchema`:

```ts
export const GENRE_SETTINGS = ["ancient", "modern"] as const;
export const GENRE_NAMES = ["han", "foreign", "mixed"] as const;
export const storyGenreSchema = z.object({ setting: z.enum(GENRE_SETTINGS), names: z.enum(GENRE_NAMES) });
```

và trong `storyConfigSchema` thêm `genre: storyGenreSchema,` sau `summary`. `types.ts` export `StoryGenre = z.infer<typeof storyGenreSchema>`, `GenreSetting`, `GenreNames`, và:

```ts
export const GENRE_SETTING_LABELS: Record<GenreSetting, { label: string; hint: string }> = {
  ancient: { label: "Cổ đại / tiên hiệp", hint: "ta/ngươi/hắn/nàng, thán từ A?/Ân, cấm vợ/chồng" },
  modern: { label: "Hiện đại", hint: "anh/cô/tôi theo quan hệ, từ đời thường, thán từ hiện đại" },
};
export const GENRE_NAMES_LABELS: Record<GenreNames, { label: string; hint: string }> = {
  han: { label: "Hán-Việt", hint: "Kế Duyên, Bắc Kinh" },
  foreign: { label: "Gốc nước ngoài", hint: "Emily, New York, Naruto" },
  mixed: { label: "Hỗn hợp", hint: "Họ Hán → Hán-Việt, tên phiên âm → gốc" },
};
```

`api.ts`: `export const storyDefaults = (genre: StoryGenre) => call("story_defaults", { genre }, (v) => storyDefaultsSchema.parse(v));`

`story-form.ts`: `storyFormSchema` thêm `genreSetting: z.enum(GENRE_SETTINGS), genreNames: z.enum(GENRE_NAMES),`; `toFormValues` thêm `genreSetting: config.genre.setting, genreNames: config.genre.names,`; `fromFormValues` thêm `genre: { setting: values.genreSetting, names: values.genreNames },` sau `summary`; `diffStoryConfig` thêm `["genre", before.genre, after.genre],` sau `summary`.

`use-story-defaults.ts`: cache theo key:

```ts
const cache = new Map<string, Promise<StoryDefaults>>();

/** Prompt gốc + rule mặc định theo genre — tải một lần mỗi tổ hợp, dùng chung. */
export function useStoryDefaults(genre: StoryGenre): StoryDefaults | undefined {
  const key = `${genre.setting}/${genre.names}`;
  const [state, setState] = useState<{ key: string; value: StoryDefaults } | undefined>();
  useEffect(() => {
    let cancelled = false;
    let pending = cache.get(key);
    if (!pending) {
      pending = storyDefaults(genre);
      cache.set(key, pending);
    }
    pending
      .then((value) => {
        if (!cancelled) setState({ key, value });
      })
      .catch((error: unknown) => {
        cache.delete(key);
        toast.error(error instanceof Error ? error.message : "Không đọc được prompt mặc định");
      });
    return () => {
      cancelled = true;
    };
  }, [key, genre]);
  return state?.key === key ? state.value : undefined;
}
```

(`genre` trong deps: truyền object ổn định — ở story-page dựng bằng `useMemo` theo hai giá trị watch.)

Run: `npx vitest run src/lib/story-form.test.ts && npm run typecheck` — typecheck sẽ đỏ ở story-page (chưa truyền genre) và test story-page (mock `storyDefaults` không có tham số vẫn OK, nhưng snapshot fixture thiếu `genre`) — Step 3 sửa.

- [ ] **Step 3: Story page — mục Thể loại (test đỏ trước)**

`story-page.test.tsx`: fixture `story` thêm `genre: { setting: "ancient", names: "han" },` sau `summary`; mock `storyDefaults: vi.fn((genre: { setting: string }) => Promise.resolve({ basePrompt: genre.setting === "modern" ? "Prompt hiện đại." : "Prompt gốc.", promptSuffix: "Đuôi.", checkRules: [] }))`. Thêm test:

```ts
  it("mục Thể loại đổi bối cảnh làm form dirty và prompt mặc định nạp lại theo genre", async () => {
    const user = userEvent.setup();
    render(<StoryPage />);
    await user.click(screen.getByRole("tab", { name: "Thể loại" }));
    await user.click(screen.getByRole("combobox", { name: "Bối cảnh" }));
    await user.click(await screen.findByRole("option", { name: /Hiện đại/ }));
    expect(screen.getByText("Có thay đổi chưa lưu")).toBeInTheDocument();
    await user.click(screen.getByRole("tab", { name: "Prompt" }));
    expect(await screen.findByText(/Prompt hiện đại\./)).toBeInTheDocument();
  });
```

Nếu Plate editor không render text thuần trong jsdom, thay assertion cuối bằng kiểm `storyDefaults` được gọi với `{ setting: "modern", names: "han" }`:

```ts
    const { storyDefaults } = await import("@/lib/api");
    expect(storyDefaults).toHaveBeenCalledWith({ setting: "modern", names: "han" });
```

Radix Select trong jsdom cần stub `Element.prototype.hasPointerCapture ??= () => false; Element.prototype.scrollIntoView ??= () => {};` — kiểm `apps/qt-ai-gui/src/test/setup.ts`, chưa có thì thêm.

Run: `npx vitest run src/components/pages/story-page.test.tsx` — Expected: FAIL (không có tab "Thể loại").

`story-page.tsx`:
- `SECTIONS` thêm `{ id: "genre", label: "Thể loại" }` sau `info`.
- Import `GENRE_NAMES_LABELS, GENRE_SETTING_LABELS, type StoryConfig` từ `@/lib/types`, `GENRE_NAMES, GENRE_SETTINGS` từ `@/lib/schema`, `useMemo` từ react.
- Trong component:

```tsx
  const genreSetting = useWatch({ control: form.control, name: "genreSetting" });
  const genreNames = useWatch({ control: form.control, name: "genreNames" });
  const genre = useMemo(
    () => ({ setting: genreSetting ?? "ancient", names: genreNames ?? "han" }),
    [genreSetting, genreNames],
  );
  const defaults = useStoryDefaults(genre);
```

- Section mới sau `info`:

```tsx
              <Section id="genre" active={active} title="Thể loại">
                <p className="text-sm text-muted-foreground">
                  Quyết định prompt và bộ rule mặc định. Prompt riêng hoặc rule riêng (nếu có) vẫn thắng.
                </p>
                <div className="grid grid-cols-2 gap-4">
                  <Field id="genreSetting" label="Bối cảnh" hint={GENRE_SETTING_LABELS[genre.setting].hint}>
                    <Select
                      value={genre.setting}
                      onValueChange={(v) => form.setValue("genreSetting", v as StoryFormValues["genreSetting"], { shouldDirty: true })}
                    >
                      <SelectTrigger id="genreSetting" aria-label="Bối cảnh">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {GENRE_SETTINGS.map((id) => (
                          <SelectItem key={id} value={id}>{GENRE_SETTING_LABELS[id].label}</SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </Field>
                  <Field id="genreNames" label="Tên riêng" hint={GENRE_NAMES_LABELS[genre.names].hint}>
                    <Select
                      value={genre.names}
                      onValueChange={(v) => form.setValue("genreNames", v as StoryFormValues["genreNames"], { shouldDirty: true })}
                    >
                      <SelectTrigger id="genreNames" aria-label="Tên riêng">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        {GENRE_NAMES.map((id) => (
                          <SelectItem key={id} value={id}>{GENRE_NAMES_LABELS[id].label}</SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </Field>
                </div>
              </Section>
```

`PromptEditor` nhận `defaults` đã theo genre; thêm `key={`${genre.setting}/${genre.names}`}` cho `<PromptEditor>` để editor remount khi đổi genre lúc đang dùng mặc định (customPrompt rỗng): `<PromptEditor key={`${genre.setting}/${genre.names}`} defaults={defaults} />`.

- [ ] **Step 4: Chạy toàn bộ check GUI**

Run: `cd apps/qt-ai-gui && npm run check && cd src-tauri && cargo test -p qt-ai-gui`
Expected: PASS. Test `no-hardcoded-colors` và theme vẫn xanh.

- [ ] **Step 5: Commit**

```bash
git add apps/qt-ai-gui
git commit -m "feat(qt-ai-gui): mục Thể loại trong Hồ sơ truyện, prompt/rule mặc định theo genre"
```

---

### Task 8: Workflow setup-story, README, kiểm cuối

**Files:**
- Modify: `apps/qt-ai-cli/antigravity/workflows/setup-story.md`
- Modify: `apps/qt-ai-gui/README.md`
- Modify: `apps/qt-web/src/lib/ai-translation.test.ts:25-26` (bỏ test `NOVEL_TRANSLATOR_BASE_PROMPT` thô nếu còn dùng hằng deprecated), `apps/qt-web/src/lib/ai-translation-prompt.ts` (xoá `NOVEL_TRANSLATOR_BASE_PROMPT` nếu không còn ai import)

- [ ] **Step 1: Workflow điền genre**

Trong `setup-story.md`, bước 4 thêm dòng sau `name`/`sourceUrl`:

```
   - `genre.setting`: `ancient` (cổ đại, tiên hiệp, huyền huyễn, cung đấu, lịch sử) hoặc `modern`
     (đô thị, hiện đại, vô hạn lưu, hệ thống thời nay). `genre.names`: `han` nếu nhân vật Trung Quốc,
     `foreign` nếu bối cảnh phương Tây / Nhật / Hàn (tên trả về Emily, Naruto), `mixed` nếu lẫn.
     Suy từ thể loại tra được và 2–3 chương đã đọc; không chắc thì giữ `ancient`/`han` và ghi chú khi trình.
```

Bước 2 sửa "thể loại" thành "thể loại (để chọn `genre`)". Kiểm tra AGENTS.md/translate.md không nhắc field story.json (grep `genre` không có gì cần sửa).

- [ ] **Step 2: README GUI**

Thêm mục sau "## Giao diện":

```markdown
## Thể loại

`story.json` có `genre: { setting: "ancient" | "modern", names: "han" | "foreign" | "mixed" }`; chọn ở trang Hồ sơ truyện, mục Thể loại. Bối cảnh quyết xưng hô, thán từ, từ gia đình, bảng thuật ngữ và bộ rule kiểm tra mặc định; tên riêng quyết phiên Hán-Việt hay trả về dạng gốc. Truyện cũ thiếu `genre` chạy như cổ đại/Hán-Việt. Prompt riêng hoặc rule riêng vẫn thắng. Chữ prompt nằm ở qt-web (`ai-translation-prompt.ts`), Rust đọc 6 bản ghép sẵn trong `crates/qt-ai-core/prompts/prompts.json` qua golden.
```

- [ ] **Step 3: Dọn hằng deprecated**

`grep -rn NOVEL_TRANSLATOR_BASE_PROMPT apps --include=*.ts --include=*.tsx | grep -v node_modules`. Nếu chỉ còn `ai-translation-prompt.ts` và test `stores the ported prompt with real Markdown line breaks`, sửa test đó dùng `composeBasePrompt(defaultStoryGenre())` rồi xoá hằng.

- [ ] **Step 4: Kiểm cuối toàn bộ**

```bash
cd apps/qt-ai-cli && npm run -s golden:check && npx vitest run && cd ../qt-web && npm run check && cd ../qt-ai-gui && npm run check && cd ../../crates/qt-ai-core && cargo test -p qt-ai-core && cargo test -p qt-ai-gui
```

Expected: "Golden khớp.", mọi test PASS, lint/typecheck 0 lỗi.

- [ ] **Step 5: Commit**

```bash
git add apps/qt-ai-cli/antigravity/workflows/setup-story.md apps/qt-ai-gui/README.md apps/qt-web/src/lib
git commit -m "docs: workflow setup-story điền genre, README thể loại; dọn hằng prompt cũ"
```

Sau đó dùng superpowers:finishing-a-development-branch.
