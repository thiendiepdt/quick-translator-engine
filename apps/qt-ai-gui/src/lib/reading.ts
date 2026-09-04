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

/** Giá trị lạ trong config (sửa tay, bản cũ) rơi về mặc định thay vì làm vùng đọc mất max-width. */
export function readingWidthOf(value: unknown): ReadingWidth {
  return isReadingWidth(value) ? value : DEFAULT_READING_WIDTH;
}
