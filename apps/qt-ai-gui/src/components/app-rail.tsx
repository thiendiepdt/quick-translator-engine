import { Activity, ArrowLeft, BookUser, CloudDownload, Download, FileType, Languages, LibraryBig, Moon, Replace, Settings2, ShieldAlert, Sun } from "lucide-react";
import { useEffect, type ReactNode } from "react";

import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useThemeActions } from "@/hooks/use-theme";
import { cn } from "@/lib/utils";
import { loadRawToc, useRawNew } from "@/store/raw-check";
import { useStoryStore, type Page } from "@/store/story";

const ITEMS: Array<{ page: Page; label: string; icon: typeof Languages }> = [
  { page: "translate", label: "Dịch", icon: Languages },
  { page: "story", label: "Hồ sơ truyện", icon: BookUser },
  { page: "raw", label: "Tải raw", icon: CloudDownload },
  { page: "vn", label: "Kiểm tra VN", icon: ShieldAlert },
  { page: "replace", label: "Tìm & thay (Ctrl+Shift+F)", icon: Replace },
  { page: "export", label: "Export", icon: Download },
  { page: "settings", label: "Cài đặt", icon: Settings2 },
];

function RailButton({
  label,
  active,
  disabled,
  badge,
  onClick,
  children,
}: {
  label: string;
  active?: boolean;
  disabled?: boolean;
  /** Số nhỏ ở góc nút (vd. số chương raw mới chưa tải). */
  badge?: number;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          aria-label={badge ? `${label} — ${badge} chương mới` : label}
          aria-current={active ? "page" : undefined}
          disabled={disabled}
          onClick={onClick}
          className={cn(
            "relative size-10 rounded-lg text-muted-foreground hover:text-foreground",
            active && "bg-accent text-accent-foreground hover:bg-accent",
          )}
        >
          {children}
          {badge ? (
            <span className="absolute -right-0.5 -top-0.5 min-w-4 rounded-full bg-primary px-1 text-center text-[10px] font-semibold leading-4 text-primary-foreground">
              {badge > 99 ? "99+" : badge}
            </span>
          ) : null}
        </Button>
      </TooltipTrigger>
      <TooltipContent side="right">{badge ? `${label} — ${badge} chương mới chưa tải` : label}</TooltipContent>
    </Tooltip>
  );
}

export function AppRail() {
  const page = useStoryStore((s) => s.page);
  const setPage = useStoryStore((s) => s.setPage);
  const closeStory = useStoryStore((s) => s.closeStory);
  const openConverter = useStoryStore((s) => s.openConverter);
  const openProcesses = useStoryStore((s) => s.openProcesses);
  const { mode, toggleMode } = useThemeActions();
  const root = useStoryStore((s) => s.root);
  const rawNew = useRawNew((s) => (root ? s.counts[root] : null));

  // Mở truyện: tải mục lục nền (chỉ khi đã đủ cài đặt tải raw) để biết số chương mới; trang Tải raw dùng lại kết quả.
  useEffect(() => {
    if (root) loadRawToc(root).catch(() => undefined);
  }, [root]);

  // Ctrl+Shift+F mở trang Tìm & thay như ô tìm trong cả dự án của các trình soạn thảo.
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key.toLowerCase() === "f") {
        event.preventDefault();
        setPage("replace");
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [setPage]);

  return (
    <nav className="flex h-full w-14 shrink-0 flex-col items-center gap-1 border-r bg-card py-3">
      <div
        className="mb-3 flex size-9 items-center justify-center rounded-lg bg-primary text-sm font-semibold text-primary-foreground"
        aria-hidden
      >
        Q
      </div>
      {/* Nút quay về đặt đầu rail, có viền + mũi tên + chữ để khác hẳn các nút trang; trước đây là icon
          xám dưới đáy nên người dùng không nhận ra. */}
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="outline"
            aria-label="Về danh sách truyện"
            onClick={closeStory}
            className="mb-1 flex h-12 w-11 flex-col items-center justify-center gap-0.5 rounded-lg border-primary/40 px-0 text-foreground hover:border-primary hover:bg-accent"
          >
            <span className="flex items-center">
              <ArrowLeft className="size-3.5" />
              <LibraryBig className="size-5" />
            </span>
            <span className="text-[10px] leading-none">Truyện</span>
          </Button>
        </TooltipTrigger>
        <TooltipContent side="right">Về danh sách truyện</TooltipContent>
      </Tooltip>
      <div className="mb-2 h-px w-8 bg-border" aria-hidden />
      {ITEMS.map(({ page: item, label, icon: Icon }) => (
        <RailButton key={item} label={label} active={page === item} badge={item === "raw" ? (rawNew ?? undefined) : undefined} onClick={() => setPage(item)}>
          <Icon className="size-5" />
        </RailButton>
      ))}
      <div className="my-1 h-px w-8 bg-border" aria-hidden />
      {/* Công cụ riêng, không gắn truyện: mở màn riêng, nút Quay lại về đúng truyện này. */}
      <RailButton label="Chuyển phồn → giản (công cụ riêng)" onClick={openConverter}>
        <FileType className="size-5" />
      </RailButton>
      <RailButton label="Tiến trình: phiên dịch, agy, khoá phiên" onClick={() => openProcesses()}>
        <Activity className="size-5" />
      </RailButton>
      <div className="flex-1" />
      <RailButton label={mode === "dark" ? "Chuyển sang sáng" : "Chuyển sang tối"} onClick={() => void toggleMode()}>
        {mode === "dark" ? <Sun className="size-5" /> : <Moon className="size-5" />}
      </RailButton>
    </nav>
  );
}
