import { ChevronDown, ChevronRight, LoaderCircle, Replace, Undo2 } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { z } from "zod";

import { ChapterRefHint } from "@/components/chapter-ref-hint";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  replaceApply,
  replaceFind,
  replaceUndo,
  replaceUndoInfo,
  type ReplaceHitRef,
  type ReplaceQuery,
} from "@/lib/api";
import { resolveChapterRef } from "@/lib/chapters";
import { replaceFindSchema, replaceUndoInfoSchema } from "@/lib/schema";
import { selectCurrentRunning, useStoryStore } from "@/store/story";

type FindResult = z.infer<typeof replaceFindSchema>;
type Hit = FindResult["hits"][number];
type UndoInfo = z.infer<typeof replaceUndoInfoSchema>;

/** Gõ xong chừng này ms mới tìm — gõ liên tục không bắn lệnh liên tục. */
const DEBOUNCE_MS = 300;
/** Mở sẵn chừng này chương đầu; chương sau bấm để mở. */
const OPEN_FIRST = 10;

const hitKey = (hit: ReplaceHitRef) => `${hit.chapter}|${hit.paragraph}|${hit.occurrence}`;

/**
 * Tìm & thay trong bản dịch đã chốt của truyện: tìm theo chữ (không regex), xem từng chỗ kèm câu xung quanh và câu
 * raw của đoạn, bỏ chọn chỗ không muốn thay, thay một lần, hoàn tác lần thay gần nhất. Tìm/thay chạy ở Rust; trang
 * chỉ nhận 500 chỗ đầu + số theo chương nên truyện nghìn chương không làm giao diện đơ.
 */
export function ReplacePage() {
  const root = useStoryStore((s) => s.root);
  const rows = useStoryStore((s) => s.snapshot?.chapters);
  const running = useStoryStore(selectCurrentRunning);
  const select = useStoryStore((s) => s.select);
  const setPage = useStoryStore((s) => s.setPage);

  const [find, setFind] = useState("");
  const [replace, setReplace] = useState("");
  const [wholeWord, setWholeWord] = useState(true);
  const [caseSensitive, setCaseSensitive] = useState(false);
  const [rawFilter, setRawFilter] = useState("");
  const [fromText, setFromText] = useState("");
  const [toText, setToText] = useState("");
  const [result, setResult] = useState<FindResult>();
  const [searching, setSearching] = useState(false);
  const [excludedChapters, setExcludedChapters] = useState<Set<string>>(new Set());
  const [excludedHits, setExcludedHits] = useState<Set<string>>(new Set());
  const [open, setOpen] = useState<Set<string>>(new Set());
  const [undo, setUndo] = useState<UndoInfo>(null);
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);
  // Tăng để tìm lại ngay (sau khi thay / hoàn tác) với cùng truy vấn.
  const [epoch, setEpoch] = useState(0);

  const chapters = useMemo(() => rows ?? [], [rows]);
  const ordinal = useMemo(() => new Map(chapters.map((row, index) => [row.id, index + 1])), [chapters]);
  const fromIndex = resolveChapterRef(chapters, fromText);
  const toIndex = resolveChapterRef(chapters, toText);
  const from = fromIndex >= 0 ? chapters[fromIndex].id : null;
  const to = toIndex >= 0 ? chapters[toIndex].id : null;
  const query: ReplaceQuery = useMemo(
    () => ({ find, replace, caseSensitive, wholeWord, rawFilter: rawFilter.trim(), from, to }),
    [find, replace, caseSensitive, wholeWord, rawFilter, from, to],
  );

  useEffect(() => {
    if (!root) return;
    let alive = true;
    void replaceUndoInfo(root).then((info) => alive && setUndo(info));
    return () => {
      alive = false;
    };
  }, [root, epoch]);

  // Ô "Thay bằng" không đổi kết quả tìm — không tìm lại khi chỉ sửa ô đó.
  const search: ReplaceQuery = useMemo(
    () => ({ find, replace: "", caseSensitive, wholeWord, rawFilter: rawFilter.trim(), from, to }),
    [find, caseSensitive, wholeWord, rawFilter, from, to],
  );
  useEffect(() => {
    if (!root) return;
    let alive = true;
    const timer = setTimeout(() => {
      if (!search.find) {
        setResult(undefined);
        setSearching(false);
        return;
      }
      setSearching(true);
      void replaceFind(root, search)
        .then((found) => {
          if (!alive) return;
          setResult(found);
          setExcludedChapters(new Set());
          setExcludedHits(new Set());
          setOpen(new Set(found.chapters.slice(0, OPEN_FIRST).map((c) => c.chapter)));
        })
        .catch((error: unknown) => alive && setStatus(error instanceof Error ? error.message : "Không tìm được"))
        .finally(() => alive && setSearching(false));
    }, DEBOUNCE_MS);
    return () => {
      alive = false;
      clearTimeout(timer);
    };
  }, [root, search, epoch]);

  const hitsByChapter = useMemo(() => {
    const groups = new Map<string, Hit[]>();
    for (const hit of result?.hits ?? []) groups.set(hit.chapter, [...(groups.get(hit.chapter) ?? []), hit]);
    return groups;
  }, [result]);

  if (!root) return null;
  const currentRoot = root;

  const selectedCount = (result?.chapters ?? []).reduce((sum, chapter) => {
    if (excludedChapters.has(chapter.chapter)) return sum;
    const off = (hitsByChapter.get(chapter.chapter) ?? []).filter((hit) => excludedHits.has(hitKey(hit))).length;
    return sum + chapter.count - off;
  }, 0);
  const canReplace = !busy && !running && selectedCount > 0 && find !== "" && find !== replace;

  function toggle<T>(set: Set<T>, value: T): Set<T> {
    const next = new Set(set);
    if (next.has(value)) next.delete(value);
    else next.add(value);
    return next;
  }

  async function apply() {
    setBusy(true);
    try {
      const excluded = (result?.hits ?? []).filter((hit) => excludedHits.has(hitKey(hit)) && !excludedChapters.has(hit.chapter));
      const done = await replaceApply(currentRoot, query, [...excludedChapters], excluded.map(({ chapter, paragraph, occurrence }) => ({ chapter, paragraph, occurrence })));
      setStatus(`Đã thay ${done.occurrences} chỗ trong ${done.chapters} chương.`);
      setEpoch((e) => e + 1);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "Không thay được");
    } finally {
      setBusy(false);
    }
  }

  async function undoLast() {
    setBusy(true);
    try {
      const done = await replaceUndo(currentRoot);
      const kept = done.conflicts.length ? ` ${done.conflicts.length} chương đã sửa tiếp sau đó nên giữ nguyên.` : "";
      setStatus(`Đã hoàn tác ${done.restored} chương.${kept}`);
      setEpoch((e) => e + 1);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "Không hoàn tác được");
    } finally {
      setBusy(false);
    }
  }

  function openChapter(id: string) {
    select(id);
    setPage("translate");
  }

  const chapterName = (id: string) => (ordinal.has(id) ? `Chương #${ordinal.get(id)}` : id);

  return (
    <div className="fine-scrollbar h-full min-h-0 overflow-y-auto">
      <div className="mx-auto flex max-w-4xl flex-col gap-4 px-6 py-6">
        <div>
          <h1 className="text-lg font-semibold">Tìm & thay</h1>
          <p className="text-xs text-muted-foreground">
            Tìm trong các chương đã dịch của truyện này. Xem từng chỗ kèm câu xung quanh và câu raw, bỏ tick chỗ nào
            không muốn thay, rồi bấm Thay. Thay nhầm thì hoàn tác được lần thay gần nhất.
          </p>
        </div>

        <div className="grid grid-cols-[auto_1fr] items-center gap-x-3 gap-y-2 rounded-lg border bg-card p-4">
          <Label htmlFor="replace-find">Tìm</Label>
          <div className="flex items-center gap-2">
            <Input id="replace-find" value={find} onChange={(e) => setFind(e.target.value)} placeholder="vd. người tộc" autoFocus />
            {searching && <LoaderCircle className="size-4 shrink-0 animate-spin text-muted-foreground" aria-label="Đang tìm" />}
          </div>
          <Label htmlFor="replace-with">Thay bằng</Label>
          <Input id="replace-with" value={replace} onChange={(e) => setReplace(e.target.value)} placeholder="vd. nhân tộc" />
          <span />
          <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm">
            <label className="flex items-center gap-1.5" title="Không khớp khi chữ cần tìm nằm trong một từ khác (tìm “ân” không dính “nhân”)">
              <input type="checkbox" checked={wholeWord} onChange={(e) => setWholeWord(e.target.checked)} />
              Nguyên từ
            </label>
            <label className="flex items-center gap-1.5">
              <input type="checkbox" checked={caseSensitive} onChange={(e) => setCaseSensitive(e.target.checked)} />
              Phân biệt hoa/thường
            </label>
          </div>
          <Label htmlFor="replace-raw" title="Chỉ tìm ở đoạn mà câu raw cùng vị trí có chữ này">
            Chỉ đoạn raw có
          </Label>
          <Input id="replace-raw" value={rawFilter} onChange={(e) => setRawFilter(e.target.value)} placeholder="tuỳ chọn, vd. 人族" className="font-mono" />
          <Label htmlFor="replace-from">Chương</Label>
          <div className="flex flex-wrap items-start gap-2">
            <div className="w-32">
              <Input id="replace-from" value={fromText} onChange={(e) => setFromText(e.target.value)} placeholder="từ (số thứ tự)" aria-label="Từ chương" />
              <ChapterRefHint text={fromText} id={from ?? ""} />
            </div>
            <span className="pt-1.5 text-muted-foreground">→</span>
            <div className="w-32">
              <Input id="replace-to" value={toText} onChange={(e) => setToText(e.target.value)} placeholder="đến (số thứ tự)" aria-label="Đến chương" />
              <ChapterRefHint text={toText} id={to ?? ""} />
            </div>
          </div>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <p className="flex-1 text-sm" role="status" aria-label="Kết quả tìm">
            {!find
              ? "Gõ chữ cần tìm."
              : !result
                ? "Đang tìm…"
                : result.total === 0
                  ? "Không có chỗ nào khớp."
                  : `${result.total} chỗ trong ${result.chapters.length} chương — đang chọn ${selectedCount}.`}
            {result && result.skipped > 0 && ` Bỏ qua ${result.skipped} chương số đoạn lệch raw.`}
          </p>
          {result && result.total > 0 && (
            <Button
              type="button"
              size="sm"
              variant="ghost"
              onClick={() => {
                const all = excludedChapters.size === result.chapters.length;
                setExcludedChapters(all ? new Set() : new Set(result.chapters.map((c) => c.chapter)));
                setExcludedHits(new Set());
              }}
            >
              {excludedChapters.size === result.chapters.length ? "Chọn hết" : "Bỏ chọn hết"}
            </Button>
          )}
          <Button type="button" size="sm" disabled={!canReplace} onClick={() => void apply()}>
            <Replace /> Thay {selectedCount} chỗ
          </Button>
        </div>
        {running && <p className="text-xs text-muted-foreground">Truyện đang dịch — bấm Dừng trước khi thay.</p>}

        {status && (
          <div className="rounded-md border bg-muted/40 px-3 py-2 text-xs">
            <p>{status}</p>
            {status.startsWith("Đã thay") && (
              <p className="mt-1 text-muted-foreground">
                Nếu là tên riêng hay thuật ngữ, sửa luôn trong{" "}
                <button type="button" className="underline" onClick={() => setPage("story")}>
                  Hồ sơ truyện → Glossary
                </button>{" "}
                để các chương dịch sau không sai lại.
              </p>
            )}
          </div>
        )}

        {undo && (
          <div className="flex items-center gap-2 rounded-md border px-3 py-2 text-xs">
            <span className="flex-1">
              Lần thay gần nhất: “{undo.find}” → “{undo.replace}” ({undo.occurrences} chỗ, {undo.chapters} chương).
            </span>
            <Button type="button" size="xs" variant="outline" disabled={busy || running} onClick={() => void undoLast()}>
              <Undo2 /> Hoàn tác
            </Button>
          </div>
        )}

        {result && result.total > 0 && (
          <ul className="flex flex-col gap-1" aria-label="Chỗ khớp theo chương">
            {result.chapters.map(({ chapter, count }) => {
              const hits = hitsByChapter.get(chapter) ?? [];
              const expanded = open.has(chapter);
              const included = !excludedChapters.has(chapter);
              return (
                <li key={chapter} className="rounded-md border">
                  <div className="flex items-center gap-2 px-2 py-1.5 text-sm">
                    <input
                      type="checkbox"
                      checked={included}
                      aria-label={`Chọn ${chapterName(chapter)}`}
                      onChange={() => setExcludedChapters((set) => toggle(set, chapter))}
                    />
                    <button
                      type="button"
                      className="flex flex-1 items-center gap-1 text-left font-medium"
                      aria-expanded={expanded}
                      onClick={() => setOpen((set) => toggle(set, chapter))}
                    >
                      {expanded ? <ChevronDown className="size-4" /> : <ChevronRight className="size-4" />}
                      {chapterName(chapter)}
                      <span className="truncate font-mono text-[11px] font-normal text-muted-foreground">{chapter}</span>
                      <span className="ml-auto rounded-full bg-muted px-1.5 text-xs font-normal text-muted-foreground">{count} chỗ</span>
                    </button>
                  </div>
                  {expanded && (
                    <ul className="flex flex-col gap-1 border-t px-2 py-1.5">
                      {hits.length === 0 && (
                        <li className="text-xs text-muted-foreground">Chưa hiện chi tiết (quá 500 chỗ) — bấm Thay vẫn thay các chỗ này.</li>
                      )}
                      {hits.map((hit) => {
                        const key = hitKey(hit);
                        const checked = included && !excludedHits.has(key);
                        return (
                          <li key={key} className="flex items-start gap-2 text-sm">
                            <input
                              type="checkbox"
                              className="mt-1"
                              checked={checked}
                              disabled={!included}
                              aria-label={`Chọn chỗ ${chapterName(chapter)} đoạn ${hit.paragraph + 1}`}
                              onChange={() => setExcludedHits((set) => toggle(set, key))}
                            />
                            <button
                              type="button"
                              className="min-w-0 flex-1 text-left"
                              title="Mở chương này"
                              onClick={() => openChapter(chapter)}
                            >
                              <span className="font-mono text-[11px] text-muted-foreground">[[{hit.paragraph + 1}]] </span>
                              {hit.before}
                              <mark className={checked && replace !== find ? "bg-transparent text-destructive line-through" : "rounded-sm bg-primary/20 text-foreground"}>
                                {hit.matched}
                              </mark>
                              {checked && replace !== find && (
                                <ins className="rounded-sm bg-primary/20 text-foreground no-underline">{replace}</ins>
                              )}
                              {hit.after}
                              {hit.raw && <span className="block truncate text-xs text-muted-foreground">raw: {hit.raw}</span>}
                            </button>
                          </li>
                        );
                      })}
                    </ul>
                  )}
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </div>
  );
}
