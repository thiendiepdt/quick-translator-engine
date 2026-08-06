import {
  AlertTriangle,
  Brain,
  Copy,
  Eraser,
  FileUp,
  LoaderCircle,
  Octagon,
  Send,
  Settings2,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { useTextFileImport } from "@/hooks/use-text-file-import";
import {
  activeAiTranslationProviderConfig,
  type AiSettings,
} from "@/lib/ai-settings";
import { baseUrlProblem, resolveAiCall } from "@/lib/ai-client";
import { generateAiText } from "@/lib/ai-text-client";
import {
  buildAiTranslationReviewPrompt,
  buildAiTranslationSystemPrompt,
  buildWorkspaceTranslationGlossary,
  checkAiTranslationViolations,
  countTranslationGlossaryEntries,
  formatAiTranslation,
  nonEmptyLineCount,
  type TranslationViolation,
} from "@/lib/ai-translation";
import { cn } from "@/lib/utils";
import { useWorkspaceCatalogStore } from "@/store/workspace-catalog";
import { useWorkspaceStore } from "@/store/workspace";

interface AiTranslationWorkspaceProps {
  aiSettings: AiSettings;
  onOpenSettings: () => void;
}

type TranslationPhase = "translating" | "retrying" | "reviewing";

function phaseLabel(phase: TranslationPhase | null, reviewRound: number): string {
  if (phase === "retrying") return "Bản đầu thiếu đoạn · đang dịch lại";
  if (phase === "reviewing") return `Đang soát lỗi · lần ${reviewRound}/3`;
  if (phase === "translating") return "Đang dịch nguyên văn";
  return "Sẵn sàng";
}

function isAbortError(error: unknown): boolean {
  return error instanceof DOMException && error.name === "AbortError";
}

export function AiTranslationWorkspace({
  aiSettings,
  onOpenSettings,
}: AiTranslationWorkspaceProps) {
  const source = useWorkspaceStore((state) => state.aiTranslationSource);
  const output = useWorkspaceStore((state) => state.aiTranslationOutput);
  const thinking = useWorkspaceStore((state) => state.aiTranslationThinking);
  const violations = useWorkspaceStore((state) => state.aiTranslationViolations);
  const localDictionaryEntries = useWorkspaceStore(
    (state) => state.localDictionaryEntries,
  );
  const knownNames = useWorkspaceStore((state) => state.knownNames);
  const setSource = useWorkspaceStore((state) => state.setAiTranslationSource);
  const setOutput = useWorkspaceStore((state) => state.setAiTranslationOutput);
  const setThinking = useWorkspaceStore((state) => state.setAiTranslationThinking);
  const setViolations = useWorkspaceStore(
    (state) => state.setAiTranslationViolations,
  );
  const clearTranslation = useWorkspaceStore((state) => state.clearAiTranslation);
  const [phase, setPhase] = useState<TranslationPhase | null>(null);
  const [reviewRound, setReviewRound] = useState(0);
  const [thinkingExpanded, setThinkingExpanded] = useState(true);
  const sourceTextareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const abortRef = useRef<AbortController | undefined>(undefined);

  const provider = aiSettings.translation.provider;
  const providerConfig = activeAiTranslationProviderConfig(aiSettings);
  const model = providerConfig.model.trim();
  const glossary = useMemo(
    () => buildWorkspaceTranslationGlossary(localDictionaryEntries, knownNames),
    [knownNames, localDictionaryEntries],
  );
  const glossaryCount = countTranslationGlossaryEntries(glossary);

  const { dropActive, dropHandlers, importFile } = useTextFileImport((text, imported) => {
    const previous = source;
    setSource(text);
    requestAnimationFrame(() => sourceTextareaRef.current?.focus());
    toast.success(`Đã nhập ${imported.kind === "file" ? imported.name : "nội dung clipboard"}`, {
      description: `${text.length.toLocaleString("vi-VN")} ký tự`,
      ...(previous
        ? { action: { label: "Hoàn tác", onClick: () => setSource(previous) } }
        : {}),
    });
  });

  useEffect(() => () => abortRef.current?.abort(), []);

  async function generate(
    systemPrompt: string,
    userMessage: string,
    signal: AbortSignal,
    workspaceChanged: () => boolean,
    streamOutput: boolean,
  ): Promise<string> {
    const config = resolveAiCall(provider, providerConfig);
    let streamedOutput = "";
    let streamedThinking = "";
    return generateAiText(config, systemPrompt, userMessage, {
      thinking: aiSettings.translation.thinking,
      signal,
      onChunk(kind, chunk) {
        if (workspaceChanged()) return;
        if (kind === "thinking") {
          streamedThinking += chunk;
          setThinking(streamedThinking);
        } else if (streamOutput) {
          streamedOutput += chunk;
          setOutput(streamedOutput);
        }
      },
    });
  }

  async function runTranslation() {
    if (phase) return;
    if (!source.trim()) {
      toast.error("Dán nguyên văn tiếng Trung trước khi dịch AI");
      return;
    }
    if (!providerConfig.apiKey.trim()) {
      toast.error(`Dịch AI cần API key ${provider === "gemini" ? "Gemini" : "DeepSeek"}`, {
        action: { label: "Mở Cài đặt", onClick: onOpenSettings },
      });
      return;
    }
    if (!model) {
      toast.error("Dịch AI cần chỉ định model", {
        action: { label: "Mở Cài đặt", onClick: onOpenSettings },
      });
      return;
    }
    const baseUrlIssue = baseUrlProblem(providerConfig.baseUrl);
    if (baseUrlIssue) {
      toast.error("Base URL proxy không hợp lệ", {
        description: baseUrlIssue,
        action: { label: "Mở Cài đặt", onClick: onOpenSettings },
      });
      return;
    }

    const requestWorkspaceId = useWorkspaceCatalogStore.getState().activeWorkspaceId;
    const workspaceChanged = () =>
      useWorkspaceCatalogStore.getState().activeWorkspaceId !== requestWorkspaceId;
    const controller = new AbortController();
    abortRef.current = controller;
    setPhase("translating");
    setReviewRound(0);
    setOutput("");
    setThinking("");
    setViolations([]);

    try {
      const systemPrompt = buildAiTranslationSystemPrompt(glossary);
      let translated = formatAiTranslation(
        await generate(systemPrompt, source, controller.signal, workspaceChanged, true),
      );
      if (workspaceChanged()) return;
      setOutput(translated);

      const sourceParagraphs = nonEmptyLineCount(source);
      let translatedParagraphs = nonEmptyLineCount(translated);
      if (translatedParagraphs + 2 <= sourceParagraphs) {
        setPhase("retrying");
        setThinking("");
        setOutput("");
        const retried = formatAiTranslation(
          await generate(systemPrompt, source, controller.signal, workspaceChanged, true),
        );
        if (workspaceChanged()) return;
        const retriedParagraphs = nonEmptyLineCount(retried);
        if (retriedParagraphs > translatedParagraphs) {
          translated = retried;
          translatedParagraphs = retriedParagraphs;
        }
        setOutput(translated);
      }

      let currentViolations = checkAiTranslationViolations(translated);
      let round = 0;
      while (currentViolations.length > 0 && round < 3) {
        const previousCount = currentViolations.length;
        round += 1;
        setPhase("reviewing");
        setReviewRound(round);
        setThinking("");
        const reviewPrompt = buildAiTranslationReviewPrompt(
          translated,
          currentViolations,
        );
        const reviewed = formatAiTranslation(
          await generate(
            reviewPrompt.system,
            reviewPrompt.user,
            controller.signal,
            workspaceChanged,
            false,
          ),
        );
        if (workspaceChanged()) return;
        const reviewedViolations = checkAiTranslationViolations(reviewed);
        if (reviewedViolations.length >= previousCount) break;
        translated = reviewed;
        currentViolations = reviewedViolations;
        setOutput(translated);
      }

      if (translatedParagraphs + 2 <= sourceParagraphs) {
        currentViolations = [
          ...currentViolations,
          {
            line: 0,
            message: `Bản dịch thiếu đoạn: raw ${sourceParagraphs} đoạn, dịch có ${translatedParagraphs} đoạn`,
            text: "",
          },
        ];
      }
      setOutput(translated);
      setViolations(currentViolations);
      toast.success("Dịch AI xong", {
        description: currentViolations.length
          ? `${round} lần soát · còn ${currentViolations.length} cảnh báo`
          : `${round} lần soát · không còn cảnh báo`,
      });
    } catch (error) {
      if (workspaceChanged()) return;
      if (controller.signal.aborted || isAbortError(error)) {
        toast.message("Đã dừng Dịch AI");
      } else {
        toast.error(error instanceof Error ? error.message : "Không thể dịch bằng AI");
      }
    } finally {
      if (!workspaceChanged()) setPhase(null);
      if (abortRef.current === controller) abortRef.current = undefined;
    }
  }

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key !== "Enter" || !(event.metaKey || event.ctrlKey) || phase) return;
      event.preventDefault();
      void runTranslation();
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  });

  function clearSource() {
    const previousSource = source;
    const previousOutput = output;
    clearTranslation();
    requestAnimationFrame(() => sourceTextareaRef.current?.focus());
    toast.message("Đã xóa khung Dịch AI", {
      action: {
        label: "Hoàn tác",
        onClick: () => {
          setSource(previousSource);
          setOutput(previousOutput);
        },
      },
    });
  }

  async function copyOutput() {
    try {
      await navigator.clipboard.writeText(output);
      toast.success("Đã sao chép bản dịch AI");
    } catch {
      toast.error("Trình duyệt không cho phép sao chép");
    }
  }

  function updateOutput(value: string) {
    setOutput(value);
    setViolations(checkAiTranslationViolations(value));
  }

  return (
    <main className="grid min-h-0 min-w-0 grid-rows-[auto_minmax(0,1fr)_auto] gap-3 overflow-hidden p-3">
      <header className="flex min-w-0 flex-wrap items-center gap-2 rounded-md border bg-card px-3 py-2">
        <div className="min-w-0">
          <h1 className="text-sm font-semibold">Dịch AI</h1>
          <p className="truncate text-[11px] text-muted-foreground">
            {provider === "gemini" ? "Gemini" : "DeepSeek"} · {model || "chưa đặt model"}
            {` · thinking ${aiSettings.translation.thinking ? "bật" : "tắt"}`}
          </p>
        </div>
        <span className="rounded border bg-muted px-2 py-1 text-[10px] text-muted-foreground">
          {glossaryCount.toLocaleString("vi-VN")} mục từ điển workspace
        </span>
        <div className="flex-1" />
        <Button type="button" variant="ghost" size="sm" onClick={onOpenSettings}>
          <Settings2 /> Cấu hình AI
        </Button>
        {phase ? (
          <Button
            type="button"
            variant="destructive"
            size="sm"
            onClick={() => abortRef.current?.abort()}
          >
            <Octagon /> Dừng
          </Button>
        ) : (
          <Button type="button" size="sm" onClick={() => void runTranslation()}>
            <Send /> Dịch AI
            <kbd className="hidden border-l border-primary-foreground/25 pl-2 text-[10px] font-normal opacity-80 sm:inline">
              Ctrl/⌘ Enter
            </kbd>
          </Button>
        )}
      </header>

      <div className="grid min-h-0 overflow-hidden rounded-lg border bg-card lg:grid-cols-2 lg:divide-x">
        <section className="grid min-h-[280px] min-w-0 grid-rows-[40px_minmax(0,1fr)_32px] lg:min-h-0">
          <header className="flex items-center gap-2 border-b px-3">
            <strong className="text-xs">Nguyên văn (CN)</strong>
            <div className="flex-1" />
            <input
              ref={fileInputRef}
              type="file"
              accept=".txt,text/plain"
              className="hidden"
              onChange={(event) => {
                const file = event.target.files?.[0];
                if (file) void importFile(file);
                event.target.value = "";
              }}
            />
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              aria-label="Nhập file cho Dịch AI"
              onClick={() => fileInputRef.current?.click()}
            >
              <FileUp />
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              aria-label="Xóa khung Dịch AI"
              disabled={!source && !output}
              onClick={clearSource}
            >
              <Eraser />
            </Button>
          </header>
          <div
            className={cn("relative min-h-0", dropActive && "bg-primary/5")}
            {...dropHandlers}
          >
            <Textarea
              ref={sourceTextareaRef}
              value={source}
              onChange={(event) => setSource(event.target.value)}
              disabled={Boolean(phase)}
              lang="zh-Hans"
              spellCheck={false}
              placeholder="Dán nguyên văn tiếng Trung hoặc thả file .txt vào đây…"
              className="absolute inset-0 h-full min-h-0 resize-none rounded-none border-0 bg-background px-6 py-5 font-serif text-[18px] leading-8 shadow-none focus-visible:ring-0"
            />
            {dropActive ? (
              <div className="pointer-events-none absolute inset-3 z-10 grid place-items-center rounded-md border-2 border-dashed border-primary bg-background/90 text-sm font-medium text-primary">
                Thả file .txt để nhập nguyên văn
              </div>
            ) : null}
          </div>
          <footer className="flex items-center justify-between border-t px-3 text-[10px] text-muted-foreground">
            <span>{source.length.toLocaleString("vi-VN")} ký tự</span>
            <span>{nonEmptyLineCount(source).toLocaleString("vi-VN")} đoạn</span>
          </footer>
        </section>

        <section className="grid min-h-[320px] min-w-0 grid-rows-[40px_auto_minmax(0,1fr)_32px] lg:min-h-0">
          <header className="flex items-center gap-2 border-b px-3">
            <strong className="text-xs">Bản dịch (VN)</strong>
            {phase ? <LoaderCircle className="size-3.5 animate-spin text-primary" /> : null}
            <span className="truncate text-[11px] text-muted-foreground">
              {phaseLabel(phase, reviewRound)}
            </span>
            <div className="flex-1" />
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              aria-label="Sao chép bản dịch AI"
              disabled={!output}
              onClick={() => void copyOutput()}
            >
              <Copy />
            </Button>
          </header>

          {thinking ? (
            <div className="border-b border-violet-500/20 bg-violet-500/5">
              <button
                type="button"
                className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-[11px] text-violet-600 dark:text-violet-300"
                onClick={() => setThinkingExpanded((value) => !value)}
              >
                <Brain className={cn("size-3.5", phase && "animate-pulse")} />
                <span className="font-medium">Quá trình suy nghĩ</span>
                <span className="ml-auto">{thinkingExpanded ? "Ẩn" : "Hiện"}</span>
              </button>
              {thinkingExpanded ? (
                <pre className="fine-scrollbar max-h-28 overflow-auto whitespace-pre-wrap px-3 pb-2 font-mono text-[10px] leading-relaxed text-violet-700/65 dark:text-violet-200/55">
                  {thinking}
                </pre>
              ) : null}
            </div>
          ) : null}

          <div className="relative min-h-0 bg-reader-paper">
            <Textarea
              value={output}
              onChange={(event) => updateOutput(event.target.value)}
              disabled={Boolean(phase)}
              lang="vi"
              spellCheck
              placeholder={phase ? "AI đang dịch…" : "Bản dịch AI sẽ xuất hiện ở đây và có thể sửa trực tiếp."}
              className="absolute inset-0 h-full min-h-0 resize-none rounded-none border-0 bg-reader-paper px-7 py-5 font-serif text-[18px] leading-8 text-reader-ink shadow-none focus-visible:ring-0"
            />
          </div>
          <footer className="flex items-center justify-between border-t px-3 text-[10px] text-muted-foreground">
            <span>{output.length.toLocaleString("vi-VN")} ký tự</span>
            <span>{nonEmptyLineCount(output).toLocaleString("vi-VN")} đoạn</span>
          </footer>
        </section>
      </div>

      <footer className="flex min-w-0 items-center gap-2 px-1 text-[11px]">
        {violations.length > 0 ? (
          <>
            <AlertTriangle className="size-3.5 shrink-0 text-amber-600 dark:text-amber-300" />
            <span className="truncate text-amber-700 dark:text-amber-300">
              Còn {violations.length} cảnh báo · {violationSummary(violations)}
            </span>
          </>
        ) : output ? (
          <span className="text-ok">Không còn cảnh báo tự động</span>
        ) : (
          <span className="text-muted-foreground">
            Prompt và vòng soát được port từ Novel Translator; dữ liệu chỉ gửi tới provider bạn chọn.
          </span>
        )}
      </footer>
    </main>
  );
}

function violationSummary(violations: TranslationViolation[]): string {
  return Array.from(new Set(violations.map((item) => item.message))).slice(0, 2).join(" · ");
}
