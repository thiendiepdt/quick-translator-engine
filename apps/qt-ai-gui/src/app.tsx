import { LoaderCircle } from "lucide-react";
import { useCallback, useEffect, type ComponentType } from "react";
import { toast } from "sonner";

import { AgyMissing } from "@/components/agy-missing";
import { AppRail } from "@/components/app-rail";
import { ExportPage } from "@/components/pages/export-page";
import { SettingsPage } from "@/components/pages/settings-page";
import { StoryPage } from "@/components/pages/story-page";
import { TranslatePage } from "@/components/pages/translate-page";
import { StoryPicker } from "@/components/story-picker";
import { useSessionEvents } from "@/hooks/use-session-events";
import { useThemeSync } from "@/hooks/use-theme";
import { agyStatus, appConfigGet, appConfigSet, pickAgyFile } from "@/lib/api";
import { useStoryStore, type Page } from "@/store/story";

const PAGES: Record<Page, ComponentType> = {
  translate: TranslatePage,
  story: StoryPage,
  export: ExportPage,
  settings: SettingsPage,
};

export default function App() {
  const agy = useStoryStore((s) => s.agy);
  const screen = useStoryStore((s) => s.screen);
  const page = useStoryStore((s) => s.page);
  const setAgy = useStoryStore((s) => s.setAgy);
  const setConfig = useStoryStore((s) => s.setConfig);
  const setPage = useStoryStore((s) => s.setPage);
  const engine = useStoryStore((s) => s.config?.engine ?? "agy");
  useSessionEvents();
  useThemeSync();

  const probe = useCallback(async () => {
    try {
      const config = await appConfigGet();
      setConfig(config);
      setAgy(await agyStatus(config.agyPath ?? undefined));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không đọc được cấu hình");
    }
  }, [setAgy, setConfig]);

  useEffect(() => {
    void probe();
  }, [probe]);

  async function pickAgy() {
    const path = await pickAgyFile();
    if (!path) return;
    const config = useStoryStore.getState().config;
    if (!config) return;
    setConfig(await appConfigSet({ ...config, agyPath: path }));
    await probe();
  }

  if (!agy) {
    return (
      <main className="flex h-full items-center justify-center text-sm text-muted-foreground">
        <LoaderCircle className="mr-2 animate-spin" /> Đang kiểm tra agy…
      </main>
    );
  }
  async function switchToApi() {
    const config = useStoryStore.getState().config;
    if (!config) return;
    try {
      setConfig(await appConfigSet({ ...config, engine: "api" }));
      setPage("settings");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được cấu hình");
    }
  }

  // Thiếu agy chỉ chặn khi động cơ là agy; API key chạy được mà không cần agy.
  if (!agy.found && engine === "agy") {
    return (
      <AgyMissing
        status={agy}
        onRetry={() => void probe()}
        onPickPath={() => void pickAgy()}
        onUseApi={() => void switchToApi()}
      />
    );
  }
  if (screen === "picker") return <StoryPicker />;
  const Current = PAGES[page];
  return (
    <div className="flex h-full">
      <AppRail />
      <main className="min-w-0 flex-1 overflow-hidden">
        <Current />
      </main>
    </div>
  );
}
