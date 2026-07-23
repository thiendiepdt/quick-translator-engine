import { zodResolver } from "@hookform/resolvers/zod";
import {
  BookType,
  Braces,
  CircleHelp,
  Languages,
  LoaderCircle,
  Send,
  Server,
  Settings2,
} from "lucide-react";
import { lazy, Suspense, useEffect, useState } from "react";
import { FormProvider, useForm, useWatch } from "react-hook-form";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
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
import { ApiError } from "@/lib/api";
import {
  type ParsedTranslationOptions,
  type TranslationOptionsValues,
  translationOptionsSchema,
} from "@/lib/schema";
import type { TranslationRequest } from "@/lib/types";
import { cn } from "@/lib/utils";
import {
  useDictionaryDefaultsQuery,
  useHealthQuery,
  useTranslationMutation,
} from "@/hooks/use-translation";
import { dictionaryPayload, useWorkspaceStore } from "@/store/workspace";

const defaultEndpoint = import.meta.env.VITE_QT_API_URL?.trim() || "/api";
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

function useDebouncedValue(value: string, delayMs: number): string {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const timeout = window.setTimeout(() => setDebounced(value), delayMs);
    return () => window.clearTimeout(timeout);
  }, [delayMs, value]);
  return debounced;
}

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
      <RailItem label="API" disabled><Braces className="size-5" /></RailItem>
      <div className="flex-1" />
      <RailItem label="Trợ giúp"><CircleHelp className="size-5" /></RailItem>
    </nav>
  );
}

export default function App() {
  const sourceText = useWorkspaceStore((state) => state.sourceText);
  const response = useWorkspaceStore((state) => state.response);
  const dictionaries = useWorkspaceStore((state) => state.dictionaries);
  const dictionaryDefaultsEndpoint = useWorkspaceStore(
    (state) => state.dictionaryDefaultsEndpoint,
  );
  const hydrateDictionaryDefaults = useWorkspaceStore(
    (state) => state.hydrateDictionaryDefaults,
  );
  const setResponse = useWorkspaceStore((state) => state.setResponse);
  const workspaceView = useWorkspaceStore((state) => state.workspaceView);
  const setWorkspaceView = useWorkspaceStore((state) => state.setWorkspaceView);
  const mobileInspectorOpen = useWorkspaceStore((state) => state.mobileInspectorOpen);
  const setMobileInspectorOpen = useWorkspaceStore((state) => state.setMobileInspectorOpen);

  const form = useForm<TranslationOptionsValues, unknown, ParsedTranslationOptions>({
    resolver: zodResolver(translationOptionsSchema),
    defaultValues: {
      endpoint: defaultEndpoint,
      pretty: true,
      wrap: false,
      prioritizedName: true,
      scanRange: 30,
      translationAlgorithm: 1,
    },
  });
  const endpoint = useWatch({ control: form.control, name: "endpoint" }) ?? defaultEndpoint;
  const normalizedEndpoint = endpoint.trim();
  const dictionaryEndpoint = useDebouncedValue(normalizedEndpoint, 400);
  const translation = useTranslationMutation();
  const health = useHealthQuery(endpoint);
  const dictionaryDefaults = useDictionaryDefaultsQuery(dictionaryEndpoint);

  useEffect(() => {
    if (dictionaryDefaults.data) {
      hydrateDictionaryDefaults(dictionaryEndpoint, dictionaryDefaults.data);
    }
  }, [dictionaryDefaults.data, dictionaryEndpoint, hydrateDictionaryDefaults]);

  const dictionaryDefaultsReady =
    dictionaryEndpoint === normalizedEndpoint &&
    dictionaryDefaultsEndpoint === normalizedEndpoint &&
    dictionaryDefaults.isSuccess;
  const dictionaryDefaultsStatus = dictionaryDefaultsReady
    ? "ready"
    : dictionaryDefaults.isError && dictionaryEndpoint === normalizedEndpoint
      ? "error"
      : "loading";

  async function submit(values: ParsedTranslationOptions) {
    if (!sourceText.trim()) {
      toast.error("Dán nguyên văn tiếng Trung trước khi dịch");
      return;
    }
    if (dictionaryDefaultsEndpoint !== values.endpoint || !dictionaryDefaults.isSuccess) {
      toast.error("Chưa tải xong từ điển mặc định từ engine");
      return;
    }

    const request: TranslationRequest = {
      text: sourceText,
      mode: "vietphrase-one",
      wrap: values.wrap,
      pretty: values.pretty,
      ranges: true,
      scanRange: values.scanRange,
      translationAlgorithm: values.translationAlgorithm,
      prioritizedName: values.prioritizedName,
      dictionaries: dictionaryPayload(dictionaries),
    };

    try {
      const result = await translation.mutateAsync({ endpoint: values.endpoint, request });
      setResponse(result);
      const rangeCount = result.sourceRanges?.length ?? 0;
      toast.success(`Dịch xong${rangeCount > 0 ? ` · ${rangeCount} range` : ""}`);
    } catch (error) {
      const requestId = error instanceof ApiError ? error.requestId : undefined;
      toast.error(error instanceof Error ? error.message : "Không thể dịch chương", {
        description: requestId ? `Request ID: ${requestId}` : undefined,
      });
    }
  }

  async function testConnection() {
    const valid = await form.trigger("endpoint");
    if (!valid) return;
    const result = await health.refetch();
    if (result.data?.status === "ok") toast.success("Cloudflare gateway đang hoạt động");
    else toast.error(result.error instanceof Error ? result.error.message : "Không kết nối được gateway");
  }

  const requestStatus = !dictionaryDefaultsReady
    ? dictionaryDefaultsStatus === "error"
      ? "Không tải được từ điển mặc định"
      : "Đang tải từ điển mặc định QT2025"
    : translation.isPending
      ? "Đang gọi Cloudflare gateway → Lambda"
      : translation.isError
        ? `Lỗi: ${translation.error.message}`
        : response && translation.isSuccess
          ? `Hoàn tất · ${response.sourceRanges?.length ?? 0} cặp range`
          : response
            ? "Văn bản mẫu · chưa gọi API"
            : "Sẵn sàng dịch một chương";

  const endpointError = form.formState.errors.endpoint?.message;

  return (
    <FormProvider {...form}>
      <form
        className="h-dvh min-h-[720px] overflow-hidden"
        onSubmit={(event) => {
          if (workspaceView !== "translate") {
            event.preventDefault();
            return;
          }
          void form.handleSubmit(submit, () => toast.error("Kiểm tra lại cấu hình request"))(event);
        }}
      >
        <header className="grid h-16 grid-cols-[minmax(200px,280px)_minmax(260px,1fr)_auto] items-center border-b bg-card md:grid-cols-[280px_minmax(300px,640px)_1fr_auto]">
          <div className="flex h-full items-center gap-3 px-4 md:px-5">
            <div className="grid size-9 shrink-0 place-items-center border border-foreground/30 font-mono text-[11px] font-bold">QT</div>
            <div className="min-w-0">
              <div className="truncate text-sm font-bold tracking-tight md:text-base">quick translator <span className="text-primary">/ engine</span></div>
              <div className="mt-1 hidden font-mono text-[9px] tracking-[0.16em] text-muted-foreground uppercase sm:block">web · vietphrase one</div>
            </div>
          </div>

          <div className="relative hidden items-center gap-2 px-2 md:flex">
            <Label htmlFor="endpoint" className="shrink-0 font-mono text-[9px] font-semibold tracking-wide text-muted-foreground uppercase">Cloudflare API</Label>
            <Input
              id="endpoint"
              aria-invalid={Boolean(endpointError)}
              className="h-9 min-w-0 bg-muted/60 font-mono text-[11px]"
              spellCheck={false}
              {...form.register("endpoint")}
            />
            {endpointError ? <span className="absolute top-[46px] left-28 z-20 rounded bg-destructive px-2 py-1 text-[9px] text-white shadow">{endpointError}</span> : null}
          </div>

          <div className="hidden items-center justify-end gap-2 px-3 md:flex">
            <Button type="button" size="sm" variant="ghost" disabled={health.isFetching} onClick={() => void testConnection()}>
              {health.isFetching ? <LoaderCircle className="animate-spin" /> : <Server />}
              {health.data?.status === "ok" ? "Gateway online" : "Test gateway"}
            </Button>
            <span className={cn("size-1.5 rounded-full bg-slate-300", health.data?.status === "ok" && "bg-emerald-600", health.isError && "bg-destructive")} />
          </div>

          <div className="flex h-full items-center justify-end gap-1 pr-3">
            <Button type="button" variant="ghost" size="icon" className={cn(workspaceView === "translate" && "xl:hidden")} aria-label="Mở cấu hình" onClick={() => setMobileInspectorOpen(true)}><Settings2 /></Button>
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
              <TranslationWorkspace isPending={translation.isPending} requestStatus={requestStatus} />
            ) : (
              <NameFilterWorkspace endpoint={normalizedEndpoint} defaultsReady={dictionaryDefaultsReady} />
            )}
          </Suspense>
          {workspaceView === "translate" ? <aside className="hidden min-h-0 border-l xl:block" aria-label="Cấu hình request">
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
              <SheetTitle>Cấu hình request</SheetTitle>
              <SheetDescription>Tùy chỉnh từ điển và engine.</SheetDescription>
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
