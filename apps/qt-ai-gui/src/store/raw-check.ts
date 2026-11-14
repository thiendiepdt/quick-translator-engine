import { create } from "zustand";

import { rawInfo, rawToc, type RawInfo, type RawToc } from "@/lib/api";

/** Mục lục tải gần nhất theo truyện; trong khoảng này trang Tải raw dùng lại, khỏi gọi extension lần nữa. */
const FRESH_MS = 5 * 60_000;

interface Entry {
  at: number;
  promise: Promise<RawToc | null>;
}

const entries = new Map<string, Entry>();

function hostOf(url: string): string {
  try {
    return new URL(url).host.replace(/^www\./, "");
  } catch {
    return url;
  }
}

/**
 * Vì sao truyện chưa tải raw được (hiện ở icon Tải raw trên thanh bên); `null` = đủ cài đặt: có link, có extension
 * cho link, extension cần auth thì đã nhập.
 */
export function rawProblem(info: RawInfo): string | null {
  const url = info.rawUrl.trim();
  if (!url) return "chưa có link tải raw";
  if (info.candidates.length === 0) {
    return info.installed === 0 ? "chưa cài extension tải raw nào" : `chưa có extension tải raw cho ${hostOf(url)}`;
  }
  if (!info.extension) return `có ${info.candidates.length} extension khớp link, chưa chọn cái nào`;
  const auth = info.extConfig?.auth;
  if (auth?.required && !auth.scheme) return `${info.extension.name} chưa nhập API key / đăng nhập`;
  return null;
}

/** Đã đủ cài đặt để tải mục lục. */
export function rawConfigured(info: RawInfo): boolean {
  return rawProblem(info) === null;
}

/** Chương mới chưa tải: sau chương cuối đã có, không khoá, chưa có file. */
export function newChapters(toc: RawToc) {
  return toc.rows.filter((row) => row.index > (toc.lastIndex ?? 0) && !row.locked && row.status !== "have");
}

interface RawNewState {
  /** Số chương mới theo truyện; `null` = chưa đủ cài đặt hoặc tải mục lục lỗi. */
  counts: Record<string, number | null>;
  /** Lý do chưa tải raw được theo truyện (`rawProblem`); `null` / thiếu = đủ cài đặt hoặc chưa kiểm. */
  problems: Record<string, string | null>;
  /** Tăng khi extension / kho / auth đổi: thanh bên kiểm lại truyện đang mở. */
  epoch: number;
  setCount: (root: string, count: number | null) => void;
  setProblem: (root: string, problem: string | null) => void;
}

export const useRawNew = create<RawNewState>()((set) => ({
  counts: {},
  problems: {},
  epoch: 0,
  setCount: (root, count) => set((state) => ({ counts: { ...state.counts, [root]: count } })),
  setProblem: (root, problem) => set((state) => ({ problems: { ...state.problems, [root]: problem } })),
}));

/** Ghi lại tình trạng cài đặt tải raw của truyện (trang Tải raw gọi mỗi khi đọc / lưu cài đặt). */
export function noteRawInfo(root: string, info: RawInfo): void {
  useRawNew.getState().setProblem(root, rawProblem(info));
}

function track(root: string, promise: Promise<RawToc | null>): Promise<RawToc | null> {
  const entry = { at: Date.now(), promise };
  entries.set(root, entry);
  promise.then(
    (toc) => useRawNew.getState().setCount(root, toc ? newChapters(toc).length : null),
    () => {
      if (entries.get(root) === entry) entries.delete(root);
      useRawNew.getState().setCount(root, null);
    },
  );
  return promise;
}

/**
 * Mục lục của truyện: dùng lại lần tải chưa quá 5 phút (kể cả lần đang chạy), không thì gọi extension. `null` khi
 * truyện chưa đủ cài đặt tải raw. `force` / `anchor` (khớp lại theo mốc): luôn tải mới.
 */
export function loadRawToc(root: string, options: { force?: boolean; anchor?: number } = {}): Promise<RawToc | null> {
  const hit = entries.get(root);
  if (hit && !options.force && options.anchor === undefined && Date.now() - hit.at < FRESH_MS) return hit.promise;
  const promise = (async () => {
    if (options.force || options.anchor !== undefined) return rawToc(root, options.anchor);
    const info = await rawInfo(root);
    noteRawInfo(root, info);
    return rawConfigured(info) ? rawToc(root) : null;
  })();
  return track(root, promise);
}

/** Trang Tải raw đổi mục lục (tải xong chương, khớp lại): giữ bản mới nhất cho lần mở sau và cho số trên thanh bên. */
export function rememberRawToc(root: string, toc: RawToc): void {
  entries.set(root, { at: entries.get(root)?.at ?? Date.now(), promise: Promise.resolve(toc) });
  useRawNew.getState().setCount(root, newChapters(toc).length);
}

/** Đổi cài đặt tải raw / dọn raw: bỏ mục lục đã nhớ. */
export function forgetRawToc(root: string): void {
  entries.delete(root);
  useRawNew.getState().setCount(root, null);
}

/** Cài / gỡ / bật tắt extension, thêm / bỏ kho, đổi auth: bỏ mọi kết quả đã nhớ, thanh bên kiểm lại. */
export function invalidateRawChecks(): void {
  entries.clear();
  useRawNew.setState((state) => ({ epoch: state.epoch + 1 }));
}

/** Cho test. */
export function resetRawChecks(): void {
  entries.clear();
  useRawNew.setState({ counts: {}, problems: {}, epoch: 0 });
}
