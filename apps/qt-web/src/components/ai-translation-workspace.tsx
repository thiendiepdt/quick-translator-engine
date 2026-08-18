import {
  AlertTriangle,
  BookOpen,
  Brain,
  CheckCircle2,
  Circle,
  Copy,
  Eraser,
  FileUp,
  ListRestart,
  LoaderCircle,
  Maximize2,
  Sparkles,
  Octagon,
  Pencil,
  Send,
  Settings2,
  Trash2,
} from "lucide-react";
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";

import { AiStoryConfigDialog } from "@/components/ai-story-config-dialog";
import { MappedText } from "@/components/mapped-text";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import {
  activeAiTranslationProviderConfig,
  type AiSettings,
} from "@/lib/ai-settings";
import { baseUrlProblem, extractStoryGlossaryWithAi, resolveAiCall } from "@/lib/ai-client";
import {
  appendAutoGlossary,
  collectGlossaryKeys,
  sanitizeExtractedGlossary,
} from "@/lib/ai-glossary";
import { generateAiText } from "@/lib/ai-text-client";
import {
  buildAiTranslationReviewPrompt,
  buildAiTranslationSystemPrompt,
  buildWorkspaceTranslationGlossary,
  checkAiTranslationViolations,
  countTranslationGlossaryEntries,
  formatAiTranslation,
  nonEmptyLineCount,
  wordCount,
  type TranslationViolation,
} from "@/lib/ai-translation";
import { appendThinking, parseThinkingLog, thinkingPinAfterScroll } from "@/lib/thinking-log";
import {
  aiParagraphRanges,
  aiParagraphsOf,
  labeledAiRepairPayload,
  labeledAiSourcePayload,
  parseLabeledAiTranslation,
  stripAiParagraphMarkers,
} from "@/lib/ai-paragraphs";
import {
  countStoryGlossaryEntries,
  storyGlossaryCategories,
  type AiTranslationChapter,
  type AutoGlossaryEntry,
  type StoryGlossary,
} from "@/lib/ai-story";
import { looksLikeTextFile, readChapterFile } from "@/lib/text-file";
import { cn } from "@/lib/utils";
import { useWorkspaceCatalogStore } from "@/store/workspace-catalog";
import { useWorkspaceStore } from "@/store/workspace";

interface AiTranslationWorkspaceProps {
  aiSettings: AiSettings;
  onOpenSettings: () => void;
}

type TranslationPhase = "translating" | "retrying" | "reviewing" | "extracting";

function phaseLabel(phase: TranslationPhase | null, reviewRound: number): string {
  if (phase === "retrying") return "Bản đầu thiếu đoạn · đang dịch lại";
  if (phase === "reviewing") return `Đang soát lỗi · lần ${reviewRound}/3`;
  if (phase === "translating") return "Đang dịch nguyên văn";
  if (phase === "extracting") return "Đang trích tên mới cho từ điển truyện";
  return "Sẵn sàng";
}

function isAbortError(error: unknown): boolean {
  return error instanceof DOMException && error.name === "AbortError";
}

const glossaryCategoryLabels: Record<string, string> = Object.fromEntries(
  storyGlossaryCategories.map(({ key, label }) => [key, label]),
);

type MappedPane = "source" | "output";

function scrollParagraphIntoView(container: HTMLDivElement | null, index: number) {
  if (!container) return;
  const target = container.querySelector<HTMLElement>(`[data-range-index="${index}"]`);
  if (!target) return;
  const containerRect = container.getBoundingClientRect();
  const targetRect = target.getBoundingClientRect();
  container.scrollTop += targetRect.top - containerRect.top - container.clientHeight / 3;
}

export function AiTranslationWorkspace({
  aiSettings,
  onOpenSettings,
}: AiTranslationWorkspaceProps) {
  const source = useWorkspaceStore((state) => state.aiTranslationSource);
  const output = useWorkspaceStore((state) => state.aiTranslationOutput);
  const thinking = useWorkspaceStore((state) => state.aiTranslationThinking);
  const violations = useWorkspaceStore((state) => state.aiTranslationViolations);
  const story = useWorkspaceStore((state) => state.aiStory);
  const chapters = useWorkspaceStore((state) => state.aiTranslationChapters);
  const activeChapterId = useWorkspaceStore(
    (state) => state.activeAiTranslationChapterId,
  );
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
  const updateStory = useWorkspaceStore((state) => state.updateAiStory);
  const importChapters = useWorkspaceStore(
    (state) => state.importAiTranslationChapters,
  );
  const selectChapter = useWorkspaceStore(
    (state) => state.selectAiTranslationChapter,
  );
  const updateChapter = useWorkspaceStore(
    (state) => state.updateAiTranslationChapter,
  );
  const removeChapter = useWorkspaceStore(
    (state) => state.removeAiTranslationChapter,
  );
  const clearChapters = useWorkspaceStore(
    (state) => state.clearAiTranslationChapters,
  );
  const [phase, setPhase] = useState<TranslationPhase | null>(null);
  const [reviewRound, setReviewRound] = useState(0);
  const [thinkingExpanded, setThinkingExpanded] = useState(true);
  const [storyConfigOpen, setStoryConfigOpen] = useState(false);
  const [queueRunning, setQueueRunning] = useState(false);
  const [dropActive, setDropActive] = useState(false);
  const [streamingOutput, setStreamingOutput] = useState<string>();
  const [streamingThinking, setStreamingThinking] = useState<string>();
  const sourceTextareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  /** Log thinking tích lũy của lần dịch đang chạy (dịch + dịch lại + các vòng soát). */
  const thinkingLogRef = useRef("");
  const [thinkingDialogOpen, setThinkingDialogOpen] = useState(false);
  const [autoGlossaryOpen, setAutoGlossaryOpen] = useState(false);
  const sourceScrollRef = useRef<HTMLDivElement>(null);
  const outputScrollRef = useRef<HTMLDivElement>(null);
  /** Bản dịch mặc định chỉ đọc; bật Sửa mới thành textarea. */
  const [outputEditing, setOutputEditing] = useState(false);
  const [sourceLinked, setSourceLinked] = useState(false);
  const [activePair, setActivePair] = useState<number>();
  const [pairScrollRequest, setPairScrollRequest] = useState<{
    pane: MappedPane;
    index: number;
  }>();
  const dragDepthRef = useRef(0);
  const abortRef = useRef<AbortController | undefined>(undefined);

  const provider = aiSettings.translation.provider;
  const providerConfig = activeAiTranslationProviderConfig(aiSettings);
  const model = providerConfig.model.trim();
  const glossary = useMemo(
    () => buildWorkspaceTranslationGlossary(localDictionaryEntries, knownNames),
    [knownNames, localDictionaryEntries],
  );
  const glossaryCount =
    countTranslationGlossaryEntries(glossary) +
    countStoryGlossaryEntries(story.glossary);

  useEffect(() => () => abortRef.current?.abort(), []);

  async function importChapterFiles(files: File[]) {
    const textFiles = files.filter(looksLikeTextFile);
    if (textFiles.length === 0) {
      toast.error("Chỉ nhận file văn bản .txt hoặc .md");
      return;
    }
    const loaded = await Promise.allSettled(
      textFiles.map(async (file) => ({
        filename: file.name,
        source: await readChapterFile(file),
      })),
    );
    const successful = loaded.flatMap((result) =>
      result.status === "fulfilled" ? [result.value] : [],
    );
    if (successful.length > 0) {
      importChapters(successful);
      requestAnimationFrame(() => sourceTextareaRef.current?.focus());
      toast.success(`Đã nhập ${successful.length} chương`, {
        description: "Đã xếp theo tên file và tự lưu vào IndexedDB",
      });
    }
    const failed = loaded.length - successful.length;
    if (failed > 0) toast.error(`${failed} file không đọc được`);
  }

  const dropHandlers = {
    onDragEnter(event: React.DragEvent) {
      if (!Array.from(event.dataTransfer.types).includes("Files")) return;
      event.preventDefault();
      dragDepthRef.current += 1;
      setDropActive(true);
    },
    onDragOver(event: React.DragEvent) {
      if (Array.from(event.dataTransfer.types).includes("Files")) event.preventDefault();
    },
    onDragLeave(event: React.DragEvent) {
      if (!Array.from(event.dataTransfer.types).includes("Files")) return;
      dragDepthRef.current = Math.max(0, dragDepthRef.current - 1);
      if (dragDepthRef.current === 0) setDropActive(false);
    },
    onDrop(event: React.DragEvent) {
      if (!Array.from(event.dataTransfer.types).includes("Files")) return;
      event.preventDefault();
      dragDepthRef.current = 0;
      setDropActive(false);
      void importChapterFiles(Array.from(event.dataTransfer.files));
    },
  };

  async function generate(
    systemPrompt: string,
    userMessage: string,
    signal: AbortSignal,
    workspaceChanged: () => boolean,
    streamOutput: boolean,
    thinkingLabel: string,
  ): Promise<string> {
    const config = resolveAiCall(provider, providerConfig);
    const thinkingBase = thinkingLogRef.current;
    let streamedOutput = "";
    let streamedThinking = "";
    const result = await generateAiText(config, systemPrompt, userMessage, {
      thinking: aiSettings.translation.thinking,
      signal,
      onChunk(kind, chunk) {
        if (workspaceChanged()) return;
        if (kind === "thinking") {
          streamedThinking += chunk;
          setStreamingThinking(appendThinking(thinkingBase, thinkingLabel, streamedThinking));
        } else if (streamOutput) {
          streamedOutput += chunk;
          setStreamingOutput(stripAiParagraphMarkers(streamedOutput));
        }
      },
    });
    if (!workspaceChanged()) {
      thinkingLogRef.current = appendThinking(thinkingBase, thinkingLabel, streamedThinking);
      setThinking(thinkingLogRef.current);
    }
    return result;
  }

  function canCallAi(): boolean {
    if (!providerConfig.apiKey.trim()) {
      toast.error(`Dịch AI cần API key ${provider === "gemini" ? "Gemini" : "DeepSeek"}`, {
        action: { label: "Mở Cài đặt", onClick: onOpenSettings },
      });
      return false;
    }
    if (!model) {
      toast.error("Dịch AI cần chỉ định model", {
        action: { label: "Mở Cài đặt", onClick: onOpenSettings },
      });
      return false;
    }
    const baseUrlIssue = baseUrlProblem(providerConfig.baseUrl);
    if (baseUrlIssue) {
      toast.error("Base URL proxy không hợp lệ", {
        description: baseUrlIssue,
        action: { label: "Mở Cài đặt", onClick: onOpenSettings },
      });
      return false;
    }
    return true;
  }

  async function translateOne(
    chapter: AiTranslationChapter | undefined,
    sourceText: string,
    controller: AbortController,
    workspaceChanged: () => boolean,
  ): Promise<{ reviewCount: number; violationCount: number }> {
    if (chapter) {
      selectChapter(chapter.id);
      updateChapter(chapter.id, {
        status: "translating",
        output: "",
        thinking: "",
        violations: [],
        reviewRound: 0,
        error: undefined,
      });
    } else {
      setOutput("");
      setThinking("");
      setViolations([]);
    }
    setPhase("translating");
    setReviewRound(0);
    setStreamingOutput("");
    setStreamingThinking("");
    setOutput("");
    setThinking("");
    thinkingLogRef.current = "";
    setViolations([]);
    setOutputEditing(false);
    setActivePair(undefined);

    // Đọc story mới nhất từ store: khi chạy hàng đợi, glossary tự thêm ở
    // chương trước phải có mặt trong prompt của chương sau, closure bị cũ.
    const freshStory = useWorkspaceStore.getState().aiStory;
    const systemPrompt = buildAiTranslationSystemPrompt(glossary, freshStory);
    const paragraphs = aiParagraphsOf(sourceText);
    const sourceParagraphs = paragraphs.length;
    const rawOutput = await generate(
      systemPrompt,
      labeledAiSourcePayload(paragraphs),
      controller.signal,
      workspaceChanged,
      true,
      "Dịch",
    );
    if (workspaceChanged()) throw new DOMException("Workspace changed", "AbortError");

    let translated: string;
    const parsed = parseLabeledAiTranslation(rawOutput, sourceParagraphs);
    if (parsed) {
      const missing = parsed.flatMap((paragraph, index) =>
        paragraph === undefined ? [index] : [],
      );
      if (missing.length > 0) {
        // Nhãn cho biết chính xác đoạn nào thiếu — chỉ dịch bổ sung đúng các
        // đoạn đó thay vì retry cả chương.
        setPhase("retrying");
        setStreamingOutput("");
        // undefined → panel hiện log đã tích lũy trong lúc chờ lượt mới stream.
        setStreamingThinking(undefined);
        const repairRaw = await generate(
          systemPrompt,
          labeledAiRepairPayload(paragraphs, missing),
          controller.signal,
          workspaceChanged,
          false,
          "Dịch bổ sung",
        );
        if (workspaceChanged()) throw new DOMException("Workspace changed", "AbortError");
        const repaired = parseLabeledAiTranslation(repairRaw, sourceParagraphs);
        if (repaired) {
          for (const index of missing) parsed[index] ??= repaired[index];
        }
      }
      // Đoạn vẫn thiếu thì giữ nguyên văn tiếng Trung: rule "CJK còn sót" sẽ
      // đánh vi phạm để vòng review dịch nốt, và người đọc thấy ngay chỗ hổng
      // thay vì mất đoạn trong im lặng. Số đoạn nhờ vậy luôn khớp raw.
      translated = `${parsed
        .map((paragraph, index) => paragraph ?? paragraphs[index])
        .join("\n\n")}\n`;
    } else {
      // Model bỏ toàn bộ nhãn — dùng nguyên output, giữ retry cả chương như
      // cũ khi thiếu đoạn rõ rệt.
      translated = formatAiTranslation(stripAiParagraphMarkers(rawOutput));
      if (nonEmptyLineCount(translated) + 2 <= sourceParagraphs) {
        setPhase("retrying");
        setOutput("");
        setStreamingOutput("");
        setStreamingThinking(undefined);
        const retried = formatAiTranslation(stripAiParagraphMarkers(
          await generate(
            systemPrompt,
            labeledAiSourcePayload(paragraphs),
            controller.signal,
            workspaceChanged,
            true,
            "Dịch lại",
          ),
        ));
        if (workspaceChanged()) throw new DOMException("Workspace changed", "AbortError");
        if (nonEmptyLineCount(retried) > nonEmptyLineCount(translated)) {
          translated = retried;
        }
      }
    }
    setOutput(translated);

    let currentViolations = checkAiTranslationViolations(
      translated,
      freshStory.checkRules,
    );
    let round = 0;
    while (currentViolations.length > 0 && round < 3) {
      const previousCount = currentViolations.length;
      round += 1;
      setPhase("reviewing");
      setReviewRound(round);
      setStreamingOutput(undefined);
      setStreamingThinking(undefined);
      if (chapter) {
        updateChapter(chapter.id, { status: "reviewing", reviewRound: round });
      }
      const reviewPrompt = buildAiTranslationReviewPrompt(translated, currentViolations);
      const reviewed = formatAiTranslation(
        await generate(
          reviewPrompt.system,
          reviewPrompt.user,
          controller.signal,
          workspaceChanged,
          false,
          `Soát lần ${round}`,
        ),
      );
      if (workspaceChanged()) throw new DOMException("Workspace changed", "AbortError");
      const reviewedViolations = checkAiTranslationViolations(
        reviewed,
        freshStory.checkRules,
      );
      // Review chỉ được sửa từ/cụm; bản sửa làm đổi số đoạn là hỏng ánh xạ.
      if (aiParagraphsOf(reviewed).length !== aiParagraphsOf(translated).length) break;
      if (reviewedViolations.length >= previousCount) break;
      translated = reviewed;
      currentViolations = reviewedViolations;
      setOutput(translated);
    }

    const finalTranslatedParagraphs = nonEmptyLineCount(translated);
    if (finalTranslatedParagraphs + 2 <= sourceParagraphs) {
      currentViolations = [
        ...currentViolations,
        {
          line: 0,
          message: `Bản dịch thiếu đoạn: raw ${sourceParagraphs} đoạn, dịch có ${finalTranslatedParagraphs} đoạn`,
          text: "",
        },
      ];
    }
    setOutput(translated);
    setViolations(currentViolations);
    if (chapter) {
      updateChapter(chapter.id, {
        status: "done",
        output: translated,
        violations: currentViolations,
        reviewRound: round,
        error: undefined,
      });
    }
    await harvestStoryGlossary(sourceText, translated, chapter, controller, workspaceChanged);

    return { reviewCount: round, violationCount: currentViolations.length };
  }

  /**
   * Vòng phản hồi glossary: trích tên riêng mới từ cặp raw ↔ bản dịch rồi tự
   * nạp vào glossary truyện (chỉ thêm key mới) để chương sau dịch nhất quán.
   * Là bước phụ trợ — mọi lỗi đều nuốt, không được phép làm hỏng chương dịch.
   */
  async function harvestStoryGlossary(
    sourceText: string,
    translated: string,
    chapter: AiTranslationChapter | undefined,
    controller: AbortController,
    workspaceChanged: () => boolean,
  ) {
    if (!aiSettings.translation.autoGlossary) return;
    setPhase("extracting");
    try {
      const latestStory = useWorkspaceStore.getState().aiStory;
      const existingKeys = collectGlossaryKeys(glossary, latestStory.glossary);
      const suggestions = await extractStoryGlossaryWithAi(
        resolveAiCall(provider, providerConfig),
        sourceText,
        translated,
        [...existingKeys],
      );
      if (workspaceChanged() || controller.signal.aborted) return;
      const pairs = sanitizeExtractedGlossary(suggestions, sourceText, translated, existingKeys);
      if (pairs.length === 0) return;
      const updated = appendAutoGlossary(
        useWorkspaceStore.getState().aiStory,
        pairs,
        chapter?.filename ?? "chuong-0",
      );
      updateStory({ glossary: updated.glossary, autoGlossaryLog: updated.autoGlossaryLog });
      const preview = pairs
        .slice(0, 3)
        .map(({ source, target }) => `${source} → ${target}`)
        .join(" · ");
      toast.message(
        `${chapter?.filename ?? "chuong-0"}: +${pairs.length} tên vào từ điển truyện`,
        { description: pairs.length > 3 ? `${preview} …` : preview },
      );
    } catch {
      // Trích tên là bước bonus; chương đã dịch xong, lỗi ở đây chỉ bị bỏ qua.
    }
  }

  async function runTranslation() {
    if (phase || queueRunning) return;
    if (!source.trim()) {
      toast.error("Dán nguyên văn hoặc nhập file chương trước khi dịch AI");
      return;
    }
    if (!canCallAi()) return;
    const requestWorkspaceId = useWorkspaceCatalogStore.getState().activeWorkspaceId;
    const workspaceChanged = () =>
      useWorkspaceCatalogStore.getState().activeWorkspaceId !== requestWorkspaceId;
    const controller = new AbortController();
    abortRef.current = controller;
    const chapter = chapters.find((item) => item.id === activeChapterId);
    try {
      const result = await translateOne(chapter, source, controller, workspaceChanged);
      toast.success("Dịch AI xong", {
        description: result.violationCount
          ? `${result.reviewCount} lần soát · còn ${result.violationCount} cảnh báo`
          : `${result.reviewCount} lần soát · không còn cảnh báo`,
      });
    } catch (error) {
      if (chapter && !workspaceChanged()) {
        updateChapter(chapter.id, controller.signal.aborted || isAbortError(error)
          ? { status: "queued", error: undefined }
          : { status: "error", error: error instanceof Error ? error.message : String(error) });
      }
      if (workspaceChanged()) return;
      if (controller.signal.aborted || isAbortError(error)) toast.message("Đã dừng Dịch AI");
      else toast.error(error instanceof Error ? error.message : "Không thể dịch bằng AI");
    } finally {
      if (!workspaceChanged()) setPhase(null);
      setStreamingOutput(undefined);
      setStreamingThinking(undefined);
      if (abortRef.current === controller) abortRef.current = undefined;
    }
  }

  async function runQueue() {
    if (phase || queueRunning || !canCallAi()) return;
    const queued = useWorkspaceStore.getState().aiTranslationChapters.filter(
      (chapter) => chapter.status !== "done",
    );
    if (queued.length === 0) {
      toast.message("Không còn chương chờ dịch");
      return;
    }
    const requestWorkspaceId = useWorkspaceCatalogStore.getState().activeWorkspaceId;
    const workspaceChanged = () =>
      useWorkspaceCatalogStore.getState().activeWorkspaceId !== requestWorkspaceId;
    const controller = new AbortController();
    abortRef.current = controller;
    setQueueRunning(true);
    let completed = 0;
    let failed = 0;
    try {
      for (const chapter of queued) {
        if (controller.signal.aborted || workspaceChanged()) break;
        if (!chapter.source.trim()) {
          updateChapter(chapter.id, { status: "error", error: "Chương không có nguyên văn" });
          failed += 1;
          continue;
        }
        try {
          await translateOne(chapter, chapter.source, controller, workspaceChanged);
          completed += 1;
        } catch (error) {
          if (controller.signal.aborted || isAbortError(error) || workspaceChanged()) {
            updateChapter(chapter.id, { status: "queued", error: undefined });
            break;
          }
          updateChapter(chapter.id, {
            status: "error",
            error: error instanceof Error ? error.message : String(error),
          });
          failed += 1;
        }
      }
      if (!workspaceChanged()) {
        toast.success(controller.signal.aborted ? "Đã dừng hàng đợi" : "Đã chạy xong hàng đợi", {
          description: `${completed} chương xong${failed ? ` · ${failed} lỗi` : ""}`,
        });
      }
    } finally {
      if (!workspaceChanged()) setPhase(null);
      setStreamingOutput(undefined);
      setStreamingThinking(undefined);
      setQueueRunning(false);
      if (abortRef.current === controller) abortRef.current = undefined;
    }
  }

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (
        event.key !== "Enter" ||
        !(event.metaKey || event.ctrlKey) ||
        phase ||
        storyConfigOpen
      ) return;
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
    setViolations(checkAiTranslationViolations(value, story.checkRules));
  }

  function confirmClearChapters() {
    if (chapters.length === 0) return;
    if (!window.confirm(`Xóa ${chapters.length} chương và toàn bộ bản dịch đã lưu?`)) return;
    clearChapters();
    toast.message("Đã xóa danh sách chương khỏi workspace");
  }

  const pendingChapterCount = chapters.filter((chapter) => chapter.status !== "done").length;
  const activeChapter = chapters.find((chapter) => chapter.id === activeChapterId);
  const visibleOutput = phase && streamingOutput !== undefined ? streamingOutput : output;
  const visibleThinking = phase && streamingThinking !== undefined
    ? streamingThinking
    : thinking;

  function removeAutoGlossaryEntry(entry: AutoGlossaryEntry) {
    const current = useWorkspaceStore.getState().aiStory;
    const glossaryCopy = Object.fromEntries(
      Object.entries(current.glossary).map(([key, group]) => [key, { ...group }]),
    ) as StoryGlossary;
    delete glossaryCopy[entry.category][entry.source];
    updateStory({
      glossary: glossaryCopy,
      autoGlossaryLog: current.autoGlossaryLog.filter((item) => item.source !== entry.source),
    });
  }

  async function copyThinking() {
    try {
      await navigator.clipboard.writeText(visibleThinking);
      toast.success("Đã sao chép quá trình suy nghĩ");
    } catch {
      toast.error("Trình duyệt không cho phép sao chép");
    }
  }

  // Ánh xạ đoạn nguồn ↔ dịch theo chỉ số; chỉ khả dụng khi số đoạn hai bên
  // khớp nhau (pipeline nhãn [[n]] đảm bảo điều này cho bản dịch mới).
  const sourceParagraphRanges = useMemo(() => aiParagraphRanges(source), [source]);
  const outputParagraphRanges = useMemo(
    () => aiParagraphRanges(visibleOutput),
    [visibleOutput],
  );
  const mappingAvailable = !phase &&
    sourceParagraphRanges.length > 0 &&
    sourceParagraphRanges.length === outputParagraphRanges.length;
  const showSourceLinked = sourceLinked && mappingAvailable;

  useEffect(() => {
    if (!mappingAvailable) setActivePair(undefined);
  }, [mappingAvailable]);

  useLayoutEffect(() => {
    if (!pairScrollRequest) return;
    const container = pairScrollRequest.pane === "source"
      ? sourceScrollRef.current
      : outputScrollRef.current;
    scrollParagraphIntoView(container, pairScrollRequest.index);
  }, [pairScrollRequest]);

  function selectPair(index: number, from: MappedPane) {
    setActivePair(index);
    if (from === "output") {
      setSourceLinked(true);
      setPairScrollRequest({ pane: "source", index });
    } else {
      setPairScrollRequest({ pane: "output", index });
    }
  }

  return (
    <>
    <main className="grid min-h-0 min-w-0 grid-rows-[auto_minmax(0,1fr)_auto] gap-3 overflow-hidden p-3">
      <header className="flex min-w-0 flex-wrap items-center gap-2 rounded-md border bg-card px-3 py-2">
        <div className="min-w-0">
          <h1 className="text-sm font-semibold">Dịch AI{story.name ? ` · ${story.name}` : ""}</h1>
          <p className="truncate text-[11px] text-muted-foreground">
            {provider === "gemini" ? "Gemini" : "DeepSeek"} · {model || "chưa đặt model"}
            {` · thinking ${aiSettings.translation.thinking ? "bật" : "tắt"}`}
          </p>
        </div>
        <span className="rounded border bg-muted px-2 py-1 text-[10px] text-muted-foreground">
          {glossaryCount.toLocaleString("vi-VN")} mục từ điển
        </span>
        <div className="flex-1" />
        {story.autoGlossaryLog.length > 0 ? (
          <Button
            type="button"
            variant="ghost"
            size="sm"
            onClick={() => setAutoGlossaryOpen(true)}
          >
            <Sparkles /> {story.autoGlossaryLog.length} tên tự thêm
          </Button>
        ) : null}
        <Button type="button" variant="ghost" size="sm" onClick={() => setStoryConfigOpen(true)}>
          <BookOpen /> Cấu hình truyện
        </Button>
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
        ) : <>
          {chapters.length > 0 ? (
            <Button type="button" variant="secondary" size="sm" disabled={pendingChapterCount === 0} onClick={() => void runQueue()}>
              <ListRestart /> Dịch hàng đợi{pendingChapterCount ? ` · ${pendingChapterCount}` : ""}
            </Button>
          ) : null}
          <Button type="button" size="sm" onClick={() => void runTranslation()}>
            <Send /> {activeChapter ? "Dịch chương" : "Dịch AI"}
            <kbd className="hidden border-l border-primary-foreground/25 pl-2 text-[10px] font-normal opacity-80 sm:inline">
              Ctrl/⌘ Enter
            </kbd>
          </Button>
        </>}
      </header>

      <div className="grid min-h-0 overflow-y-auto rounded-lg border bg-card lg:grid-cols-[230px_minmax(0,1fr)_minmax(0,1fr)] lg:overflow-hidden lg:divide-x">
        <aside className="flex min-h-[220px] min-w-0 flex-col border-b lg:min-h-0 lg:border-b-0">
            <header className="flex h-10 shrink-0 items-center gap-2 border-b px-3">
              <strong className="text-xs">Chương</strong>
              <span className="text-[10px] text-muted-foreground">{chapters.length || 1}</span>
              <div className="flex-1" />
              <Button
                type="button"
                variant="ghost"
                size="icon-sm"
                aria-label="Xóa toàn bộ chương"
                disabled={Boolean(phase) || chapters.length === 0}
                onClick={confirmClearChapters}
              >
                <Trash2 />
              </Button>
            </header>
            <div className="fine-scrollbar min-h-0 flex-1 overflow-y-auto p-1.5">
              {chapters.length === 0 ? (
                // Buffer dán tay hiện như một chương mặc định để layout luôn
                // ổn định; nhập file là danh sách thật thay chỗ.
                <div className="flex items-center rounded-md bg-accent">
                  <div className="flex min-w-0 flex-1 items-center gap-2 px-2 py-2">
                    {output ? (
                      violations.length > 0 ? (
                        <AlertTriangle className="size-3.5 shrink-0 text-amber-600 dark:text-amber-300" />
                      ) : (
                        <CheckCircle2 className="size-3.5 shrink-0 text-ok" />
                      )
                    ) : (
                      <Circle className="size-3.5 shrink-0 text-muted-foreground" />
                    )}
                    <span className="min-w-0 flex-1">
                      <span className="block truncate text-xs font-medium">chuong-0</span>
                      <span className="block truncate text-[10px] text-muted-foreground">
                        {output
                          ? `${violations.length} cảnh báo`
                          : "Dán nguyên văn hoặc nhập file"}
                      </span>
                    </span>
                  </div>
                </div>
              ) : null}
              {chapters.map((chapter) => (
                <div
                  key={chapter.id}
                  className={cn(
                    "group flex items-center rounded-md",
                    chapter.id === activeChapterId ? "bg-accent" : "hover:bg-muted/70",
                  )}
                >
                  <button
                    type="button"
                    disabled={Boolean(phase)}
                    className="flex min-w-0 flex-1 items-center gap-2 px-2 py-2 text-left disabled:cursor-default"
                    onClick={() => selectChapter(chapter.id)}
                  >
                    {chapter.status === "done" ? (
                      <CheckCircle2 className="size-3.5 shrink-0 text-ok" />
                    ) : chapter.status === "translating" || chapter.status === "reviewing" ? (
                      <LoaderCircle className="size-3.5 shrink-0 animate-spin text-primary" />
                    ) : chapter.status === "error" ? (
                      <AlertTriangle className="size-3.5 shrink-0 text-destructive" />
                    ) : (
                      <Circle className="size-3.5 shrink-0 text-muted-foreground" />
                    )}
                    <span className="min-w-0 flex-1">
                      <span className="block truncate text-xs font-medium">{chapter.filename}</span>
                      <span className="block truncate text-[10px] text-muted-foreground">
                        {chapter.status === "done"
                          ? `${chapter.violations.length} cảnh báo`
                          : chapter.error || chapterStatusLabel(chapter)}
                      </span>
                    </span>
                  </button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon-sm"
                    className="mr-1 opacity-0 group-hover:opacity-100 focus-visible:opacity-100"
                    aria-label={`Xóa ${chapter.filename}`}
                    disabled={Boolean(phase)}
                    onClick={() => {
                      if (window.confirm(`Xóa ${chapter.filename} và bản dịch đã lưu?`)) {
                        removeChapter(chapter.id);
                      }
                    }}
                  >
                    <Trash2 />
                  </Button>
                </div>
              ))}
              <button
                type="button"
                disabled={Boolean(phase)}
                onClick={() => fileInputRef.current?.click()}
                className="mt-1.5 flex w-full items-center justify-center gap-2 rounded-md border border-dashed px-3 py-2.5 text-xs font-medium text-muted-foreground transition-colors hover:border-primary/50 hover:bg-primary/5 hover:text-primary disabled:pointer-events-none disabled:opacity-50"
              >
                <FileUp className="size-4" /> Nhập chương từ file
              </button>
            </div>
            {chapters.length > 0 ? (
              <footer className="flex h-8 shrink-0 items-center justify-between border-t px-3 text-[10px] text-muted-foreground">
                <span>{chapters.filter((chapter) => chapter.status === "done").length} xong</span>
                <span>{pendingChapterCount} chờ</span>
              </footer>
            ) : null}
          </aside>
        <section className="grid min-h-[280px] min-w-0 grid-rows-[40px_minmax(0,1fr)_32px] lg:min-h-0">
          <header className="flex items-center gap-2 border-b px-3">
            <strong className="min-w-0 truncate text-xs">
              {activeChapter ? activeChapter.filename : "Nguyên văn (CN)"}
            </strong>
            <div className="flex-1" />
            <input
              ref={fileInputRef}
              type="file"
              accept=".txt,.md,text/plain,text/markdown"
              multiple
              aria-label="Chọn tệp chương"
              className="hidden"
              onChange={(event) => {
                const files = Array.from(event.target.files ?? []);
                if (files.length > 0) void importChapterFiles(files);
                event.target.value = "";
              }}
            />
            <Button
              type="button"
              variant="ghost"
              size="sm"
              title="Nhập nhiều file chương vào hàng đợi Dịch AI"
              onClick={() => fileInputRef.current?.click()}
            >
              <FileUp /> Nhập chương
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="sm"
              title="Xóa nguyên văn và bản dịch của khung Dịch AI"
              disabled={!source && !output}
              onClick={clearSource}
            >
              <Eraser /> Xóa
            </Button>
            <Tabs
              value={showSourceLinked ? "linked" : "raw"}
              onValueChange={(value) => setSourceLinked(value === "linked")}
            >
              <TabsList className="h-7">
                <TabsTrigger value="raw" className="text-[11px]">Gốc</TabsTrigger>
                <TabsTrigger value="linked" disabled={!mappingAvailable} className="text-[11px]">
                  Đối chiếu
                </TabsTrigger>
              </TabsList>
            </Tabs>
          </header>
          <div
            className={cn("relative min-h-0", dropActive && "bg-primary/5")}
            {...dropHandlers}
          >
            {showSourceLinked ? (
              <div
                ref={sourceScrollRef}
                lang="zh-Hans"
                className="fine-scrollbar absolute inset-0 overflow-auto"
              >
                <MappedText
                  text={source}
                  ranges={sourceParagraphRanges}
                  activeRange={activePair}
                  onRangeSelect={(index) => selectPair(index, "source")}
                  emptyMessage="Chưa có nguyên văn."
                  className="min-h-full px-6 py-5 font-serif text-[18px] leading-8"
                />
              </div>
            ) : (
              <Textarea
                ref={sourceTextareaRef}
                value={source}
                onChange={(event) => setSource(event.target.value)}
                disabled={Boolean(phase)}
                lang="zh-Hans"
                spellCheck={false}
                placeholder="Dán nguyên văn tiếng Trung hoặc thả nhiều file chương vào đây…"
                className="absolute inset-0 h-full min-h-0 resize-none rounded-none border-0 bg-transparent px-6 py-5 font-serif text-[18px] leading-8 shadow-none focus-visible:ring-0"
              />
            )}
            {dropActive ? (
              <div className="pointer-events-none absolute inset-3 z-10 grid place-items-center rounded-md border-2 border-dashed border-primary bg-background/90 text-sm font-medium text-primary">
                Thả các file .txt để thêm vào hàng đợi
              </div>
            ) : null}
          </div>
          <footer className="flex items-center justify-between border-t px-3 text-[10px] text-muted-foreground">
            <span>{source.length.toLocaleString("vi-VN")} ký tự</span>
            <span>{nonEmptyLineCount(source).toLocaleString("vi-VN")} đoạn</span>
          </footer>
        </section>

        <section className="flex min-h-[320px] min-w-0 flex-col lg:min-h-0">
          <header className="flex h-10 shrink-0 items-center gap-2 border-b px-3">
            <strong className="text-xs">Bản dịch (VN)</strong>
            {phase ? <LoaderCircle className="size-3.5 animate-spin text-primary" /> : null}
            <span className="truncate text-[11px] text-muted-foreground">
              {phaseLabel(phase, reviewRound)}
            </span>
            <div className="flex-1" />
            <Button
              type="button"
              variant={outputEditing ? "secondary" : "ghost"}
              size="sm"
              title="Bản dịch mặc định chỉ đọc — bật Sửa để chỉnh trực tiếp"
              disabled={Boolean(phase)}
              onClick={() => setOutputEditing((value) => !value)}
            >
              <Pencil /> {outputEditing ? "Xong" : "Sửa"}
            </Button>
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              aria-label="Sao chép bản dịch AI"
              disabled={!output || Boolean(phase)}
              onClick={() => void copyOutput()}
            >
              <Copy />
            </Button>
          </header>

          {visibleThinking ? (
            <div className="shrink-0 border-b border-thinking/20 bg-thinking/5">
              <div className="flex w-full items-center gap-1 px-3 py-1 text-[11px] text-thinking">
                <button
                  type="button"
                  className="flex min-w-0 flex-1 items-center gap-2 py-0.5 text-left"
                  onClick={() => setThinkingExpanded((value) => !value)}
                >
                  <Brain className={cn("size-3.5", phase && "animate-pulse")} />
                  <span className="font-medium">Quá trình suy nghĩ</span>
                  <span className="ml-auto">{thinkingExpanded ? "Ẩn" : "Hiện"}</span>
                </button>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon-sm"
                  aria-label="Sao chép quá trình suy nghĩ"
                  onClick={() => void copyThinking()}
                >
                  <Copy />
                </Button>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon-sm"
                  aria-label="Mở quá trình suy nghĩ"
                  onClick={() => setThinkingDialogOpen(true)}
                >
                  <Maximize2 />
                </Button>
              </div>
              {thinkingExpanded ? (
                <ThinkingLogView
                  text={visibleThinking}
                  className="max-h-28 px-3 pb-2 text-[10px] leading-relaxed text-thinking/70"
                />
              ) : null}
            </div>
          ) : null}

          <div className="relative min-h-0 flex-1 bg-reader-paper">
            {phase || outputEditing ? (
              <Textarea
                value={visibleOutput}
                onChange={(event) => updateOutput(event.target.value)}
                disabled={Boolean(phase)}
                lang="vi"
                spellCheck
                placeholder={phase ? "AI đang dịch…" : "Bản dịch AI sẽ xuất hiện ở đây."}
                className="absolute inset-0 h-full min-h-0 resize-none rounded-none border-0 bg-reader-paper px-7 py-5 font-serif text-[18px] leading-8 text-reader-ink shadow-none focus-visible:ring-0"
              />
            ) : mappingAvailable && visibleOutput ? (
              <div
                ref={outputScrollRef}
                lang="vi"
                className="fine-scrollbar absolute inset-0 overflow-auto"
              >
                <MappedText
                  text={visibleOutput}
                  ranges={outputParagraphRanges}
                  activeRange={activePair}
                  onRangeSelect={(index) => selectPair(index, "output")}
                  emptyMessage="Bản dịch AI sẽ xuất hiện ở đây."
                  className="min-h-full px-7 py-5 font-serif text-[18px] leading-8 text-reader-ink"
                />
              </div>
            ) : (
              <div className="fine-scrollbar absolute inset-0 overflow-auto px-7 py-5 font-serif text-[18px] leading-8 whitespace-pre-wrap text-reader-ink">
                {visibleOutput || (
                  <span className="font-sans text-sm text-muted-foreground">
                    Bản dịch AI sẽ xuất hiện ở đây · mặc định chỉ đọc, bấm Sửa để chỉnh trực tiếp.
                  </span>
                )}
              </div>
            )}
          </div>
          <footer className="flex h-8 shrink-0 items-center justify-between border-t px-3 text-[10px] text-muted-foreground">
            <span>{wordCount(visibleOutput).toLocaleString("vi-VN")} chữ</span>
            <span>{nonEmptyLineCount(visibleOutput).toLocaleString("vi-VN")} đoạn</span>
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
            Tự lưu truyện, chương, prompt, từ điển và kết quả vào IndexedDB.
          </span>
        )}
      </footer>
    </main>
    {storyConfigOpen ? (
      <AiStoryConfigDialog
        open
        onOpenChange={setStoryConfigOpen}
        story={story}
        aiSettings={aiSettings}
        onSave={(value) => updateStory(value)}
      />
    ) : null}
    {autoGlossaryOpen ? (
      <Dialog open onOpenChange={setAutoGlossaryOpen}>
        <DialogContent className="flex max-h-[80vh] flex-col gap-0 overflow-hidden p-0 sm:max-w-xl">
          <DialogHeader className="shrink-0 border-b px-6 py-4">
            <DialogTitle className="flex items-center gap-2.5 text-sm">
              <Sparkles className="size-4 text-thinking" />
              Tên tự thêm vào từ điển truyện
            </DialogTitle>
          </DialogHeader>
          <div className="fine-scrollbar min-h-0 flex-1 overflow-y-auto px-6 py-4">
            {story.autoGlossaryLog.length === 0 ? (
              <p className="text-sm text-muted-foreground">Đã gỡ hết các tên tự thêm.</p>
            ) : (
              <ul className="space-y-1">
                {story.autoGlossaryLog.map((entry) => (
                  <li key={entry.source} className="flex items-center gap-2 rounded-md px-1 py-0.5 text-sm hover:bg-accent/50">
                    <span className="shrink-0 font-medium">{entry.source}</span>
                    <span className="text-muted-foreground">→</span>
                    <span className="min-w-0 flex-1 truncate">{entry.target}</span>
                    <span className="shrink-0 text-[10px] text-muted-foreground">
                      {glossaryCategoryLabels[entry.category] ?? entry.category}
                      {entry.chapter ? ` · ${entry.chapter}` : ""}
                    </span>
                    <Button
                      type="button"
                      variant="ghost"
                      size="icon-sm"
                      aria-label={`Gỡ ${entry.source}`}
                      onClick={() => removeAutoGlossaryEntry(entry)}
                    >
                      <Trash2 />
                    </Button>
                  </li>
                ))}
              </ul>
            )}
            <p className="pt-3 text-[11px] text-muted-foreground">
              Tên tự trích từ bản dịch xong, chỉ thêm mục chưa có — muốn sửa target thì vào Cấu hình truyện, gỡ ở đây sẽ xóa khỏi từ điển truyện.
            </p>
          </div>
        </DialogContent>
      </Dialog>
    ) : null}
    {thinkingDialogOpen ? (
      <Dialog open onOpenChange={setThinkingDialogOpen}>
        <DialogContent className="flex h-[85vh] flex-col gap-0 overflow-hidden p-0 sm:max-w-2xl">
          <DialogHeader className="shrink-0 border-b px-6 py-4">
            <DialogTitle className="flex items-center gap-2.5 text-sm">
              <span className="flex size-7 items-center justify-center rounded-full bg-thinking">
                <Brain className={cn("size-4 text-thinking-foreground", phase && "animate-pulse")} />
              </span>
              Quá trình suy nghĩ
              {phase ? (
                <span className="font-normal text-muted-foreground">
                  · {phaseLabel(phase, reviewRound)}
                </span>
              ) : null}
            </DialogTitle>
          </DialogHeader>
          <ThinkingRichView
            text={visibleThinking}
            className="min-h-0 flex-1 bg-reader-paper px-6 py-5"
          />
          <div className="flex shrink-0 items-center justify-between border-t px-6 py-3">
            <span className="text-[11px] text-muted-foreground">
              {wordCount(visibleThinking).toLocaleString("vi-VN")} chữ
            </span>
            <Button type="button" size="sm" onClick={() => void copyThinking()}>
              <Copy /> Sao chép
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    ) : null}
    </>
  );
}

/**
 * Bám đáy khi stream: chỉ nhả khi người dùng thật sự cuộn lên (scrollTop
 * giảm) — content mọc thêm không được phép nhả, tránh auto scroll chết ngẫu
 * nhiên khi scroll event chen giữa chunk mới và effect.
 */
function usePinnedScroll<T extends HTMLElement>(dep: unknown) {
  const ref = useRef<T>(null);
  const pinnedRef = useRef(true);
  const lastScrollTopRef = useRef(0);
  useLayoutEffect(() => {
    const node = ref.current;
    if (node && pinnedRef.current) node.scrollTop = node.scrollHeight;
  }, [dep]);
  function onScroll(event: React.UIEvent<T>) {
    const node = event.currentTarget;
    pinnedRef.current = thinkingPinAfterScroll(
      pinnedRef.current,
      lastScrollTopRef.current,
      node,
    );
    lastScrollTopRef.current = node.scrollTop;
  }
  return { ref, onScroll };
}

/** Dải thinking gọn trong panel bản dịch. */
function ThinkingLogView({ text, className }: { text: string; className?: string }) {
  const { ref, onScroll } = usePinnedScroll<HTMLPreElement>(text);
  return (
    <pre
      ref={ref}
      onScroll={onScroll}
      className={cn("fine-scrollbar overflow-auto whitespace-pre-wrap font-mono", className)}
    >
      {text}
    </pre>
  );
}

/** Đoạn text thinking với **đậm** inline của model render thành strong thật. */
function thinkingInline(text: string): React.ReactNode[] {
  return text.split(/\*\*(.+?)\*\*/gs).map((part, index) =>
    index % 2 === 1 ? (
      <strong key={index} className="font-semibold text-reader-ink">{part}</strong>
    ) : (
      part
    ),
  );
}

/**
 * Bản trình bày đầy đủ cho dialog: mỗi lượt gọi model một section có chip
 * nhãn, dòng `**Tiêu đề**` của Gemini thành heading, còn lại là prose.
 */
function ThinkingRichView({ text, className }: { text: string; className?: string }) {
  const { ref, onScroll } = usePinnedScroll<HTMLDivElement>(text);
  const segments = parseThinkingLog(text);
  return (
    <div
      ref={ref}
      onScroll={onScroll}
      className={cn("fine-scrollbar overflow-y-auto", className)}
    >
      <div className="space-y-7 pb-2">
        {segments.map((segment, segmentIndex) => (
          <section key={segmentIndex}>
            {segment.label ? (
              <div className="mb-4 flex items-center gap-3">
                <span className="shrink-0 rounded-full bg-thinking px-3 py-1 text-[11px] font-semibold tracking-wide text-thinking-foreground">
                  {segment.label}
                </span>
                <div className="h-px flex-1 bg-gradient-to-r from-thinking/40 to-transparent" />
              </div>
            ) : null}
            <div className="space-y-3.5">
              {segment.text.split(/\n{2,}/).map((block, blockIndex) => {
                const heading = /^\*\*(.+?)\*\*$/s.exec(block.trim());
                return heading ? (
                  <h4
                    key={blockIndex}
                    className="flex items-center gap-2 pt-2.5 text-[15px] font-semibold tracking-tight text-reader-ink"
                  >
                    <span className="size-1.5 shrink-0 rounded-full bg-thinking" aria-hidden />
                    {heading[1]}
                  </h4>
                ) : (
                  <p
                    key={blockIndex}
                    className="whitespace-pre-wrap pl-3.5 text-sm leading-7 text-reader-ink/85"
                  >
                    {thinkingInline(block)}
                  </p>
                );
              })}
            </div>
          </section>
        ))}
      </div>
    </div>
  );
}

function chapterStatusLabel(chapter: AiTranslationChapter): string {
  if (chapter.status === "translating") return "Đang dịch";
  if (chapter.status === "reviewing") return `Đang soát · lần ${chapter.reviewRound}/3`;
  if (chapter.status === "error") return "Lỗi";
  if (chapter.status === "done") return "Đã dịch";
  return "Chờ dịch";
}

function violationSummary(violations: TranslationViolation[]): string {
  return Array.from(new Set(violations.map((item) => item.message))).slice(0, 2).join(" · ");
}
