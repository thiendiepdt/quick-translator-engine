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

/** Biến của mẫu tên file — hàng chip dưới ô "Mẫu tên file", bấm để chèn vào chỗ con trỏ. */
export const TEMPLATE_TOKENS = [
  { token: "{chuong:4}", hint: "Số thứ tự chương, đủ 4 chữ số (0001) — bắt buộc có số thứ tự" },
  { token: "{chuong}", hint: "Số thứ tự chương, không thêm số 0 (1)" },
  { token: "{chapterTitle}", hint: "Tiêu đề chương theo mục lục" },
  { token: "{chapter-id}", hint: "Mã chương bên trang nguồn" },
  { token: "{volume}", hint: "Tên quyển (nguồn có chia quyển)" },
] as const;

/** Chèn `token` vào `text` thay đoạn [start, end); trả chuỗi mới và vị trí con trỏ sau token. */
export function insertAt(text: string, token: string, start: number, end: number): { text: string; caret: number } {
  const from = Math.max(0, Math.min(start, text.length));
  const to = Math.max(from, Math.min(end, text.length));
  return { text: text.slice(0, from) + token + text.slice(to), caret: from + token.length };
}

/** Mã truyện trong link SangTacViet: `…/truyen/qidian/1/1036370336/` → `1036370336`. */
export function stvStoryId(url: string): string | null {
  return /\/truyen\/[^/]+\/\d+\/(\d+)/.exec(url)?.[1] ?? null;
}

export interface TemplatePreset {
  label: string;
  /** Tên file tool đó tạo ra cho link `url` (hiện ở tooltip). */
  example: (url: string) => string;
  /** Mẫu cho link tải raw `url`; `null` khi không dựng được (vd. mẫu STV mà link không phải STV). */
  template: (url: string) => string | null;
  /** Vì sao nút tắt khi `template` trả `null`. */
  unavailable?: string;
}

/**
 * Mẫu đặt tên của các tool tải truyện khác, để chương tải về nối đúng thứ tự file raw cũ:
 * - Chinese Web Downloader (book-downloader, trang fanqie / qimao): `0001_<mã chương>`.
 * - STV Web Downloader (book-downloader, trang SangTacViet): `<mã truyện>_0001_<tiêu đề>` (mã truyện lấy từ link,
 *   ghi thẳng vào mẫu như mẫu đoán từ tên file cũ).
 * - Novel Reader (app đọc truyện): `0001 <tiêu đề>`.
 * - DTH (dichtienghoa-app, mẫu mặc định `{thu_tu} - {ten_chuong}`): `0001 - <tiêu đề>`; bản không dấu cách
 *   `0001-<tiêu đề>`.
 */
export const TEMPLATE_PRESETS: TemplatePreset[] = [
  { label: "Chinese Web Downloader", example: () => "0001_<mã chương>.txt (fanqie, qimao)", template: () => "{chuong:4}_{chapter-id}" },
  {
    label: "STV Web Downloader",
    example: (url) => `${stvStoryId(url) ?? "<mã truyện>"}_0001_第1章 标题.txt`,
    template: (url) => {
      const id = stvStoryId(url);
      return id ? `${id}_{chuong:4}_{chapterTitle}` : null;
    },
    unavailable: "Cần link tải raw của SangTacViet (…/truyen/<nguồn>/1/<mã truyện>/) để lấy mã truyện",
  },
  { label: "Novel Reader", example: () => "0001 第1章 标题.txt", template: () => "{chuong:4} {chapterTitle}" },
  { label: "DTH", example: () => "0001 - 第1章 标题.txt", template: () => "{chuong:4} - {chapterTitle}" },
  // DTH cho người dùng tự đổi mẫu; hay gặp kiểu bỏ dấu cách quanh gạch ngang.
  { label: "DTH không cách", example: () => "0001-第1章 标题.txt", template: () => "{chuong:4}-{chapterTitle}" },
];

export const RAW_STATUS_LABELS = { have: "Đã có", missing: "Chưa có", locked: "Khoá" } as const;

/**
 * Chọn chương như book-downloader: bấm thường bật / tắt một chương. Giữ Shift thì áp cho cả dải từ chương bấm
 * trước (`last`) tới chương này: chương này đang chưa chọn thì chọn cả dải, đang chọn thì bỏ cả dải.
 */
export function toggleSelection(selected: ReadonlySet<string>, order: readonly string[], id: string, last: string | undefined, shift: boolean): Set<string> {
  const next = new Set(selected);
  const on = !selected.has(id);
  const from = last === undefined ? -1 : order.indexOf(last);
  const to = order.indexOf(id);
  const range = shift && from !== -1 && to !== -1 ? order.slice(Math.min(from, to), Math.max(from, to) + 1) : [id];
  for (const each of range) {
    if (on) next.add(each);
    else next.delete(each);
  }
  return next;
}
