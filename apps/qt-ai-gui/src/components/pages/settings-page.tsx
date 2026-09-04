import { zodResolver } from "@hookform/resolvers/zod";
import { FolderSearch } from "lucide-react";
import { useEffect, type ComponentProps, type ReactNode } from "react";
import { useForm, type FieldPath, type FieldPathValue, type UseFormReturn } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { PalettePicker } from "@/components/palette-picker";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { useReadingWidth } from "@/hooks/use-reading-width";
import { useThemeActions } from "@/hooks/use-theme";
import { agyStatus, appConfigSet, pickAgyFile, saveSettings, storySnapshot } from "@/lib/api";
import { apiSettingsFromForm, engineFormFromConfig, engineFormSchema } from "@/lib/engine-form";
import { READING_WIDTH_LABELS, READING_WIDTHS } from "@/lib/reading";
import { DEFAULT_API_MODELS, OPENAI_REASONING_EFFORTS } from "@/lib/schema";
import { THEME_MODE_LABELS, THEME_MODES } from "@/lib/theme";
import { API_PROVIDER_LABELS, ENGINE_LABELS, type ApiProvider, type Engine } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

const settingsFormSchema = engineFormSchema.extend({
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

/** Ô chọn kiểu radio giống chọn chế độ sáng/tối. */
function Choice<T extends string>({
  label,
  value,
  options,
  labels,
  onChange,
  disabled,
}: {
  label: string;
  value: T;
  options: readonly T[];
  labels: Record<T, string>;
  onChange: (value: T) => void;
  disabled?: boolean;
}) {
  return (
    <div role="radiogroup" aria-label={label} className="inline-flex w-fit rounded-md border p-0.5">
      {options.map((option) => (
        <button
          key={option}
          type="button"
          role="radio"
          aria-checked={value === option}
          disabled={disabled}
          onClick={() => onChange(option)}
          className={cn(
            "rounded-[calc(var(--radius)-2px)] px-3 py-1.5 text-sm disabled:opacity-50",
            value === option ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:bg-accent",
          )}
        >
          {labels[option]}
        </button>
      ))}
    </div>
  );
}

const ENGINES: readonly Engine[] = ["agy", "api"];
const API_PROVIDERS: readonly ApiProvider[] = ["gemini", "openai"];

/**
 * Card "Động cơ dịch": chọn agy hay API key; API thì key/model/base URL theo provider đang chọn
 * (provider kia vẫn giữ giá trị trong form, đổi qua lại không mất key).
 */
function EngineCard({ form, running }: { form: UseFormReturn<SettingsForm>; running: boolean }) {
  const engine = form.watch("engine");
  const provider = form.watch("apiProvider");
  const thinking = form.watch("thinking");
  const reasoningEffort = form.watch("reasoningEffort");
  const set = <K extends FieldPath<SettingsForm>>(name: K, value: FieldPathValue<SettingsForm, K>) =>
    form.setValue(name, value, { shouldDirty: true });
  const text = (
    name: "geminiApiKey" | "geminiModel" | "geminiBaseUrl" | "openaiApiKey" | "openaiModel" | "openaiBaseUrl",
    label: string,
    hint: string,
    props: ComponentProps<"input"> = {},
  ) => (
    <div className="flex flex-col gap-1.5">
      <Label htmlFor={name}>{label}</Label>
      <Input id={name} autoComplete="off" spellCheck={false} className="font-mono text-xs" {...props} {...form.register(name)} />
      <p className="text-xs text-muted-foreground">{hint}</p>
    </div>
  );
  return (
    <Card
      title="Động cơ dịch"
      description="agy dùng quota Antigravity qua agent; API key gọi thẳng model bằng key của bạn, không cần cài agy."
    >
      <Choice label="Động cơ dịch" value={engine} options={ENGINES} labels={ENGINE_LABELS} onChange={(v) => set("engine", v)} disabled={running} />
      {engine === "api" && (
        <div className="flex flex-col gap-4 rounded-md border bg-muted/25 p-4">
          <div className="flex flex-col gap-1.5">
            <Label>Nhà cung cấp</Label>
            <Choice
              label="Nhà cung cấp API"
              value={provider}
              options={API_PROVIDERS}
              labels={API_PROVIDER_LABELS}
              onChange={(v) => set("apiProvider", v)}
              disabled={running}
            />
          </div>
          {provider === "gemini" ? (
            <>
              {text("geminiApiKey", "API key Google AI", "Chỉ lưu trong config.json của app trên máy này.", { type: "password" })}
              {text("geminiModel", "Model", `Trống = ${DEFAULT_API_MODELS.gemini}.`, { placeholder: DEFAULT_API_MODELS.gemini })}
              {text("geminiBaseUrl", "Base URL", "Trống = endpoint chính thức của Google; điền khi dùng relay Gemini.", {
                placeholder: "https://generativelanguage.googleapis.com",
              })}
              <div className="flex items-center justify-between rounded-md bg-background/60 px-3 py-2">
                <Label htmlFor="thinking" className="text-xs font-normal">
                  Thinking (Gemini 3.x: high ↔ minimal)
                </Label>
                <Switch id="thinking" checked={thinking} onCheckedChange={(v) => set("thinking", v)} disabled={running} />
              </div>
            </>
          ) : (
            <>
              {text("openaiApiKey", "API key", "Key OpenAI hoặc key của hub tương thích.", { type: "password" })}
              {text("openaiModel", "Model", `Trống = ${DEFAULT_API_MODELS.openai}; dùng hub thì gõ model của hub (vd gemini-3.7-flash).`, {
                placeholder: DEFAULT_API_MODELS.openai,
              })}
              {text("openaiBaseUrl", "Base URL", "Trống = https://api.openai.com/v1; hub riêng thì điền tới hết /v1.", {
                placeholder: "https://api.openai.com/v1",
              })}
              <div className="flex items-center justify-between gap-2">
                <Label htmlFor="reasoningEffort" className="text-xs font-normal">
                  Mức nghĩ (reasoning_effort)
                </Label>
                <Select value={reasoningEffort} onValueChange={(v) => set("reasoningEffort", v as SettingsForm["reasoningEffort"])} disabled={running}>
                  <SelectTrigger id="reasoningEffort" className="w-40" aria-label="Mức reasoning OpenAI">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {OPENAI_REASONING_EFFORTS.map((effort) => (
                      <SelectItem key={effort} value={effort}>
                        {effort}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </>
          )}
        </div>
      )}
    </Card>
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
  const reading = useReadingWidth();
  const form = useForm<SettingsForm>({ resolver: zodResolver(settingsFormSchema) });

  useEffect(() => {
    if (config && settings) {
      form.reset({
        ...engineFormFromConfig(config),
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
        engine: values.engine,
        api: apiSettingsFromForm(values),
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
          <div className="flex flex-col gap-1.5">
            <Label>Chiều ngang văn bản đọc</Label>
            <Choice
              label="Chiều ngang văn bản đọc"
              value={reading.width}
              options={READING_WIDTHS}
              labels={READING_WIDTH_LABELS}
              onChange={(v) => void reading.setWidth(v)}
            />
            <p className="text-xs text-muted-foreground">Cũng đổi được ngay trên thanh tab của trang đọc.</p>
          </div>
        </Card>
        <form onSubmit={(e) => void submit(e)} className="flex flex-col gap-6">
          <EngineCard form={form} running={running} />
          <Card title="App" description="Antigravity CLI và giới hạn phiên (chỉ dùng khi động cơ là agy).">
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
