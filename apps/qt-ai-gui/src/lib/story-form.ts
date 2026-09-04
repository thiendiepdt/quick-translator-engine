import { z } from "zod";

import { GLOSSARY_KEYS, type StoryConfig } from "@/lib/types";

const pairSchema = z.object({ source: z.string(), target: z.string() });
const ruleSchema = z.object({ pattern: z.string(), flags: z.string(), message: z.string() });

export const storyFormSchema = z.object({
  name: z.string(),
  sourceUrl: z.string(),
  protagonist: z.string(),
  summary: z.string(),
  customPrompt: z.string(),
  voice: z.string(),
  toneRules: z.string(),
  avoid: z.string(),
  signaturePhrases: z.array(pairSchema),
  glossary: z.object({
    names: z.array(pairSchema),
    places: z.array(pairSchema),
    items: z.array(pairSchema),
    creatures: z.array(pairSchema),
    skills: z.array(pairSchema),
    common: z.array(pairSchema),
    signature_phrases: z.array(pairSchema),
  }),
  checkRules: z.array(ruleSchema),
  autoGlossary: z.enum(["inherit", "on", "off"]),
});

export type StoryFormValues = z.infer<typeof storyFormSchema>;
export type Pair = z.infer<typeof pairSchema>;

const toPairs = (record: Record<string, string>): Pair[] =>
  Object.entries(record).map(([source, target]) => ({ source, target }));
const fromPairs = (pairs: Pair[]): Record<string, string> =>
  Object.fromEntries(
    pairs.filter((p) => p.source.trim() && p.target.trim()).map((p) => [p.source.trim(), p.target.trim()]),
  );
const lines = (list: string[]) => list.join("\n");
const fromLines = (text: string) =>
  text
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter(Boolean);

export function toFormValues(config: StoryConfig): StoryFormValues {
  return {
    name: config.name,
    sourceUrl: config.sourceUrl,
    protagonist: config.protagonist,
    summary: config.summary,
    customPrompt: config.customPrompt,
    voice: config.style.voice,
    toneRules: lines(config.style.toneRules),
    avoid: lines(config.style.avoid),
    signaturePhrases: toPairs(config.style.signaturePhrases),
    glossary: {
      names: toPairs(config.glossary.names),
      places: toPairs(config.glossary.places),
      items: toPairs(config.glossary.items),
      creatures: toPairs(config.glossary.creatures),
      skills: toPairs(config.glossary.skills),
      common: toPairs(config.glossary.common),
      signature_phrases: toPairs(config.glossary.signature_phrases),
    },
    checkRules: config.checkRules.map((rule) => ({
      pattern: rule.pattern,
      flags: rule.flags ?? "",
      message: rule.message,
    })),
    autoGlossary: config.autoGlossary,
  };
}

/** Form → config; `autoGlossaryLog` không sửa trên form nên lấy từ `base`. */
export function fromFormValues(values: StoryFormValues, base: StoryConfig): StoryConfig {
  return {
    name: values.name,
    sourceUrl: values.sourceUrl,
    protagonist: values.protagonist,
    summary: values.summary,
    glossary: {
      names: fromPairs(values.glossary.names),
      places: fromPairs(values.glossary.places),
      items: fromPairs(values.glossary.items),
      creatures: fromPairs(values.glossary.creatures),
      skills: fromPairs(values.glossary.skills),
      common: fromPairs(values.glossary.common),
      signature_phrases: fromPairs(values.glossary.signature_phrases),
    },
    style: {
      voice: values.voice,
      toneRules: fromLines(values.toneRules),
      signaturePhrases: fromPairs(values.signaturePhrases),
      avoid: fromLines(values.avoid),
    },
    customPrompt: values.customPrompt,
    checkRules: values.checkRules
      .filter((rule) => rule.pattern.trim() && rule.message.trim())
      .map((rule) => ({
        pattern: rule.pattern,
        ...(rule.flags.trim() ? { flags: rule.flags.trim() } : {}),
        message: rule.message,
      })),
    autoGlossaryLog: base.autoGlossaryLog,
    autoGlossary: values.autoGlossary,
  };
}

/** Chế độ sửa dạng văn bản của bảng glossary: mỗi dòng `Hán=Việt`. */
export function pairsToText(pairs: Pair[]): string {
  return pairs.map((pair) => `${pair.source}=${pair.target}`).join("\n");
}

/** Dòng trống bỏ; dòng không có `=` giữ source và target rỗng để người dùng thấy mà sửa. */
export function textToPairs(text: string): Pair[] {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => {
      const index = line.indexOf("=");
      return index < 0
        ? { source: line, target: "" }
        : { source: line.slice(0, index).trim(), target: line.slice(index + 1).trim() };
    });
}

export interface DiffLine {
  field: string;
  before: string;
  after: string;
}

/** So từng field cấp 1 (glossary/style tách theo nhóm) — đủ để người dùng duyệt kết quả AI điền. */
export function diffStoryConfig(before: StoryConfig, after: StoryConfig): DiffLine[] {
  const entries: Array<[string, unknown, unknown]> = [
    ["name", before.name, after.name],
    ["sourceUrl", before.sourceUrl, after.sourceUrl],
    ["protagonist", before.protagonist, after.protagonist],
    ["summary", before.summary, after.summary],
    ["customPrompt", before.customPrompt, after.customPrompt],
    ["style.voice", before.style.voice, after.style.voice],
    ["style.toneRules", before.style.toneRules, after.style.toneRules],
    ["style.signaturePhrases", before.style.signaturePhrases, after.style.signaturePhrases],
    ["style.avoid", before.style.avoid, after.style.avoid],
    ...GLOSSARY_KEYS.map((key): [string, unknown, unknown] => [
      `glossary.${key}`,
      before.glossary[key],
      after.glossary[key],
    ]),
    ["checkRules", before.checkRules, after.checkRules],
    ["autoGlossary", before.autoGlossary, after.autoGlossary],
  ];
  return entries.flatMap(([field, b, a]) => {
    const beforeText = JSON.stringify(b, null, 1);
    const afterText = JSON.stringify(a, null, 1);
    return beforeText === afterText ? [] : [{ field, before: beforeText, after: afterText }];
  });
}
