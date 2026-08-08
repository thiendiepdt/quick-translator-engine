import type { TextRange } from "@/lib/types";

/**
 * Dịch AI theo đoạn đánh số: mỗi đoạn nguyên văn (một dòng không rỗng) được
 * gắn nhãn `[[n]]` trước khi gửi đi, model phải giữ nhãn trong bản dịch. Nhờ
 * đó biết chính xác đoạn nào thiếu để dịch bổ sung đúng đoạn, và có ánh xạ
 * 1-1 nguồn ↔ dịch cho tính năng click đối chiếu.
 */

const MARKER_PATTERN = /\[\[(\d{1,4})\]\]/g;

/** Mỗi dòng không rỗng là một đoạn — cùng đơn vị với nonEmptyLineCount. */
export function aiParagraphsOf(text: string): string[] {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
}

/** Offset từng đoạn trong văn bản gốc, dùng cho MappedText. */
export function aiParagraphRanges(text: string): TextRange[] {
  const ranges: TextRange[] = [];
  let offset = 0;
  for (const line of text.split("\n")) {
    const raw = line.endsWith("\r") ? line.slice(0, -1) : line;
    const trimmed = raw.trim();
    if (trimmed) {
      const leading = raw.length - raw.trimStart().length;
      ranges.push({ start: offset + leading, length: trimmed.length });
    }
    offset += line.length + 1;
  }
  return ranges;
}

function alignedFormatHeader(count: number): string {
  return [
    `Nguyên văn gồm ${count} đoạn, mỗi đoạn mở đầu bằng nhãn dạng [[n]].`,
    "Bản dịch BẮT BUỘC giữ đúng định dạng đó: mỗi đoạn dịch mở đầu bằng nhãn [[n]] của đoạn nguyên văn tương ứng,",
    `đủ và đúng thứ tự từ [[1]] đến [[${count}]]; mỗi nhãn xuất hiện đúng một lần; không gộp, không tách, không thêm hay bỏ đoạn.`,
    "Ngoài nhãn, không thêm chú thích hay lời dẫn nào khác.",
  ].join("\n");
}

/** User message cho lượt dịch chính: header format + từng đoạn gắn nhãn. */
export function labeledAiSourcePayload(paragraphs: string[]): string {
  const body = paragraphs
    .map((paragraph, index) => `[[${index + 1}]] ${paragraph}`)
    .join("\n\n");
  return `${alignedFormatHeader(paragraphs.length)}\n\n${body}`;
}

/** User message cho lượt dịch bổ sung các đoạn bị thiếu. */
export function labeledAiRepairPayload(
  paragraphs: string[],
  missingIndexes: number[],
): string {
  const body = missingIndexes
    .map((index) => `[[${index + 1}]] ${paragraphs[index]}`)
    .join("\n\n");
  return [
    "Bản dịch trước bị thiếu các đoạn dưới đây (trích từ cùng chương, giữ nguyên nhãn [[n]] gốc).",
    "Dịch bổ sung đúng các đoạn này theo toàn bộ quy tắc đã cho: mỗi đoạn dịch mở đầu bằng nhãn [[n]] tương ứng,",
    "không gộp, không bỏ, không thêm chú thích.",
    "",
    body,
  ].join("\n");
}

/**
 * Tách bản dịch có nhãn thành mảng đoạn theo chỉ số. Trả về undefined khi
 * model bỏ toàn bộ nhãn (không còn căn cứ ghép); đoạn thiếu giữ undefined.
 */
export function parseLabeledAiTranslation(
  output: string,
  count: number,
): Array<string | undefined> | undefined {
  const matches = [...output.matchAll(MARKER_PATTERN)];
  if (matches.length === 0) return undefined;

  const paragraphs: Array<string | undefined> = Array.from(
    { length: count },
    () => undefined,
  );
  for (let index = 0; index < matches.length; index += 1) {
    const match = matches[index];
    const paragraphIndex = Number(match[1]) - 1;
    if (paragraphIndex < 0 || paragraphIndex >= count) continue;
    const start = (match.index ?? 0) + match[0].length;
    const end = index + 1 < matches.length
      ? matches[index + 1].index ?? output.length
      : output.length;
    const text = output
      .slice(start, end)
      .replace(/\s*\n+\s*/g, " ")
      .trim();
    if (text && paragraphs[paragraphIndex] === undefined) {
      paragraphs[paragraphIndex] = text;
    }
  }
  return paragraphs;
}

/** Xóa nhãn khỏi văn bản đang stream để hiển thị sạch cho người dùng. */
export function stripAiParagraphMarkers(text: string): string {
  return text.replace(/\[\[\d{1,4}\]\] ?/g, "");
}
