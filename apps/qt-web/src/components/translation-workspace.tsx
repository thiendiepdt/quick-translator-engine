import {
  Braces,
  Copy,
  FileText,
  LoaderCircle,
  Pin,
  PinOff,
  RotateCcw,
  Sparkles,
} from "lucide-react";
import {
  type MouseEvent,
  useLayoutEffect,
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
import { rangeText } from "@/lib/ranges";
import type { DictionaryUpdateKey } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useWorkspaceStore } from "@/store/workspace";

interface TranslationWorkspaceProps {
  isPending: boolean;
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

export function TranslationWorkspace({ isPending, requestStatus }: TranslationWorkspaceProps) {
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
  const [scrollRequest, setScrollRequest] = useState<ScrollRequest>();
  const [contextSelection, setContextSelection] =
    useState<DictionaryUpdateSelection>();
  const [dictionaryUpdateKey, setDictionaryUpdateKey] =
    useState<DictionaryUpdateKey>();
  const sourceScrollRef = useRef<HTMLDivElement>(null);
  const outputScrollRef = useRef<HTMLDivElement>(null);

  const sourceRanges = response?.sourceRanges ?? [];
  const targetRanges = response?.targetRanges ?? [];
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
    const rangeIndex = Number(rangeElement.dataset.rangeIndex);
    const source = rangeText(sourceText, sourceRanges[rangeIndex]);
    const fullTarget = rangeText(response?.translated ?? "", targetRanges[rangeIndex]);
    if (!source || !fullTarget) {
      setContextSelection(undefined);
      return;
    }

    const selection = window.getSelection();
    const selectedTarget =
      selection &&
      !selection.isCollapsed &&
      selection.rangeCount > 0 &&
      rangeElement.contains(selection.getRangeAt(0).commonAncestorContainer)
        ? selection.toString().trim()
        : "";
    setActiveRange(rangeIndex);
    setContextSelection({
      source,
      target: selectedTarget || fullTarget,
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
        <section className={cn("min-h-0 min-w-0 grid-rows-[40px_minmax(0,1fr)]", mobilePane === "source" ? "grid" : "hidden lg:grid")} aria-label="Nguyên văn">
          <header className="flex items-center justify-between gap-3 border-b pr-2 pl-4">
            <strong className="hidden text-xs lg:block">Nguyên văn</strong>
            {paneTabs}
            <Tabs value={sourceView} onValueChange={(value) => setSourceView(value as "raw" | "linked")}>
              <TabsList className="h-7">
                <TabsTrigger value="raw" className="text-[11px]">Gốc</TabsTrigger>
                <TabsTrigger value="linked" disabled={!response} className="text-[11px]">Đối chiếu</TabsTrigger>
              </TabsList>
            </Tabs>
          </header>
          <div ref={sourceScrollRef} lang="zh-Hans" className="fine-scrollbar relative min-h-0 overflow-auto">
            {sourceView === "raw" ? (
              <Textarea
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
                      onRangeSelect={(rangeIndex) => selectRange(rangeIndex, "output")}
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
                    Update VietPhrase
                  </ContextMenuItem>
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("names")}
                  >
                    Update Name (chính)
                  </ContextMenuItem>
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("names2")}
                  >
                    Update Name (phụ)
                  </ContextMenuItem>
                  <ContextMenuSeparator />
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("chinesePhienAmWords")}
                  >
                    Update Phiên Âm
                  </ContextMenuItem>
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("danhTu")}
                  >
                    Update Danh Từ
                  </ContextMenuItem>
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("hauTu")}
                  >
                    Update Hậu Từ
                  </ContextMenuItem>
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("hoNguoi")}
                  >
                    Update Họ Người
                  </ContextMenuItem>
                  <ContextMenuItem
                    disabled={!contextSelection}
                    onSelect={() => beginDictionaryUpdate("luatNhan")}
                  >
                    Update Luật Nhân
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
                    Copy To Việt
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
                    Copy To Clipboard
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
                  ? "Click một cụm để đối chiếu · click phải để cập nhật từ điển"
                  : <><span lang="zh-Hans" className="truncate">{activeSource || "∅"}</span><span>↔</span><strong className="truncate text-foreground">{activeTarget || "∅"}</strong></>}
              </div>
            ) : null}
          </div>
        </section>
      </div>

      <footer className="flex min-w-0 items-center justify-between gap-3 px-1">
        <div className="flex min-w-0 items-center gap-2 text-[11px] text-muted-foreground">
          {isPending ? <LoaderCircle className="size-3.5 shrink-0 animate-spin text-primary" /> : <span className="size-1.5 shrink-0 rounded-full bg-ok" />}
          <span className="truncate">{requestStatus}</span>
          {/* Số ký tự chỉ có nghĩa khi đã có chữ — bằng 0 thì nó là nhiễu. */}
          {sourceText.length > 0 ? (
            <span
              className="hidden shrink-0 font-mono text-[10px] sm:inline"
              title="ký tự nguyên văn / ký tự bản dịch"
            >
              · {sourceText.length.toLocaleString("vi-VN")}
              {" / "}
              {(response?.translated.length ?? 0).toLocaleString("vi-VN")}
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
        dictionaryKey={dictionaryUpdateKey}
        selection={contextSelection}
        localEntries={localDictionaryEntries}
        onOpenChange={(open) => {
          if (!open) setDictionaryUpdateKey(undefined);
        }}
        onSave={saveLocalDictionaryEntries}
        onRemove={removeLocalDictionaryEntries}
      />
    </main>
  );
}
