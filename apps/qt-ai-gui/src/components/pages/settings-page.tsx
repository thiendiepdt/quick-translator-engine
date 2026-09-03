import { zodResolver } from "@hookform/resolvers/zod";
import { FolderSearch } from "lucide-react";
import { useEffect, type ComponentProps, type ReactNode } from "react";
import { useForm } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { PalettePicker } from "@/components/palette-picker";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useThemeActions } from "@/hooks/use-theme";
import { agyStatus, appConfigSet, pickAgyFile, saveSettings, storySnapshot } from "@/lib/api";
import { THEME_MODE_LABELS, THEME_MODES } from "@/lib/theme";
import { cn } from "@/lib/utils";
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
const NUMERIC = new Set<keyof SettingsForm>(["maxSessions", "chaptersPerSession", "maxReviewRounds", "minLengthRatio"]);

function Card({ title, description, children }: { title: string; description: string; children: ReactNode }) {
  return (
    <section className="rounded-lg border bg-card p-5">
      <h2 className="text-base font-semibold">{title}</h2>
      <p className="mb-4 text-sm text-muted-foreground">{description}</p>
      <div className="flex flex-col gap-4">{children}</div>
    </section>
  );
}

export function SettingsPage() {
  const root = useStoryStore((s) => s.root);
  const config = useStoryStore((s) => s.config);
  const settings = useStoryStore((s) => s.snapshot?.settings);
  const agy = useStoryStore((s) => s.agy);
  const running = useStoryStore((s) => s.session.status === "running");
  const setConfig = useStoryStore((s) => s.setConfig);
  const setAgy = useStoryStore((s) => s.setAgy);
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const theme = useThemeActions();
  const form = useForm<SettingsForm>({ resolver: zodResolver(settingsFormSchema) });

  useEffect(() => {
    if (config && settings) {
      form.reset({
        agyPath: config.agyPath ?? "",
        model: config.model ?? "",
        maxSessions: config.maxSessions,
        chaptersPerSession: settings.chaptersPerSession,
        maxReviewRounds: settings.maxReviewRounds,
        minLengthRatio: settings.minLengthRatio,
      });
    }
  }, [config, settings, form]);

  const submit = form.handleSubmit(async (values) => {
    if (!config || !root) return;
    try {
      const next = await appConfigSet({
        ...config,
        agyPath: values.agyPath.trim() || null,
        model: values.model.trim() || null,
        maxSessions: values.maxSessions,
      });
      setConfig(next);
      setAgy(await agyStatus(next.agyPath ?? undefined));
      await saveSettings(root, {
        chaptersPerSession: values.chaptersPerSession,
        maxReviewRounds: values.maxReviewRounds,
        minLengthRatio: values.minLengthRatio,
      });
      setSnapshot(await storySnapshot(root));
      toast.success("Đã lưu cài đặt");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được cài đặt");
    }
  });

  async function pickAgy() {
    const path = await pickAgyFile();
    if (path) form.setValue("agyPath", path, { shouldDirty: true });
  }

  const field = (name: keyof SettingsForm, label: string, hint: string, props: ComponentProps<"input"> = {}) => (
    <div className="flex flex-col gap-1.5">
      <Label htmlFor={name}>{label}</Label>
      <Input
        id={name}
        {...props}
        {...form.register(name, { valueAsNumber: NUMERIC.has(name) })}
        aria-invalid={Boolean(form.formState.errors[name])}
      />
      <p className={cn("text-xs", form.formState.errors[name] ? "text-destructive" : "text-muted-foreground")}>
        {form.formState.errors[name]?.message ?? hint}
      </p>
    </div>
  );

  return (
    <div className="fine-scrollbar h-full overflow-y-auto">
      <div className="mx-auto flex max-w-3xl flex-col gap-6 px-8 py-8">
        <header>
          <h1 className="text-2xl font-semibold tracking-tight">Cài đặt</h1>
          <p className="text-sm text-muted-foreground">
            Giao diện và App dùng chung mọi truyện; "Truyện này" ghi vào state.json của truyện đang mở.
          </p>
        </header>
        <Card title="Giao diện" description="Áp dụng ngay, lưu vào cấu hình app.">
          <PalettePicker value={theme.palette} onChange={(p) => void theme.setPalette(p)} />
          <div className="flex flex-col gap-1.5">
            <Label>Chế độ</Label>
            <div role="radiogroup" aria-label="Chế độ sáng tối" className="inline-flex w-fit rounded-md border p-0.5">
              {THEME_MODES.map((mode) => (
                <button
                  key={mode}
                  type="button"
                  role="radio"
                  aria-checked={theme.mode === mode}
                  onClick={() => void theme.setMode(mode)}
                  className={cn(
                    "rounded-[calc(var(--radius)-2px)] px-3 py-1.5 text-sm",
                    theme.mode === mode ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:bg-accent",
                  )}
                >
                  {THEME_MODE_LABELS[mode]}
                </button>
              ))}
            </div>
          </div>
        </Card>
        <form onSubmit={(e) => void submit(e)} className="flex flex-col gap-6">
          <Card title="App" description="Antigravity CLI và giới hạn phiên.">
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="agyPath">Đường dẫn agy</Label>
              <div className="flex gap-1">
                <Input id="agyPath" {...form.register("agyPath")} placeholder="Trống = tự tìm trong PATH" />
                <Button type="button" variant="outline" size="icon" aria-label="Chọn file agy" onClick={() => void pickAgy()}>
                  <FolderSearch />
                </Button>
              </div>
              <p className="text-xs text-muted-foreground">
                {agy?.found ? `Đang dùng: ${agy.path}${agy.version ? ` (${agy.version})` : ""}` : "Chưa tìm thấy agy."}
              </p>
            </div>
            {field("model", "Model mặc định", "Trống = model mặc định của agy; danh sách ở dropdown trang Dịch.")}
            {field("maxSessions", "Số phiên tối đa mỗi lần Bắt đầu", "Cầu dao chống chạy vô hạn; mặc định 50.", {
              type: "number",
              min: 1,
              max: 1000,
            })}
          </Card>
          <Card title="Truyện này" description="Ghi vào state.json của truyện đang mở.">
            {field("chaptersPerSession", "Chương / phiên", "Agent dừng sau số chương này để giữ context sạch; mặc định 10.", {
              type: "number",
              min: 1,
              max: 100,
            })}
            {field("maxReviewRounds", "Số vòng soát tối đa", "Hết vòng mà chỉ còn vi phạm rule thì chốt kèm cảnh báo; mặc định 3.", {
              type: "number",
              min: 0,
              max: 10,
            })}
            {field("minLengthRatio", "Tỉ lệ ký tự dịch/raw tối thiểu", "Dưới ngưỡng coi là dịch thiếu; mặc định 0.75.", {
              type: "number",
              step: 0.05,
              min: 0.1,
              max: 3,
            })}
          </Card>
          <div className="flex justify-end">
            <Button type="submit" disabled={running || !form.formState.isDirty}>
              Lưu App + Truyện này
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
