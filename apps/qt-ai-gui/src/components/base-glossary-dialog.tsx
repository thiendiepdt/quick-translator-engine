import { LoaderCircle, RotateCcw } from "lucide-react";
import { useEffect, useState } from "react";
import { FormProvider, useForm } from "react-hook-form";
import { toast } from "sonner";

import { Choice } from "@/components/choice";
import { GlossaryEditor } from "@/components/glossary-editor";
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
import { GENRE_SETTINGS } from "@/lib/schema";
import { glossaryToPairs, pairsToGlossary, type StoryFormValues } from "@/lib/story-form";
import { GENRE_SETTING_LABELS, GLOSSARY_KEYS, GLOSSARY_LABELS, type BaseView, type GenreSetting } from "@/lib/types";
import { useStoryStore } from "@/store/story";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/**
 * Kho glossary chung theo bối cảnh: nền cho mọi truyện cùng bối cảnh (truyện đè key trùng, vẫn lọc theo
 * chương). Dùng lại GlossaryEditor của truyện qua form RHF riêng chỉ đụng tới field `glossary`.
 */
export function BaseGlossaryDialog({ open, onOpenChange }: Props) {
  const bump = useStoryStore((s) => s.bumpBaseVersion);
  const [setting, setSetting] = useState<GenreSetting>("ancient");
  const [loaded, setLoaded] = useState<{ setting: GenreSetting; view: BaseView } | undefined>();
  const [busy, setBusy] = useState(false);
  const form = useForm<StoryFormValues>({ defaultValues: { glossary: glossaryToPairs(undefined) } });
  const dirty = form.formState.isDirty;
  const view = loaded?.setting === setting ? loaded.view : undefined;

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    baseGet("glossary", setting, undefined)
      .then((next) => {
        if (cancelled) return;
        setLoaded({ setting, view: next });
        form.reset({ glossary: glossaryToPairs(next.glossary) });
      })
      .catch((error: unknown) => toast.error(error instanceof Error ? error.message : "Không đọc được glossary chung"));
    return () => {
      cancelled = true;
    };
  }, [open, setting, form]);

  async function run(action: () => Promise<BaseView>, done: string) {
    setBusy(true);
    try {
      const next = await action();
      setLoaded({ setting, view: next });
      form.reset({ glossary: glossaryToPairs(next.glossary) });
      bump();
      toast.success(done);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được");
    } finally {
      setBusy(false);
    }
  }

  const save = () => {
    const glossary = pairsToGlossary(form.getValues("glossary"));
    void run(() => baseSave("glossary", setting, undefined, { glossary }), "Đã lưu glossary chung");
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex max-h-[92vh] w-[min(96vw,64rem)] flex-col">
        <DialogHeader>
          <DialogTitle>Glossary chung</DialogTitle>
          <DialogDescription>
            Nền cho mọi truyện cùng bối cảnh; glossary riêng của truyện đè mục trùng. Chỉ mục có mặt trong chương mới
            vào prompt.
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
            <p className="text-xs text-muted-foreground">{GENRE_SETTING_LABELS[setting].hint}</p>
          </div>
          <span className="rounded-full bg-muted px-2 py-0.5 text-xs text-muted-foreground">
            {view ? (view.source === "file" ? "đã sửa" : "bản cứng") : "…"}
          </span>
        </div>
        <FormProvider {...form}>
          <div className="fine-scrollbar flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto">
            {view ? (
              GLOSSARY_KEYS.map((key) => (
                <GlossaryEditor key={`${setting}-${key}`} name={`glossary.${key}`} label={GLOSSARY_LABELS[key]} />
              ))
            ) : (
              <div role="status" className="grid min-h-[200px] place-items-center text-sm text-muted-foreground">
                Đang tải…
              </div>
            )}
          </div>
        </FormProvider>
        <DialogFooter>
          <Button
            type="button"
            variant="ghost"
            disabled={busy || view?.source !== "file"}
            onClick={() => void run(() => baseReset("glossary", setting, undefined), "Đã về bản cứng")}
          >
            <RotateCcw /> Về mặc định
          </Button>
          <Button type="button" disabled={busy || !dirty} onClick={save}>
            {busy && <LoaderCircle className="animate-spin" />} Lưu
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
