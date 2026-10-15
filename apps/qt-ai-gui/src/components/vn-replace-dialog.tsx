import { Wrench } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { vnReplace, vnSettingsGet } from "@/lib/api";
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

function suggestion(replacements: VnReplacement[], from: string, setting: GenreSetting): string {
  const wanted = setting === "ancient" ? "ancient" : "modern";
  return (
    replacements.find((r) => r.from === from && (r.setting === wanted || r.setting === "any"))?.to ??
    replacements.find((r) => r.from === from)?.to ??
    ""
  );
}

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
  const [from, setFrom] = useState(translated[0]?.matched ?? "");
  /** Cụm thay người dùng tự gõ; null = dùng gợi ý từ bảng thay. */
  const [typedTo, setTypedTo] = useState<string | null>(null);
  const [table, setTable] = useState<VnReplacement[]>([]);
  const [scope, setScope] = useState<Scope>("selected");
  const [ticked, setTicked] = useState<string[]>(() => [...new Set(translated.map((hit) => hit.chapter))]);
  const [rangeFrom, setRangeFrom] = useState("");
  const [rangeTo, setRangeTo] = useState("");
  const rows = useStoryStore((s) => s.snapshot?.chapters);
  const to = typedTo ?? suggestion(table, from, setting);

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
  const sameTerm = useMemo(
    () => allHits.filter((hit) => hit.side === "out" && hit.status !== "ok" && hit.matched === from),
    [allHits, from],
  );
  const chaptersWithTerm = useMemo(() => [...new Set(sameTerm.map((hit) => hit.chapter))], [sameTerm]);
  const skipped = translated.filter((hit) => hit.matched !== from).length;
  const target =
    scope === "selected"
      ? translated.filter((hit) => hit.matched === from)
      : scope === "chapters"
        ? sameTerm.filter((hit) => ticked.includes(hit.chapter))
        : sameTerm;
  const targetChapters = [...new Set(target.map((hit) => hit.chapter))];
  const found = occurrencesOf(target);
  const rawOnly = hits.length > 0 && translated.length === 0;

  /** Tick các chương có chỗ khớp nằm trong khoảng; ô trống = từ đầu / tới cuối. Nhận số thứ tự (#12) hoặc mã chương. */
  function tickRange() {
    const list = rows ?? [];
    const position = new Map(list.map((row, index) => [row.id, index]));
    const lo = rangeFrom.trim() ? resolveChapterRef(list, rangeFrom) : 0;
    const hi = rangeTo.trim() ? resolveChapterRef(list, rangeTo) : list.length - 1;
    if (lo < 0 || hi < 0 || lo > hi) {
      toast.error("Khoảng chương không hợp lệ");
      return;
    }
    setTicked(chaptersWithTerm.filter((id) => {
      const at = position.get(id);
      return at !== undefined && at >= lo && at <= hi;
    }));
  }

  async function run() {
    setBusy(true);
    try {
      const pairs = new Map<string, [string, number]>();
      for (const hit of target) pairs.set(`${hit.chapter}\u0000${hit.paragraph}`, [hit.chapter, hit.paragraph]);
      const request =
        scope === "selected"
          ? { from, to, chapters: targetChapters, paragraphs: [...pairs.values()] }
          : { from, to, chapters: targetChapters };
      const outcome = await vnReplace(root, request);
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
          Chỉ thay trong bản dịch (out/), nguyên từ, phân biệt hoa thường. Hoàn tác lần thay gần nhất ở trang Tìm &amp; thay.
        </DialogDescription>
      </DialogHeader>
      <div className="grid grid-cols-2 gap-3">
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="vn-from">Cụm gốc</Label>
          <Input id="vn-from" value={from} onChange={(e) => setFrom(e.target.value)} />
        </div>
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="vn-to">Cụm thay</Label>
          <Input id="vn-to" value={to} onChange={(e) => setTypedTo(e.target.value)} />
        </div>
      </div>
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
            <Input aria-label="Từ chương" placeholder="Từ (#1 hoặc mã)" className="h-8 w-32" value={rangeFrom} onChange={(e) => setRangeFrom(e.target.value)} />
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
      {rawOnly && (
        <p className="text-sm text-status-warning">
          Các chỗ đang chọn chỉ có ở raw (chương chưa dịch) — dịch xong mới thay được.
        </p>
      )}
      {scope === "selected" && skipped > 0 && !rawOnly && (
        <p className="text-sm text-muted-foreground">Bỏ qua {skipped} chỗ khác cụm gốc.</p>
      )}
      <p className="rounded-md bg-muted p-3 text-sm">
        {scope === "selected"
          ? `Sẽ thay ${found} chỗ ở ${targetChapters.length} chương.`
          : `Sẽ thay mọi chỗ "${from}" trong ${targetChapters.length} chương (đã phát hiện ${found} chỗ).`}
      </p>
      <DialogFooter>
        <Button variant="outline" disabled={busy} onClick={() => onOpenChange(false)}>
          Huỷ
        </Button>
        <Button disabled={busy || !from.trim() || !to.trim() || target.length === 0} onClick={() => void run()}>
          <Wrench /> Thay
        </Button>
      </DialogFooter>
    </>
  );
}
