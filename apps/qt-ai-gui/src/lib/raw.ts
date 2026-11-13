/** Mẫu tên file raw — cùng luật với Rust `qt_ai_core::raw_download::template` để báo lỗi ngay khi gõ. */

const TOKEN = /\{([^{}]*)\}/g;
// eslint-disable-next-line no-control-regex
const FORBIDDEN = /[\\/:*?"<>|\u0000-\u001f\u007f]/g;
export const MAX_NAME_CHARS = 120;

type Token = { kind: "index"; width: number } | { kind: "id" } | { kind: "title" } | { kind: "volume" };

function parseToken(token: string): Token | undefined {
  if (token === "chuong") return { kind: "index", width: 0 };
  if (token === "chapter-id") return { kind: "id" };
  if (token === "chapterTitle") return { kind: "title" };
  if (token === "volume") return { kind: "volume" };
  const match = /^chuong:([1-9])$/.exec(token);
  return match ? { kind: "index", width: Number(match[1]) } : undefined;
}

/** Lỗi của mẫu, hoặc `null` nếu hợp lệ. */
export function validateTemplate(template: string): string | null {
  const text = template.trim();
  if (!text) return "Mẫu tên file không được để trống";
  let hasIndex = false;
  for (const match of text.matchAll(TOKEN)) {
    const token = parseToken(match[1] ?? "");
    if (!token) return `Token ${match[0]} không hợp lệ — có {chuong}, {chuong:4}, {chapter-id}, {chapterTitle}, {volume}`;
    if (token.kind === "index") hasIndex = true;
  }
  return hasIndex ? null : "Mẫu phải có {chuong} hoặc {chuong:4} — app sắp chương theo tên file";
}

export function sanitizeName(name: string): string {
  const squashed = name.replace(FORBIDDEN, " ").split(/\s+/).filter(Boolean).join(" ");
  return Array.from(squashed).slice(0, MAX_NAME_CHARS).join("").replace(/[ .]+$/, "").trimStart();
}

export function renderName(template: string, parts: { index: number; id: string; title: string; volume?: string }): string {
  const rendered = template.trim().replace(TOKEN, (whole, inner: string) => {
    const token = parseToken(inner);
    if (!token) return whole;
    if (token.kind === "index") return String(parts.index).padStart(token.width, "0");
    if (token.kind === "id") return parts.id;
    if (token.kind === "title") return parts.title;
    return parts.volume ?? "";
  });
  return sanitizeName(rendered) || String(parts.index).padStart(4, "0");
}

export const RAW_STATUS_LABELS = { have: "Đã có", missing: "Chưa có", locked: "Khoá" } as const;
