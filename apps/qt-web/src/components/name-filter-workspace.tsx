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
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Textarea } from "@/components/ui/textarea";
import { useNameFilterMutation } from "@/hooks/use-translation";
import { activeAiProviderConfig, type AiSettings } from "@/lib/ai-settings";
import { ApiError } from "@/lib/api";
import {
  isNameFilterMode,
  readStoredNameApprovalThreshold,
  readStoredNameFilterMode,
  storeNameApprovalThreshold,
  storeNameFilterMode,
} from "@/lib/name-filter-mode";
import type { NameCandidate, NameFilterMode, NameFilterRequest } from "@/lib/types";
import { cn } from "@/lib/utils";
import { dictionaryPayload, useWorkspaceStore } from "@/store/workspace";
import { useWorkspaceCatalogStore } from "@/store/workspace-catalog";

interface NameFilterWorkspaceProps {
  endpoint: string;
  defaultsReady: boolean;
  aiSettings?: AiSettings;
}

const entityLabels: Record<NameCandidate["entityType"], string> = {
  person: "Nhân vật",
  location: "Địa danh",
  organization: "Tổ chức",
  title: "Danh hiệu",
  unknown: "Chưa rõ",
};

export function NameFilterWorkspace({
  endpoint,
  defaultsReady,
  aiSettings,
}: NameFilterWorkspaceProps) {
  const sourceText = useWorkspaceStore((state) => state.sourceText);
  const setSourceText = useWorkspaceStore((state) => state.setSourceText);
  const dictionaries = useWorkspaceStore((state) => state.dictionaries);
  const response = useWorkspaceStore((state) => state.nameFilterResponse);
  const setResponse = useWorkspaceStore((state) => state.setNameFilterResponse);
  const knownNames = useWorkspaceStore((state) => state.knownNames);
  const rejectedNames = useWorkspaceStore((state) => state.rejectedNames);
  const acceptName = useWorkspaceStore((state) => state.acceptNameCandidate);
  const undoAcceptedName = useWorkspaceStore((state) => state.undoAcceptedNameCandidate);
  const rejectName = useWorkspaceStore((state) => state.rejectNameCandidate);
  const restoreRejectedName = useWorkspaceStore(
    (state) => state.restoreRejectedNameCandidate,
  );
  const restoreAllRejectedNames = useWorkspaceStore(
    (state) => state.restoreAllRejectedNameCandidates,
  );
  const clearMemory = useWorkspaceStore((state) => state.clearNameMemory);
  const mutation = useNameFilterMutation();
  const [mode, setMode] = useState<NameFilterMode>(readStoredNameFilterMode);
  const [approvalThreshold, setApprovalThreshold] = useState(readStoredNameApprovalThreshold);
  const [approvalThresholdDraft, setApprovalThresholdDraft] = useState(() =>
    String(readStoredNameApprovalThreshold()),
  );
  const [aiExtractEnabled, setAiExtractEnabled] = useState(false);
  const [aiEnabled, setAiEnabled] = useState(false);
  const [search, setSearch] = useState("");
  const [activeText, setActiveText] = useState<string>();
  const [candidateView, setCandidateView] = useState<"pending" | "rejected">("pending");

  useEffect(() => {
    storeNameFilterMode(mode);
  }, [mode]);

  useEffect(() => {
    storeNameApprovalThreshold(approvalThreshold);
  }, [approvalThreshold]);

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
  const rejectedCandidates = useMemo(() => {
    const responseCandidates = new Map(
      (response?.candidates ?? []).map((candidate) => [candidate.text, candidate]),
    );
    const needle = search.trim().toLocaleLowerCase();
    return rejectedNames
      .map((text) => ({ text, candidate: responseCandidates.get(text) }))
      .filter(({ text, candidate }) => {
        if (!needle) return true;
        const details = candidate
          ? `${candidate.suggested} ${entityLabels[candidate.entityType]}`
          : "";
        return `${text} ${details}`.toLocaleLowerCase().includes(needle);
      });
  }, [rejectedNames, response?.candidates, search]);

  const visibleActiveText = candidates.some((candidate) => candidate.text === activeText)
    ? activeText
    : undefined;

  async function runFilter() {
    if (!sourceText.trim()) {
      toast.error("Dán chương tiếng Trung trước khi lọc tên");
      return;
    }
    if (!defaultsReady) {
      toast.error("Chưa tải xong từ điển mặc định từ bộ máy");
      return;
    }
    const wantsAi = aiExtractEnabled || aiEnabled;
    // Key/model của đúng provider đang chọn — không dùng chéo giữa hai bên.
    const providerConfig = aiSettings ? activeAiProviderConfig(aiSettings) : undefined;
    const apiKey = providerConfig?.apiKey.trim() ?? "";
    if (wantsAi && !apiKey) {
      toast.error("Tính năng AI cần API key của bạn", {
        description: "Nhập key DeepSeek/Gemini trong Cài đặt (biểu tượng bánh răng).",
      });
      return;
    }
    const model = providerConfig?.model.trim() ?? "";
    if (wantsAi && aiSettings?.provider === "gemini" && !model) {
      toast.error("Gemini cần chỉ định model", {
        description: "Nhập model (ví dụ gemini-2.5-flash) trong Cài đặt.",
      });
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
      ...(wantsAi && aiSettings
        ? {
            ai: {
              provider: aiSettings.provider,
              apiKey,
              ...(model ? { model } : {}),
            },
          }
        : {}),
      aiExtract: { enabled: aiExtractEnabled, minConfidence: 0.65 },
      aiFallback: {
        enabled: aiEnabled,
        minConfidence: 0.65,
        minRuleConfidence: 0.4,
        maxRuleConfidence: 0.82,
        maxCandidates: 25,
      },
      dictionaries: dictionaryPayload(dictionaries),
    };
    const requestWorkspaceId = useWorkspaceCatalogStore.getState().activeWorkspaceId;
    try {
      const next = await mutation.mutateAsync({ endpoint, request });
      if (
        useWorkspaceCatalogStore.getState().activeWorkspaceId !==
        requestWorkspaceId
      ) return;
      setResponse(next);
      setActiveText(next.candidates[0]?.text);
      if (next.warnings?.length) {
        toast.warning("Lọc xong nhưng dịch vụ tùy chọn có cảnh báo", {
          description: next.warnings.join(" · "),
        });
      } else {
        toast.success(`Tìm thấy ${next.candidates.length.toLocaleString("vi-VN")} ứng viên`);
      }
    } catch (error) {
      if (
        useWorkspaceCatalogStore.getState().activeWorkspaceId !==
        requestWorkspaceId
      ) return;
      const requestId = error instanceof ApiError ? error.requestId : undefined;
      toast.error(error instanceof Error ? error.message : "Không thể lọc tên", {
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
    toast.success(`Đã thêm ${candidate.text}=${normalized} vào Tên 2`);
  }

  function acceptHighConfidence() {
    const highConfidence = candidates.filter(
      (candidate) =>
        candidate.score >= approvalThreshold / 100 && !knownNames[candidate.text],
    );
    for (const candidate of highConfidence) acceptName(candidate.text, candidate.suggested);
    toast.success(
      `Đã duyệt ${highConfidence.length} tên có độ tin cậy ≥ ${approvalThreshold}%`,
    );
  }

  function updateApprovalThresholdDraft(value: string) {
    setApprovalThresholdDraft(value);
    if (!/^\d{1,3}$/.test(value)) return;
    const next = Number(value);
    if (next <= 100) setApprovalThreshold(next);
  }

  function commitApprovalThreshold() {
    const parsed = Number(approvalThresholdDraft);
    const next = Number.isFinite(parsed)
      ? Math.min(100, Math.max(0, Math.round(parsed)))
      : approvalThreshold;
    setApprovalThreshold(next);
    setApprovalThresholdDraft(String(next));
  }

  return (
    <main className="grid min-h-0 min-w-0 grid-rows-[64px_minmax(0,1fr)_50px] bg-background px-3 pb-3 md:px-4 md:pb-4">
      <div className="flex min-w-0 items-center gap-3 px-1">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <strong className="text-sm">Lọc tên theo chương</strong>
            <Badge variant="outline" className="font-mono text-[9px]">BỘ NHỚ TÊN</Badge>
          </div>
          <p className="truncate text-[10px] text-muted-foreground">
            Quy tắc QT + ngữ cảnh + bộ nhớ; AI chỉ chạy khi bật.
          </p>
        </div>
        <Tabs
          value={mode}
          onValueChange={(value) => {
            if (isNameFilterMode(value)) setMode(value);
          }}
          className="ml-auto"
        >
          <TabsList className="h-9">
            <TabsTrigger value="qt" className="text-[10px]">QT cũ</TabsTrigger>
            <TabsTrigger value="hybrid" className="text-[10px]">Kết hợp</TabsTrigger>
          </TabsList>
        </Tabs>
        <ProviderToggle label="Trích AI" icon={<BrainCircuit />} checked={aiExtractEnabled} onCheckedChange={setAiExtractEnabled} />
        <ProviderToggle label="AI" icon={<Sparkles />} checked={aiEnabled} onCheckedChange={setAiEnabled} />
        <Button type="button" disabled={mutation.isPending || !defaultsReady} onClick={() => void runFilter()}>
          {mutation.isPending ? <LoaderCircle className="animate-spin" /> : <Filter />}
          Lọc tên
        </Button>
      </div>

      <div className="grid min-h-0 overflow-hidden rounded-lg border bg-card shadow-[var(--shadow-panel)] lg:grid-cols-[minmax(320px,0.78fr)_minmax(520px,1.22fr)]">
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
          <div className="flex items-center justify-between gap-3 border-t bg-muted/35 px-4 py-2 text-[10px] text-muted-foreground">
            <span>
              <strong className="text-foreground">Bộ nhớ truyện:</strong>{" "}
              {Object.keys(knownNames).length.toLocaleString("vi-VN")} đã duyệt
            </span>
            <Button
              type="button"
              size="sm"
              variant="outline"
              disabled={rejectedNames.length === 0}
              onClick={() => setCandidateView("rejected")}
              className={cn(
                "h-7 gap-1.5 border-destructive/30 px-2 text-[10px] text-destructive hover:bg-destructive/5 hover:text-destructive",
                candidateView === "rejected" && "bg-destructive/10",
              )}
              aria-label={`Xem ${rejectedNames.length.toLocaleString("vi-VN")} tên đã loại`}
            >
              <X /> Tên đã loại
              <span className="rounded-full bg-destructive/10 px-1.5 py-0.5 font-mono font-semibold">
                {rejectedNames.length.toLocaleString("vi-VN")}
              </span>
            </Button>
          </div>
        </section>

        <section className="grid min-h-0 grid-rows-[46px_minmax(0,1fr)]" aria-label="Ứng viên tên">
          <header className="flex items-center gap-3 border-b px-3">
            <div className="relative min-w-0 flex-1">
              <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={search}
                onChange={(event) => setSearch(event.target.value)}
                placeholder={
                  candidateView === "pending"
                    ? "Tìm chữ Hán, tên Việt hoặc loại thực thể…"
                    : "Tìm trong các tên đã loại…"
                }
                className="h-8 pl-8 text-xs"
              />
            </div>
            <Tabs
              value={candidateView}
              onValueChange={(value) => {
                if (value === "pending" || value === "rejected") setCandidateView(value);
              }}
              className="shrink-0"
            >
              <TabsList className="h-8">
                <TabsTrigger value="pending" className="px-2 text-[10px]">
                  Chờ duyệt {candidates.length.toLocaleString("vi-VN")}
                </TabsTrigger>
                <TabsTrigger value="rejected" className="px-2 text-[10px]">
                  Đã loại {rejectedNames.length.toLocaleString("vi-VN")}
                </TabsTrigger>
              </TabsList>
            </Tabs>
            {candidateView === "pending" ? (
              <>
                <div className="relative w-20 shrink-0">
                  <Input
                    type="number"
                    inputMode="numeric"
                    min={0}
                    max={100}
                    step={1}
                    value={approvalThresholdDraft}
                    onChange={(event) => updateApprovalThresholdDraft(event.target.value)}
                    onBlur={commitApprovalThreshold}
                    onKeyDown={(event) => {
                      if (event.key === "Enter") event.currentTarget.blur();
                    }}
                    aria-label="Ngưỡng duyệt (%)"
                    className="h-8 pr-6 text-right font-mono text-xs [appearance:textfield] [&::-webkit-inner-spin-button]:appearance-none [&::-webkit-outer-spin-button]:appearance-none"
                  />
                  <span className="pointer-events-none absolute top-1/2 right-2 -translate-y-1/2 text-[10px] text-muted-foreground">
                    %
                  </span>
                </div>
                <Button
                  type="button"
                  size="sm"
                  variant="outline"
                  disabled={
                    !candidates.some(
                      (candidate) =>
                        candidate.score >= approvalThreshold / 100 &&
                        !knownNames[candidate.text],
                    )
                  }
                  onClick={acceptHighConfidence}
                >
                  <ShieldCheck /> Duyệt
                </Button>
              </>
            ) : (
              <Button
                type="button"
                size="sm"
                variant="outline"
                disabled={rejectedNames.length === 0}
                onClick={() => {
                  const restoredCount = rejectedNames.length;
                  restoreAllRejectedNames();
                  setCandidateView("pending");
                  toast.success(
                    `Đã đưa ${restoredCount.toLocaleString("vi-VN")} tên về danh sách chờ duyệt`,
                    { description: "Lọc lại chương nếu chưa thấy các tên vừa khôi phục." },
                  );
                }}
              >
                <RotateCcw /> Khôi phục tất cả
              </Button>
            )}
          </header>
          <div className="fine-scrollbar min-h-0 overflow-auto">
            {candidateView === "rejected" ? (
              rejectedCandidates.length > 0 ? (
                <div className="divide-y">
                  {rejectedCandidates.map(({ text, candidate }) => (
                    <RejectedNameRow
                      key={text}
                      text={text}
                      candidate={candidate}
                      onRestore={() => {
                        restoreRejectedName(text);
                        toast.success(`Đã đưa ${text} về danh sách chờ duyệt`);
                      }}
                    />
                  ))}
                </div>
              ) : (
                <EmptyCandidateList message="Không có tên đã loại khớp tìm kiếm." />
              )
            ) : candidates.length > 0 ? (
              <div className="divide-y">
                {candidates.map((candidate) => (
                  <CandidateRow
                    key={candidate.text}
                    candidate={candidate}
                    active={visibleActiveText === candidate.text}
                    acceptedValue={knownNames[candidate.text]}
                    onActivate={() => setActiveText(candidate.text)}
                    onAccept={(suggested) => accept(candidate, suggested)}
                    onUndo={() => {
                      undoAcceptedName(candidate.text);
                      toast.message(`Đã bỏ duyệt ${candidate.text}`);
                    }}
                    onReject={() => {
                      rejectName(candidate.text);
                      toast.message(`Đã loại ${candidate.text} khỏi bộ nhớ truyện`);
                    }}
                  />
                ))}
              </div>
            ) : (
              <EmptyCandidateList
                message={
                  response
                    ? "Không có bản ghi khớp bộ lọc hiện tại."
                    : "Bấm “Lọc tên” để tạo danh sách ứng viên cần duyệt."
                }
              />
            )}
          </div>
        </section>
      </div>

      <footer className="flex min-w-0 items-center justify-between gap-3 px-1 text-[10px] text-muted-foreground">
        <div className="truncate">
          {response ? (
            <>
              Quy tắc {response.stats.ruleCandidates} · AI trích {response.stats.aiExtractedCandidates} · AI đã duyệt {response.stats.aiReviewed}
              {response.capabilities.aiConfigured ? ` · AI: ${response.capabilities.aiProvider ?? "?"}` : ""}
            </>
          ) : <>Bộ nhớ tên được lưu theo không gian làm việc và dùng lại ở chương kế tiếp.</>}
        </div>
        <Button
          type="button"
          size="sm"
          variant="ghost"
          disabled={Object.keys(knownNames).length === 0 && rejectedNames.length === 0}
          onClick={() => {
            clearMemory();
            toast.message("Đã xóa bộ nhớ lọc tên của truyện");
          }}
        >
          <RotateCcw /> Xóa bộ nhớ
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

function EmptyCandidateList({ message }: { message: string }) {
  return (
    <div className="grid h-full min-h-64 place-items-center px-8 text-center text-sm text-muted-foreground">
      <div>
        <Filter className="mx-auto mb-3 size-8 opacity-35" />
        {message}
      </div>
    </div>
  );
}

function RejectedNameRow({
  text,
  candidate,
  onRestore,
}: {
  text: string;
  candidate?: NameCandidate;
  onRestore: () => void;
}) {
  return (
    <div className="flex items-center gap-4 px-4 py-3">
      <div className="min-w-0 flex-1">
        <div className="truncate text-base font-semibold" title={text}>
          {text}
        </div>
        <div className="mt-1 truncate text-[10px] text-muted-foreground">
          {candidate ? (
            <>
              Gợi ý trước đó: <strong className="text-foreground">{candidate.suggested}</strong>
              {" · "}
              {entityLabels[candidate.entityType]}
            </>
          ) : (
            "Khôi phục rồi lọc lại chương để cập nhật gợi ý."
          )}
        </div>
      </div>
      <Button
        type="button"
        size="sm"
        variant="outline"
        onClick={onRestore}
        aria-label={`Khôi phục ${text}`}
      >
        <RotateCcw /> Khôi phục
      </Button>
    </div>
  );
}

function CandidateRow({
  candidate,
  active,
  acceptedValue,
  onActivate,
  onAccept,
  onUndo,
  onReject,
}: {
  candidate: NameCandidate;
  active: boolean;
  acceptedValue?: string;
  onActivate: () => void;
  onAccept: (suggested: string) => void;
  onUndo: () => void;
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
            <>
              <Badge className="h-8 rounded-md bg-ok px-2 text-ok-foreground"><Check /> Đã duyệt</Badge>
              <Button
                type="button"
                size="icon-sm"
                variant="outline"
                aria-label={`Bỏ duyệt ${candidate.text}`}
                title="Bỏ duyệt"
                onClick={(event) => {
                  event.stopPropagation();
                  onUndo();
                }}
              >
                <RotateCcw />
              </Button>
            </>
          ) : (
            <Button type="button" size="icon-sm" variant="outline" className="text-ok" aria-label={`Duyệt ${candidate.text}`} onClick={(event) => { event.stopPropagation(); onAccept(suggested); }}>
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
      <mark className="rounded bg-highlight px-1 font-semibold text-highlight-foreground">{value.slice(start + 1, end)}</mark>
      {value.slice(end + 1)}
    </>
  );
}
