import { zodResolver } from "@hookform/resolvers/zod";
import { BookA, FileText, FolderSearch, ListChecks } from "lucide-react";
import { useEffect, useState, type ComponentProps, type ReactNode } from "react";
import { useForm, type FieldPath, type FieldPathValue, type UseFormReturn } from "react-hook-form";
import { toast } from "sonner";
import { z } from "zod";

import { BaseGlossaryDialog } from "@/components/base-glossary-dialog";
import { BasePromptDialog } from "@/components/base-prompt-dialog";
import { BaseRulesDialog } from "@/components/base-rules-dialog";
import { Choice } from "@/components/choice";
import { EffortTable } from "@/components/effort-table";
import { PalettePicker } from "@/components/palette-picker";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useReadingWidth } from "@/hooks/use-reading-width";
import { useThemeActions } from "@/hooks/use-theme";
import { agyStatus, appConfigSet, pickAgyFile, pickFolder, saveSettings, storySnapshot } from "@/lib/api";
import { apiSettingsFromForm, engineFormFromConfig, engineFormSchema } from "@/lib/engine-form";
import { READING_WIDTH_LABELS, READING_WIDTHS } from "@/lib/reading";
import { DEFAULT_API_MODELS, GEMINI_EFFORTS, OPENAI_EFFORTS } from "@/lib/schema";
import { THEME_MODE_LABELS, THEME_MODES } from "@/lib/theme";
import { API_PROVIDER_LABELS, ENGINE_LABELS, type ApiProvider, type BaseKind, type Engine } from "@/lib/types";
import { cn } from "@/lib/utils";
import { selectCurrentRunning, useStoryStore } from "@/store/story";

const settingsFormSchema = engineFormSchema.extend({
  agyPath: z.string(),
  model: z.string(),
  maxSessions: z.number().int().min(1).max(1000),
  maxParallel: z.number().int().min(1).max(20),
  chaptersPerSession: z.number().int().min(1).max(100),
  maxReviewRounds: z.number().int().min(0).max(10),
  minLengthRatio: z.number().min(0.1).max(3),
});
type SettingsForm = z.infer<typeof settingsFormSchema>;
const NUMERIC = new Set<keyof SettingsForm>([
  "maxSessions",
  "maxParallel",
  "chaptersPerSession",
  "maxReviewRounds",
  "minLengthRatio",
]);

function Card({ title, description, children }: { title: string; description: string; children: ReactNode }) {
  return (
    <section className="rounded-lg border bg-card p-5">
      <h2 className="text-base font-semibold">{title}</h2>
      <p className="mb-4 text-sm text-muted-foreground">{description}</p>
      <div className="flex flex-col gap-4">{children}</div>
    </section>
  );
}

const ENGINES: readonly Engine[] = ["agy", "api"];
const API_PROVIDERS: readonly ApiProvider[] = ["gemini", "openai"];

const FORM_ID = "settings-form";

/**
 * Footer cố định dưới khung cuộn (ngoài form, nút Lưu nối vào form qua thuộc tính form=): người dùng
 * sửa ô nào cũng thấy nút Lưu ngay trước mắt thay vì phải cuộn xuống cuối trang. Chưa sửa gì thì nút mờ.
 */
function SaveBar({ dirty, running, saving, onReset }: { dirty: boolean; running: boolean; saving: boolean; onReset: () => void }) {
  const status = !dirty
    ? "Giao diện và Thư viện tự lưu; ba mục dưới sửa xong bấm Lưu."
    : running
      ? "Có thay đổi chưa lưu. Dừng dịch rồi mới lưu được."
      : "Có thay đổi chưa lưu.";
  return (
    <footer data-testid="save-bar" className="shrink-0 border-t bg-background px-8 py-3">
      <div className="mx-auto flex max-w-3xl items-center justify-between gap-3">
        <p className={cn("text-sm", dirty ? "font-medium text-foreground" : "text-muted-foreground")}>{status}</p>
        <div className="flex shrink-0 gap-2">
          <Button type="button" variant="ghost" size="sm" onClick={onReset} disabled={!dirty || saving}>
            Hoàn tác
          </Button>
          <Button type="submit" form={FORM_ID} size="sm" disabled={!dirty || running || saving}>
            Lưu App + Truyện này
          </Button>
        </div>
      </div>
    </footer>
  );
}

/**
 * Card "Động cơ dịch": chọn agy hay API key; API thì key/model/base URL theo provider đang chọn
 * (provider kia vẫn giữ giá trị trong form, đổi qua lại không mất key).
 */
function EngineCard({ form, running }: { form: UseFormReturn<SettingsForm>; running: boolean }) {
  const engine = form.watch("engine");
  const provider = form.watch("apiProvider");
  const geminiEffort = form.watch("geminiEffort");
  const openaiEffort = form.watch("openaiEffort");
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
              <EffortTable
                idPrefix="gemini"
                values={geminiEffort}
                options={GEMINI_EFFORTS}
                hint="Gemini 3.x: thinkingLevel; 2.5: minimal = tắt, mức khác = tự động. Mặc định model = không gửi tham số."
                disabled={running}
                onChange={(step, v) => set(`geminiEffort.${step}`, v as (typeof GEMINI_EFFORTS)[number])}
              />
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
              <EffortTable
                idPrefix="openai"
                values={openaiEffort}
                options={OPENAI_EFFORTS}
                hint="Gửi reasoning_effort cho từng bước. Mặc định model = không gửi tham số; hub lạ không nhận thì chọn mục này."
                disabled={running}
                onChange={(step, v) => set(`openaiEffort.${step}`, v as (typeof OPENAI_EFFORTS)[number])}
              />
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
  const running = useStoryStore(selectCurrentRunning);
  const setConfig = useStoryStore((s) => s.setConfig);
  const setAgy = useStoryStore((s) => s.setAgy);
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const theme = useThemeActions();
  const reading = useReadingWidth();
  const form = useForm<SettingsForm>({ resolver: zodResolver(settingsFormSchema) });
  const [baseDialog, setBaseDialog] = useState<BaseKind | undefined>();

  useEffect(() => {
    if (config && settings) {
      form.reset({
        ...engineFormFromConfig(config),
        agyPath: config.agyPath ?? "",
        model: config.model ?? "",
        maxSessions: config.maxSessions,
        maxParallel: config.maxParallel,
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
        maxParallel: values.maxParallel,
      });
      setConfig(next);
      if (next.engine === "agy") setAgy(await agyStatus(next.agyPath ?? undefined));
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

  /** Thư viện lưu ngay (không qua form) như theme, để picker thấy tức thì. */
  async function setLibrary(next: string | null) {
    if (!config) return;
    try {
      setConfig(await appConfigSet({ ...config, libraryRoot: next }));
      toast.success(next ? "Đã đổi thư viện" : "Đã bỏ thư viện");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được cấu hình");
    }
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
    <div className="flex h-full flex-col">
      <div className="fine-scrollbar min-h-0 flex-1 overflow-y-auto">
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
          <Card title="Thư viện" description="Folder cha chứa mọi truyện: Tạo truyện mới ghi vào đây, màn chọn truyện liệt kê con trực tiếp.">
            <div className="flex items-center gap-2">
              <p className="min-w-0 flex-1 truncate font-mono text-xs" title={config?.libraryRoot ?? undefined}>
                {config?.libraryRoot ?? <span className="text-muted-foreground">Chưa chọn</span>}
              </p>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => {
                  void pickFolder("Chọn thư viện (folder cha chứa truyện)").then((root) => {
                    if (root) void setLibrary(root);
                  });
                }}
              >
                <FolderSearch /> Chọn
              </Button>
              {config?.libraryRoot && (
                <Button type="button" variant="ghost" size="sm" onClick={() => void setLibrary(null)}>
                  Bỏ
                </Button>
              )}
            </div>
          </Card>
          <Card
            title="Bản mặc định"
            description="Prompt, rule và glossary dùng cho mọi truyện chưa có bản riêng. Sửa ở đây là mọi truyện đang dùng mặc định ăn theo, kể cả phiên agy."
          >
            <div className="flex flex-wrap gap-2">
              <Button type="button" variant="outline" size="sm" onClick={() => setBaseDialog("prompt")}>
                <FileText /> Prompt mặc định
              </Button>
              <Button type="button" variant="outline" size="sm" onClick={() => setBaseDialog("rules")}>
                <ListChecks /> Rule mặc định
              </Button>
              <Button type="button" variant="outline" size="sm" onClick={() => setBaseDialog("glossary")}>
                <BookA /> Glossary chung
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              File nằm trong thư mục cấu hình app, mục <code>base/</code>. Về mặc định = xoá file.
            </p>
          </Card>
          <form id={FORM_ID} onSubmit={(e) => void submit(e)} className="flex flex-col gap-6">
            <EngineCard form={form} running={running} />
            <Card title="App" description="Giới hạn phiên chung; đường dẫn agy và model chỉ dùng khi động cơ là agy.">
              {field("maxParallel", "Số truyện dịch song song", "Mỗi truyện một phiên, tối đa 20; API hub dễ trả 429 nếu để cao thì hạ xuống. Mặc định 20.", {
                type: "number",
                min: 1,
                max: 20,
              })}
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
          </form>
        </div>
      </div>
      <SaveBar dirty={form.formState.isDirty} running={running} saving={form.formState.isSubmitting} onReset={() => form.reset()} />
      <BasePromptDialog open={baseDialog === "prompt"} onOpenChange={(o) => setBaseDialog(o ? "prompt" : undefined)} />
      <BaseRulesDialog open={baseDialog === "rules"} onOpenChange={(o) => setBaseDialog(o ? "rules" : undefined)} />
      <BaseGlossaryDialog open={baseDialog === "glossary"} onOpenChange={(o) => setBaseDialog(o ? "glossary" : undefined)} />
    </div>
  );
}
