import { ChevronDown, ChevronRight, LoaderCircle, Replace, Undo2 } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { z } from "zod";

import { ChapterRefHint } from "@/components/chapter-ref-hint";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  replaceApply,
  replaceFind,
  replaceUndo,
  replaceUndoInfo,
  storySnapshot,
  type ReplaceHitRef,
  type ReplaceProfileRef,
  type ReplaceQuery,
} from "@/lib/api";
import { chapterLabel } from "@/lib/cast-addressing";
import { resolveChapterRef } from "@/lib/chapters";
import { replaceFindSchema, replaceUndoInfoSchema } from "@/lib/schema";
import { GLOSSARY_LABELS, type GlossaryKey } from "@/lib/types";
import { selectCurrentRunning, useStoryStore } from "@/store/story";

type FindResult = z.infer<typeof replaceFindSchema>;
type Hit = FindResult["hits"][number];
type ProfileHit = FindResult["profile"][number];
type UndoInfo = z.infer<typeof replaceUndoInfoSchema>;

/** Gõ xong chừng này ms mới tìm — gõ liên tục không bắn lệnh liên tục. */
const DEBOUNCE_MS = 300;
/** Mở sẵn chừng này chương đầu; chương sau bấm để mở. */
const OPEN_FIRST = 10;
/** Tab hồ sơ hiện từng khúc chừng này chỗ (sổ tiêu đề có thể cả nghìn dòng). */
const PROFILE_PAGE = 100;

const hitKey = (hit: ReplaceHitRef) => `${hit.chapter}|${hit.paragraph}|${hit.occurrence}`;
const profileKey = (hit: ReplaceProfileRef) => `${hit.field}|${hit.key}|${hit.index}|${hit.occurrence}`;

type Tab = "chapters" | "profile" | "cast" | "titles";
const TABS: Array<{ tab: Tab; label: string }> = [
  { tab: "chapters", label: "Chương" },
  { tab: "profile", label: "Hồ sơ" },
  { tab: "cast", label: "Nhân vật" },
  { tab: "titles", label: "Tiêu đề chương" },
];

/** Ô hồ sơ (Rust `ProfileRef.field`) nằm ở tab nào: cast.json → Nhân vật, titles.json → Tiêu đề, còn lại story.json. */
function tabOf(field: string): Tab {
  if (field === "addressing" || field === "narration") return "cast";
  if (field === "titles") return "titles";
  return "profile";
}

const FIELD_LABELS: Record<string, string> = {
  signaturePhrases: "Cụm từ đặc trưng (style)",
  protagonist: "Nhân vật chính",
  summary: "Tóm tắt",
  voice: "Giọng kể / voice",
  toneRules: "Tone rules",
  avoid: "Cách diễn đạt cần tránh",
  customPrompt: "Prompt",
  addressing: "Mốc đổi xưng hô",
  narration: "Đại từ lời kể",
  titles: "Sổ tiêu đề chương",
};

function fieldLabel(field: string): string {
  const group = field.startsWith("glossary.") ? field.slice("glossary.".length) : "";
  if (group in GLOSSARY_LABELS) return `Glossary · ${GLOSSARY_LABELS[group as GlossaryKey]}`;
  return FIELD_LABELS[field] ?? field;
}

/** Vị trí của chỗ khớp trong ô: khoá Hán, dòng thứ mấy, hay mốc từ chương nào. */
function profileWhere(hit: ProfileHit): string {
  if (hit.field === "toneRules" || hit.field === "avoid") return `dòng ${hit.index + 1}`;
  if (hit.chapter) return `${hit.key} · từ ch.${chapterLabel(hit.chapter)}`;
  return hit.key;
}

function tabCounts(result: FindResult | undefined): Record<Tab, number> {
  const counts: Record<Tab, number> = { chapters: result?.total ?? 0, profile: 0, cast: 0, titles: 0 };
  for (const hit of result?.profile ?? []) counts[tabOf(hit.field)] += 1;
  return counts;
}

const FILE_LABELS: Record<string, string> = { "story.json": "Hồ sơ truyện", "cast.json": "bảng nhân vật", "titles.json": "sổ tiêu đề" };

function toggle<T>(set: Set<T>, value: T): Set<T> {
  const next = new Set(set);
  if (next.has(value)) next.delete(value);
  else next.add(value);
  return next;
}

/** Chữ quanh chỗ khớp; chỗ đang chọn thay thì gạch chữ cũ, hiện chữ mới ngay sau. */
function Snippet({ hit, replacing, replace }: { hit: { before: string; matched: string; after: string }; replacing: boolean; replace: string }) {
  return (
    <>
      {hit.before}
      <mark className={replacing ? "bg-transparent text-destructive line-through" : "rounded-sm bg-primary/20 text-foreground"}>{hit.matched}</mark>
      {replacing && <ins className="rounded-sm bg-primary/20 text-foreground no-underline">{replace}</ins>}
      {hit.after}
    </>
  );
}

/** Chỗ khớp của một tab hồ sơ, gộp theo ô; tick ở đầu nhóm chọn / bỏ cả nhóm. */
function ProfileHits({
  hits,
  excluded,
  setExcluded,
  find,
  replace,
}: {
  hits: ProfileHit[];
  excluded: Set<string>;
  setExcluded: (update: (set: Set<string>) => Set<string>) => void;
  find: string;
  replace: string;
}) {
  const [limit, setLimit] = useState(PROFILE_PAGE);
  const groups = useMemo(() => {
    const byField = new Map<string, ProfileHit[]>();
    for (const hit of hits) byField.set(hit.field, [...(byField.get(hit.field) ?? []), hit]);
    return [...byField];
  }, [hits]);
  const shown = new Set(hits.slice(0, limit));

  return (
    <div className="flex flex-col gap-1">
      {groups.map(([field, all]) => {
        const visible = all.filter((hit) => shown.has(hit));
        if (visible.length === 0) return null;
        const on = all.filter((hit) => !excluded.has(profileKey(hit))).length;
        const label = fieldLabel(field);
        return (
          <section key={field} className="rounded-md border" aria-label={label}>
            <label className="flex items-center gap-2 px-2 py-1.5 text-sm font-medium">
              <input
                type="checkbox"
                checked={on === all.length}
                aria-label={`Chọn cả nhóm ${label}`}
                onChange={() =>
                  setExcluded((set) => {
                    const next = new Set(set);
                    for (const hit of all) {
                      if (on === all.length) next.add(profileKey(hit));
                      else next.delete(profileKey(hit));
                    }
                    return next;
                  })
                }
              />
              {label}
              <span className="ml-auto rounded-full bg-muted px-1.5 text-xs font-normal text-muted-foreground">{all.length} chỗ</span>
            </label>
            <ul className="flex flex-col gap-1 border-t px-2 py-1.5">
              {visible.map((hit) => {
                const key = profileKey(hit);
                const checked = !excluded.has(key);
                const where = profileWhere(hit);
                const nth = hit.occurrence > 0 ? ` chỗ ${hit.occurrence + 1}` : "";
                return (
                  <li key={key} className="flex items-start gap-2 text-sm">
                    <input
                      type="checkbox"
                      className="mt-1"
                      checked={checked}
                      aria-label={`Chọn ${label}${where ? ` ${where}` : ""}${nth}`}
                      onChange={() => setExcluded((set) => toggle(set, key))}
                    />
                    <span className="min-w-0 flex-1">
                      {where && <span className="mr-1.5 font-mono text-[11px] text-muted-foreground">{where}</span>}
                      <Snippet hit={hit} replacing={checked && replace !== find} replace={replace} />
                    </span>
                  </li>
                );
              })}
            </ul>
          </section>
        );
      })}
      {hits.length > limit && (
        <Button type="button" size="xs" variant="ghost" className="self-start" onClick={() => setLimit((l) => l + PROFILE_PAGE)}>
          Hiện thêm {Math.min(PROFILE_PAGE, hits.length - limit)} / còn {hits.length - limit}
        </Button>
      )}
    </div>
  );
}

/**
 * Tìm & thay trong bản dịch đã chốt và hồ sơ của truyện: tìm theo chữ (không regex), xem từng chỗ kèm câu xung
 * quanh và câu raw của đoạn, bỏ chọn chỗ không muốn thay, thay một lần, hoàn tác lần thay gần nhất. Kết quả chia tab
 * Chương / Hồ sơ (story.json) / Nhân vật (cast.json) / Tiêu đề chương (titles.json) — đổi tên trong chương mà hồ sơ
 * giữ tên cũ thì chương dịch sau quay về tên cũ. Tìm/thay chạy ở Rust; trang chỉ nhận 500 chỗ đầu trong chương + số
 * theo chương nên truyện nghìn chương không làm giao diện đơ.
 */
export function ReplacePage() {
  const root = useStoryStore((s) => s.root);
  const rows = useStoryStore((s) => s.snapshot?.chapters);
  const running = useStoryStore(selectCurrentRunning);
  const select = useStoryStore((s) => s.select);
  const setPage = useStoryStore((s) => s.setPage);
  const setSnapshot = useStoryStore((s) => s.setSnapshot);

  const [find, setFind] = useState("");
  const [replace, setReplace] = useState("");
  const [wholeWord, setWholeWord] = useState(true);
  const [caseSensitive, setCaseSensitive] = useState(false);
  const [rawFilter, setRawFilter] = useState("");
  const [fromText, setFromText] = useState("");
  const [toText, setToText] = useState("");
  const [result, setResult] = useState<FindResult>();
  // Tăng mỗi lần có kết quả mới để danh sách hồ sơ về khúc đầu.
  const [resultId, setResultId] = useState(0);
  const [searching, setSearching] = useState(false);
  const [tab, setTab] = useState<Tab>("chapters");
  const [excludedChapters, setExcludedChapters] = useState<Set<string>>(new Set());
  const [excludedHits, setExcludedHits] = useState<Set<string>>(new Set());
  const [excludedProfile, setExcludedProfile] = useState<Set<string>>(new Set());
  const [open, setOpen] = useState<Set<string>>(new Set());
  const [undo, setUndo] = useState<UndoInfo>(null);
  const [status, setStatus] = useState("");
  // Lần thay vừa rồi không có chỗ nào trong hồ sơ → nhắc thêm vào glossary.
  const [suggestGlossary, setSuggestGlossary] = useState(false);
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
          setResultId((id) => id + 1);
          setExcludedChapters(new Set());
          setExcludedHits(new Set());
          setExcludedProfile(new Set());
          setOpen(new Set(found.chapters.slice(0, OPEN_FIRST).map((c) => c.chapter)));
          // Giữ tab đang xem nếu nó còn chỗ khớp, không thì sang tab đầu tiên có.
          const counts = tabCounts(found);
          setTab((current) => (counts[current] > 0 ? current : (TABS.find((t) => counts[t.tab] > 0)?.tab ?? "chapters")));
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
  const profileByTab = useMemo(() => {
    const groups: Record<Tab, ProfileHit[]> = { chapters: [], profile: [], cast: [], titles: [] };
    for (const hit of result?.profile ?? []) groups[tabOf(hit.field)].push(hit);
    return groups;
  }, [result]);

  if (!root) return null;
  const currentRoot = root;

  const counts = tabCounts(result);
  const profileHits = result?.profile ?? [];
  const chapterSelected = (result?.chapters ?? []).reduce((sum, chapter) => {
    if (excludedChapters.has(chapter.chapter)) return sum;
    const off = (hitsByChapter.get(chapter.chapter) ?? []).filter((hit) => excludedHits.has(hitKey(hit))).length;
    return sum + chapter.count - off;
  }, 0);
  const selectedCount = chapterSelected + profileHits.filter((hit) => !excludedProfile.has(profileKey(hit))).length;
  const anyHit = (result?.total ?? 0) > 0 || profileHits.length > 0;
  const canReplace = !busy && !running && selectedCount > 0 && find !== "" && find !== replace;

  /** Làm mới snapshot để trang Hồ sơ truyện đọc story.json vừa đổi. */
  function refreshStory() {
    void storySnapshot(currentRoot)
      .then(setSnapshot)
      .catch(() => undefined);
  }

  async function apply() {
    setBusy(true);
    try {
      const excluded = (result?.hits ?? []).filter((hit) => excludedHits.has(hitKey(hit)) && !excludedChapters.has(hit.chapter));
      const profile = profileHits.filter((hit) => excludedProfile.has(profileKey(hit)));
      const done = await replaceApply(
        currentRoot,
        query,
        [...excludedChapters],
        excluded.map(({ chapter, paragraph, occurrence }) => ({ chapter, paragraph, occurrence })),
        profile.map(({ field, key, index, occurrence }) => ({ field, key, index, occurrence })),
      );
      const inProfile = done.profile > 0 ? ` và ${done.profile} chỗ trong hồ sơ` : "";
      setStatus(`Đã thay ${done.occurrences} chỗ trong ${done.chapters} chương${inProfile}.`);
      setSuggestGlossary(done.occurrences > 0 && done.profile === 0 && profileHits.length === 0);
      if (done.profile > 0) refreshStory();
      setEpoch((e) => e + 1);
    } catch (error) {
      setStatus(error instanceof Error ? error.message : "Không thay được");
      setSuggestGlossary(false);
    } finally {
      setBusy(false);
    }
  }

  async function undoLast() {
    setBusy(true);
    try {
      const done = await replaceUndo(currentRoot);
      const files = done.profileRestored > 0 ? ` và ${done.profileRestored} file hồ sơ` : "";
      const kept = done.conflicts.length ? ` ${done.conflicts.length} chương đã sửa tiếp sau đó nên giữ nguyên.` : "";
      const keptFiles = done.profileConflicts.length
        ? ` ${done.profileConflicts.map((name) => FILE_LABELS[name] ?? name).join(", ")} đã sửa tiếp sau đó nên giữ nguyên.`
        : "";
      setStatus(`Đã hoàn tác ${done.restored} chương${files}.${kept}${keptFiles}`);
      setSuggestGlossary(false);
      if (done.profileRestored > 0) refreshStory();
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
  const summary = [
    result && result.total > 0 ? `${result.total} chỗ trong ${result.chapters.length} chương` : "",
    profileHits.length > 0 ? `${profileHits.length} chỗ trong hồ sơ` : "",
  ]
    .filter(Boolean)
    .join(", ");

  return (
    <div className="fine-scrollbar h-full min-h-0 overflow-y-auto">
      <div className="mx-auto flex max-w-4xl flex-col gap-4 px-6 py-6">
        <div>
          <h1 className="text-lg font-semibold">Tìm & thay</h1>
          <p className="text-xs text-muted-foreground">
            Tìm trong các chương đã dịch và hồ sơ của truyện này (glossary, nhân vật, tiêu đề chương). Xem từng chỗ kèm câu
            xung quanh và câu raw, bỏ tick chỗ nào không muốn thay, rồi bấm Thay. Thay nhầm thì hoàn tác được lần thay gần
            nhất.
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
          <Label
            htmlFor="replace-raw"
            title="Chương: chỉ đoạn mà câu raw cùng vị trí có chữ này. Hồ sơ: chỉ mục có khoá Hán chứa chữ này"
          >
            Chỉ đoạn raw có
          </Label>
          <Input id="replace-raw" value={rawFilter} onChange={(e) => setRawFilter(e.target.value)} placeholder="tuỳ chọn, vd. 人族" className="font-mono" />
          <Label htmlFor="replace-from" title="Chỉ áp cho tab Chương — hồ sơ không gắn với chương">
            Chương
          </Label>
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
            {!find ? "Gõ chữ cần tìm." : !result ? "Đang tìm…" : !anyHit ? "Không có chỗ nào khớp." : `${summary} — đang chọn ${selectedCount}.`}
            {result && result.skipped > 0 && ` Bỏ qua ${result.skipped} chương số đoạn lệch raw.`}
          </p>
          {result && anyHit && (
            <Button
              type="button"
              size="sm"
              variant="ghost"
              onClick={() => {
                const none = selectedCount === 0;
                setExcludedChapters(none ? new Set() : new Set(result.chapters.map((c) => c.chapter)));
                setExcludedHits(new Set());
                setExcludedProfile(none ? new Set() : new Set(profileHits.map(profileKey)));
              }}
            >
              {selectedCount === 0 ? "Chọn hết" : "Bỏ chọn hết"}
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
            {suggestGlossary && (
              <p className="mt-1 text-muted-foreground">
                Hồ sơ truyện chưa có chữ này. Nếu là tên riêng hay thuật ngữ, thêm vào{" "}
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
              Lần thay gần nhất: “{undo.find}” → “{undo.replace}” ({undo.occurrences} chỗ, {undo.chapters} chương
              {undo.profile > 0 && `, ${undo.profile} chỗ trong hồ sơ`}).
            </span>
            <Button type="button" size="xs" variant="outline" disabled={busy || running} onClick={() => void undoLast()}>
              <Undo2 /> Hoàn tác
            </Button>
          </div>
        )}

        {result && anyHit && (
          <Tabs value={tab} onValueChange={(value) => setTab(value as Tab)}>
            <TabsList aria-label="Nơi có chỗ khớp">
              {TABS.map(({ tab: value, label }) => (
                <TabsTrigger key={value} value={value} disabled={counts[value] === 0}>
                  {label}{" "}
                  <span className="rounded-full bg-muted px-1.5 text-xs font-normal text-muted-foreground">{counts[value]}</span>
                </TabsTrigger>
              ))}
            </TabsList>
            <TabsContent value="chapters">
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
                                  <Snippet hit={hit} replacing={checked && replace !== find} replace={replace} />
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
            </TabsContent>
            {TABS.filter(({ tab: value }) => value !== "chapters").map(({ tab: value }) => (
              <TabsContent key={value} value={value}>
                <ProfileHits
                  key={resultId}
                  hits={profileByTab[value]}
                  excluded={excludedProfile}
                  setExcluded={setExcludedProfile}
                  find={find}
                  replace={replace}
                />
              </TabsContent>
            ))}
          </Tabs>
        )}
      </div>
    </div>
  );
}
