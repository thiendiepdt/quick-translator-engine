import type { Cast } from "@/lib/types";

/** Nhãn ngắn của chương như `chapter_label` bên core: số thứ tự đầu id (`0150_74…` → `150`). */
export function chapterLabel(id: string): string {
  const digits = id.split(/[_-]/).find((part) => part.length > 0 && part.length <= 5 && /^\d+$/.test(part));
  return digits === undefined ? id : digits.replace(/^0+(?=\d)/, "");
}

/** Cặp xưng hô đang áp theo cast.json: mốc muộn nhất của mỗi cặp (so id chương theo số). */
export function latestAddressing(cast: Cast): Record<string, { target: string; from: string }> {
  const result: Record<string, { target: string; from: string }> = {};
  for (const [key, timeline] of Object.entries(cast.addressing)) {
    const latest = [...timeline.changes].sort((a, b) => a.from.localeCompare(b.from, undefined, { numeric: true })).at(-1);
    if (latest) result[key] = { target: latest.target, from: latest.from };
  }
  return result;
}
