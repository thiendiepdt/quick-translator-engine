import {
  BrainCircuit,
  Check,
  Filter,
  LoaderCircle,
  RotateCcw,
  Search,
  ShieldCheck,
  Sparkles,
  X,
} from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { useNameFilterMutation } from "@/hooks/use-translation";
import { ApiError } from "@/lib/api";
import type { NameCandidate, NameFilterRequest } from "@/lib/types";
import { cn } from "@/lib/utils";
import { dictionaryPayload, useWorkspaceStore } from "@/store/workspace";

interface NameFilterWorkspaceProps {
  endpoint: string;
  defaultsReady: boolean;
}

const entityLabels: Record<NameCandidate["entityType"], string> = {
  person: "Nhân vật",
  location: "Địa danh",
  organization: "Tổ chức",
  title: "Danh hiệu",
  unknown: "Chưa rõ",
};

export function NameFilterWorkspace({ endpoint, defaultsReady }: NameFilterWorkspaceProps) {
  const sourceText = useWorkspaceStore((state) => state.sourceText);
  const setSourceText = useWorkspaceStore((state) => state.setSourceText);
  const dictionaries = useWorkspaceStore((state) => state.dictionaries);
  const response = useWorkspaceStore((state) => state.nameFilterResponse);
  const setResponse = useWorkspaceStore((state) => state.setNameFilterResponse);
  const knownNames = useWorkspaceStore((state) => state.knownNames);
  const rejectedNames = useWorkspaceStore((state) => state.rejectedNames);
  const acceptName = useWorkspaceStore((state) => state.acceptNameCandidate);
  const rejectName = useWorkspaceStore((state) => state.rejectNameCandidate);
  const clearMemory = useWorkspaceStore((state) => state.clearNameMemory);
  const nameMemoryId = useWorkspaceStore((state) => state.nameMemoryId);
  const switchNameMemory = useWorkspaceStore((state) => state.switchNameMemory);
  const mutation = useNameFilterMutation();
  const [mode, setMode] = useState<"qt" | "hybrid">("hybrid");
  const [nerEnabled, setNerEnabled] = useState(false);
  const [aiEnabled, setAiEnabled] = useState(false);
  const [search, setSearch] = useState("");
  const [activeText, setActiveText] = useState<string>();
  const [memoryIdDraft, setMemoryIdDraft] = useState(nameMemoryId);

  const rejected = useMemo(() => new Set(rejectedNames), [rejectedNames]);
  const candidates = useMemo(() => {
    const needle = search.trim().toLocaleLowerCase();
    return (response?.candidates ?? []).filter((candidate) => {
      if (rejected.has(candidate.text)) return false;
      if (!needle) return true;
      return `${candidate.text} ${candidate.suggested} ${entityLabels[candidate.entityType]}`
        .toLocaleLowerCase()
        .includes(needle);
    });
  }, [rejected, response?.candidates, search]);

  const visibleActiveText = candidates.some((candidate) => candidate.text === activeText)
    ? activeText
    : undefined;

  async function runFilter() {
    if (!sourceText.trim()) {
      toast.error("Dán chương tiếng Trung trước khi lọc name");
      return;
    }
    if (!defaultsReady) {
      toast.error("Chưa tải xong từ điển mặc định từ engine");
      return;
    }
    const request: NameFilterRequest = {
      text: sourceText,
      mode,
      minOccurrences: mode === "qt" ? 2 : 2,
      minConfidence: mode === "qt" ? 0.55 : 0.6,
      maxCandidates: 300,
      knownNames,
      rejectedNames,
      ner: { enabled: nerEnabled, minConfidence: 0.65 },
      aiFallback: {
        enabled: aiEnabled,
        minConfidence: 0.65,
        minRuleConfidence: 0.4,
        maxRuleConfidence: 0.82,
        maxCandidates: 25,
      },
      dictionaries: dictionaryPayload(dictionaries),
    };
    try {
      const next = await mutation.mutateAsync({ endpoint, request });
      setResponse(next);
      setActiveText(next.candidates[0]?.text);
      if (next.warnings?.length) {
        toast.warning("Lọc xong nhưng provider tùy chọn có cảnh báo", {
          description: next.warnings.join(" · "),
        });
      } else {
        toast.success(`Tìm thấy ${next.candidates.length.toLocaleString("vi-VN")} candidate`);
      }
    } catch (error) {
      const requestId = error instanceof ApiError ? error.requestId : undefined;
      toast.error(error instanceof Error ? error.message : "Không thể lọc name", {
        description: requestId ? `Request ID: ${requestId}` : undefined,
      });
    }
  }

  function accept(candidate: NameCandidate, suggested: string) {
    const normalized = suggested.trim();
    if (!normalized) {
      toast.error("Tên tiếng Việt không được để trống");
      return;
    }
    acceptName(candidate.text, normalized);
    toast.success(`Đã thêm ${candidate.text}=${normalized} vào Names2`);
  }

  function acceptHighConfidence() {
    const highConfidence = candidates.filter(
      (candidate) => candidate.score >= 0.85 && !knownNames[candidate.text],
    );
    for (const candidate of highConfidence) acceptName(candidate.text, candidate.suggested);
    toast.success(`Đã duyệt ${highConfidence.length} name có confidence ≥ 85%`);
  }

  return (
    <main className="grid min-h-0 min-w-0 grid-rows-[64px_minmax(0,1fr)_50px] bg-background px-3 pb-3 md:px-4 md:pb-4">
      <div className="flex min-w-0 items-center gap-3 px-1">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <strong className="text-sm">Lọc name theo chương</strong>
            <Badge variant="outline" className="font-mono text-[9px]">BOOK MEMORY</Badge>
          </div>
          <p className="truncate text-[10px] text-muted-foreground">
            Rules QT + ngữ cảnh + memory; ONNX và AI chỉ chạy khi bật.
          </p>
        </div>
        <Label className="ml-auto hidden items-center gap-2 text-[10px] font-semibold lg:flex">
          Mã truyện
          <Input
            value={memoryIdDraft}
            onChange={(event) => setMemoryIdDraft(event.target.value)}
            onBlur={() => switchNameMemory(memoryIdDraft)}
            onKeyDown={(event) => {
              if (event.key !== "Enter") return;
              event.preventDefault();
              switchNameMemory(memoryIdDraft);
              event.currentTarget.blur();
            }}
            className="h-8 w-36 font-mono text-[10px]"
            aria-label="Mã memory của truyện"
          />
        </Label>
        <Tabs value={mode} onValueChange={(value) => setMode(value as "qt" | "hybrid")} className="ml-auto lg:ml-0">
          <TabsList className="h-9">
            <TabsTrigger value="qt" className="text-[10px]">QT LEGACY</TabsTrigger>
            <TabsTrigger value="hybrid" className="text-[10px]">HYBRID</TabsTrigger>
          </TabsList>
        </Tabs>
        <ProviderToggle label="ONNX" icon={<BrainCircuit />} checked={nerEnabled} onCheckedChange={setNerEnabled} />
        <ProviderToggle label="AI" icon={<Sparkles />} checked={aiEnabled} onCheckedChange={setAiEnabled} />
        <Button type="button" disabled={mutation.isPending || !defaultsReady} onClick={() => void runFilter()}>
          {mutation.isPending ? <LoaderCircle className="animate-spin" /> : <Filter />}
          Lọc name
        </Button>
      </div>

      <div className="grid min-h-0 overflow-hidden rounded-lg border bg-card shadow-[0_12px_40px_rgba(28,44,72,0.07)] lg:grid-cols-[minmax(320px,0.78fr)_minmax(520px,1.22fr)]">
        <section className="grid min-h-0 grid-rows-[46px_minmax(0,1fr)_auto] border-r" aria-label="Chương nguồn">
          <header className="flex items-center justify-between border-b px-4">
            <strong className="text-xs tracking-wide uppercase">Chương nguồn</strong>
            <span className="font-mono text-[10px] text-muted-foreground">
              {sourceText.length.toLocaleString("vi-VN")} ký tự
            </span>
          </header>
          <Textarea
            value={sourceText}
            onChange={(event) => setSourceText(event.target.value)}
            placeholder="Dán một chương tiếng Trung vào đây…"
            className="h-full min-h-full resize-none rounded-none border-0 bg-transparent px-6 py-5 text-[17px] leading-8 shadow-none focus-visible:ring-0"
          />
          <div className="border-t bg-muted/35 px-4 py-3 text-[10px] text-muted-foreground">
            <strong className="text-foreground">Memory truyện:</strong>{" "}
            {Object.keys(knownNames).length.toLocaleString("vi-VN")} đã duyệt ·{" "}
            {rejectedNames.length.toLocaleString("vi-VN")} đã loại
          </div>
        </section>

        <section className="grid min-h-0 grid-rows-[46px_minmax(0,1fr)]" aria-label="Name candidates">
          <header className="flex items-center gap-3 border-b px-3">
            <div className="relative min-w-0 flex-1">
              <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={search}
                onChange={(event) => setSearch(event.target.value)}
                placeholder="Tìm chữ Hán, tên Việt hoặc loại entity…"
                className="h-8 pl-8 text-xs"
              />
            </div>
            <span className="shrink-0 font-mono text-[10px] text-muted-foreground">
              {candidates.length.toLocaleString("vi-VN")} records
            </span>
            <Button type="button" size="sm" variant="outline" disabled={!candidates.some((candidate) => candidate.score >= 0.85 && !knownNames[candidate.text])} onClick={acceptHighConfidence}>
              <ShieldCheck /> Duyệt ≥85%
            </Button>
          </header>
          <div className="fine-scrollbar min-h-0 overflow-auto">
            {candidates.length > 0 ? (
              <div className="divide-y">
                {candidates.map((candidate) => (
                  <CandidateRow
                    key={candidate.text}
                    candidate={candidate}
                    active={visibleActiveText === candidate.text}
                    acceptedValue={knownNames[candidate.text]}
                    onActivate={() => setActiveText(candidate.text)}
                    onAccept={(suggested) => accept(candidate, suggested)}
                    onReject={() => {
                      rejectName(candidate.text);
                      toast.message(`Đã loại ${candidate.text} khỏi memory truyện`);
                    }}
                  />
                ))}
              </div>
            ) : (
              <div className="grid h-full min-h-64 place-items-center px-8 text-center text-sm text-muted-foreground">
                <div>
                  <Filter className="mx-auto mb-3 size-8 opacity-35" />
                  {response ? "Không có record khớp bộ lọc hiện tại." : "Bấm “Lọc name” để tạo danh sách candidate cần duyệt."}
                </div>
              </div>
            )}
          </div>
        </section>
      </div>

      <footer className="flex min-w-0 items-center justify-between gap-3 px-1 text-[10px] text-muted-foreground">
        <div className="truncate">
          {response ? (
            <>
              Rules {response.stats.ruleCandidates} · NER {response.stats.nerCandidates} · AI reviewed {response.stats.aiReviewed}
              {response.capabilities.nerConfigured ? " · ONNX ready" : ""}
              {response.capabilities.aiConfigured ? " · Gemini ready" : ""}
            </>
          ) : <>Memory <strong>{nameMemoryId}</strong> được lưu trong trình duyệt và gửi lại ở chương kế tiếp.</>}
        </div>
        <Button
          type="button"
          size="sm"
          variant="ghost"
          disabled={Object.keys(knownNames).length === 0 && rejectedNames.length === 0}
          onClick={() => {
            clearMemory();
            toast.message("Đã xóa memory lọc name của truyện");
          }}
        >
          <RotateCcw /> Xóa memory
        </Button>
      </footer>
    </main>
  );
}

function ProviderToggle({
  label,
  icon,
  checked,
  onCheckedChange,
}: {
  label: string;
  icon: React.ReactNode;
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
}) {
  return (
    <Label className="flex h-9 items-center gap-2 rounded-md border px-2.5 text-[10px] font-semibold">
      <span className="[&_svg]:size-3.5">{icon}</span>
      {label}
      <Switch size="sm" checked={checked} onCheckedChange={onCheckedChange} />
    </Label>
  );
}

function CandidateRow({
  candidate,
  active,
  acceptedValue,
  onActivate,
  onAccept,
  onReject,
}: {
  candidate: NameCandidate;
  active: boolean;
  acceptedValue?: string;
  onActivate: () => void;
  onAccept: (suggested: string) => void;
  onReject: () => void;
}) {
  const [suggested, setSuggested] = useState(acceptedValue ?? candidate.suggested);
  return (
    <article className={cn("grid gap-2 px-3 py-3 transition-colors", active && "bg-primary/5")} onClick={onActivate}>
      <div className="grid grid-cols-[minmax(88px,0.55fr)_minmax(140px,1fr)_auto] items-center gap-2">
        <div className="min-w-0">
          <div className="truncate text-base font-semibold" title={candidate.text}>{candidate.text}</div>
          <div className="mt-0.5 flex items-center gap-1.5">
            <Badge variant={candidate.entityType === "unknown" ? "outline" : "secondary"} className="text-[9px]">
              {entityLabels[candidate.entityType]}
            </Badge>
            <span className="font-mono text-[9px] text-muted-foreground">{Math.round(candidate.score * 100)}%</span>
            <span className="font-mono text-[9px] text-muted-foreground">×{candidate.occurrences}</span>
          </div>
        </div>
        <Input
          value={suggested}
          onChange={(event) => setSuggested(event.target.value)}
          onClick={(event) => event.stopPropagation()}
          aria-label={`Tên tiếng Việt cho ${candidate.text}`}
          className="h-9 font-medium"
        />
        <div className="flex items-center gap-1">
          {acceptedValue ? (
            <Badge className="h-8 rounded-md bg-emerald-600 px-2"><Check /> Đã duyệt</Badge>
          ) : (
            <Button type="button" size="icon-sm" variant="outline" className="text-emerald-700" aria-label={`Duyệt ${candidate.text}`} onClick={(event) => { event.stopPropagation(); onAccept(suggested); }}>
              <Check />
            </Button>
          )}
          <Button type="button" size="icon-sm" variant="ghost" className="text-destructive" aria-label={`Loại ${candidate.text}`} onClick={(event) => { event.stopPropagation(); onReject(); }}>
            <X />
          </Button>
        </div>
      </div>
      {active && candidate.contexts[0] ? (
        <div className="rounded border bg-muted/45 px-3 py-2 text-xs leading-6 text-muted-foreground">
          <HighlightedContext value={candidate.contexts[0]} />
        </div>
      ) : null}
      {active ? (
        <div className="flex flex-wrap gap-1">
          {candidate.sources.map((source) => <Badge key={source} variant="outline" className="font-mono text-[8px]">{source}</Badge>)}
          {candidate.reasons.slice(0, 2).map((reason) => <span key={reason} className="text-[9px] text-muted-foreground">· {reason}</span>)}
        </div>
      ) : null}
    </article>
  );
}

function HighlightedContext({ value }: { value: string }) {
  const start = value.indexOf("【");
  const end = value.indexOf("】", start + 1);
  if (start < 0 || end < 0) return value;
  return (
    <>
      {value.slice(0, start)}
      <mark className="rounded bg-amber-200 px-1 font-semibold text-amber-950">{value.slice(start + 1, end)}</mark>
      {value.slice(end + 1)}
    </>
  );
}
