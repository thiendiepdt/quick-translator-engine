import { zodResolver } from "@hookform/resolvers/zod";
import { Download, Sparkles, Upload } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { FormProvider, useForm, useWatch } from "react-hook-form";
import { toast } from "sonner";

import { AiFillDialog } from "@/components/ai-fill-dialog";
import { CheckRulesEditor } from "@/components/check-rules-editor";
import { GlossaryEditor } from "@/components/glossary-editor";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { saveStory, storySnapshot } from "@/lib/api";
import { storyConfigSchema } from "@/lib/schema";
import { fromFormValues, storyFormSchema, toFormValues, type StoryFormValues } from "@/lib/story-form";
import { GLOSSARY_KEYS, GLOSSARY_LABELS, type StoryConfig } from "@/lib/types";
import { useStoryStore } from "@/store/story";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function StoryFormDialog({ open, onOpenChange }: Props) {
  const root = useStoryStore((s) => s.root);
  const story = useStoryStore((s) => s.snapshot?.story);
  const running = useStoryStore((s) => s.session.status === "running");
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const form = useForm<StoryFormValues>({
    resolver: zodResolver(storyFormSchema),
    defaultValues: story ? toFormValues(story) : undefined,
  });
  const autoGlossary = useWatch({ control: form.control, name: "autoGlossary" });
  const [fillOpen, setFillOpen] = useState(false);
  const fileInput = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open && story) form.reset(toFormValues(story));
  }, [open, story, form]);

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
      onOpenChange(false);
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
      form.reset(toFormValues({ ...parsed, autoGlossaryLog: currentStory.autoGlossaryLog }));
      toast.message("Đã nạp JSON vào form — bấm Lưu để ghi");
    } catch {
      toast.error("File không đúng schema story.json của qt-web");
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="flex max-h-[90vh] max-w-4xl flex-col">
        <DialogHeader>
          <DialogTitle>Hồ sơ truyện</DialogTitle>
        </DialogHeader>
        <FormProvider {...form}>
          <form onSubmit={(e) => void submit(e)} className="flex min-h-0 flex-1 flex-col gap-3">
            <Tabs defaultValue="info" className="flex min-h-0 flex-1 flex-col">
              <TabsList className="w-fit">
                <TabsTrigger value="info">Thông tin</TabsTrigger>
                <TabsTrigger value="style">Style</TabsTrigger>
                <TabsTrigger value="glossary">Glossary</TabsTrigger>
                <TabsTrigger value="rules">Rule & prompt</TabsTrigger>
              </TabsList>
              <ScrollArea className="min-h-0 flex-1 pr-3">
                <TabsContent value="info" className="flex flex-col gap-3">
                  <div className="grid grid-cols-2 gap-3">
                    <div>
                      <Label htmlFor="name">Tên truyện</Label>
                      <Input id="name" {...form.register("name")} />
                    </div>
                    <div>
                      <Label htmlFor="sourceUrl">Link nguồn</Label>
                      <Input id="sourceUrl" {...form.register("sourceUrl")} placeholder="https://…" />
                    </div>
                    <div>
                      <Label htmlFor="protagonist">Nhân vật chính</Label>
                      <Input id="protagonist" {...form.register("protagonist")} placeholder="Tên Hán-Việt" />
                    </div>
                    <div>
                      <Label htmlFor="autoGlossary">Tự thêm tên riêng</Label>
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
                    </div>
                  </div>
                  <div>
                    <Label htmlFor="summary">Tóm tắt</Label>
                    <Textarea
                      id="summary"
                      rows={5}
                      {...form.register("summary")}
                      placeholder="Bối cảnh, tuyến nhân vật, mạch truyện chính…"
                    />
                  </div>
                  <Button
                    type="button"
                    variant="secondary"
                    className="w-fit"
                    disabled={running}
                    onClick={() => setFillOpen(true)}
                  >
                    <Sparkles /> AI điền từ tên + link
                  </Button>
                </TabsContent>
                <TabsContent value="style" className="flex flex-col gap-3">
                  <div>
                    <Label htmlFor="voice">Giọng kể / voice</Label>
                    <Input id="voice" {...form.register("voice")} />
                  </div>
                  <div>
                    <Label htmlFor="toneRules">Tone rules · mỗi dòng một rule</Label>
                    <Textarea id="toneRules" rows={5} {...form.register("toneRules")} />
                  </div>
                  <div>
                    <Label htmlFor="avoid">Cách diễn đạt cần tránh · mỗi dòng một mục</Label>
                    <Textarea id="avoid" rows={4} {...form.register("avoid")} />
                  </div>
                  <GlossaryEditor name="signaturePhrases" label="Cụm từ đặc trưng (style)" />
                </TabsContent>
                <TabsContent value="glossary" className="flex flex-col gap-4">
                  {GLOSSARY_KEYS.map((key) => (
                    <GlossaryEditor key={key} name={`glossary.${key}`} label={GLOSSARY_LABELS[key]} />
                  ))}
                </TabsContent>
                <TabsContent value="rules" className="flex flex-col gap-3">
                  <CheckRulesEditor />
                  <div>
                    <Label htmlFor="customPrompt">
                      Custom prompt <span className="text-xs text-muted-foreground">(trống = prompt mặc định)</span>
                    </Label>
                    <Textarea
                      id="customPrompt"
                      rows={8}
                      className="font-mono text-xs"
                      {...form.register("customPrompt")}
                    />
                  </div>
                </TabsContent>
              </ScrollArea>
            </Tabs>
            <DialogFooter className="items-center">
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
              <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
                Đóng
              </Button>
              <Button type="submit" disabled={running || form.formState.isSubmitting}>
                Lưu
              </Button>
            </DialogFooter>
          </form>
        </FormProvider>
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
      </DialogContent>
    </Dialog>
  );
}
