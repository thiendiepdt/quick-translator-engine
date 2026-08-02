import { z } from "zod";

import { dictionaryKeys, type DictionaryDefaults } from "@/lib/types";

export const translationOptionsSchema = z.object({
  mode: z.enum(["hanviet", "vietphrase", "vietphrase-one"]),
  pretty: z.boolean(),
  wrap: z.boolean(),
  prioritizedName: z.boolean(),
  scanRange: z.coerce.number().int().min(1).max(100),
  translationAlgorithm: z.coerce.number().int().min(0).max(2).transform((value) => value as 0 | 1 | 2),
});

export type TranslationOptionsValues = z.input<typeof translationOptionsSchema>;
export type ParsedTranslationOptions = z.output<typeof translationOptionsSchema>;

const textRangeSchema = z.object({
  start: z.number().int().nonnegative(),
  length: z.number().int().nonnegative(),
});

const nameCandidateSchema = z.object({
  text: z.string(),
  suggested: z.string(),
  entityType: z.enum(["person", "location", "organization", "title", "unknown"]),
  score: z.number().min(0).max(1),
  occurrences: z.number().int().nonnegative(),
  ranges: z.array(textRangeSchema),
  contexts: z.array(z.string()),
  reasons: z.array(z.string()),
  sources: z.array(z.string()),
  known: z.boolean(),
});

export const nameFilterResponseSchema = z.object({
  candidates: z.array(nameCandidateSchema),
  stats: z.object({
    scannedCharacters: z.number().int().nonnegative(),
    ruleCandidates: z.number().int().nonnegative(),
    aiExtractedCandidates: z.number().int().nonnegative(),
    aiReviewed: z.number().int().nonnegative(),
  }),
  capabilities: z.object({
    aiConfigured: z.boolean(),
  }),
  warnings: z.array(z.string()).optional(),
});

export const translationResponseSchema = z
  .object({
    translated: z.string(),
    sourceRanges: z.array(textRangeSchema).optional(),
    targetRanges: z.array(textRangeSchema).optional(),
  })
  .refine(
    (value) => (value.sourceRanges?.length ?? 0) === (value.targetRanges?.length ?? 0),
    "sourceRanges và targetRanges phải có cùng số phần tử",
  );

export const engineStatusSchema = z.object({
  ready: z.boolean(),
  dataDir: z.string().optional(),
  message: z.string(),
});

export const openedTextFileSchema = z.object({
  path: z.string(),
  name: z.string(),
  content: z.string(),
});

export const dictionaryDefaultsSchema: z.ZodType<DictionaryDefaults> = z.object({
  names: z.string(),
  names2: z.string(),
  luatNhan: z.string(),
  pronouns: z.string(),
  danhTu: z.string(),
  hoNguoi: z.string(),
  hauTu: z.string(),
  ignoredChinesePhrases: z.string(),
});

export const dictionaryPayloadSchema = z.object(
  Object.fromEntries(dictionaryKeys.map((key) => [key, z.string().optional()])),
);
