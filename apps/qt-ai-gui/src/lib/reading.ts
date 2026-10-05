export const READING_WIDTHS = ["narrow", "normal", "wide", "full"] as const;
export type ReadingWidth = (typeof READING_WIDTHS)[number];
export const DEFAULT_READING_WIDTH: ReadingWidth = "normal";

export const READING_WIDTH_LABELS: Record<ReadingWidth, string> = {
  narrow: "Hẹp",
  normal: "Vừa",
  wide: "Rộng",
  full: "Toàn màn",
};

export function isReadingWidth(value: unknown): value is ReadingWidth {
  return typeof value === "string" && (READING_WIDTHS as readonly string[]).includes(value);
}

export interface NumberedLine {
  /** Số thứ tự đoạn (1-based); dòng trống không có số. */
  n?: number;
  text: string;
}

/**
 * Tách văn bản thành dòng và đánh số các dòng không rỗng. Harness coi mỗi dòng không rỗng là một đoạn và gắn
 * nhãn [[n]] theo đúng thứ tự này, nên số ở đây trùng với [[n]] trong cảnh báo, yêu cầu sửa và bản nháp.
 */
export function numberLines(text: string): NumberedLine[] {
  let n = 0;
  return text.split(/\r?\n/).map((line) => (line.trim() === "" ? { text: line } : { n: ++n, text: line }));
}

/** Cùng định nghĩa với rule cứng "CJK còn sót" của harness (`\p{Script=Han}`): 《》【】「」 không tính. */
const HAN_RUN = /\p{Script=Han}+/gu;

export function hasHan(text: string): boolean {
  return /\p{Script=Han}/u.test(text);
}

/** Cắt một dòng thành các khúc xen kẽ thường / chữ Hán để tô riêng phần còn sót. */
export function splitHan(text: string): { text: string; han: boolean }[] {
  const parts: { text: string; han: boolean }[] = [];
  let last = 0;
  for (const match of text.matchAll(HAN_RUN)) {
    if (match.index > last) parts.push({ text: text.slice(last, match.index), han: false });
    parts.push({ text: match[0], han: true });
    last = match.index + match[0].length;
  }
  if (last < text.length) parts.push({ text: text.slice(last), han: false });
  return parts;
}

/** Số ký tự không tính khoảng trắng — đúng cách harness đo độ dài khi tính tỉ lệ dịch/gốc. */
export function countChars(text: string): number {
  return [...text.replace(/\s+/gu, "")].length;
}

/** Số chữ (tiếng) của văn bản tiếng Việt: cụm cách nhau bởi khoảng trắng, bỏ cụm chỉ gồm dấu câu ("—", "……"). */
export function countWords(text: string): number {
  return text.split(/\s+/u).filter((token) => /[\p{L}\p{N}]/u.test(token)).length;
}

/** Giá trị lạ trong config (sửa tay, bản cũ) rơi về mặc định thay vì làm vùng đọc mất max-width. */
export function readingWidthOf(value: unknown): ReadingWidth {
  return isReadingWidth(value) ? value : DEFAULT_READING_WIDTH;
}
