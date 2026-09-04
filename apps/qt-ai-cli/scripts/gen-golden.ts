/**
 * Sinh golden fixtures cho crate Rust qt-ai-core từ code TS THẬT của qt-web.
 * Chạy: npm run golden          → ghi file
 *       npm run golden:check    → so với file đang có, exit 1 nếu lệch (bắt drift)
 */
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { composeBasePrompt, genreKey, PROMPT_GENRE_COMBOS } from "@/lib/ai-translation-prompt";
import {
  defaultAiCheckRules, buildAiTranslationSystemPrompt, checkAiTranslationViolations,
  filterTranslationGlossaryForSource, formatAiTranslation, glossaryEntryMatchesSource,
} from "@/lib/ai-translation";
import {
  aiParagraphsOf, labeledAiRepairPayload, labeledAiSourcePayload,
  parseLabeledAiTranslation, stripAiParagraphMarkers,
} from "@/lib/ai-paragraphs";
import { appendAutoGlossary, collectGlossaryKeys, sanitizeExtractedGlossary } from "@/lib/ai-glossary";
import { emptyAiStoryConfig, naturalChapterCompare, normalizeAiStoryConfig, type StoryGenre } from "@/lib/ai-story";

const HERE = dirname(fileURLToPath(import.meta.url));
const CORE = resolve(HERE, "../../../crates/qt-ai-core");
const FIXTURES = join(CORE, "tests", "fixtures");
const CHECK_ONLY = process.argv.includes("--check");

const CH1 = "赵静文抬头看向远方的高塔。\n\n她沉默了很久没有说话。";
const CH2 = "第二天早上，赵静文和他们一起出发了。\r\n  慕容雪羽也来了。\r\n";

function storyFull() {
  const story = emptyAiStoryConfig();
  story.name = "Phương Thốn Đạo Chủ";
  story.protagonist = "Triệu Tĩnh Văn";
  story.summary = "Tóm tắt 1 câu.";
  story.glossary.names = { "赵静文": "Triệu Tĩnh Văn", "慕容雪羽": "Mộ Dung Tuyết Vũ", "不出现": "Không Xuất Hiện" };
  story.glossary.places = { "高塔": "Cao Tháp" };
  story.glossary.signature_phrases = { "哼": "Hừ" };
  story.style = { voice: "lạnh, kiệm lời", toneRules: ["ta/ngươi", "không dùng mình/tôi"], signaturePhrases: { "方寸": "Phương Thốn" }, avoid: ["anh ấy"] };
  return story;
}
function storyCustomPrompt() {
  const story = storyFull();
  story.customPrompt = "  Prompt riêng của truyện.\nDòng 2.  ";
  return story;
}
function storyRules() {
  const story = storyFull();
  story.checkRules = [
    { pattern: "cấm", message: "Từ cấm" },
    { pattern: "(", message: "regex hỏng — phải bị bỏ qua" },
    { pattern: "\\p{Script=Han}", flags: "u", message: "CJK riêng của truyện (đè rule bắt buộc)" },
  ];
  return story;
}
function storyOnlyStyle() {
  const story = emptyAiStoryConfig();
  story.style.voice = "hào sảng";
  return story;
}

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

// Rust không port logic ghép: xuất sẵn 6 base theo key `setting/names` để tra bảng.
const prompts = (() => {
  const noStory = buildAiTranslationSystemPrompt({});
  const legacy = composeBasePrompt({ setting: "ancient", names: "han" });
  return {
    bases: Object.fromEntries(PROMPT_GENRE_COMBOS.map((g) => [genreKey(g), composeBasePrompt(g)])),
    suffix: noStory.slice(legacy.length),
  };
})();

const story = {
  normalize: [
    { input: {}, output: emptyAiStoryConfig() as unknown },
    {
      input: {
        name: "A", laField: 1, glossary: { names: { "赵静文": "Triệu Tĩnh Văn", " ": "x", k: 5 }, la: { a: "b" } },
        style: { voice: "v", tone_rules: ["r", 3], signature_phrases: { "哼": "Hừ" }, toneRules: null },
        checkRules: [{ pattern: "x", message: "m" }, { pattern: "", message: "m" }, { pattern: "y", flags: "i", message: "n" }, "rác"],
        autoGlossaryLog: [{ source: " 高塔 ", target: "cao tháp", category: "places", chapter: "0001" }, { source: "a", target: "b", category: "sai" }],
        autoGlossary: "off",
      },
      output: null as unknown,
    },
    { input: { genre: { setting: "modern", names: "sai" } }, output: null as unknown },
  ],
  sort: {
    input: ["chuong-0010", "chuong-0002", "0001", "10", "2", "Chuong-0001", "chuong-0002b"],
    sorted: [] as string[],
  },
};
story.normalize[1]!.output = normalizeAiStoryConfig(story.normalize[1]!.input);
story.normalize[2]!.output = normalizeAiStoryConfig(story.normalize[2]!.input);
story.sort.sorted = [...story.sort.input].sort(naturalChapterCompare);

const p1 = aiParagraphsOf(CH1);
const p2 = aiParagraphsOf(CH2);
const PARSE_INPUTS: Array<[string, number]> = [
  ["[[1]] A\n\n[[2]] B", 2],
  ["[[2]]  B dòng 1\n   dòng 2 \n[[1]] A [[1]] lặp [[9]] ngoài [[0]] không", 2],
  ["không nhãn", 2],
  ["[[1]]   \n[[2]] chỉ 2", 3],
];
const paragraphs = {
  paragraphsOf: [{ text: CH1, result: p1 }, { text: CH2, result: p2 }, { text: "  \n\n", result: [] as string[] }],
  sourcePayload: [{ paragraphs: p1, payload: labeledAiSourcePayload(p1) }, { paragraphs: [] as string[], payload: labeledAiSourcePayload([]) }],
  repairPayload: [{ paragraphs: p1, missing: [1], payload: labeledAiRepairPayload(p1, [1]) }],
  parse: PARSE_INPUTS.map(([output, count]) => {
    const parsed = parseLabeledAiTranslation(output, count);
    return { output, count, result: parsed === undefined ? null : parsed.map((x) => x ?? null) };
  }),
  strip: [{ input: "[[1]] A [[12]]B[[3]]", output: stripAiParagraphMarkers("[[1]] A [[12]]B[[3]]") }],
  format: [
    { input: "a  \r\n\r\n\r\nb\nc\n\n", output: formatAiTranslation("a  \r\n\r\n\r\nb\nc\n\n") },
    { input: "\n \n", output: formatAiTranslation("\n \n") },
  ],
};

const promptCases = [
  { name: "no-story", story: null, source: null },
  { name: "empty-story", story: emptyAiStoryConfig(), source: CH1 },
  { name: "full-filtered", story: storyFull(), source: CH1 },
  { name: "full-unfiltered", story: storyFull(), source: null },
  { name: "custom-prompt", story: storyCustomPrompt(), source: CH2 },
  { name: "only-style", story: storyOnlyStyle(), source: CH1 },
  { name: "rules", story: storyRules(), source: CH1 },
  ...PROMPT_GENRE_COMBOS.map((g) => ({ name: `genre-${genreKey(g)}`, story: storyGenre(g), source: CH1 })),
  { name: "genre-custom-prompt", story: storyGenreCustomPrompt(), source: CH1 },
].map((c) => ({
  ...c,
  prompt: buildAiTranslationSystemPrompt({}, c.story ?? undefined, c.source ?? undefined),
}));

const CHECK_TEXT = [
  "Triệu Tĩnh Văn ngẩng đầu，nhìn tòa 高塔 nơi xa.",
  "But nàng im lặng. Vợ của hắn và anh ấy đều biết.",
  "\"Mình không đi,\" nàng nói. Hừm. Ơ? Thập phần kỳ lạ…",
  "Não hải hắn KHẤP HUYẾT, cư nhiên Cư Nhiên. Lãnh diễm.",
  "Dòng sạch không vi phạm gì cả.",
  "",
].join("\n");
const SIMPLE_RULES = [{ pattern: "x", message: "m" }];
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

const full = storyFull();
const glossaryEntries: unknown[] = [
  { source: "赵静文", target: "Triệu Tĩnh Văn", category: "names" },   // đã có → loại
  { source: "沉默", target: "im lặng", category: "common" },            // hợp lệ
  { source: "远方", target: "Nơi Xa", category: "lạ" },                 // target không có trong dịch → loại
  { source: "高塔", target: "tòa tháp cao", category: "places" },        // đã có (places) → loại
  { source: "塔", target: "tháp", category: "places" },                  // 1 chữ Hán → loại
  { source: "沉默", target: "im lặng", category: "common" },             // trùng → loại
  { source: "抬头", target: "ngẩng đầu", category: "kỳ lạ" },           // category lạ → names
  { source: "不在raw", target: "ngẩng", category: "names" },             // không có trong raw → loại
  "rác", null, { source: 1, target: "x" },
];
const translation = "Triệu Tĩnh Văn ngẩng đầu nhìn về phía tòa tháp cao nơi xa.\n\nNàng im lặng hồi lâu không nói lời nào.\n";
const existing = collectGlossaryKeys({}, full.glossary);
const sanitized = sanitizeExtractedGlossary(glossaryEntries, CH1, translation, existing);
const glossary = {
  matches: [
    { source: "赵静文", text: CH1, result: glossaryEntryMatchesSource("赵静文", CH1) },
    { source: "赵静文", text: "静文来了", result: glossaryEntryMatchesSource("赵静文", "静文来了") },
    { source: "慕容雪羽", text: "雪羽来了", result: glossaryEntryMatchesSource("慕容雪羽", "雪羽来了") },
    { source: "慕容雪羽", text: "容雪羽来了", result: glossaryEntryMatchesSource("慕容雪羽", "容雪羽来了") },
    { source: "高塔", text: "塔", result: glossaryEntryMatchesSource("高塔", "塔") },
    { source: "不出现", text: CH1, result: glossaryEntryMatchesSource("不出现", CH1) },
  ],
  filter: [{ glossary: full.glossary, source: CH2, result: filterTranslationGlossaryForSource(full.glossary, CH2) }],
  sanitize: [{ entries: glossaryEntries, raw: CH1, translation, existingKeys: [...existing].sort(), result: sanitized }],
  append: [{ story: full, pairs: sanitized, chapter: "0001", result: appendAutoGlossary(full, sanitized, "0001") }],
};

const outputs: Array<[string, unknown]> = [
  [join(CORE, "prompts", "prompts.json"), prompts],
  [join(FIXTURES, "story.json"), story],
  [join(FIXTURES, "paragraphs.json"), paragraphs],
  [join(FIXTURES, "prompt.json"), { cases: promptCases }],
  [join(FIXTURES, "check.json"), check],
  [join(FIXTURES, "glossary.json"), glossary],
];

let drift = 0;
for (const [path, value] of outputs) {
  const text = `${JSON.stringify(value, null, 2)}\n`;
  if (CHECK_ONLY) {
    const current = existsSync(path) ? readFileSync(path, "utf8") : "";
    if (current !== text) {
      console.error(`LỆCH: ${path} — chạy npm run golden rồi port lại Rust cho khớp.`);
      drift += 1;
    }
    continue;
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, text, "utf8");
  console.log(`Đã ghi ${path}`);
}
if (CHECK_ONLY) {
  console.log(drift === 0 ? "Golden khớp." : `${drift} file lệch.`);
  process.exit(drift === 0 ? 0 : 1);
}
