import { Wrench } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { type VnReplaceRequest, vnReplace, vnSettingsGet } from "@/lib/api";
import { resolveChapterRef } from "@/lib/chapters";
import type { GenreSetting, VnHitView, VnReplacement } from "@/lib/types";
import { useStoryStore } from "@/store/story";

interface Props {
  root: string;
  setting: GenreSetting;
  /** Chỗ đang chọn ở tab Kiểm tra VN. */
  hits: VnHitView[];
  /** Mọi chỗ khớp của truyện (để tính phạm vi theo chương). */
  allHits: VnHitView[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDone: () => void;
}

type Scope = "selected" | "chapters" | "all";

/** Một cụm phát hiện trong các chỗ đang chọn. */
interface Row {
  /** Cụm khớp (matched) — khoá của dòng, để lấy các chỗ của nó. */
  term: string;
  from: string;
  /** Cụm thay người dùng tự gõ; null = dùng gợi ý từ bảng thay. */
  typedTo: string | null;
  included: boolean;
}

const WORD = /[\p{L}\p{N}\p{M}]/u;

/** Cụm thay của `from` trong bảng: ưu tiên dòng hợp bối cảnh truyện (hoặc "any"), không có thì dòng đầu. */
function lookup(table: VnReplacement[], from: string, setting: GenreSetting): string | undefined {
  const wanted = setting === "ancient" ? "ancient" : "modern";
  const rows = table.filter((r) => r.from === from);
  return (rows.find((r) => r.setting === wanted || r.setting === "any") ?? rows[0])?.to;
}

/** Các chỗ `part` nằm trong `text` mà ký tự kề hai đầu không phải chữ/số (chỉ xét đầu nào của `part` là chữ/số). */
function boundedAt(text: string, part: string): number[] {
  const found: number[] = [];
  const end = (at: number) => at + part.length;
  for (let at = text.indexOf(part); at >= 0;) {
    const leftOk = !WORD.test(part.charAt(0)) || at === 0 || !WORD.test(text.charAt(at - 1));
    const rightOk = !WORD.test(part.charAt(part.length - 1)) || end(at) === text.length || !WORD.test(text.charAt(end(at)));
    if (leftOk && rightOk) {
      found.push(at);
      at = text.indexOf(part, end(at));
    } else {
      at = text.indexOf(part, at + 1);
    }
  }
  return found;
}

/** Gợi ý cụm thay: đúng cụm trong bảng; không có thì thay phần dài nhất của cụm có trong bảng ("khỉ Việt Nam" → "khỉ Thái Lan"). */
function suggestion(table: VnReplacement[], from: string, setting: GenreSetting): string {
  const exact = lookup(table, from, setting);
  if (exact !== undefined) return exact;
  const parts = [...new Set(table.map((r) => r.from))].filter((part) => part.trim() && part !== from).sort((x, y) => y.length - x.length);
  for (const part of parts) {
    const at = boundedAt(from, part);
    if (at.length === 0) continue;
    const to = lookup(table, part, setting) ?? "";
    let out = "";
    let cursor = 0;
    for (const start of at) {
      out += from.slice(cursor, start) + to;
      cursor = start + part.length;
    }
    return out + from.slice(cursor);
  }
  return "";
}

const unique = <T,>(list: T[]) => [...new Set(list)];

/** Số lần xuất hiện của các chỗ: gộp theo (chương, đoạn), mỗi đoạn lấy count lớn nhất. */
function occurrencesOf(hits: VnHitView[]): number {
  const perParagraph = new Map<string, number>();
  for (const hit of hits) {
    const key = `${hit.chapter}\u0000${hit.paragraph}`;
    perParagraph.set(key, Math.max(perParagraph.get(key) ?? 0, hit.count));
  }
  let total = 0;
  for (const count of perParagraph.values()) total += count;
  return total;
}

/** Thay cụm nhạy cảm trong bản dịch: chỗ đang chọn, các chương tick, hoặc mọi chương có chỗ khớp. Có hoàn tác ở trang Tìm & thay. */
export function VnReplaceDialog({ root, setting, hits, allHits, open, onOpenChange, onDone }: Props) {
  const [busy, setBusy] = useState(false);
  return (
    <Dialog open={open} onOpenChange={(value) => !busy && onOpenChange(value)}>
      <DialogContent>
        {/* Nội dung chỉ gắn khi hộp thoại mở, nên mỗi lần mở là một form mới. */}
        <VnReplaceForm
          root={root}
          setting={setting}
          hits={hits}
          allHits={allHits}
          busy={busy}
          setBusy={setBusy}
          onOpenChange={onOpenChange}
          onDone={onDone}
        />
      </DialogContent>
    </Dialog>
  );
}

function VnReplaceForm({
  root,
  setting,
  hits,
  allHits,
  busy,
  setBusy,
  onOpenChange,
  onDone,
}: Omit<Props, "open"> & { busy: boolean; setBusy: (busy: boolean) => void }) {
  const translated = hits.filter((hit) => hit.side === "out");
  const [rows, setRows] = useState<Row[]>(() =>
    unique(translated.map((hit) => hit.matched)).map((term) => ({ term, from: term, typedTo: null, included: true })),
  );
  const [table, setTable] = useState<VnReplacement[]>([]);
  const [scope, setScope] = useState<Scope>("selected");
  const [ticked, setTicked] = useState<string[]>(() => unique(translated.map((hit) => hit.chapter)));
  const [rangeFrom, setRangeFrom] = useState("");
  const [rangeTo, setRangeTo] = useState("");
  const chapterRows = useStoryStore((s) => s.snapshot?.chapters);

  useEffect(() => {
    let alive = true;
    vnSettingsGet()
      .then((settings) => {
        if (alive) setTable(settings.replacements);
      })
      .catch((error: unknown) => {
        if (alive) toast.error(error instanceof Error ? error.message : "Không đọc được bảng thay");
      });
    return () => {
      alive = false;
    };
  }, []);

  // Chỗ đã "Không sao" không tính: chương chỉ còn những chỗ đó không được đề xuất / tick.
  const pending = useMemo(() => allHits.filter((hit) => hit.side === "out" && hit.status !== "ok"), [allHits]);
  const chaptersWithTerm = unique(rows.flatMap((row) => pending.filter((hit) => hit.matched === row.term).map((hit) => hit.chapter)));
  const plans = rows.map((row) => {
    const to = row.typedTo ?? suggestion(table, row.from, setting);
    const termHits = (scope === "selected" ? translated : pending).filter((hit) => hit.matched === row.term);
    const target = scope === "chapters" ? termHits.filter((hit) => ticked.includes(hit.chapter)) : termHits;
    // Cụm thay trùng cụm gốc: backend bỏ qua, nên không tính.
    const active = row.included && row.from.trim() !== "" && to.trim() !== "" && row.from.trim() !== to.trim() && target.length > 0;
    return { row, to, target, active };
  });
  const active = plans.filter((plan) => plan.active);
  const targetChapters = unique(active.flatMap((plan) => plan.target.map((hit) => hit.chapter)));
  const found = active.reduce((sum, plan) => sum + occurrencesOf(plan.target), 0);
  const rawOnly = hits.length > 0 && translated.length === 0;

  function updateRow(index: number, change: Partial<Row>) {
    setRows((list) => list.map((row, at) => (at === index ? { ...row, ...change } : row)));
  }

  /** Tick các chương có chỗ khớp nằm trong khoảng; ô trống = từ đầu / tới cuối. Nhận số thứ tự (#12) hoặc mã chương. */
  function tickRange() {
    const list = chapterRows ?? [];
    const position = new Map(list.map((row, index) => [row.id, index]));
    const lo = rangeFrom.trim() ? resolveChapterRef(list, rangeFrom) : 0;
    const hi = rangeTo.trim() ? resolveChapterRef(list, rangeTo) : list.length - 1;
    if (lo < 0 || hi < 0 || lo > hi) {
      toast.error("Khoảng chương không hợp lệ");
      return;
    }
    setTicked(
      chaptersWithTerm.filter((id) => {
        const at = position.get(id);
        return at !== undefined && at >= lo && at <= hi;
      }),
    );
  }

  async function run() {
    setBusy(true);
    try {
      const requests: VnReplaceRequest[] = active.map(({ row, to, target }) => {
        const chapters = unique(target.map((hit) => hit.chapter));
        if (scope !== "selected") return { from: row.from, to, chapters };
        const pairs = new Map<string, [string, number]>();
        for (const hit of target) pairs.set(`${hit.chapter}\u0000${hit.paragraph}`, [hit.chapter, hit.paragraph]);
        return { from: row.from, to, chapters, paragraphs: [...pairs.values()] };
      });
      const outcome = await vnReplace(root, requests);
      toast.success(`Đã thay ${outcome.occurrences} chỗ ở ${outcome.chapters} chương — hoàn tác ở trang Tìm & thay`);
      onOpenChange(false);
      onDone();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không thay được");
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <DialogHeader>
        <DialogTitle>Thay cụm nhạy cảm</DialogTitle>
        <DialogDescription>
          Chỉ thay trong bản dịch (out/), nguyên từ, phân biệt hoa thường. Mọi cặp thay chung một lần hoàn tác ở trang Tìm &amp; thay.
        </DialogDescription>
      </DialogHeader>
      {rows.length > 0 && (
        <div className="flex max-h-64 flex-col gap-2 overflow-auto">
          <div className="grid grid-cols-[minmax(0,8rem)_1fr_1fr] gap-2 text-xs text-muted-foreground">
            <span>Cụm phát hiện</span>
            <span>Cụm gốc</span>
            <span>Cụm thay</span>
          </div>
          {plans.map(({ row, to }, index) => (
            <div key={row.term} className="grid grid-cols-[minmax(0,8rem)_1fr_1fr] items-start gap-2">
              <label className="flex items-center gap-2 pt-2 text-sm">
                <input
                  type="checkbox"
                  aria-label={`Thay cụm ${index + 1}`}
                  checked={row.included}
                  onChange={(e) => updateRow(index, { included: e.target.checked })}
                />
                <span className="truncate" title={row.term}>
                  {row.term}
                </span>
              </label>
              <Input aria-label={`Cụm gốc ${index + 1}`} value={row.from} onChange={(e) => updateRow(index, { from: e.target.value })} />
              <div className="flex flex-col gap-1">
                <Input aria-label={`Cụm thay ${index + 1}`} value={to} onChange={(e) => updateRow(index, { typedTo: e.target.value })} />
                {!to.trim() && <span className="text-xs text-muted-foreground">Bỏ trống = không thay</span>}
              </div>
            </div>
          ))}
        </div>
      )}
      <fieldset className="flex flex-col gap-1 text-sm">
        <label className="flex items-center gap-2">
          <input type="radio" name="vn-scope" checked={scope === "selected"} onChange={() => setScope("selected")} />
          Chỉ các chỗ đang chọn
        </label>
        <label className="flex items-center gap-2">
          <input type="radio" name="vn-scope" checked={scope === "chapters"} onChange={() => setScope("chapters")} />
          Chọn chương
        </label>
        {scope === "chapters" && (
          <div className="ml-6 flex items-center gap-2">
            <Input
              aria-label="Từ chương"
              placeholder="Từ (#1 hoặc mã)"
              className="h-8 w-32"
              value={rangeFrom}
              onChange={(e) => setRangeFrom(e.target.value)}
            />
            <Input aria-label="Tới chương" placeholder="Tới" className="h-8 w-32" value={rangeTo} onChange={(e) => setRangeTo(e.target.value)} />
            <Button type="button" size="sm" variant="outline" onClick={tickRange}>
              Tick khoảng
            </Button>
          </div>
        )}
        {scope === "chapters" && (
          <div className="ml-6 flex max-h-32 flex-wrap gap-2 overflow-auto">
            {chaptersWithTerm.map((chapter) => (
              <label key={chapter} className="flex items-center gap-1 font-mono text-xs">
                <input
                  type="checkbox"
                  aria-label={chapter}
                  checked={ticked.includes(chapter)}
                  onChange={(e) => setTicked((list) => (e.target.checked ? [...list, chapter] : list.filter((c) => c !== chapter)))}
                />
                {chapter}
              </label>
            ))}
          </div>
        )}
        <label className="flex items-center gap-2">
          <input type="radio" name="vn-scope" checked={scope === "all"} onChange={() => setScope("all")} />
          Mọi chương có chỗ khớp
        </label>
      </fieldset>
      {rawOnly && <p className="text-sm text-status-warning">Các chỗ đang chọn chỉ có ở raw (chương chưa dịch) — dịch xong mới thay được.</p>}
      <p className="rounded-md bg-muted p-3 text-sm">
        {scope === "selected"
          ? `Sẽ thay ${found} chỗ ở ${targetChapters.length} chương.`
          : `Sẽ thay mọi chỗ của ${active.length} cụm trong ${targetChapters.length} chương (đã phát hiện ${found} chỗ).`}
      </p>
      <DialogFooter>
        <Button variant="outline" disabled={busy} onClick={() => onOpenChange(false)}>
          Huỷ
        </Button>
        <Button disabled={busy || active.length === 0} onClick={() => void run()}>
          <Wrench /> Thay
        </Button>
      </DialogFooter>
    </>
  );
}
