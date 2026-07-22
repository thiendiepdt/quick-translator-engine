import { z } from "zod";

import { dictionaryKeys } from "@/lib/types";

const endpointSchema = z.string().trim().min(1, "Nhập URL Cloudflare Worker").refine(
  (value) => {
    if (value.startsWith("/")) return true;
    try {
      const parsed = new URL(value);
      return parsed.protocol === "https:" || parsed.hostname === "localhost";
    } catch {
      return false;
    }
  },
  "Dùng HTTPS URL hoặc đường dẫn same-origin bắt đầu bằng /",
);

export const translationOptionsSchema = z.object({
  endpoint: endpointSchema,
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

export const healthResponseSchema = z.object({ status: z.literal("ok") });

export const dictionaryPayloadSchema = z.object(
  Object.fromEntries(dictionaryKeys.map((key) => [key, z.string().optional()])),
);
