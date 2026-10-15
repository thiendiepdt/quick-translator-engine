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

/** Thay cụm nhạy cảm trong bản dịch: chỗ đang chọn, các chương tick, hoặc mọi chương có chỗ khớp. Có hoàn tác ở trang Tìm & thay. */
export function VnReplaceDialog({ root, setting, hits, allHits, open, onOpenChange, onDone }: Props) {
  const translated = hits.filter((hit) => hit.side === "out");
  const initialFrom = translated[0]?.matched ?? "";
  const [from, setFrom] = useState(initialFrom);
  const [to, setTo] = useState("");
  const [scope, setScope] = useState<Scope>("selected");
  const [ticked, setTicked] = useState<string[]>([]);
  const [rangeFrom, setRangeFrom] = useState("");
  const [rangeTo, setRangeTo] = useState("");
  const [busy, setBusy] = useState(false);
  const rows = useStoryStore((s) => s.snapshot?.chapters);

  useEffect(() => {
    if (!open) return;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- đặt lại form mỗi lần mở hộp thoại
    setFrom(initialFrom);
    setScope("selected");
    setTicked([...new Set(translated.map((hit) => hit.chapter))]);
    void vnSettingsGet().then((settings) => setTo(suggestion(settings.replacements, initialFrom, setting)));
    // eslint-disable-next-line react-hooks/exhaustive-deps -- chỉ nạp lại khi mở hộp thoại
  }, [open]);

  const sameTerm = useMemo(() => allHits.filter((hit) => hit.side === "out" && hit.matched === from), [allHits, from]);
  const chaptersWithTerm = useMemo(() => [...new Set(sameTerm.map((hit) => hit.chapter))], [sameTerm]);
  const target =
    scope === "selected"
      ? translated.filter((hit) => hit.matched === from)
      : scope === "chapters"
        ? sameTerm.filter((hit) => ticked.includes(hit.chapter))
        : sameTerm;
  const targetChapters = [...new Set(target.map((hit) => hit.chapter))];

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
      const request =
        scope === "selected"
          ? { from, to, chapters: targetChapters, paragraphs: target.map((hit) => [hit.chapter, hit.paragraph] as [string, number]) }
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
    <Dialog open={open} onOpenChange={(value) => !busy && onOpenChange(value)}>
      <DialogContent>
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
            <Input id="vn-to" value={to} onChange={(e) => setTo(e.target.value)} />
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
        <p className="rounded-md bg-muted p-3 text-sm">
          Sẽ thay {target.length} chỗ ở {targetChapters.length} chương.
        </p>
        <DialogFooter>
          <Button variant="outline" disabled={busy} onClick={() => onOpenChange(false)}>
            Huỷ
          </Button>
          <Button disabled={busy || !from.trim() || !to.trim() || target.length === 0} onClick={() => void run()}>
            <Wrench /> Thay
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
