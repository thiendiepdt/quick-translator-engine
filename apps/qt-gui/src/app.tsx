import { zodResolver } from "@hookform/resolvers/zod";
import { useQueryClient } from "@tanstack/react-query";
import {
  BookType,
  CircleHelp,
  Database,
  FolderOpen,
  HardDrive,
  Languages,
  LoaderCircle,
  Send,
  Settings2,
} from "lucide-react";
import { lazy, Suspense, useEffect } from "react";
import { FormProvider, useForm, useWatch } from "react-hook-form";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  useDictionaryDefaultsQuery,
  useEngineStatusQuery,
  useLoadEngineMutation,
  useTranslationMutation,
} from "@/hooks/use-translation";
import { ApiError, chooseDataDirectory } from "@/lib/api";
import {
  type ParsedTranslationOptions,
  type TranslationOptionsValues,
  translationOptionsSchema,
} from "@/lib/schema";
import type { TranslationRequest } from "@/lib/types";
import { cn } from "@/lib/utils";
import {
  dictionaryPatchPayload,
  dictionaryPayload,
  useWorkspaceStore,
} from "@/store/workspace";

const DictionaryInspector = lazy(() =>
  import("@/components/dictionary-inspector").then((module) => ({
    default: module.DictionaryInspector,
  })),
);
const TranslationWorkspace = lazy(() =>
  import("@/components/translation-workspace").then((module) => ({
    default: module.TranslationWorkspace,
  })),
);
const NameFilterWorkspace = lazy(() =>
  import("@/components/name-filter-workspace").then((module) => ({
    default: module.NameFilterWorkspace,
  })),
);

function InspectorFallback() {
  return (
    <div className="flex h-full flex-col gap-3 bg-card p-5" aria-label="Đang tải cấu hình">
      <div className="h-3 w-28 animate-pulse rounded bg-muted" />
      <div className="h-6 w-44 animate-pulse rounded bg-muted" />
      <div className="mt-3 grid grid-cols-2 gap-2">
        {Array.from({ length: 8 }, (_, index) => (
          <div key={index} className="h-9 animate-pulse rounded bg-muted" />
        ))}
      </div>
      <div className="mt-3 h-32 animate-pulse rounded bg-muted" />
    </div>
  );
}

function RailItem({
  label,
  active = false,
  onClick,
  disabled = false,
  children,
}: {
  label: string;
  active?: boolean;
  onClick?: () => void;
  disabled?: boolean;
  children: React.ReactNode;
}) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          type="button"
          disabled={disabled}
          onClick={onClick}
          className={cn(
            "relative grid min-h-14 w-full place-items-center text-[10px] font-semibold text-slate-400 transition-colors hover:bg-white/8 hover:text-white",
            active && "bg-white/8 text-white before:absolute before:inset-y-2 before:left-0 before:w-0.5 before:bg-blue-400",
            disabled && "opacity-45",
          )}
          aria-label={label}
        >
          <span className="flex flex-col items-center gap-1">{children}<span>{label}</span></span>
        </button>
      </TooltipTrigger>
      <TooltipContent side="right" sideOffset={6}>{label}</TooltipContent>
    </Tooltip>
  );
}

function NavigationRail({
  active,
  onChange,
}: {
  active: "translate" | "names";
  onChange: (value: "translate" | "names") => void;
}) {
  return (
    <nav className="hidden min-h-0 flex-col items-center bg-[var(--sidebar)] text-[var(--sidebar-foreground)] md:flex" aria-label="Điều hướng chính">
      <div className="my-3 grid size-9 place-items-center border border-white/25 font-mono text-xs font-bold">QT</div>
      <RailItem label="Dịch" active={active === "translate"} onClick={() => onChange("translate")}><Languages className="size-5" /></RailItem>
      <RailItem label="Tên" active={active === "names"} onClick={() => onChange("names")}><BookType className="size-5" /></RailItem>
      <RailItem label="Local" disabled><Database className="size-5" /></RailItem>
      <div className="flex-1" />
      <RailItem label="Trợ giúp"><CircleHelp className="size-5" /></RailItem>
    </nav>
  );
}

export default function App() {
  const sourceText = useWorkspaceStore((state) => state.sourceText);
  const response = useWorkspaceStore((state) => state.response);
  const dictionaries = useWorkspaceStore((state) => state.dictionaries);
  const localDictionaryEntries = useWorkspaceStore((state) => state.localDictionaryEntries);
  const dictionaryDefaultsSource = useWorkspaceStore(
    (state) => state.dictionaryDefaultsSource,
  );
  const hydrateDictionaryDefaults = useWorkspaceStore(
    (state) => state.hydrateDictionaryDefaults,
  );
  const setResponse = useWorkspaceStore((state) => state.setResponse);
  const workspaceView = useWorkspaceStore((state) => state.workspaceView);
  const setWorkspaceView = useWorkspaceStore((state) => state.setWorkspaceView);
  const mobileInspectorOpen = useWorkspaceStore((state) => state.mobileInspectorOpen);
  const setMobileInspectorOpen = useWorkspaceStore((state) => state.setMobileInspectorOpen);
  const queryClient = useQueryClient();

  const form = useForm<TranslationOptionsValues, unknown, ParsedTranslationOptions>({
    resolver: zodResolver(translationOptionsSchema),
    defaultValues: {
      mode: "vietphrase-one",
      pretty: true,
      wrap: false,
      prioritizedName: true,
      scanRange: 30,
      translationAlgorithm: 1,
    },
  });
  const engineStatus = useEngineStatusQuery();
  const loadEngine = useLoadEngineMutation();
  const currentStatus = engineStatus.data;
  const dataDir = currentStatus?.ready ? currentStatus.dataDir : undefined;
  const dictionaryDefaults = useDictionaryDefaultsQuery(dataDir);
  const translation = useTranslationMutation();
  const selectedMode = useWatch({ control: form.control, name: "mode" }) ?? "vietphrase-one";

  useEffect(() => {
    if (dataDir && dictionaryDefaults.data) {
      hydrateDictionaryDefaults(dataDir, dictionaryDefaults.data);
    }
  }, [dataDir, dictionaryDefaults.data, hydrateDictionaryDefaults]);

  const dictionaryDefaultsReady =
    Boolean(dataDir) &&
    dictionaryDefaultsSource === dataDir &&
    dictionaryDefaults.isSuccess;
  const dictionaryDefaultsStatus = dictionaryDefaultsReady
    ? "ready"
    : dictionaryDefaults.isError
      ? "error"
      : "loading";

  async function selectDataDirectory() {
    try {
      const selected = await chooseDataDirectory();
      if (!selected) return;
      const status = await loadEngine.mutateAsync(selected);
      queryClient.setQueryData(["desktop", "engine-status"], status);
      toast.success("Đã nạp bộ từ điển local", { description: status.dataDir });
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không nạp được thư mục dữ liệu");
    }
  }

  async function submit(values: ParsedTranslationOptions) {
    if (!sourceText.trim()) {
      toast.error("Dán hoặc mở nguyên văn tiếng Trung trước khi dịch");
      return;
    }
    if (!dictionaryDefaultsReady) {
      toast.error("Chưa nạp xong từ điển local");
      return;
    }

    const request: TranslationRequest = {
      text: sourceText,
      mode: values.mode,
      wrap: values.wrap,
      pretty: values.pretty,
      ranges: true,
      scanRange: values.scanRange,
      translationAlgorithm: values.translationAlgorithm,
      prioritizedName: values.prioritizedName,
      dictionaries: dictionaryPayload(dictionaries),
      dictionaryPatches: dictionaryPatchPayload(localDictionaryEntries),
    };

    try {
      const result = await translation.mutateAsync(request);
      setResponse(result);
      const rangeCount = result.sourceRanges?.length ?? 0;
      toast.success(`Dịch xong${rangeCount > 0 ? ` · ${rangeCount} range` : ""}`);
    } catch (error) {
      toast.error(
        error instanceof ApiError || error instanceof Error
          ? error.message
          : "Không thể dịch chương",
      );
    }
  }

  const requestStatus = !currentStatus?.ready
    ? currentStatus?.message ?? "Chọn thư mục dữ liệu QT2025 để bắt đầu"
    : !dictionaryDefaultsReady
      ? dictionaryDefaultsStatus === "error"
        ? "Không tải được từ điển mặc định"
        : "Đang nạp từ điển local"
      : translation.isPending
        ? "Đang dịch trực tiếp bằng qt-core"
        : translation.isError
          ? `Lỗi: ${translation.error.message}`
          : response && translation.isSuccess
            ? `Hoàn tất · ${response.sourceRanges?.length ?? 0} cặp range`
            : response
              ? "Văn bản mẫu · chưa chạy engine"
              : "Engine local sẵn sàng";

  return (
    <FormProvider {...form}>
      <form
        className="h-dvh min-h-[700px] overflow-hidden"
        onSubmit={(event) => {
          if (workspaceView !== "translate") {
            event.preventDefault();
            return;
          }
          void form.handleSubmit(submit, () => toast.error("Kiểm tra lại cấu hình engine"))(event);
        }}
      >
        <header className="grid h-16 grid-cols-[minmax(230px,320px)_minmax(0,1fr)_auto] items-center border-b bg-card">
          <div className="flex h-full items-center gap-3 px-4 md:px-5">
            <div className="grid size-9 shrink-0 place-items-center border border-foreground/30 font-mono text-[11px] font-bold">QT</div>
            <div className="min-w-0">
              <div className="truncate text-sm font-bold tracking-tight md:text-base">quick translator <span className="text-primary">/ gui</span></div>
              <div className="mt-1 hidden font-mono text-[9px] tracking-[0.16em] text-muted-foreground uppercase sm:block">tauri · local engine</div>
            </div>
          </div>

          <div className="hidden min-w-0 items-center gap-3 px-3 md:flex">
            <HardDrive className={cn("size-4 shrink-0", currentStatus?.ready ? "text-emerald-600" : "text-muted-foreground")} />
            <div className="min-w-0">
              <div className="font-mono text-[9px] font-semibold tracking-wide text-muted-foreground uppercase">Data directory</div>
              <div className="truncate font-mono text-[11px]" title={dataDir}>
                {dataDir ?? "Chưa chọn dữ liệu QT2025"}
              </div>
            </div>
            <Button
              type="button"
              size="sm"
              variant="ghost"
              className="ml-auto shrink-0"
              disabled={loadEngine.isPending}
              onClick={() => void selectDataDirectory()}
            >
              {loadEngine.isPending ? <LoaderCircle className="animate-spin" /> : <FolderOpen />}
              Chọn thư mục
            </Button>
          </div>

          <div className="flex h-full items-center justify-end gap-1 pr-3">
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className={cn(workspaceView === "translate" && "xl:hidden")}
              aria-label="Mở cấu hình"
              onClick={() => setMobileInspectorOpen(true)}
            >
              <Settings2 />
            </Button>
            {workspaceView === "translate" ? (
              <Button type="submit" className="h-10 px-4" disabled={translation.isPending || !dictionaryDefaultsReady}>
                {translation.isPending || dictionaryDefaultsStatus === "loading" ? <LoaderCircle className="animate-spin" /> : <Send />}
                <span className="hidden sm:inline">Dịch chương</span>
              </Button>
            ) : null}
          </div>
        </header>

        <div className={cn(
          "grid h-[calc(100dvh-4rem)] min-h-0 md:grid-cols-[64px_minmax(0,1fr)]",
          workspaceView === "translate" && "xl:grid-cols-[64px_minmax(0,1fr)_340px]",
        )}>
          <NavigationRail active={workspaceView} onChange={setWorkspaceView} />
          <Suspense fallback={<div className="m-4 animate-pulse rounded-lg border bg-card" />}>
            {workspaceView === "translate" ? (
              <TranslationWorkspace
                isPending={translation.isPending}
                requestStatus={requestStatus}
                mode={selectedMode}
              />
            ) : (
              <NameFilterWorkspace defaultsReady={dictionaryDefaultsReady} />
            )}
          </Suspense>
          {workspaceView === "translate" ? <aside className="hidden min-h-0 border-l xl:block" aria-label="Cấu hình engine">
            <Suspense fallback={<InspectorFallback />}>
              <DictionaryInspector
                defaultsStatus={dictionaryDefaultsStatus}
                defaultsError={dictionaryDefaults.error?.message}
                onRetry={() => void dictionaryDefaults.refetch()}
              />
            </Suspense>
          </aside> : null}
        </div>

        <Sheet open={mobileInspectorOpen} onOpenChange={setMobileInspectorOpen}>
          <SheetContent side="right" className={cn("w-[min(92vw,390px)] gap-0 p-0 sm:max-w-[390px]", workspaceView === "translate" && "xl:hidden")}>
            <SheetHeader className="sr-only">
              <SheetTitle>Cấu hình engine</SheetTitle>
              <SheetDescription>Tùy chỉnh từ điển và engine local.</SheetDescription>
            </SheetHeader>
            <Suspense fallback={<InspectorFallback />}>
              <DictionaryInspector
                mobile
                defaultsStatus={dictionaryDefaultsStatus}
                defaultsError={dictionaryDefaults.error?.message}
                onRetry={() => void dictionaryDefaults.refetch()}
              />
            </Suspense>
          </SheetContent>
        </Sheet>
      </form>
    </FormProvider>
  );
}
