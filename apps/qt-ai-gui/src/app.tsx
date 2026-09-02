import { LoaderCircle } from "lucide-react";
import { useCallback, useEffect } from "react";
import { toast } from "sonner";

import { AgyMissing } from "@/components/agy-missing";
import { StoryPicker } from "@/components/story-picker";
import { Workbench } from "@/components/workbench";
import { useSessionEvents } from "@/hooks/use-session-events";
import { agyStatus, appConfigGet, appConfigSet, pickAgyFile } from "@/lib/api";
import { useStoryStore } from "@/store/story";

export default function App() {
  const agy = useStoryStore((s) => s.agy);
  const screen = useStoryStore((s) => s.screen);
  const setAgy = useStoryStore((s) => s.setAgy);
  const setConfig = useStoryStore((s) => s.setConfig);
  useSessionEvents();

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
      <main className="flex h-screen items-center justify-center text-sm text-muted-foreground">
        <LoaderCircle className="mr-2 animate-spin" /> Đang kiểm tra agy…
      </main>
    );
  }
  if (!agy.found) {
    return <AgyMissing status={agy} onRetry={() => void probe()} onPickPath={() => void pickAgy()} />;
  }
  return screen === "picker" ? <StoryPicker /> : <Workbench />;
}
