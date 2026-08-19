import { zodResolver } from "@hookform/resolvers/zod";
import {
  BookType,
  CircleHelp,
  Languages,
  PanelRight,
  Settings2,
  Sparkles,
} from "lucide-react";
import { lazy, Suspense, useEffect, useState } from "react";
import { FormProvider, useForm, useWatch } from "react-hook-form";
import { toast } from "sonner";

import { ensurePersistentStorage } from "@/lib/persistent-storage";
import {
  aiStoryIsEmpty,
  readStoryMirror,
  writeStoryMirror,
} from "@/lib/story-mirror";

import { SettingsDialog } from "@/components/settings-dialog";
import { WorkspaceSwitcher } from "@/components/workspace-switcher";
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
import { ApiError } from "@/lib/api";
import {
  readStoredEngineSettings,
  storeEngineSettings,
} from "@/lib/engine-settings";
import {
  type ParsedTranslationOptions,
  type TranslationOptionsValues,
  translationOptionsSchema,
} from "@/lib/schema";
import { readStoredAiSettings, storeAiSettings } from "@/lib/ai-settings";
import {
  pathForWorkspaceView,
  workspaceViewFromPath,
} from "@/lib/workspace-route";
import { readStoredEndpoint, storeEndpoint } from "@/lib/endpoint-setting";
import type { TranslationRequest } from "@/lib/types";
import { cn } from "@/lib/utils";
import {
  useDictionaryDefaultsQuery,
  useHealthQuery,
  useTranslationMutation,
} from "@/hooks/use-translation";
import {
  dictionaryPatchPayload,
  dictionaryPayload,
  type WorkspaceView,
  useWorkspaceStore,
} from "@/store/workspace";
import { useWorkspaceCatalogStore } from "@/store/workspace-catalog";

const configuredEndpoint = import.meta.env.VITE_QT_API_URL?.trim() || "/api";
const defaultEndpoint = readStoredEndpoint(configuredEndpoint);
const defaultEngineSettings = readStoredEngineSettings();
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
const AiTranslationWorkspace = lazy(() =>
  import("@/components/ai-translation-workspace").then((module) => ({
    default: module.AiTranslationWorkspace,
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
    <div className="flex h-full flex-col gap-3 bg-card p-4" aria-label="Đang tải cấu hình">
      <div className="h-6 w-40 animate-pulse rounded bg-muted" />
      <div className="mt-1 grid grid-cols-2 gap-2">
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
  children,
}: {
  label: string;
  active?: boolean;
  onClick?: () => void;
  children: React.ReactNode;
}) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          type="button"
          onClick={onClick}
          className={cn(
            "relative grid min-h-13 w-full place-items-center text-[10px] font-semibold text-sidebar-foreground/55 transition-colors hover:bg-sidebar-accent hover:text-sidebar-foreground",
            active &&
              "bg-sidebar-accent text-sidebar-foreground before:absolute before:inset-y-2 before:left-0 before:w-0.5 before:bg-sidebar-ring",
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
  onOpenSettings,
}: {
  active: WorkspaceView;
  onChange: (value: WorkspaceView) => void;
  onOpenSettings: () => void;
}) {
  return (
    <nav className="hidden min-h-0 flex-col items-center bg-sidebar text-sidebar-foreground md:flex" aria-label="Điều hướng chính">
      <div className="my-3 grid size-9 place-items-center rounded-sm border border-sidebar-foreground/25 font-mono text-xs font-bold">QT</div>
      <RailItem label="Convert" active={active === "translate"} onClick={() => onChange("translate")}><Languages className="size-5" /></RailItem>
      <RailItem label="Dịch AI" active={active === "ai-translate"} onClick={() => onChange("ai-translate")}><Sparkles className="size-5" /></RailItem>
      <RailItem label="Tên" active={active === "names"} onClick={() => onChange("names")}><BookType className="size-5" /></RailItem>
      <div className="flex-1" />
      <RailItem label="Cài đặt" onClick={onOpenSettings}><Settings2 className="size-5" /></RailItem>
      <RailItem label="Trợ giúp"><CircleHelp className="size-5" /></RailItem>
    </nav>
  );
}

export default function App() {
  const sourceText = useWorkspaceStore((state) => state.sourceText);
  const response = useWorkspaceStore((state) => state.response);
  const dictionaries = useWorkspaceStore((state) => state.dictionaries);
  const localDictionaryEntries = useWorkspaceStore(
    (state) => state.localDictionaryEntries,
  );
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
  const activeWorkspaceId = useWorkspaceCatalogStore(
    (state) => state.activeWorkspaceId,
  );
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [aiSettings, setAiSettings] = useState(readStoredAiSettings);

  const form = useForm<TranslationOptionsValues, unknown, ParsedTranslationOptions>({
    resolver: zodResolver(translationOptionsSchema),
    defaultValues: {
      endpoint: defaultEndpoint,
      ...defaultEngineSettings,
    },
  });
  const endpoint = useWatch({ control: form.control, name: "endpoint" }) ?? defaultEndpoint;
  const pretty = useWatch({ control: form.control, name: "pretty" });
  const wrap = useWatch({ control: form.control, name: "wrap" });
  const prioritizedName = useWatch({ control: form.control, name: "prioritizedName" });
  const scanRange = useWatch({ control: form.control, name: "scanRange" });
  const translationAlgorithm = useWatch({
    control: form.control,
    name: "translationAlgorithm",
  });
  const normalizedEndpoint = endpoint.trim();
  const dictionaryEndpoint = useDebouncedValue(normalizedEndpoint, 400);
  const translation = useTranslationMutation();
  const health = useHealthQuery(endpoint);
  const dictionaryDefaults = useDictionaryDefaultsQuery(dictionaryEndpoint);

  // Xin chế độ persistent cho IndexedDB ngay khi vào app: không được cấp thì
  // trình duyệt có quyền tự dọn dữ liệu (mất công dịch) — cảnh báo một lần.
  useEffect(() => {
    void ensurePersistentStorage().then((status) => {
      if (status !== "denied" && status !== "unsupported") return;
      const warnedKey = "qt-web-persist-warning-shown";
      try {
        if (window.localStorage.getItem(warnedKey)) return;
        window.localStorage.setItem(warnedKey, "1");
      } catch {
        // localStorage bị chặn thì vẫn cảnh báo, chấp nhận lặp lại.
      }
      toast.warning("Trình duyệt chưa cho lưu dữ liệu vĩnh viễn", {
        duration: 12_000,
        description:
          "IndexedDB có thể bị tự dọn khi thiếu dung lượng. Cài trang thành ứng dụng (Install app) hoặc dùng thường xuyên để được cấp quyền, và nhớ tải các chương đã dịch về máy làm backup.",
      });
    });
  }, []);

  // Cấu hình truyện có bản sao dự phòng trong localStorage: IDB hỏng riêng lẻ
  // thì khôi phục lại (không chống được eviction cả origin — persist() lo).
  useEffect(() => {
    const persistApi = useWorkspaceStore.persist;
    const workspaceKey = () => persistApi.getOptions().name ?? "qt-web-workspace-v1";
    function maybeRestoreFromMirror() {
      const current = useWorkspaceStore.getState().aiStory;
      if (!aiStoryIsEmpty(current)) return;
      const mirror = readStoryMirror(workspaceKey());
      if (!mirror || aiStoryIsEmpty(mirror)) return;
      useWorkspaceStore.getState().updateAiStory(mirror);
      toast.message("Đã khôi phục cấu hình truyện từ bản sao dự phòng", {
        description:
          "IndexedDB không còn cấu hình truyện — đã lấy lại bản sao trong localStorage.",
      });
    }
    if (persistApi.hasHydrated()) maybeRestoreFromMirror();
    // Đổi workspace cũng rehydrate → kiểm tra lại với key mới.
    const unsubscribeHydration = persistApi.onFinishHydration(maybeRestoreFromMirror);
    let lastStory = useWorkspaceStore.getState().aiStory;
    const unsubscribeStore = useWorkspaceStore.subscribe((state) => {
      if (state.aiStory === lastStory) return;
      lastStory = state.aiStory;
      // Không ghi đè mirror tốt bằng config trống (lúc chuyển workspace).
      if (!aiStoryIsEmpty(state.aiStory)) writeStoryMirror(workspaceKey(), state.aiStory);
    });
    return () => {
      unsubscribeHydration();
      unsubscribeStore();
    };
  }, []);

  useEffect(() => {
    storeEndpoint(endpoint);
  }, [endpoint]);

  useEffect(() => {
    storeAiSettings(aiSettings);
  }, [aiSettings]);

  // Mỗi khung có URL riêng (/, /dich-ai, /loc-ten) để bookmark/F5 giữ nguyên
  // tab; back/forward của trình duyệt cũng chuyển tab.
  useEffect(() => {
    const path = pathForWorkspaceView(workspaceView);
    if (window.location.pathname !== path) {
      window.history.pushState(null, "", path);
    }
  }, [workspaceView]);

  useEffect(() => {
    function onPopState() {
      setWorkspaceView(workspaceViewFromPath(window.location.pathname) ?? "translate");
    }
    window.addEventListener("popstate", onPopState);
    return () => window.removeEventListener("popstate", onPopState);
  }, [setWorkspaceView]);

  useEffect(() => {
    storeEngineSettings({
      pretty,
      wrap,
      prioritizedName,
      scanRange,
      translationAlgorithm,
    });
  }, [pretty, prioritizedName, scanRange, translationAlgorithm, wrap]);

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
  const canTranslate =
    workspaceView === "translate" && !translation.isPending && dictionaryDefaultsReady;

  async function submit(values: ParsedTranslationOptions) {
    if (!sourceText.trim()) {
      toast.error("Dán nguyên văn tiếng Trung trước khi dịch");
      return;
    }
    if (dictionaryDefaultsEndpoint !== values.endpoint || !dictionaryDefaults.isSuccess) {
      toast.error("Chưa tải xong từ điển mặc định từ bộ máy dịch");
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
      dictionaryPatches: dictionaryPatchPayload(localDictionaryEntries),
    };
    const requestWorkspaceId = activeWorkspaceId;

    try {
      const result = await translation.mutateAsync({ endpoint: values.endpoint, request });
      if (
        useWorkspaceCatalogStore.getState().activeWorkspaceId !==
        requestWorkspaceId
      ) return;
      setResponse(result);
      const rangeCount = result.sourceRanges?.length ?? 0;
      toast.success(`Dịch xong${rangeCount > 0 ? ` · ${rangeCount} cặp` : ""}`);
    } catch (error) {
      if (
        useWorkspaceCatalogStore.getState().activeWorkspaceId !==
        requestWorkspaceId
      ) return;
      const requestId = error instanceof ApiError ? error.requestId : undefined;
      toast.error(error instanceof Error ? error.message : "Không thể dịch chương", {
        description: requestId ? `Request ID: ${requestId}` : undefined,
      });
    }
  }

  function runTranslate() {
    void form.handleSubmit(submit, () => {
      setSettingsOpen(true);
      toast.error("Endpoint chưa hợp lệ");
    })();
  }

  // ⌘/Ctrl + Enter dịch ngay, không phải rời tay khỏi bàn phím để bấm nút.
  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key !== "Enter" || !(event.metaKey || event.ctrlKey)) return;
      if (!canTranslate) return;
      event.preventDefault();
      runTranslate();
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  });

  async function testConnection() {
    const valid = await form.trigger("endpoint");
    if (!valid) return;
    const result = await health.refetch();
    if (result.data?.status === "ok") toast.success("API đang hoạt động");
    else toast.error(result.error instanceof Error ? result.error.message : "Không kết nối được API");
  }

  const requestStatus = !dictionaryDefaultsReady
    ? dictionaryDefaultsStatus === "error"
      ? "Không tải được từ điển mặc định"
      : "Đang tải từ điển mặc định"
    : translation.isPending
      ? "Đang dịch…"
      : translation.isError
        ? `Lỗi: ${translation.error.message}`
        : response && translation.isSuccess
          ? `Xong · ${response.sourceRanges?.length ?? 0} cặp`
          : response
            ? "Văn bản mẫu · chưa gọi API"
            : "Sẵn sàng";

  const gatewayStatus = health.data?.status === "ok" ? "ok" : health.isError ? "error" : "unknown";
  const touchedCount = Object.values(dictionaries).filter(({ touched }) => touched).length;
  const endpointInvalid = Boolean(form.formState.errors.endpoint);

  return (
    <FormProvider {...form}>
      <form
        className="flex h-dvh min-h-0 flex-col overflow-hidden"
        onSubmit={(event) => {
          event.preventDefault();
          if (workspaceView !== "translate") return;
          runTranslate();
        }}
      >
        {/* Thanh trên gom toàn bộ chrome toàn cục. Endpoint và nút kiểm tra
            gateway đã chuyển vào dialog Cài đặt — chỉ còn một chấm trạng thái. */}
        <header className="flex h-14 shrink-0 items-center gap-3 border-b bg-card px-3 md:px-4">
          <div className="grid size-9 shrink-0 place-items-center rounded-sm border border-foreground/25 font-mono text-[11px] font-bold md:hidden">QT</div>
          <div className="min-w-0 truncate text-sm font-bold tracking-tight md:text-base">
            Quick Translator <span className="text-primary">/ Engine</span>
          </div>
          {/* Chip đã thay cho dấu ngoặc đơn, nên không lồng thêm ngoặc vào trong.
              Bỏ mono in hoa dãn chữ để cùng nhịp với wordmark viết hoa đầu từ. */}
          <span className="hidden shrink-0 rounded border border-primary/25 bg-primary/6 px-2 py-0.5 text-xs font-medium text-primary sm:block">
            {workspaceView === "translate"
              ? "VietPhrase 1 nghĩa"
              : workspaceView === "ai-translate"
                ? "Dịch AI trung thành raw"
                : "Lọc Names"}
          </span>

          <select
            aria-label="Khung làm việc"
            className="h-8 min-w-0 rounded border bg-background px-2 text-xs md:hidden"
            value={workspaceView}
            onChange={(event) => setWorkspaceView(event.target.value as WorkspaceView)}
          >
            <option value="translate">Convert</option>
            <option value="ai-translate">Dịch AI</option>
            <option value="names">Tên</option>
          </select>

          <div className="flex-1" />

          <WorkspaceSwitcher />

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                type="button"
                variant="ghost"
                size="icon"
                aria-label="Cài đặt endpoint và giao diện"
                onClick={() => setSettingsOpen(true)}
              >
                <span className="relative">
                  <Settings2 />
                  <span
                    className={cn(
                      "absolute -top-0.5 -right-0.5 size-1.5 rounded-full",
                      endpointInvalid || gatewayStatus === "error"
                        ? "bg-destructive"
                        : gatewayStatus === "ok"
                          ? "bg-ok"
                          : "bg-transparent",
                    )}
                  />
                </span>
              </Button>
            </TooltipTrigger>
            <TooltipContent>Cài đặt · giao diện</TooltipContent>
          </Tooltip>

          {workspaceView === "translate" ? (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  className="xl:hidden"
                  aria-label="Mở từ điển và bộ máy"
                  onClick={() => setMobileInspectorOpen(true)}
                >
                  <span className="relative">
                    <PanelRight />
                    {touchedCount > 0 ? (
                      <span className="absolute -top-0.5 -right-0.5 size-1.5 rounded-full bg-ok" />
                    ) : null}
                  </span>
                </Button>
              </TooltipTrigger>
              <TooltipContent>
                Từ điển &amp; bộ máy{touchedCount > 0 ? ` · ${touchedCount} đã sửa` : ""}
              </TooltipContent>
            </Tooltip>
          ) : null}

        </header>

        <div className={cn(
          "grid min-h-0 flex-1 md:grid-cols-[56px_minmax(0,1fr)]",
          workspaceView === "translate" && "xl:grid-cols-[56px_minmax(0,1fr)_320px]",
        )}>
          <NavigationRail
            active={workspaceView}
            onChange={setWorkspaceView}
            onOpenSettings={() => setSettingsOpen(true)}
          />
          <Suspense fallback={<div className="m-3 animate-pulse rounded-lg border bg-card" />}>
            {workspaceView === "translate" ? (
              <TranslationWorkspace
                key={activeWorkspaceId}
                endpoint={normalizedEndpoint}
                canTranslate={canTranslate}
                isPending={translation.isPending}
                onTranslate={runTranslate}
                aiSettings={aiSettings}
                onOpenSettings={() => setSettingsOpen(true)}
                requestStatus={requestStatus}
              />
            ) : workspaceView === "ai-translate" ? (
              <AiTranslationWorkspace
                key={activeWorkspaceId}
                aiSettings={aiSettings}
                onOpenSettings={() => setSettingsOpen(true)}
              />
            ) : (
              <NameFilterWorkspace
                key={activeWorkspaceId}
                endpoint={normalizedEndpoint}
                defaultsReady={dictionaryDefaultsReady}
                aiSettings={aiSettings}
                onOpenSettings={() => setSettingsOpen(true)}
              />
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
          <SheetContent side="right" className={cn("w-[min(92vw,390px)] gap-0 overflow-hidden p-0 sm:max-w-[390px]", workspaceView === "translate" && "xl:hidden")}>
            <SheetHeader className="sr-only">
              <SheetTitle>Cấu hình request</SheetTitle>
              <SheetDescription>Tùy chỉnh từ điển và bộ máy dịch.</SheetDescription>
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

        <SettingsDialog
          open={settingsOpen}
          onOpenChange={setSettingsOpen}
          gatewayStatus={gatewayStatus}
          gatewayChecking={health.isFetching}
          onTestGateway={() => void testConnection()}
          aiSettings={aiSettings}
          onAiSettingsChange={setAiSettings}
        />
      </form>
    </FormProvider>
  );
}
