import { LoaderCircle, RotateCcw } from "lucide-react";
import { lazy, Suspense, useEffect, useState } from "react";
import { toast } from "sonner";

import { Choice } from "@/components/choice";
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
import { NAMES_CHOICE_LABELS, SETTING_CHOICE_LABELS } from "@/lib/genre-labels";
import { GENRE_NAMES, GENRE_SETTINGS } from "@/lib/schema";
import type { BaseView, GenreNames, GenreSetting } from "@/lib/types";
import { useStoryStore } from "@/store/story";

// Plate + remark nặng; chỉ tải khi mở dialog.
const PlatePromptEditor = lazy(async () => {
  const module = await import("@/components/plate-prompt-editor");
  return { default: module.PlatePromptEditor };
});

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/**
 * Sửa base prompt của app theo genre (9 tổ hợp). Lưu = ghi file base/prompts/<setting>-<names>.md;
 * Về mặc định = xoá file. Truyện đang dùng prompt mặc định ăn theo (bumpBaseVersion nạp lại defaults).
 */
export function BasePromptDialog({ open, onOpenChange }: Props) {
  const bump = useStoryStore((s) => s.bumpBaseVersion);
  const [setting, setSetting] = useState<GenreSetting>("ancient");
  const [names, setNames] = useState<GenreNames>("han");
  const [view, setView] = useState<BaseView | undefined>();
  const [draft, setDraft] = useState<string | undefined>();
  const [version, setVersion] = useState(0);
  const [busy, setBusy] = useState(false);
  const dirty = draft !== undefined && draft !== view?.text;

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    setView(undefined);
    setDraft(undefined);
    baseGet("prompt", setting, names)
      .then((next) => {
        if (cancelled) return;
        setView(next);
        setVersion((v) => v + 1);
      })
      .catch((error: unknown) => toast.error(error instanceof Error ? error.message : "Không đọc được base prompt"));
    return () => {
      cancelled = true;
    };
  }, [open, setting, names]);

  function pick<T>(apply: (value: T) => void) {
    return (value: T) => {
      if (dirty && !window.confirm("Bỏ thay đổi chưa lưu?")) return;
      apply(value);
    };
  }

  async function run(action: () => Promise<BaseView>, done: string) {
    setBusy(true);
    try {
      const next = await action();
      setView(next);
      setDraft(undefined);
      setVersion((v) => v + 1);
      bump();
      toast.success(done);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được");
    } finally {
      setBusy(false);
    }
  }

  const placeholder = (text: string) => (
    <div role="status" className="grid min-h-[420px] place-items-center text-sm text-muted-foreground">
      {text}
    </div>
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex max-h-[92vh] w-[min(96vw,64rem)] flex-col">
        <DialogHeader>
          <DialogTitle>Prompt mặc định</DialogTitle>
          <DialogDescription>
            Bản dùng cho mọi truyện chưa có prompt riêng. Sửa nguyên văn theo từng tổ hợp bối cảnh × tên riêng.
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
              onChange={pick(setSetting)}
              disabled={busy}
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <Label>Tên riêng</Label>
            <Choice
              label="Tên riêng"
              value={names}
              options={GENRE_NAMES}
              labels={NAMES_CHOICE_LABELS}
              onChange={pick(setNames)}
              disabled={busy}
            />
          </div>
          <span className="rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground">
            {view ? (view.source === "file" ? "đã sửa" : "bản cứng") : "…"}
          </span>
        </div>
        <div className="fine-scrollbar min-h-0 flex-1 overflow-y-auto">
          {view?.text !== undefined ? (
            <Suspense fallback={placeholder("Đang tải editor…")}>
              <PlatePromptEditor key={version} initialValue={view.text} onChange={setDraft} />
            </Suspense>
          ) : (
            placeholder("Đang tải…")
          )}
        </div>
        <DialogFooter>
          <Button
            type="button"
            variant="ghost"
            disabled={busy || view?.source !== "file"}
            onClick={() => void run(() => baseReset("prompt", setting, names), "Đã về bản cứng")}
          >
            <RotateCcw /> Về mặc định
          </Button>
          <Button
            type="button"
            disabled={busy || !dirty}
            onClick={() => void run(() => baseSave("prompt", setting, names, { text: draft ?? "" }), "Đã lưu prompt mặc định")}
          >
            {busy && <LoaderCircle className="animate-spin" />} Lưu
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
