import {
  AlertTriangle,
  BookOpen,
  FileDown,
  FileText,
  FileUp,
  Languages,
  LoaderCircle,
  Palette,
  Plus,
  RotateCcw,
  Save,
  Search,
  Settings2,
  ShieldCheck,
  Trash2,
} from "lucide-react";
import { lazy, Suspense, useEffect, useRef, useState } from "react";
import { toast } from "sonner";

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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import {
  activeAiTranslationProviderConfig,
  type AiSettings,
} from "@/lib/ai-settings";
import { baseUrlProblem, resolveAiCall } from "@/lib/ai-client";
import { NOVEL_TRANSLATOR_BASE_PROMPT } from "@/lib/ai-translation-prompt";
import { DEFAULT_AI_CHECK_RULES } from "@/lib/ai-translation";
import { fillAiStoryConfig } from "@/lib/ai-story-fill";
import {
  normalizeAiStoryConfig,
  storyGlossaryCategories,
  type AiStoryConfig,
  type StoryGlossaryKey,
  parseAiStoryConfigJson,
} from "@/lib/ai-story";

const PlatePromptEditor = lazy(async () => {
  const module = await import("@/components/plate-prompt-editor");
  return { default: module.PlatePromptEditor };
});

interface AiStoryConfigDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  story: AiStoryConfig;
  aiSettings: AiSettings;
  onSave: (story: AiStoryConfig) => void;
}

function listText(values: string[]): string {
  return values.join("\n");
}

function parseList(value: string): string[] {
  return value.split(/\r?\n/).map((item) => item.trim()).filter(Boolean);
}

function pairEntries(value: Record<string, string>): Array<[string, string]> {
  return Object.entries(value);
}

export function AiStoryConfigDialog({
  open,
  onOpenChange,
  story,
  aiSettings,
  onSave,
}: AiStoryConfigDialogProps) {
  const [draft, setDraft] = useState(() => normalizeAiStoryConfig(story));
  const importFileRef = useRef<HTMLInputElement>(null);

  async function importConfigFile(file: File | undefined) {
    if (!file) return;
    const parsed = parseAiStoryConfigJson(await file.text());
    if (!parsed) {
      toast.error("File cấu hình không hợp lệ", {
        description: "Cần file JSON xuất từ chính dialog này.",
      });
      return;
    }
    setDraft(parsed);
    toast.success("Đã nạp cấu hình từ file", {
      description: "Kiểm tra lại rồi bấm Lưu cấu hình để áp dụng.",
    });
  }

  function exportConfigFile() {
    const normalized = normalizeAiStoryConfig(draft);
    const filename = `${
      (normalized.name.trim() || "cau-hinh-truyen").replace(/[\\/:*?"<>|]/g, "").slice(0, 60) ||
      "cau-hinh-truyen"
    }.json`;
    const blob = new Blob([JSON.stringify(normalized, null, 2)], {
      type: "application/json",
    });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = filename;
    anchor.click();
    URL.revokeObjectURL(url);
  }
  const hasGeminiKey = aiSettings.gemini.apiKey.trim().length > 0;
  const [filling, setFilling] = useState(false);
  const [promptEditorVersion, setPromptEditorVersion] = useState(0);
  const fillAbortRef = useRef<AbortController | undefined>(undefined);

  useEffect(() => () => fillAbortRef.current?.abort(), []);

  function patch(values: Partial<AiStoryConfig>) {
    setDraft((current) => ({ ...current, ...values }));
  }

  function updateGlossary(
    category: StoryGlossaryKey,
    entries: Array<[string, string]>,
  ) {
    patch({
      glossary: {
        ...draft.glossary,
        [category]: Object.fromEntries(entries),
      },
    });
  }

  async function aiFill() {
    const provider = aiSettings.gemini.apiKey.trim()
      ? "gemini"
      : aiSettings.translation.provider;
    // Dùng model Gemini của Dịch AI thay vì model lọc tên: bản flash-lite
    // mặc định của lọc tên không hỗ trợ Google Search grounding.
    const providerConfig = provider === "gemini" && aiSettings.gemini.apiKey.trim()
      ? { ...aiSettings.gemini, model: aiSettings.translation.models.gemini }
      : activeAiTranslationProviderConfig(aiSettings);
    if (!providerConfig.apiKey.trim()) {
      toast.error("AI fill cần API key Gemini hoặc provider Dịch AI");
      return;
    }
    const problem = baseUrlProblem(providerConfig.baseUrl);
    if (problem) {
      toast.error("Base URL AI không hợp lệ", { description: problem });
      return;
    }
    setFilling(true);
    const controller = new AbortController();
    fillAbortRef.current = controller;
    try {
      const result = await fillAiStoryConfig(
        resolveAiCall(provider, providerConfig),
        draft,
        controller.signal,
      );
      setDraft((current) => normalizeAiStoryConfig({ ...current, ...result.values }));
      if (provider !== "gemini") {
        toast.warning("Đã điền bằng provider Dịch AI — không có web search", {
          description: "Nhập API key Gemini để AI fill tra cứu Google Search.",
        });
      } else if (result.googleSearchUsed) {
        toast.success("Đã điền thông tin truyện", {
          description: "Gemini đã tra cứu bằng Google Search.",
        });
      } else {
        toast.warning("Gemini không tra Google Search lần này", {
          description: "Kết quả có thể không chính xác — kiểm tra lại hoặc bấm AI fill lần nữa.",
        });
      }
    } catch (error) {
      if (controller.signal.aborted) return;
      toast.error(error instanceof Error ? error.message : "Không thể AI fill");
    } finally {
      setFilling(false);
      if (fillAbortRef.current === controller) fillAbortRef.current = undefined;
    }
  }

  function save() {
    for (const rule of draft.checkRules) {
      try {
        new RegExp(rule.pattern, rule.flags);
      } catch (error) {
        toast.error(`Regex không hợp lệ: ${rule.pattern}`, {
          description: error instanceof Error ? error.message : undefined,
        });
        return;
      }
    }
    onSave(normalizeAiStoryConfig(draft));
    onOpenChange(false);
    toast.success("Đã lưu cấu hình truyện vào IndexedDB");
  }

  const effectiveRules = draft.checkRules.length > 0
    ? draft.checkRules
    : DEFAULT_AI_CHECK_RULES;

  function editRule(index: number, field: "pattern" | "flags" | "message", value: string) {
    const rules = effectiveRules.map((rule) => ({ ...rule }));
    rules[index] = { ...rules[index], [field]: value };
    patch({ checkRules: rules });
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(nextOpen) => {
        if (!nextOpen) fillAbortRef.current?.abort();
        onOpenChange(nextOpen);
      }}
    >
      <DialogContent className="h-[92dvh] w-[min(94vw,1040px)]">
        <div className="shrink-0 border-b px-6 py-4 pr-14">
          <DialogHeader>
            <div className="flex items-center gap-2">
              <Settings2 className="size-4 text-primary" />
              <DialogTitle>Cấu hình truyện</DialogTitle>
            </div>
            <DialogDescription>Thiết lập riêng cho workspace này.</DialogDescription>
          </DialogHeader>
        </div>

        <Tabs defaultValue="info" className="min-h-0 flex-1 gap-0">
          <div className="fine-scrollbar shrink-0 overflow-x-auto overflow-y-hidden px-6 pt-4">
            <TabsList className="w-max border">
              <TabsTrigger value="info">
                <BookOpen /> Thông tin
              </TabsTrigger>
              <TabsTrigger value="glossary">
                <Languages /> Từ điển
              </TabsTrigger>
              <TabsTrigger value="style">
                <Palette /> Style
              </TabsTrigger>
              <TabsTrigger value="prompt">
                <FileText /> Prompt
              </TabsTrigger>
              <TabsTrigger value="rules">
                <ShieldCheck /> Kiểm tra
              </TabsTrigger>
            </TabsList>
          </div>

          <TabsContent value="info" className="fine-scrollbar flex min-h-0 flex-col overflow-y-auto px-6 py-5">
            <div className="grid min-h-0 flex-1 gap-x-8 gap-y-5 lg:grid-cols-2">
              <div className="grid content-start gap-4">
                <div className="grid gap-2">
                  <Label htmlFor="story-name">Tên truyện</Label>
                  <div className="flex gap-2">
                    <Input
                      id="story-name"
                      className="min-w-0 flex-1 bg-card"
                      value={draft.name}
                      onChange={(event) => patch({ name: event.target.value })}
                      placeholder="Tên truyện Trung Quốc"
                    />
                    <Button
                      type="button"
                      variant="outline"
                      title={hasGeminiKey
                        ? "Gemini tra cứu Google Search rồi tự điền thông tin truyện"
                        : "Cần API key Gemini để tra Google Search — vào Cài đặt để nhập"}
                      onClick={() => void aiFill()}
                      disabled={filling}
                    >
                      {filling ? <LoaderCircle className="animate-spin" /> : <Search />}
                      AI fill
                    </Button>
                  </div>
                  {!hasGeminiKey ? (
                    <p className="flex items-center gap-1.5 text-xs text-amber-700 dark:text-amber-300">
                      <AlertTriangle className="size-3.5 shrink-0" />
                      AI fill cần API key Gemini để tra Google Search — nhập ở Cài đặt, mục &ldquo;AI cho lọc tên &amp; từ điển&rdquo;, chọn Gemini. Thiếu key sẽ dùng provider Dịch AI và dễ bịa thông tin.
                    </p>
                  ) : null}
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="story-url">Link nguồn</Label>
                  <Input
                    id="story-url"
                    type="url"
                    autoComplete="off"
                    spellCheck={false}
                    className="bg-card font-mono text-xs"
                    value={draft.sourceUrl}
                    onChange={(event) => patch({ sourceUrl: event.target.value })}
                    placeholder="https://fanqienovel.com/page/..."
                  />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="story-protagonist">Nhân vật chính</Label>
                  <Input
                    id="story-protagonist"
                    className="bg-card"
                    value={draft.protagonist}
                    onChange={(event) => patch({ protagonist: event.target.value })}
                    placeholder="Tên Hán-Việt"
                  />
                </div>
              </div>
              <div className="flex min-h-0 flex-col gap-2">
                <Label htmlFor="story-summary">Tóm tắt</Label>
                <Textarea
                  id="story-summary"
                  value={draft.summary}
                  onChange={(event) => patch({ summary: event.target.value })}
                  placeholder="Tóm tắt bối cảnh, tuyến nhân vật và mạch truyện chính…"
                  className="min-h-52 flex-1 resize-none bg-card leading-relaxed"
                />
              </div>
            </div>
          </TabsContent>

          <TabsContent value="glossary" className="fine-scrollbar min-h-0 overflow-y-auto px-6 py-5">
            <div className="grid gap-4">
              <div className="flex items-center justify-between rounded-lg border p-4">
                <div>
                  <Label htmlFor="story-auto-glossary" className="text-sm font-semibold">
                    Tự thêm tên từ bản dịch
                  </Label>
                  <p className="pt-1 text-xs text-muted-foreground">
                    Sau mỗi chương, trích tên riêng mới nạp vào từ điển truyện. Cài đặt của truyện được ưu tiên hơn toggle chung trong Cấu hình AI.
                  </p>
                </div>
                <Select
                  value={draft.autoGlossary}
                  onValueChange={(autoGlossary) =>
                    setDraft({
                      ...draft,
                      autoGlossary: autoGlossary as AiStoryConfig["autoGlossary"],
                    })
                  }
                >
                  <SelectTrigger id="story-auto-glossary" className="w-44 shrink-0">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="inherit">Theo Cấu hình AI</SelectItem>
                    <SelectItem value="on">Bật cho truyện này</SelectItem>
                    <SelectItem value="off">Tắt cho truyện này</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              {storyGlossaryCategories.map(({ key, label }) => {
                const entries = pairEntries(draft.glossary[key]);
                return (
                  <section key={key} className="grid gap-3 rounded-lg border p-4">
                    <div className="flex items-center justify-between">
                      <h3 className="text-sm font-semibold">{label}</h3>
                      <span className="text-xs text-muted-foreground">{entries.length} mục</span>
                    </div>
                    {entries.map(([source, target], index) => (
                      <div key={`${key}-${index}`} className="grid grid-cols-[1fr_1fr_32px] gap-2">
                        <Input
                          value={source}
                          aria-label={`${label} CN ${index + 1}`}
                          className="bg-card"
                          placeholder="Hán tự"
                          onChange={(event) => {
                            const next = [...entries];
                            next[index] = [event.target.value, target];
                            updateGlossary(key, next);
                          }}
                        />
                        <Input
                          value={target}
                          aria-label={`${label} VN ${index + 1}`}
                          className="bg-card"
                          placeholder="Tiếng Việt"
                          onChange={(event) => {
                            const next = [...entries];
                            next[index] = [source, event.target.value];
                            updateGlossary(key, next);
                          }}
                        />
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon-sm"
                          aria-label={`Xóa ${source}`}
                          onClick={() => updateGlossary(key, entries.filter((_, itemIndex) => itemIndex !== index))}
                        >
                          <Trash2 />
                        </Button>
                      </div>
                    ))}
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="w-fit"
                      onClick={() => updateGlossary(key, [...entries, ["", ""]])}
                    >
                      <Plus /> Thêm mục
                    </Button>
                  </section>
                );
              })}
            </div>
          </TabsContent>

          <TabsContent value="style" className="fine-scrollbar min-h-0 overflow-y-auto px-6 py-5">
            <div className="grid gap-5">
              <div className="grid gap-1.5">
                <Label htmlFor="story-voice">Voice nhân vật chính</Label>
                <Textarea
                  id="story-voice"
                  value={draft.style.voice}
                  onChange={(event) => patch({ style: { ...draft.style, voice: event.target.value } })}
                  className="min-h-24 resize-y bg-card"
                />
              </div>
              <div className="grid gap-1.5">
                <Label htmlFor="story-tone">Tone rules · mỗi dòng một rule</Label>
                <Textarea
                  id="story-tone"
                  value={listText(draft.style.toneRules)}
                  onChange={(event) => patch({ style: { ...draft.style, toneRules: parseList(event.target.value) } })}
                  className="min-h-32 resize-y bg-card"
                />
              </div>
              <div className="grid gap-2">
                <div className="flex items-center justify-between">
                  <Label>Cụm từ đặc trưng</Label>
                  <span className="text-xs text-muted-foreground">
                    {Object.keys(draft.style.signaturePhrases).length} mục
                  </span>
                </div>
                {pairEntries(draft.style.signaturePhrases).map(([source, target], index, entries) => (
                  <div key={index} className="grid grid-cols-[1fr_1fr_32px] gap-2">
                    <Input
                      value={source}
                      aria-label={`Cụm đặc trưng CN ${index + 1}`}
                      className="bg-card"
                      placeholder="Hán tự"
                      onChange={(event) => {
                        const next = [...entries];
                        next[index] = [event.target.value, target];
                        patch({ style: { ...draft.style, signaturePhrases: Object.fromEntries(next) } });
                      }}
                    />
                    <Input
                      value={target}
                      aria-label={`Cụm đặc trưng VN ${index + 1}`}
                      className="bg-card"
                      placeholder="Tiếng Việt"
                      onChange={(event) => {
                        const next = [...entries];
                        next[index] = [source, event.target.value];
                        patch({ style: { ...draft.style, signaturePhrases: Object.fromEntries(next) } });
                      }}
                    />
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon-sm"
                      aria-label={`Xóa cụm ${source}`}
                      onClick={() => patch({
                        style: {
                          ...draft.style,
                          signaturePhrases: Object.fromEntries(
                            entries.filter((_, itemIndex) => itemIndex !== index),
                          ),
                        },
                      })}
                    >
                      <Trash2 />
                    </Button>
                  </div>
                ))}
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  className="w-fit"
                  onClick={() => patch({
                    style: {
                      ...draft.style,
                      signaturePhrases: {
                        ...draft.style.signaturePhrases,
                        "": "",
                      },
                    },
                  })}
                >
                  <Plus /> Thêm cụm
                </Button>
              </div>
              <div className="grid gap-1.5">
                <Label htmlFor="story-avoid">Cách diễn đạt cần tránh · mỗi dòng một mục</Label>
                <Textarea
                  id="story-avoid"
                  value={listText(draft.style.avoid)}
                  onChange={(event) => patch({ style: { ...draft.style, avoid: parseList(event.target.value) } })}
                  className="min-h-32 resize-y bg-card"
                />
              </div>
            </div>
          </TabsContent>

          <TabsContent value="prompt" className="fine-scrollbar min-h-0 overflow-y-auto px-6 py-5">
            <div className="grid gap-3">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <h3 className="text-sm font-semibold">Prompt dịch thuật</h3>
                  <p className="text-xs text-muted-foreground">
                    Từ điển, style và thông tin truyện vẫn được nối vào phía sau.
                  </p>
                </div>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    patch({ customPrompt: "" });
                    setPromptEditorVersion((version) => version + 1);
                  }}
                >
                  <RotateCcw /> Mặc định
                </Button>
              </div>
              <Suspense
                fallback={(
                  <div
                    role="status"
                    className="grid min-h-[480px] place-items-center rounded-sm border border-border/60 bg-reader-paper text-sm text-muted-foreground"
                  >
                    Đang tải editor…
                  </div>
                )}
              >
                <PlatePromptEditor
                  key={promptEditorVersion}
                  initialValue={draft.customPrompt || NOVEL_TRANSLATOR_BASE_PROMPT}
                  onChange={(customPrompt) => patch({ customPrompt })}
                />
              </Suspense>
            </div>
          </TabsContent>

          <TabsContent value="rules" className="fine-scrollbar min-h-0 overflow-y-auto px-6 py-5">
            <div className="grid gap-3">
              <div className="flex items-start justify-between gap-4">
                <div>
                  <h3 className="text-sm font-semibold">Rules kiểm tra sau dịch</h3>
                  <p className="text-xs text-muted-foreground">
                    Review tối đa 3 lần và chỉ giữ bản làm giảm số vi phạm. Rule bắt Hán tự còn
                    sót luôn được áp, kể cả khi thay danh sách này.
                  </p>
                </div>
                <Button type="button" variant="outline" size="sm" onClick={() => patch({ checkRules: [] })}>
                  <RotateCcw /> Mặc định
                </Button>
              </div>
              <div className="grid grid-cols-[minmax(0,1fr)_56px_minmax(0,1fr)_32px] gap-2 px-1 text-[10px] text-muted-foreground">
                <span>Regex</span><span>Flags</span><span>Mô tả lỗi</span><span />
              </div>
              {effectiveRules.map((rule, index) => (
                <div key={index} className="grid grid-cols-[minmax(0,1fr)_56px_minmax(0,1fr)_32px] gap-2">
                  <Input
                    value={rule.pattern}
                    aria-label={`Regex ${index + 1}`}
                    className="bg-card font-mono text-xs"
                    onChange={(event) => editRule(index, "pattern", event.target.value)}
                  />
                  <Input
                    value={rule.flags ?? ""}
                    aria-label={`Flags ${index + 1}`}
                    className="bg-card font-mono text-xs"
                    placeholder="i"
                    onChange={(event) => editRule(index, "flags", event.target.value)}
                  />
                  <Input
                    value={rule.message}
                    aria-label={`Mô tả rule ${index + 1}`}
                    className="bg-card"
                    onChange={(event) => editRule(index, "message", event.target.value)}
                  />
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon-sm"
                    aria-label={`Xóa rule ${index + 1}`}
                    onClick={() => patch({ checkRules: effectiveRules.filter((_, itemIndex) => itemIndex !== index) })}
                  >
                    <Trash2 />
                  </Button>
                </div>
              ))}
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="w-fit"
                onClick={() => patch({ checkRules: [...effectiveRules, { pattern: "", message: "" }] })}
              >
                <Plus /> Thêm rule
              </Button>
            </div>
          </TabsContent>
        </Tabs>

        <DialogFooter className="shrink-0 border-t px-6 py-4">
          <div className="mr-auto flex items-center gap-2">
            <input
              ref={importFileRef}
              type="file"
              accept=".json,application/json"
              className="hidden"
              aria-label="Chọn file cấu hình truyện"
              onChange={(event) => {
                void importConfigFile(event.target.files?.[0]);
                event.target.value = "";
              }}
            />
            <Button
              type="button"
              variant="outline"
              onClick={() => importFileRef.current?.click()}
            >
              <FileUp /> Nhập JSON
            </Button>
            <Button type="button" variant="outline" onClick={exportConfigFile}>
              <FileDown /> Xuất JSON
            </Button>
          </div>
          <Button
            type="button"
            variant="outline"
            onClick={() => {
              fillAbortRef.current?.abort();
              onOpenChange(false);
            }}
          >
            Hủy
          </Button>
          <Button type="button" disabled={filling} onClick={save}>
            <Save /> Lưu cấu hình
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
