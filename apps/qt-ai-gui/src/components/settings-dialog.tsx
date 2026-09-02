import { zodResolver } from "@hookform/resolvers/zod";
import { FolderSearch } from "lucide-react";
import { useEffect, type ComponentProps } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { agyStatus, appConfigSet, pickAgyFile, saveSettings, storySnapshot } from "@/lib/api";
import { useStoryStore } from "@/store/story";

const settingsFormSchema = z.object({
  agyPath: z.string(),
  model: z.string(),
  maxSessions: z.number().int().min(1).max(1000),
  chaptersPerSession: z.number().int().min(1).max(100),
  maxReviewRounds: z.number().int().min(0).max(10),
  minLengthRatio: z.number().min(0.1).max(3),
});
type SettingsForm = z.infer<typeof settingsFormSchema>;
const NUMERIC_FIELDS = new Set<keyof SettingsForm>([
  "maxSessions",
  "chaptersPerSession",
  "maxReviewRounds",
  "minLengthRatio",
]);

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function SettingsDialog({ open, onOpenChange }: Props) {
  const root = useStoryStore((s) => s.root);
  const config = useStoryStore((s) => s.config);
  const settings = useStoryStore((s) => s.snapshot?.settings);
  const running = useStoryStore((s) => s.session.status === "running");
  const setConfig = useStoryStore((s) => s.setConfig);
  const setAgy = useStoryStore((s) => s.setAgy);
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const form = useForm<SettingsForm>({ resolver: zodResolver(settingsFormSchema) });

  useEffect(() => {
    if (open && config && settings) {
      form.reset({
        agyPath: config.agyPath ?? "",
        model: config.model ?? "",
        maxSessions: config.maxSessions,
        chaptersPerSession: settings.chaptersPerSession,
        maxReviewRounds: settings.maxReviewRounds,
        minLengthRatio: settings.minLengthRatio,
      });
    }
  }, [open, config, settings, form]);

  const submit = form.handleSubmit(async (values) => {
    if (!config || !root) return;
    try {
      const nextConfig = await appConfigSet({
        ...config,
        agyPath: values.agyPath.trim() || null,
        model: values.model.trim() || null,
        maxSessions: values.maxSessions,
      });
      setConfig(nextConfig);
      setAgy(await agyStatus(nextConfig.agyPath ?? undefined));
      await saveSettings(root, {
        chaptersPerSession: values.chaptersPerSession,
        maxReviewRounds: values.maxReviewRounds,
        minLengthRatio: values.minLengthRatio,
      });
      setSnapshot(await storySnapshot(root));
      toast.success("Đã lưu cài đặt");
      onOpenChange(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được cài đặt");
    }
  });

  async function pickAgy() {
    const path = await pickAgyFile();
    if (path) form.setValue("agyPath", path, { shouldDirty: true });
  }

  const field = (name: keyof SettingsForm, label: string, hint: string, props: ComponentProps<"input"> = {}) => (
    <div>
      <Label htmlFor={name}>{label}</Label>
      <Input
        id={name}
        {...props}
        {...form.register(name, { valueAsNumber: NUMERIC_FIELDS.has(name) })}
        aria-invalid={Boolean(form.formState.errors[name])}
      />
      <p className="text-xs text-muted-foreground">{form.formState.errors[name]?.message ?? hint}</p>
    </div>
  );

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Cài đặt</DialogTitle>
          <DialogDescription>
            Phần "App" dùng chung mọi truyện; phần "Truyện này" ghi vào state.json của truyện đang mở.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={(e) => void submit(e)} className="flex flex-col gap-4">
          <section className="flex flex-col gap-3">
            <h3 className="text-sm font-semibold">App</h3>
            <div>
              <Label htmlFor="agyPath">Đường dẫn agy</Label>
              <div className="flex gap-1">
                <Input id="agyPath" {...form.register("agyPath")} placeholder="Trống = tự tìm trong PATH" />
                <Button
                  type="button"
                  variant="outline"
                  size="icon"
                  aria-label="Chọn file agy"
                  onClick={() => void pickAgy()}
                >
                  <FolderSearch />
                </Button>
              </div>
            </div>
            {field(
              "model",
              "Model mặc định",
              "Trống = model mặc định của agy; danh sách xem ở dropdown trên bàn dịch.",
            )}
            {field("maxSessions", "Số phiên tối đa mỗi lần Bắt đầu", "Cầu dao chống chạy vô hạn; mặc định 50.", {
              type: "number",
              min: 1,
              max: 1000,
            })}
          </section>
          <section className="flex flex-col gap-3">
            <h3 className="text-sm font-semibold">Truyện này</h3>
            {field(
              "chaptersPerSession",
              "Chương / phiên",
              "Agent dừng sau số chương này để giữ context sạch; mặc định 10.",
              { type: "number", min: 1, max: 100 },
            )}
            {field(
              "maxReviewRounds",
              "Số vòng soát tối đa",
              "Hết vòng mà chỉ còn vi phạm rule thì chốt kèm cảnh báo; mặc định 3.",
              { type: "number", min: 0, max: 10 },
            )}
            {field(
              "minLengthRatio",
              "Tỉ lệ ký tự dịch/raw tối thiểu",
              "Dưới ngưỡng coi là dịch thiếu; mặc định 0.75.",
              { type: "number", step: 0.05, min: 0.1, max: 3 },
            )}
          </section>
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Đóng
            </Button>
            <Button type="submit" disabled={running}>
              Lưu
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
