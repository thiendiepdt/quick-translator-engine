import type { TextRange } from "@/lib/types";

export type TextSegment =
  | { kind: "plain"; text: string; key: string }
  | { kind: "mapped"; text: string; rangeIndex: number; key: string };

export function buildTextSegments(text: string, ranges: TextRange[]): TextSegment[] {
  const ordered = ranges
    .map((range, rangeIndex) => ({ ...range, rangeIndex }))
    .filter(
      ({ start, length }) =>
        Number.isInteger(start) &&
        Number.isInteger(length) &&
        start >= 0 &&
        length >= 0 &&
        start + length <= text.length,
    )
    .sort((left, right) => left.start - right.start || left.rangeIndex - right.rangeIndex);

  const segments: TextSegment[] = [];
  let cursor = 0;

  for (const item of ordered) {
    if (item.start < cursor) continue;
    if (item.start > cursor) {
      segments.push({
        kind: "plain",
        text: text.slice(cursor, item.start),
        key: `plain-${cursor}-${item.start}`,
      });
    }
    if (item.length > 0) {
      segments.push({
        kind: "mapped",
        text: text.slice(item.start, item.start + item.length),
        rangeIndex: item.rangeIndex,
        key: `mapped-${item.rangeIndex}-${item.start}`,
      });
    }
    cursor = item.start + item.length;
  }

  if (cursor < text.length) {
    segments.push({
      kind: "plain",
      text: text.slice(cursor),
      key: `plain-${cursor}-${text.length}`,
    });
  }

  return segments;
}

export function rangeText(text: string, range: TextRange | undefined): string {
  if (!range) return "";
  return text.slice(range.start, range.start + range.length);
}
