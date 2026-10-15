import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { bookInfoGet, bookInfoSave, hanVietName, sourceFetchImage, sourceFetchText, type BookMeta } from "@/lib/api";
import { BOOK_SOURCE_LABELS, detectBookSource, parseSourceBook, type BookSourceId } from "@/lib/book-source";
import { imageToJpeg, renderDefaultCover, splitStoryName } from "@/lib/epub-cover";

export type CoverStatus =
  | { kind: "loading" }
  /** `cover.jpg` lấy từ nguồn. */
  | { kind: "fetched"; source: string }
  /** Không có link hoặc link không hỗ trợ → bìa mặc định. */
  | { kind: "unsupported" }
  /** Nguồn không có ảnh bìa → bìa mặc định. */
  | { kind: "no-cover"; source: string }
  /** Tải lỗi → bìa mặc định, cho Thử lại. */
  | { kind: "error"; message: string };

const errorMessage = (error: unknown) => (error instanceof Error ? error.message : String(error));

export const sourceLabel = (source: string) => BOOK_SOURCE_LABELS[source as BookSourceId] ?? source;

const HAN = /\p{Script=Han}/u;

/** Bút danh chữ Hán → Hán-Việt (giữ bản gốc để trang tiêu đề ghi kèm); tên không có chữ Hán giữ nguyên. */
async function toHanViet(name: string): Promise<{ author: string; original: string }> {
  const trimmed = name.trim();
  return HAN.test(trimmed) ? { author: await hanVietName(trimmed), original: trimmed } : { author: trimmed, original: "" };
}

/**
 * Tác giả + ảnh bìa cho export EPUB. Mở lần đầu (chưa có `cover.jpg`, chưa từng lấy) thì tự tải từ link nguồn;
 * tải lỗi không ghi mốc nên lần mở sau tự thử lại, và có `refetch` để bấm tay. Tác giả chỉ điền từ nguồn khi
 * ô đang trống (hoặc còn chữ Hán) — không đè chữ người dùng đã sửa. Bút danh chữ Hán đổi sang Hán-Việt, bản gốc lưu
 * ở `authorOriginal`; `book.json` cũ còn tên chữ Hán thì đổi lúc mở.
 */
export function useBookInfo(root: string | undefined, story: { name: string; sourceUrl: string } | undefined, enabled: boolean) {
  const [meta, setMeta] = useState<BookMeta>({ author: "", authorOriginal: "", source: "", coverUrl: "", fetchedAt: 0 });
  const [author, setAuthor] = useState("");
  const [cover, setCover] = useState<Uint8Array | undefined>();
  const [status, setStatus] = useState<CoverStatus>({ kind: "loading" });
  const [busy, setBusy] = useState(false);
  const [defaultCover, setDefaultCover] = useState<Uint8Array | undefined>();
  const alive = useRef(true);
  // Bản mới nhất cho các callback async (cập nhật cùng lúc với setMeta, không gán trong render).
  const metaRef = useRef(meta);
  const name = story?.name ?? "";
  const sourceUrl = story?.sourceUrl ?? "";

  useEffect(() => {
    alive.current = true;
    return () => {
      alive.current = false;
    };
  }, []);

  const save = useCallback(
    async (next: BookMeta, jpeg?: Uint8Array, clearCover = false) => {
      if (!root) return;
      await bookInfoSave(root, next, jpeg, clearCover);
      metaRef.current = next;
      if (!alive.current) return;
      setMeta(next);
    },
    [root],
  );

  const refetch = useCallback(
    async (currentAuthor: string) => {
      if (!root) return;
      setBusy(true);
      setStatus({ kind: "loading" });
      try {
        const ref = detectBookSource(sourceUrl);
        if (!ref) {
          await save({ ...metaRef.current, author: currentAuthor, source: "", coverUrl: "", fetchedAt: Date.now() }, undefined, true);
          if (!alive.current) return;
          setCover(undefined);
          setStatus({ kind: "unsupported" });
          return;
        }
        const book = parseSourceBook(ref, await sourceFetchText(ref.fetchUrl));
        const fetched = await toHanViet(book.author);
        const keep = currentAuthor.trim() !== "" && !HAN.test(currentAuthor);
        const nextAuthor = keep ? currentAuthor : fetched.author;
        const authorOriginal = fetched.original || metaRef.current.authorOriginal;
        if (alive.current) setAuthor(nextAuthor);
        if (book.coverUrls.length === 0) {
          await save({ author: nextAuthor, authorOriginal, source: ref.source, coverUrl: "", fetchedAt: Date.now() }, undefined, true);
          if (!alive.current) return;
          setCover(undefined);
          setStatus({ kind: "no-cover", source: ref.source });
          return;
        }
        let lastError: unknown;
        for (const url of book.coverUrls) {
          try {
            const image = await sourceFetchImage(url);
            const jpeg = await imageToJpeg(image.bytes, image.mime);
            await save({ author: nextAuthor, authorOriginal, source: ref.source, coverUrl: url, fetchedAt: Date.now() }, jpeg);
            if (!alive.current) return;
            setCover(jpeg);
            setStatus({ kind: "fetched", source: ref.source });
            return;
          } catch (error) {
            lastError = error;
          }
        }
        // Có tác giả nhưng ảnh lỗi: lưu tác giả, không ghi mốc để lần mở sau thử lại ảnh.
        await save({ ...metaRef.current, author: nextAuthor, authorOriginal, source: ref.source, fetchedAt: 0 });
        if (alive.current) setStatus({ kind: "error", message: errorMessage(lastError) });
      } catch (error) {
        if (alive.current) setStatus({ kind: "error", message: errorMessage(error) });
      } finally {
        if (alive.current) setBusy(false);
      }
    },
    [root, sourceUrl, save],
  );

  useEffect(() => {
    if (!enabled || !root) return;
    let cancelled = false;
    void (async () => {
      try {
        const info = await bookInfoGet(root);
        if (cancelled) return;
        let loaded = info.meta;
        metaRef.current = loaded;
        if (HAN.test(loaded.author)) {
          // Lưu từ bản trước khi có phiên âm: đổi một lần rồi ghi lại.
          const converted = await toHanViet(loaded.author);
          if (cancelled) return;
          loaded = { ...loaded, author: converted.author, authorOriginal: loaded.authorOriginal || converted.original };
          await save(loaded);
          if (cancelled) return;
        }
        setMeta(loaded);
        setAuthor(loaded.author);
        setCover(info.cover);
        if (info.cover) {
          setStatus({ kind: "fetched", source: loaded.source });
        } else if (loaded.fetchedAt > 0) {
          setStatus(loaded.source ? { kind: "no-cover", source: loaded.source } : { kind: "unsupported" });
        } else {
          await refetch(loaded.author);
        }
      } catch (error) {
        if (!cancelled) setStatus({ kind: "error", message: errorMessage(error) });
      }
    })();
    return () => {
      cancelled = true;
    };
    // Chỉ nạp khi bật EPUB hoặc đổi truyện; refetch đổi theo link nhưng không tự tải lại.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [enabled, root]);

  /** Lưu tác giả khi rời ô nhập. */
  const commitAuthor = useCallback(async () => {
    if (author === metaRef.current.author) return;
    await save({ ...metaRef.current, author });
  }, [author, save]);

  // Bìa mặc định vẽ lại theo tên + tác giả đã lưu (không vẽ lại mỗi phím gõ).
  const committedAuthor = meta.author;
  useEffect(() => {
    if (!enabled || cover || status.kind === "loading") return;
    let cancelled = false;
    renderDefaultCover({ ...splitStoryName(name), author: committedAuthor })
      .then((jpeg) => !cancelled && setDefaultCover(jpeg))
      .catch(() => !cancelled && setDefaultCover(undefined));
    return () => {
      cancelled = true;
    };
  }, [enabled, cover, status.kind, name, committedAuthor]);

  const shown = cover ?? defaultCover;
  const previewUrl = useMemo(
    () => (shown ? URL.createObjectURL(new Blob([shown as BlobPart], { type: "image/jpeg" })) : undefined),
    [shown],
  );
  useEffect(() => () => (previewUrl ? URL.revokeObjectURL(previewUrl) : undefined), [previewUrl]);

  /** JPEG đưa vào sách: bìa đã lưu, không có thì bìa mặc định vẽ theo tác giả đang nhập. */
  const coverForExport = useCallback(async () => {
    if (cover) return cover;
    try {
      return await renderDefaultCover({ ...splitStoryName(name), author });
    } catch {
      return undefined;
    }
  }, [cover, name, author]);

  return {
    author,
    setAuthor,
    commitAuthor,
    status,
    busy,
    previewUrl,
    isDefaultCover: !cover,
    refetch: () => refetch(author),
    coverForExport,
  };
}
