import { Braces, Copy, FileText, LoaderCircle, RotateCcw, Sparkles } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { MappedText } from "@/components/mapped-text";
import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { rangeText } from "@/lib/ranges";
import { cn } from "@/lib/utils";
import { useWorkspaceStore } from "@/store/workspace";

interface TranslationWorkspaceProps {
  isPending: boolean;
  requestStatus: string;
}

export function TranslationWorkspace({ isPending, requestStatus }: TranslationWorkspaceProps) {
  const sourceText = useWorkspaceStore((state) => state.sourceText);
  const response = useWorkspaceStore((state) => state.response);
  const activeRange = useWorkspaceStore((state) => state.activeRange);
  const sourceView = useWorkspaceStore((state) => state.sourceView);
  const outputView = useWorkspaceStore((state) => state.outputView);
  const setSourceText = useWorkspaceStore((state) => state.setSourceText);
  const setActiveRange = useWorkspaceStore((state) => state.setActiveRange);
  const setSourceView = useWorkspaceStore((state) => state.setSourceView);
  const setOutputView = useWorkspaceStore((state) => state.setOutputView);
  const clearWorkspace = useWorkspaceStore((state) => state.clearWorkspace);
  const loadSample = useWorkspaceStore((state) => state.loadSample);
  const [mobilePane, setMobilePane] = useState<"source" | "output">("source");

  const sourceRanges = response?.sourceRanges ?? [];
  const targetRanges = response?.targetRanges ?? [];
  const activeSource = rangeText(sourceText, activeRange === undefined ? undefined : sourceRanges[activeRange]);
  const activeTarget = rangeText(response?.translated ?? "", activeRange === undefined ? undefined : targetRanges[activeRange]);
  const hasMapping = Boolean(response && sourceRanges.length > 0 && targetRanges.length > 0);

  async function copyOutput() {
    if (!response?.translated) return;
    try {
      await navigator.clipboard.writeText(response.translated);
      toast.success("Đã sao chép bản dịch");
    } catch {
      toast.error("Trình duyệt không cho phép sao chép");
    }
  }

  const paneTabs = (
    <Tabs value={mobilePane} onValueChange={(value) => setMobilePane(value as "source" | "output")} className="lg:hidden">
      <TabsList className="h-8">
        <TabsTrigger value="source" className="text-xs"><FileText /> Nguyên văn</TabsTrigger>
        <TabsTrigger value="output" className="text-xs"><Sparkles /> Bản dịch</TabsTrigger>
      </TabsList>
    </Tabs>
  );

  return (
    <main className="grid min-h-0 min-w-0 grid-rows-[50px_minmax(0,1fr)_54px] bg-background px-3 pb-3 md:px-4 md:pb-4">
      <div className="flex items-center justify-between gap-3 px-1">
        <div className="hidden min-w-0 items-center gap-2 text-xs lg:flex">
          <strong>中文 · Nguyên văn</strong><span className="text-muted-foreground">→</span><strong>Tiếng Việt · Một nghĩa</strong>
          <span className="ml-1 rounded border border-primary/25 bg-primary/6 px-2 py-1 font-mono text-[9px] font-semibold tracking-wide text-primary">VIETPHRASE-ONE</span>
        </div>
        {paneTabs}
        <div className="ml-auto shrink-0 font-mono text-[10px] text-muted-foreground">
          {sourceText.length.toLocaleString("vi-VN")} ký tự nguồn
          <span className="mx-1.5">·</span>
          {(response?.translated.length ?? 0).toLocaleString("vi-VN")} ký tự đích
        </div>
      </div>

      <div className="grid min-h-0 min-w-0 overflow-hidden rounded-lg border bg-card shadow-[0_12px_40px_rgba(28,44,72,0.07)] lg:grid-cols-2">
        <section className={cn("min-h-0 min-w-0 grid-rows-[48px_minmax(0,1fr)]", mobilePane === "source" ? "grid" : "hidden lg:grid")} aria-label="Nguyên văn">
          <header className="flex items-center justify-between gap-3 border-b px-4">
            <div className="flex items-baseline gap-2"><strong className="text-xs tracking-wide uppercase">Source</strong><span className="font-mono text-[9px] text-muted-foreground">UTF-16</span></div>
            <Tabs value={sourceView} onValueChange={(value) => setSourceView(value as "raw" | "linked")}>
              <TabsList className="h-8">
                <TabsTrigger value="raw" className="text-[10px]">RAW</TabsTrigger>
                <TabsTrigger value="linked" disabled={!response} className="text-[10px]">LINKED</TabsTrigger>
              </TabsList>
            </Tabs>
          </header>
          <div className="fine-scrollbar relative min-h-0 overflow-auto">
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
                onRangeSelect={setActiveRange}
                emptyMessage="Chưa có nguyên văn."
                className="min-h-full px-7 py-6 text-[18px] leading-9"
              />
            )}
          </div>
        </section>

        <section className={cn("min-h-0 min-w-0 grid-rows-[48px_minmax(0,1fr)] border-l", mobilePane === "output" ? "grid border-l-0" : "hidden lg:grid")} aria-label="Bản dịch">
          <header className="flex items-center justify-between gap-3 border-b px-4">
            <div className="flex items-baseline gap-2"><strong className="text-xs tracking-wide uppercase">Output</strong><span className="font-mono text-[9px] text-muted-foreground">READER</span></div>
            <Tabs value={outputView} onValueChange={(value) => setOutputView(value as "output" | "json")}>
              <TabsList variant="line" className="h-8">
                <TabsTrigger value="output" className="text-[10px]">BẢN DỊCH</TabsTrigger>
                <TabsTrigger value="json" disabled={!response} className="text-[10px]">RAW JSON</TabsTrigger>
              </TabsList>
            </Tabs>
          </header>
          <div className="fine-scrollbar relative min-h-0 overflow-auto bg-[var(--reader-paper)]">
            {outputView === "json" ? (
              <pre className="min-h-full overflow-auto bg-slate-950 p-6 font-mono text-xs leading-6 text-slate-200">
                {response ? JSON.stringify(response, null, 2) : ""}
              </pre>
            ) : (
              <MappedText
                text={response?.translated ?? ""}
                ranges={targetRanges}
                activeRange={activeRange}
                onRangeSelect={setActiveRange}
                emptyMessage={isPending ? "Đang dịch chương…" : "Bản dịch sẽ xuất hiện ở đây. Chọn “Dùng văn bản mẫu” để thử range mapping mà không gọi API."}
                className="reader-output min-h-full px-8 py-8 font-serif text-[21px] leading-[2.05] text-[var(--reader-ink)] md:px-10"
              />
            )}
            {hasMapping && outputView === "output" ? (
              <div className="pointer-events-none sticky bottom-4 mx-5 mt-auto flex w-fit max-w-[calc(100%-2.5rem)] items-center gap-2 border bg-[var(--reader-paper)]/95 px-3 py-2 text-[10px] text-muted-foreground shadow-sm backdrop-blur">
                <span className="font-semibold text-[var(--reader-accent)]">↔</span>
                {activeRange === undefined ? "Click chữ ở hai phía để xem phần tương ứng" : <><span className="truncate">{activeSource || "∅"}</span><span>↔</span><strong className="truncate text-foreground">{activeTarget || "∅"}</strong></>}
              </div>
            ) : null}
          </div>
        </section>
      </div>

      <footer className="flex min-w-0 items-center justify-between gap-3 px-1">
        <div className="flex min-w-0 items-center gap-2 text-[11px] text-muted-foreground">
          {isPending ? <LoaderCircle className="size-3.5 shrink-0 animate-spin text-primary" /> : <span className="size-1.5 shrink-0 rounded-full bg-emerald-600" />}
          <span className="truncate">{requestStatus}</span>
        </div>
        <div className="flex shrink-0 items-center gap-1">
          <Button type="button" variant="ghost" size="sm" onClick={loadSample}><Braces /> Dùng văn bản mẫu</Button>
          <Button type="button" variant="ghost" size="sm" disabled={!sourceText && !response} onClick={clearWorkspace}><RotateCcw /> Xóa workspace</Button>
          <Button type="button" variant="outline" size="sm" disabled={!response?.translated} onClick={() => void copyOutput()}><Copy /> Sao chép output</Button>
        </div>
      </footer>
    </main>
  );
}
