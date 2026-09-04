import { zodResolver } from "@hookform/resolvers/zod";
import { Download, Save, Sparkles, Upload } from "lucide-react";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { FormProvider, useForm, useWatch } from "react-hook-form";
import { toast } from "sonner";

import { AiFillDialog } from "@/components/ai-fill-dialog";
import { CheckRulesEditor } from "@/components/check-rules-editor";
import { GlossaryEditor } from "@/components/glossary-editor";
import { PromptEditor } from "@/components/prompt-editor";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { useStoryDefaults } from "@/hooks/use-story-defaults";
import { saveStory, storySnapshot } from "@/lib/api";
import { storyConfigSchema } from "@/lib/schema";
import { fromFormValues, storyFormSchema, toFormValues, type StoryFormValues } from "@/lib/story-form";
import { GLOSSARY_KEYS, GLOSSARY_LABELS, type StoryConfig } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

const SECTIONS = [
  { id: "info", label: "Thông tin" },
  { id: "style", label: "Style" },
  { id: "glossary", label: "Glossary" },
  { id: "rules", label: "Rule kiểm tra" },
  { id: "prompt", label: "Prompt" },
] as const;
type SectionId = (typeof SECTIONS)[number]["id"];

function Field({ id, label, hint, children }: { id: string; label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="flex flex-col gap-1.5">
      <Label htmlFor={id}>{label}</Label>
      {children}
      {hint && <p className="text-xs text-muted-foreground">{hint}</p>}
    </div>
  );
}

/** Mỗi mục là một tab: chỉ mục đang chọn được render — glossary + rule + prompt cùng lúc là quá dài và lag. */
function Section({ id, active, title, children }: { id: SectionId; active: SectionId; title: string; children: ReactNode }) {
  if (id !== active) return null;
  return (
    <section id={`sec-${id}`} role="tabpanel" aria-label={title}>
      <h2 className="mb-3 text-lg font-semibold tracking-tight">{title}</h2>
      <div className="flex flex-col gap-4">{children}</div>
    </section>
  );
}

export function StoryPage() {
  const root = useStoryStore((s) => s.root);
  const story = useStoryStore((s) => s.snapshot?.story);
  const running = useStoryStore((s) => s.session.status === "running");
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const form = useForm<StoryFormValues>({
    resolver: zodResolver(storyFormSchema),
    defaultValues: story ? toFormValues(story) : undefined,
  });
  const autoGlossary = useWatch({ control: form.control, name: "autoGlossary" });
  const defaults = useStoryDefaults();
  const [fillOpen, setFillOpen] = useState(false);
  const [active, setActive] = useState<SectionId>("info");
  const fileInput = useRef<HTMLInputElement>(null);
  const scroller = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (story) form.reset(toFormValues(story));
  }, [story, form]);

  if (!root || !story) return null;
  const currentRoot = root;
  const currentStory = story;

  async function persist(config: StoryConfig) {
    await saveStory(currentRoot, config);
    setSnapshot(await storySnapshot(currentRoot));
    toast.success("Đã lưu hồ sơ truyện");
  }

  const submit = form.handleSubmit(async (values) => {
    try {
      await persist(fromFormValues(values, currentStory));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được");
    }
  });

  function exportJson() {
    const config = fromFormValues(form.getValues(), currentStory);
    const blob = new Blob([`${JSON.stringify(config, null, 2)}\n`], { type: "application/json" });
    const link = document.createElement("a");
    link.href = URL.createObjectURL(blob);
    link.download = `${config.name || "story"}.story.json`;
    link.click();
    URL.revokeObjectURL(link.href);
  }

  async function importJson(file: File) {
    try {
      const parsed = storyConfigSchema.parse(JSON.parse(await file.text()));
      form.reset(toFormValues({ ...parsed, autoGlossaryLog: currentStory.autoGlossaryLog }), {
        keepDefaultValues: true,
      });
      toast.message("Đã nạp JSON vào form — bấm Lưu để ghi");
    } catch {
      toast.error("File không đúng schema story.json của qt-web");
    }
  }

  function jumpTo(id: SectionId) {
    setActive(id);
    if (scroller.current) scroller.current.scrollTop = 0;
  }

  const dirty = form.formState.isDirty;

  return (
    <FormProvider {...form}>
      <form onSubmit={(e) => void submit(e)} className="flex h-full flex-col">
        <div className="grid min-h-0 flex-1 grid-cols-[200px_1fr]">
          <nav className="border-r bg-card/50 p-4">
            <p className="mb-3 text-xs font-medium tracking-widest text-muted-foreground uppercase">Hồ sơ truyện</p>
            <ul className="flex flex-col gap-0.5" role="tablist" aria-label="Mục hồ sơ truyện">
              {SECTIONS.map((section) => (
                <li key={section.id}>
                  <button
                    type="button"
                    role="tab"
                    aria-selected={active === section.id}
                    onClick={() => jumpTo(section.id)}
                    className={cn(
                      "w-full rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent",
                      active === section.id && "bg-accent font-medium",
                    )}
                  >
                    {section.label}
                  </button>
                </li>
              ))}
            </ul>
          </nav>
          <div ref={scroller} className="fine-scrollbar min-h-0 overflow-y-auto">
            <div className="mx-auto flex max-w-3xl flex-col gap-10 px-8 py-8">
              {/* Phần thông tin form ở mục khác vẫn nằm trong react-hook-form dù không render. */}
              <Section id="info" active={active} title="Thông tin">
                <div className="grid grid-cols-2 gap-4">
                  <Field id="name" label="Tên truyện">
                    <Input id="name" {...form.register("name")} />
                  </Field>
                  <Field id="sourceUrl" label="Link nguồn">
                    <Input id="sourceUrl" {...form.register("sourceUrl")} placeholder="https://…" />
                  </Field>
                  <Field id="protagonist" label="Nhân vật chính" hint="Tên Hán-Việt dùng xuyên suốt.">
                    <Input id="protagonist" {...form.register("protagonist")} />
                  </Field>
                  <Field id="autoGlossary" label="Tự thêm tên riêng">
                    <Select
                      value={autoGlossary}
                      onValueChange={(v) =>
                        form.setValue("autoGlossary", v as StoryFormValues["autoGlossary"], { shouldDirty: true })
                      }
                    >
                      <SelectTrigger id="autoGlossary">
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="inherit">Theo mặc định (bật)</SelectItem>
                        <SelectItem value="on">Bật</SelectItem>
                        <SelectItem value="off">Tắt</SelectItem>
                      </SelectContent>
                    </Select>
                  </Field>
                </div>
                <Field
                  id="summary"
                  label="Tóm tắt"
                  hint="Bối cảnh, tuyến nhân vật, mạch truyện chính — agent đọc để dịch nhất quán."
                >
                  <Textarea id="summary" rows={6} {...form.register("summary")} />
                </Field>
                <Button
                  type="button"
                  variant="secondary"
                  className="w-fit"
                  disabled={running}
                  onClick={() => setFillOpen(true)}
                >
                  <Sparkles /> AI điền từ tên + link
                </Button>
              </Section>
              <Section id="style" active={active} title="Style">
                <Field id="voice" label="Giọng kể / voice">
                  <Input id="voice" {...form.register("voice")} />
                </Field>
                <Field id="toneRules" label="Tone rules" hint="Mỗi dòng một rule.">
                  <Textarea id="toneRules" rows={5} {...form.register("toneRules")} />
                </Field>
                <Field id="avoid" label="Cách diễn đạt cần tránh" hint="Mỗi dòng một mục.">
                  <Textarea id="avoid" rows={4} {...form.register("avoid")} />
                </Field>
                <GlossaryEditor name="signaturePhrases" label="Cụm từ đặc trưng (style)" />
              </Section>
              <Section id="glossary" active={active} title="Glossary">
                {GLOSSARY_KEYS.map((key) => (
                  <GlossaryEditor key={key} name={`glossary.${key}`} label={GLOSSARY_LABELS[key]} />
                ))}
              </Section>
              <Section id="rules" active={active} title="Rule kiểm tra">
                <CheckRulesEditor defaults={defaults} />
              </Section>
              <Section id="prompt" active={active} title="Prompt">
                <PromptEditor defaults={defaults} />
              </Section>
            </div>
          </div>
        </div>
        <footer className="flex items-center gap-2 border-t bg-card px-5 py-3">
          <input
            ref={fileInput}
            type="file"
            accept=".json"
            className="hidden"
            aria-label="Chọn file story.json"
            onChange={(e) => {
              const f = e.target.files?.[0];
              if (f) void importJson(f);
              e.target.value = "";
            }}
          />
          <Button type="button" variant="ghost" size="sm" onClick={() => fileInput.current?.click()}>
            <Upload /> Nhập JSON
          </Button>
          <Button type="button" variant="ghost" size="sm" onClick={exportJson}>
            <Download /> Xuất JSON
          </Button>
          <div className="flex-1" />
          {dirty && <span className="text-xs text-muted-foreground">Có thay đổi chưa lưu</span>}
          <Button type="submit" disabled={running || !dirty || form.formState.isSubmitting}>
            <Save /> Lưu
          </Button>
        </footer>
        <AiFillDialog
          root={currentRoot}
          initialName={form.getValues("name")}
          initialUrl={form.getValues("sourceUrl")}
          open={fillOpen}
          onOpenChange={setFillOpen}
          onApply={(after) => {
            setFillOpen(false);
            void persist(after)
              .then(() => form.reset(toFormValues(after)))
              .catch((e: unknown) => toast.error(e instanceof Error ? e.message : "Không lưu được"));
          }}
        />
      </form>
    </FormProvider>
  );
}
