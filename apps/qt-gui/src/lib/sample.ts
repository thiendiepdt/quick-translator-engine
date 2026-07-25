import type { TextRange, TranslationResponse } from "@/lib/types";

export const sampleSource = "萧炎看着她，轻声说道：“云韵，这一趟务必要小心。”";
export const sampleTarget =
  "Tiêu Viêm nhìn nàng, khẽ nói: “Vân Vận, chuyến đi này nhất định phải cẩn thận.”";

const samplePairs = [
  ["萧炎", "Tiêu Viêm"],
  ["看着", "nhìn"],
  ["她", "nàng"],
  ["轻声说道", "khẽ nói"],
  ["云韵", "Vân Vận"],
  ["这一趟", "chuyến đi này"],
  ["务必要", "nhất định phải"],
  ["小心", "cẩn thận"],
] as const;

function locate(text: string, fragment: string): TextRange {
  return { start: text.indexOf(fragment), length: fragment.length };
}

export const sampleResponse: TranslationResponse = {
  translated: sampleTarget,
  sourceRanges: samplePairs.map(([source]) => locate(sampleSource, source)),
  targetRanges: samplePairs.map(([, target]) => locate(sampleTarget, target)),
};

export const sampleDictionaryValues = {
  names: "萧炎=Tiêu Viêm\n云韵=Vân Vận",
  pronouns: "她=nàng",
} as const;
