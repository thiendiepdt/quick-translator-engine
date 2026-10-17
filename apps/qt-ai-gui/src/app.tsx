import { LoaderCircle } from "lucide-react";
import { useCallback, useEffect, useRef, type ComponentType } from "react";
import { toast } from "sonner";

import { AgyMissing } from "@/components/agy-missing";
import { AppRail } from "@/components/app-rail";
import { ExportPage } from "@/components/pages/export-page";
import { ReplacePage } from "@/components/pages/replace-page";
import { SettingsPage } from "@/components/pages/settings-page";
import { StoryPage } from "@/components/pages/story-page";
import { TranslatePage } from "@/components/pages/translate-page";
import { VnCheckPage } from "@/components/pages/vn-check-page";
import { StoryPicker } from "@/components/story-picker";
import { StorySidebar } from "@/components/story-sidebar";
import { WhatsNewDialog } from "@/components/whats-new";
import { useSessionEvents } from "@/hooks/use-session-events";
import { useThemeSync } from "@/hooks/use-theme";
import { useUndoFallback } from "@/hooks/use-undo-fallback";
import { useUpdateCheck } from "@/hooks/use-update-check";
import { useWhatsNew } from "@/hooks/use-whats-new";
import { agyStatus, appConfigGet, appConfigSet, pickAgyFile } from "@/lib/api";
import { useStoryStore, type Page } from "@/store/story";

const PAGES: Record<Page, ComponentType> = {
  translate: TranslatePage,
  story: StoryPage,
  vn: VnCheckPage,
  replace: ReplacePage,
  export: ExportPage,
  settings: SettingsPage,
};

export default function App() {
  const agy = useStoryStore((s) => s.agy);
  const screen = useStoryStore((s) => s.screen);
  const page = useStoryStore((s) => s.page);
  const root = useStoryStore((s) => s.root);
  const setAgy = useStoryStore((s) => s.setAgy);
  const setConfig = useStoryStore((s) => s.setConfig);
  const setPage = useStoryStore((s) => s.setPage);
  const config = useStoryStore((s) => s.config);
  const engine = config?.engine ?? "api";
  useSessionEvents();
  useThemeSync();
  useUndoFallback();
  useUpdateCheck();
  // Dấu hiệu đã dùng app trước bản có "Có gì mới" (chưa có khoá last-seen): config.json trên đĩa đã có truyện gần đây
  // hoặc thư viện. Không dùng khoá palette/theme trong localStorage: useThemeSync ghi chúng ngay khi config nạp,
  // nên máy cài mới cũng có trước khi kịp kiểm tra.
  const whatsNew = useWhatsNew(config ? config.recent.length > 0 || config.libraryRoot !== null : undefined);

  // Chỉ dò agy khi động cơ là agy: người dùng API key không phải chờ, không bị màn "Chưa thấy agy".
  // `probing` chặn dò chồng: mỗi lượt dò spawn `agy --version` + `agy models` (~2s), dò chồng liên tục
  // từng làm hàng chục tiến trình agy chạy song song và mọi lệnh khác của app treo theo.
  const probing = useRef(false);
  const probe = useCallback(async () => {
    if (probing.current) return;
    probing.current = true;
    try {
      const config = await appConfigGet();
      setConfig(config);
      if (config.engine === "agy") {
        try {
          setAgy(await agyStatus(config.agyPath ?? undefined));
        } catch (error) {
          // Dò hỏng → hiện màn "Chưa thấy agy" có nút thử lại thay vì xoay vô hạn.
          const message = error instanceof Error ? error.message : "Không dò được agy";
          setAgy({ found: false, path: null, version: null, models: [], message });
        }
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không đọc được cấu hình");
    } finally {
      probing.current = false;
    }
  }, [setAgy, setConfig]);

  useEffect(() => {
    void probe();
  }, [probe]);

  // Đổi sang agy giữa chừng (Cài đặt) mà chưa dò lần nào thì dò lúc đó. Không phụ thuộc `config`:
  // mỗi lượt dò lại setConfig → object mới → effect chạy lại → vòng lặp dò vô hạn.
  useEffect(() => {
    if (engine === "agy" && !agy) void probe();
  }, [engine, agy, probe]);

  async function pickAgy() {
    const path = await pickAgyFile();
    if (!path) return;
    const config = useStoryStore.getState().config;
    if (!config) return;
    setConfig(await appConfigSet({ ...config, agyPath: path }));
    await probe();
  }

  if (!config || (engine === "agy" && !agy)) {
    return (
      <main className="flex h-full items-center justify-center text-sm text-muted-foreground">
        <LoaderCircle className="mr-2 animate-spin" /> {config ? "Đang kiểm tra agy…" : "Đang đọc cấu hình…"}
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
  if (engine === "agy" && agy && !agy.found) {
    return (
      <AgyMissing
        status={agy}
        onRetry={() => void probe()}
        onPickPath={() => void pickAgy()}
        onUseApi={() => void switchToApi()}
      />
    );
  }
  // Cài đặt chỉ có trong truyện: ở màn chọn truyện, "Xem tất cả" mở danh sách ngay trong hộp thoại.
  const whatsNewDialog = (
    <WhatsNewDialog
      version={whatsNew.version}
      since={whatsNew.since}
      onClose={whatsNew.dismiss}
      onShowAll={
        screen === "workbench"
          ? () => {
              setPage("settings");
              whatsNew.dismiss();
            }
          : undefined
      }
    />
  );
  if (screen === "picker") {
    return (
      <>
        <StoryPicker />
        {whatsNewDialog}
      </>
    );
  }
  const Current = PAGES[page];
  return (
    <div className="flex h-full">
      <AppRail />
      <main className="min-w-0 flex-1 overflow-hidden">
        {/* Dựng lại trang khi đổi truyện: kết quả tìm, hộp thoại, form của truyện trước không được áp sang truyện mới. */}
        <Current key={root} />
      </main>
      <StorySidebar />
      {whatsNewDialog}
    </div>
  );
}
