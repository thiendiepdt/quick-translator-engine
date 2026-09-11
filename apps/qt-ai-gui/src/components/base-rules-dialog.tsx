import { LoaderCircle, RotateCcw } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Choice } from "@/components/choice";
import { RuleTable, type RuleRow } from "@/components/rule-table";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { baseGet, baseReset, baseSave } from "@/lib/api";
import { SETTING_CHOICE_LABELS } from "@/lib/genre-labels";
import { rowsOf, rulesOf } from "@/lib/rule-rows";
import { GENRE_SETTINGS } from "@/lib/schema";
import type { BaseView, GenreSetting } from "@/lib/types";
import { useStoryStore } from "@/store/story";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/** Sửa bộ rule mặc định theo bối cảnh: file base/rules/<setting>.json thay bộ cứng đã lọc. */
export function BaseRulesDialog({ open, onOpenChange }: Props) {
  const bump = useStoryStore((s) => s.bumpBaseVersion);
  const [setting, setSetting] = useState<GenreSetting>("ancient");
  // State khoá theo bối cảnh đang chọn: đổi bối cảnh là bản cũ tự biến mất, không reset trong effect.
  const [loaded, setLoaded] = useState<{ setting: GenreSetting; view: BaseView } | undefined>();
  const [edit, setEdit] = useState<{ setting: GenreSetting; rows: RuleRow[] } | undefined>();
  const [busy, setBusy] = useState(false);
  const view = loaded?.setting === setting ? loaded.view : undefined;
  const rows = edit?.setting === setting ? edit.rows : rowsOf(view?.rules);
  const dirty = edit?.setting === setting;

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    baseGet("rules", setting, undefined)
      .then((next) => {
        if (!cancelled) setLoaded({ setting, view: next });
      })
      .catch((error: unknown) => toast.error(error instanceof Error ? error.message : "Không đọc được rule mặc định"));
    return () => {
      cancelled = true;
    };
  }, [open, setting]);

  async function run(action: () => Promise<BaseView>, done: string) {
    setBusy(true);
    try {
      const next = await action();
      setLoaded({ setting, view: next });
      setEdit(undefined);
      bump();
      toast.success(done);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex max-h-[92vh] w-[min(96vw,64rem)] flex-col">
        <DialogHeader>
          <DialogTitle>Rule kiểm tra mặc định</DialogTitle>
          <DialogDescription>
            Bộ rule chạy cho truyện chưa có rule riêng, theo bối cảnh. Rule "còn Hán tự" luôn chạy, không nằm ở đây.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-wrap items-end gap-4">
          <div className="flex flex-col gap-1.5">
            <Label>Bối cảnh</Label>
            <Choice
              label="Bối cảnh"
              value={setting}
              options={GENRE_SETTINGS}
              labels={SETTING_CHOICE_LABELS}
              onChange={(value) => {
                if (dirty && !window.confirm("Bỏ thay đổi chưa lưu?")) return;
                setSetting(value);
              }}
              disabled={busy}
            />
          </div>
          <span className="rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground">
            {view ? (view.source === "file" ? "đã sửa" : "bản cứng") : "…"} · {rows.length}
          </span>
        </div>
        <div className="fine-scrollbar min-h-0 flex-1 overflow-y-auto">
          {view ? (
            <RuleTable rows={rows} onChange={(next) => setEdit({ setting, rows: next })} />
          ) : (
            <div role="status" className="grid min-h-[200px] place-items-center text-sm text-muted-foreground">
              Đang tải…
            </div>
          )}
        </div>
        <DialogFooter>
          <Button
            type="button"
            variant="ghost"
            disabled={busy || view?.source !== "file"}
            onClick={() => void run(() => baseReset("rules", setting, undefined), "Đã về bản cứng")}
          >
            <RotateCcw /> Về mặc định
          </Button>
          <Button
            type="button"
            disabled={busy || !dirty}
            onClick={() => void run(() => baseSave("rules", setting, undefined, { rules: rulesOf(rows) }), "Đã lưu rule mặc định")}
          >
            {busy && <LoaderCircle className="animate-spin" />} Lưu
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
