import {
  Braces,
  Copy,
  Eraser,
  FileText,
  LoaderCircle,
  Pin,
  PinOff,
  RotateCcw,
  Send,
  Sparkles,
} from "lucide-react";
import {
  type KeyboardEvent,
  type MouseEvent,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { toast } from "sonner";

import {
  DictionaryUpdateDialog,
  type DictionaryUpdateSelection,
} from "@/components/dictionary-update-dialog";
import { MappedText } from "@/components/mapped-text";
import { Button } from "@/components/ui/button";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { buildTextSegments, rangeText } from "@/lib/ranges";
import { candidateContext } from "@/lib/ai-client";
import type { AiSettings } from "@/lib/ai-settings";
import type { DictionaryUpdateKey } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useWorkspaceStore } from "@/store/workspace";

interface TranslationWorkspaceProps {
  endpoint: string;
  canTranslate: boolean;
  isPending: boolean;
  onTranslate: () => void;
  aiSettings?: AiSettings;
  onOpenSettings?: () => void;
  requestStatus: string;
}

type MappedPane = "source" | "output";

interface ScrollRequest {
  pane: MappedPane;
  rangeIndex: number;
}

function scrollRangeIntoView(container: HTMLDivElement | null, rangeIndex: number) {
  if (!container) return;
  const target = container.querySelector<HTMLElement>(
    `[data-range-index="${rangeIndex}"]`,
  );
  if (!target) return;

  const containerRect = container.getBoundingClientRect();
  const targetRect = target.getBoundingClientRect();
  const targetTop = container.scrollTop + targetRect.top - containerRect.top;
  const centeredTop = targetTop - (container.clientHeight - targetRect.height) / 2;
  const reduceMotion =
    typeof window.matchMedia === "function" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  container.scrollTo({
    top: Math.max(0, centeredTop),
    behavior: reduceMotion ? "auto" : "smooth",
  });
}

export function TranslationWorkspace({
  endpoint,
  canTranslate,
  isPending,
  onTranslate,
  aiSettings,
  onOpenSettings,
  requestStatus,
}: TranslationWorkspaceProps) {
  const sourceText = useWorkspaceStore((state) => state.sourceText);
  const response = useWorkspaceStore((state) => state.response);
  const activeRange = useWorkspaceStore((state) => state.activeRange);
  const sourceView = useWorkspaceStore((state) => state.sourceView);
  const outputView = useWorkspaceStore((state) => state.outputView);
  const rangePinEnabled = useWorkspaceStore((state) => state.rangePinEnabled);
  const setSourceText = useWorkspaceStore((state) => state.setSourceText);
  const setActiveRange = useWorkspaceStore((state) => state.setActiveRange);
  const setSourceView = useWorkspaceStore((state) => state.setSourceView);
  const setOutputView = useWorkspaceStore((state) => state.setOutputView);
  const setRangePinEnabled = useWorkspaceStore((state) => state.setRangePinEnabled);
  const clearWorkspace = useWorkspaceStore((state) => state.clearWorkspace);
  const loadSample = useWorkspaceStore((state) => state.loadSample);
  const localDictionaryEntries = useWorkspaceStore(
    (state) => state.localDictionaryEntries,
  );
  const saveLocalDictionaryEntries = useWorkspaceStore(
    (state) => state.saveLocalDictionaryEntries,
  );
  const removeLocalDictionaryEntries = useWorkspaceStore(
    (state) => state.removeLocalDictionaryEntries,
  );
  const [mobilePane, setMobilePane] = useState<"source" | "output">("source");
  const [selectedOutputRangeIndices, setSelectedOutputRangeIndices] = useState<number[]>([]);
  const [scrollRequest, setScrollRequest] = useState<ScrollRequest>();
  const [contextSelection, setContextSelection] =
    useState<DictionaryUpdateSelection>();
  const [dictionaryUpdateKey, setDictionaryUpdateKey] =
    useState<DictionaryUpdateKey>();
  const sourceScrollRef = useRef<HTMLDivElement>(null);
  const sourceTextareaRef = useRef<HTMLTextAreaElement>(null);
  const outputScrollRef = useRef<HTMLDivElement>(null);
  const outputSelectionAnchorRef = useRef<number | undefined>(undefined);
  const outputSelectionFocusRef = useRef<number | undefined>(undefined);

  const sourceRanges = response?.sourceRanges ?? [];
  const targetRanges = useMemo(() => response?.targetRanges ?? [], [response?.targetRanges]);
  const selectableTargetRangeIndices = useMemo(
    () =>
      buildTextSegments(response?.translated ?? "", targetRanges).flatMap((segment) =>
        segment.kind === "mapped" ? [segment.rangeIndex] : [],
      ),
    [response?.translated, targetRanges],
  );
  const activeSource = rangeText(sourceText, activeRange === undefined ? undefined : sourceRanges[activeRange]);
  const activeTarget = rangeText(response?.translated ?? "", activeRange === undefined ? undefined : targetRanges[activeRange]);
  const hasMapping = Boolean(response && sourceRanges.length > 0 && targetRanges.length > 0);

  useLayoutEffect(() => {
    if (!scrollRequest) return;
    const container =
      scrollRequest.pane === "source" ? sourceScrollRef.current : outputScrollRef.current;
    scrollRangeIntoView(container, scrollRequest.rangeIndex);
  }, [scrollRequest]);

  function selectRange(rangeIndex: number, pane: MappedPane) {
    setActiveRange(rangeIndex);
    if (pane === "output") {
      setSelectedOutputRangeIndices([rangeIndex]);
      outputSelectionAnchorRef.current = rangeIndex;
      outputSelectionFocusRef.current = rangeIndex;
    } else {
      setSelectedOutputRangeIndices([]);
      outputSelectionAnchorRef.current = undefined;
      outputSelectionFocusRef.current = undefined;
    }
    if (!rangePinEnabled) return;

    if (pane === "source") {
      setOutputView("output");
      setMobilePane("output");
      setScrollRequest({ pane: "output", rangeIndex });
    } else {
      setSourceView("linked");
      setMobilePane("source");
      setScrollRequest({ pane: "source", rangeIndex });
    }
  }

  function extendOutputRangeSelection(
    event: KeyboardEvent<HTMLSpanElement>,
    rangeIndex: number,
  ) {
    if (!event.shiftKey || (event.key !== "ArrowLeft" && event.key !== "ArrowRight")) return;

    const anchor = selectableTargetRangeIndices.includes(outputSelectionAnchorRef.current ?? -1)
      ? outputSelectionAnchorRef.current ?? rangeIndex
      : rangeIndex;
    const focus = selectableTargetRangeIndices.includes(outputSelectionFocusRef.current ?? -1)
      ? outputSelectionFocusRef.current ?? rangeIndex
      : rangeIndex;
    const anchorPosition = selectableTargetRangeIndices.indexOf(anchor);
    const focusPosition = selectableTargetRangeIndices.indexOf(focus);
    const nextFocusPosition = focusPosition + (event.key === "ArrowLeft" ? -1 : 1);
    if (
      anchorPosition < 0 ||
      focusPosition < 0 ||
      nextFocusPosition < 0 ||
      nextFocusPosition >= selectableTargetRangeIndices.length
    ) return;

    event.preventDefault();
    const nextFocus = selectableTargetRangeIndices[nextFocusPosition];
    outputSelectionAnchorRef.current = anchor;
    outputSelectionFocusRef.current = nextFocus;
    const start = Math.min(anchorPosition, nextFocusPosition);
    const end = Math.max(anchorPosition, nextFocusPosition);
    const next = selectableTargetRangeIndices.slice(start, end + 1);
    setSelectedOutputRangeIndices(next);
    setActiveRange(anchor);
  }

  function toggleRangePin() {
    const enabled = !rangePinEnabled;
    setRangePinEnabled(enabled);
    if (!enabled) setScrollRequest(undefined);
  }

  async function copyOutput() {
    if (!response?.translated) return;
    try {
      await navigator.clipboard.writeText(response.translated);
      toast.success("Đã sao chép bản dịch");
    } catch {
      toast.error("Trình duyệt không cho phép sao chép");
    }
  }

  function captureOutputSelection(event: MouseEvent<HTMLDivElement>) {
    const target = event.target;
    const rangeElement =
      target instanceof Element
        ? target.closest<HTMLElement>("[data-range-index]")
        : null;
    if (!rangeElement || !event.currentTarget.contains(rangeElement)) {
      setContextSelection(undefined);
      return;
    }
    const clickedRangeIndex = Number(rangeElement.dataset.rangeIndex);
    const selection = window.getSelection();
    const browserRange =
      selection &&
      !selection.isCollapsed &&
      selection.rangeCount > 0 &&
      event.currentTarget.contains(selection.getRangeAt(0).commonAncestorContainer)
        ? selection.getRangeAt(0)
        : undefined;
    const hasKeyboardSelection =
      selectedOutputRangeIndices.length > 1 &&
      selectedOutputRangeIndices.includes(clickedRangeIndex);
    const selectedRangeIndices = hasKeyboardSelection
      ? selectedOutputRangeIndices
      : browserRange
      ? Array.from(event.currentTarget.querySelectorAll<HTMLElement>("[data-range-index]"))
          .filter((element) => browserRange.intersectsNode(element))
          .map((element) => Number(element.dataset.rangeIndex))
          .filter((rangeIndex) => Number.isInteger(rangeIndex))
          .filter((rangeIndex, index, values) => values.indexOf(rangeIndex) === index)
          .sort((left, right) => (targetRanges[left]?.start ?? 0) - (targetRanges[right]?.start ?? 0))
      : selectedOutputRangeIndices.includes(clickedRangeIndex)
        ? selectedOutputRangeIndices
        : [clickedRangeIndex];
    const selectedRanges = selectedRangeIndices
      .map((rangeIndex) => ({
        source: sourceRanges[rangeIndex],
        target: targetRanges[rangeIndex],
      }))
      .filter(({ source, target }) => source && target);
    if (selectedRanges.length === 0) {
      setContextSelection(undefined);
      return;
    }

    const sourceStart = Math.min(...selectedRanges.map(({ source }) => source.start));
    const sourceEnd = Math.max(...selectedRanges.map(({ source }) => source.start + source.length));
    const targetStart = Math.min(...selectedRanges.map(({ target }) => target.start));
    const targetEnd = Math.max(...selectedRanges.map(({ target }) => target.start + target.length));
    const selectedSource = sourceText.slice(sourceStart, sourceEnd).trim();
    const partialTarget =
      selectedRangeIndices.length === 1 && browserRange
        ? selection?.toString().trim() ?? ""
        : "";
    const selectedTarget =
      partialTarget || response?.translated.slice(targetStart, targetEnd).trim() || "";
    if (!selectedSource || !selectedTarget) {
      setContextSelection(undefined);
      return;
    }

    setActiveRange(selectedRangeIndices[0] ?? clickedRangeIndex);
    setContextSelection({
      source: selectedSource,
      target: selectedTarget,
    });
  }

  function beginDictionaryUpdate(key: DictionaryUpdateKey) {
    if (!contextSelection) return;
    setDictionaryUpdateKey(key);
  }

  async function copyContextText(text: string, label: string) {
    try {
      await navigator.clipboard.writeText(text);
      toast.success(label);
    } catch {
      toast.error("Trình duyệt không cho phép sao chép");
    }
  }

  // Xóa nhanh để dán chương mới: chỉ xóa nguyên văn (nút "Xóa" ở thanh dưới
  // mới reset cả bản dịch), focus lại ô nhập và cho hoàn tác qua toast để
  // không mất chương dài vì một cú bấm nhầm.
  function clearSourceForNewPaste() {
    const previous = sourceText;
    setSourceText("");
    setSourceView("raw");
    requestAnimationFrame(() => sourceTextareaRef.current?.focus());
    toast.message("Đã xóa nguyên văn", {
      action: { label: "Hoàn tác", onClick: () => setSourceText(previous) },
    });
  }

  // Bộ chuyển khung chỉ tồn tại dưới lg, nơi mỗi lần chỉ hiện được một khung.
  const paneTabs = (
    <Tabs value={mobilePane} onValueChange={(value) => setMobilePane(value as "source" | "output")} className="lg:hidden">
      <TabsList className="h-8">
        <TabsTrigger value="source" className="text-xs"><FileText /> Nguyên văn</TabsTrigger>
        <TabsTrigger value="output" className="text-xs"><Sparkles /> Bản dịch</TabsTrigger>
      </TabsList>
    </Tabs>
  );

  return (
    // Bản dịch tiếng Việt dài hơn nguyên văn tiếng Trung khoảng một phần ba,
    // nên chia đôi đều làm khung phải luôn chật hơn — cho nó rộng hơn.
    <main className="grid min-h-0 min-w-0 grid-rows-[minmax(0,1fr)_44px] bg-background px-3 pt-3 pb-1 md:px-4 md:pt-4 md:pb-2">
      <div className="grid min-h-0 min-w-0 overflow-hidden rounded-lg border bg-card shadow-[var(--shadow-panel)] lg:grid-cols-[minmax(0,0.88fr)_minmax(0,1.12fr)]">
        <section className={cn("min-h-0 min-w-0 grid-rows-[40px_minmax(0,1fr)_44px]", mobilePane === "source" ? "grid" : "hidden lg:grid")} aria-label="Nguyên văn">
          <header className="flex items-center justify-between gap-3 border-b pr-2 pl-4">
            <strong className="hidden text-xs lg:block">Nguyên văn</strong>
            {paneTabs}
            <div className="flex items-center gap-1">
              <Button
                type="button"
                variant="ghost"
                size="sm"
                disabled={!sourceText}
                aria-label="Xóa nguyên văn để dán chương mới"
                title="Xóa nguyên văn để dán chương mới"
                onClick={clearSourceForNewPaste}
              >
                <Eraser /> Xóa
              </Button>
              <Tabs value={sourceView} onValueChange={(value) => setSourceView(value as "raw" | "linked")}>
                <TabsList className="h-7">
                  <TabsTrigger value="raw" className="text-[11px]">Gốc</TabsTrigger>
                  <TabsTrigger value="linked" disabled={!response} className="text-[11px]">Đối chiếu</TabsTrigger>
                </TabsList>
              </Tabs>
            </div>
          </header>
          <div ref={sourceScrollRef} lang="zh-Hans" className="fine-scrollbar relative min-h-0 overflow-auto">
            {sourceView === "raw" ? (
              <Textarea
                ref={sourceTextareaRef}
                value={sourceText}
                onChange={(event) => setSourceText(event.target.value)}
                placeholder="Dán một chương tiếng Trung vào đây…"
                aria-label="Nguyên văn tiếng Trung"
                className="h-full min-h-full resize-none rounded-none border-0 bg-transparent px-7 py-6 text-[18px] leading-9 shadow-none focus-visible:ring-0"
              />
            ) : (
              <MappedText
                text={sourceText}
                ranges={sourceRanges}
                activeRange={activeRange}
                onRangeSelect={(rangeIndex) => selectRange(rangeIndex, "source")}
                emptyMessage="Chưa có nguyên văn."
                className="min-h-full px-7 py-6 text-[18px] leading-9"
              />
            )}
          </div>
          <footer className="flex items-center justify-between gap-3 border-t bg-card/95 px-3 backdrop-blur">
            <span className="shrink-0 text-[11px] text-muted-foreground">
              <strong className="font-mono font-medium text-foreground">
                {sourceText.length.toLocaleString("vi-VN")}
              </strong>{" "}
              ký tự
            </span>
            <Button
              type="button"
              size="sm"
              disabled={!canTranslate}
              title="Dịch chương (Ctrl/⌘ + Enter)"
              onClick={onTranslate}
            >
              {isPending ? <LoaderCircle className="animate-spin" /> : <Send />}
              <span>Dịch chương</span>
              <kbd className="hidden border-l border-primary-foreground/25 pl-2 font-sans text-[10px] font-normal opacity-80 sm:inline">
                Ctrl/⌘ Enter
              </kbd>
            </Button>
          </footer>
        </section>

        <section className={cn("min-h-0 min-w-0 grid-rows-[40px_minmax(0,1fr)] border-l", mobilePane === "output" ? "grid border-l-0" : "hidden lg:grid")} aria-label="Bản dịch">
          <header className="flex items-center justify-between gap-2 border-b pr-2 pl-4">
            <strong className="hidden text-xs lg:block">Bản dịch</strong>
            {paneTabs}
            <div className="flex items-center gap-1">
              {/* Pin thuộc về hành vi của khung này, nên đặt ngay tại đây thay
                  vì nằm trên một dải riêng chạy ngang toàn màn hình. */}
              <Button
                type="button"
                variant={rangePinEnabled ? "secondary" : "ghost"}
                size="icon-sm"
                aria-label={rangePinEnabled ? "Tắt tự cuộn range" : "Bật tự cuộn range"}
                aria-pressed={rangePinEnabled}
                title={
                  rangePinEnabled
                    ? "Đang bật: click một cặp sẽ cuộn tới cặp tương ứng"
                    : "Đang tắt: click chỉ làm nổi bật cặp tương ứng"
                }
                onClick={toggleRangePin}
              >
                {rangePinEnabled ? <Pin /> : <PinOff />}
              </Button>
              <Tabs value={outputView} onValueChange={(value) => setOutputView(value as "output" | "json")}>
                <TabsList className="h-7">
                  <TabsTrigger value="output" className="text-[11px]">Văn bản</TabsTrigger>
                  <TabsTrigger value="json" disabled={!response} className="text-[11px]">JSON</TabsTrigger>
                </TabsList>
              </Tabs>
            </div>
          </header>
          <div ref={outputScrollRef} lang="vi" className="fine-scrollbar relative min-h-0 overflow-auto bg-reader-paper">
            {outputView === "json" ? (
              <pre className="min-h-full overflow-auto bg-code p-6 font-mono text-xs leading-6 text-code-foreground">
                {response ? JSON.stringify(response, null, 2) : ""}
              </pre>
            ) : (
              <ContextMenu>
                <ContextMenuTrigger asChild>
                  <div
                    className="min-h-full"
                    onContextMenuCapture={captureOutputSelection}
                  >
              <MappedText
                text={response?.translated ?? ""}
                ranges={targetRanges}
                activeRange={activeRange}
                activeRanges={selectedOutputRangeIndices.length > 0 ? selectedOutputRangeIndices : undefined}
                onRangeSelect={(rangeIndex) => selectRange(rangeIndex, "output")}
                onRangeKeyDown={extendOutputRangeSelection}
                emptyMessage={isPending ? "Đang dịch chương…" : "Bản dịch sẽ xuất hiện ở đây. Chọn “Dùng văn bản mẫu” để thử range mapping mà không gọi API."}
                      className="min-h-full px-8 py-8 font-serif text-[21px] leading-[2.05] text-reader-ink md:px-10"
                    />
                  </div>
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("vietPhrase")}
                  >
                    Cập nhật VietPhrase
                  </ContextMenuItem>
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("names")}
                  >
                    Cập nhật Tên
                  </ContextMenuItem>
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("names2")}
                  >
                    Cập nhật Tên 2
                  </ContextMenuItem>
                  <ContextMenuSeparator />
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("chinesePhienAmWords")}
                  >
                    Cập nhật Phiên Âm
                  </ContextMenuItem>
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("danhTu")}
                  >
                    Cập nhật Danh Từ
                  </ContextMenuItem>
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("hauTu")}
                  >
                    Cập nhật Hậu Từ
                  </ContextMenuItem>
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("hoNguoi")}
                  >
                    Cập nhật Họ Người
                  </ContextMenuItem>
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("luatNhan")}
                  >
                    Cập nhật Luật Nhân
                  </ContextMenuItem>
                  <ContextMenuSeparator />
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => {
                      if (contextSelection) {
                        void copyContextText(
                          contextSelection.target,
                          "Đã sao chép nghĩa tiếng Việt",
                        );
                      }
                    }}
                  >
                    Sao chép tiếng Việt
                  </ContextMenuItem>
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => {
                      if (contextSelection) {
                        void copyContextText(
                          `${contextSelection.source}=${contextSelection.target}`,
                          "Đã sao chép cặp từ",
                        );
                      }
                    }}
                  >
                    Sao chép cặp từ
                  </ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>
            )}
            {/* Câu hướng dẫn chỉ hiện khi chưa chọn cặp nào; chọn rồi thì thanh
                này chuyển hẳn sang việc của nó là hiển thị cặp đang xem. */}
            {hasMapping && outputView === "output" ? (
              <div className="pointer-events-none sticky bottom-4 mx-5 mt-auto flex w-fit max-w-[calc(100%-2.5rem)] items-center gap-2 rounded-md border bg-reader-paper/95 px-3 py-1.5 text-[11px] text-muted-foreground shadow-sm backdrop-blur">
                <span className="font-semibold text-pair">↔</span>
                {activeRange === undefined
                  ? "Click một cụm để đối chiếu · kéo chọn nhiều cụm rồi click phải để cập nhật từ điển"
                  : <>
                    <span lang="zh-Hans" className="truncate">{activeSource || "∅"}</span>
                    <span>↔</span>
                    <strong className="truncate text-foreground">{activeTarget || "∅"}</strong>
                    {selectedOutputRangeIndices.length > 1 ? (
                      <span className="shrink-0">· Đã chọn {selectedOutputRangeIndices.length} cụm</span>
                    ) : selectedOutputRangeIndices.length === 1 ? (
                      <span className="hidden shrink-0 sm:inline">· Shift + ←/→ để chọn thêm</span>
                    ) : null}
                  </>}
              </div>
            ) : null}
          </div>
        </section>
      </div>

      <footer className="flex min-w-0 items-center justify-between gap-3 px-1">
        <div className="flex min-w-0 items-center gap-2 text-[11px] text-muted-foreground">
          {isPending ? <LoaderCircle className="size-3.5 shrink-0 animate-spin text-primary" /> : <span className="size-1.5 shrink-0 rounded-full bg-ok" />}
          <span className="truncate">{requestStatus}</span>
          {response?.translated ? (
            <span
              className="hidden shrink-0 font-mono text-[10px] sm:inline"
              title="Số ký tự bản dịch"
            >
              · {response.translated.length.toLocaleString("vi-VN")} ký tự bản dịch
            </span>
          ) : null}
        </div>
        <div className="flex shrink-0 items-center gap-1">
          <Button type="button" variant="ghost" size="sm" onClick={loadSample}><Braces /> Văn bản mẫu</Button>
          <Button type="button" variant="ghost" size="sm" disabled={!sourceText && !response} onClick={clearWorkspace}><RotateCcw /> Xóa</Button>
          <Button type="button" variant="outline" size="sm" disabled={!response?.translated} onClick={() => void copyOutput()}><Copy /> Sao chép</Button>
        </div>
      </footer>
      <DictionaryUpdateDialog
        key={`${dictionaryUpdateKey ?? "closed"}-${contextSelection?.source ?? ""}-${contextSelection?.target ?? ""}`}
        open={dictionaryUpdateKey !== undefined}
        endpoint={endpoint}
        dictionaryKey={dictionaryUpdateKey}
        selection={contextSelection}
        context={contextSelection ? candidateContext(sourceText, contextSelection.source, 120) : ""}
        aiSettings={aiSettings}
        localEntries={localDictionaryEntries}
        onOpenChange={(open) => {
          if (!open) setDictionaryUpdateKey(undefined);
        }}
        onOpenAiSettings={() => {
          setDictionaryUpdateKey(undefined);
          onOpenSettings?.();
        }}
        onSave={saveLocalDictionaryEntries}
        onRemove={removeLocalDictionaryEntries}
      />
    </main>
  );
}
