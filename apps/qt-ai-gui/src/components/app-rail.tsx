import { BookUser, Download, Languages, LibraryBig, Moon, Settings2, Sun } from "lucide-react";
import type { ReactNode } from "react";

import { Button } from "@/components/ui/button";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { useThemeActions } from "@/hooks/use-theme";
import { cn } from "@/lib/utils";
import { useStoryStore, type Page } from "@/store/story";

const ITEMS: Array<{ page: Page; label: string; icon: typeof Languages }> = [
  { page: "translate", label: "Dịch", icon: Languages },
  { page: "story", label: "Hồ sơ truyện", icon: BookUser },
  { page: "export", label: "Export", icon: Download },
  { page: "settings", label: "Cài đặt", icon: Settings2 },
];

function RailButton({
  label,
  active,
  disabled,
  onClick,
  children,
}: {
  label: string;
  active?: boolean;
  disabled?: boolean;
  onClick: () => void;
  children: ReactNode;
}) {
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          aria-label={label}
          aria-current={active ? "page" : undefined}
          disabled={disabled}
          onClick={onClick}
          className={cn(
            "size-10 rounded-lg text-muted-foreground hover:text-foreground",
            active && "bg-accent text-accent-foreground hover:bg-accent",
          )}
        >
          {children}
        </Button>
      </TooltipTrigger>
      <TooltipContent side="right">{label}</TooltipContent>
    </Tooltip>
  );
}

export function AppRail() {
  const page = useStoryStore((s) => s.page);
  const setPage = useStoryStore((s) => s.setPage);
  const running = useStoryStore((s) => s.session.status === "running");
  const closeStory = useStoryStore((s) => s.closeStory);
  const { mode, toggleMode } = useThemeActions();
  return (
    <nav className="flex h-full w-14 shrink-0 flex-col items-center gap-1 border-r bg-card py-3">
      <div
        className="mb-3 flex size-9 items-center justify-center rounded-lg bg-primary text-sm font-semibold text-primary-foreground"
        aria-hidden
      >
        Q
      </div>
      {ITEMS.map(({ page: item, label, icon: Icon }) => (
        <RailButton key={item} label={label} active={page === item} onClick={() => setPage(item)}>
          <Icon className="size-5" />
        </RailButton>
      ))}
      <div className="flex-1" />
      <RailButton label={mode === "dark" ? "Chuyển sang sáng" : "Chuyển sang tối"} onClick={() => void toggleMode()}>
        {mode === "dark" ? <Sun className="size-5" /> : <Moon className="size-5" />}
      </RailButton>
      <RailButton
        label={running ? "Dừng phiên trước khi đổi truyện" : "Về danh sách truyện"}
        disabled={running}
        onClick={closeStory}
      >
        <LibraryBig className="size-5" />
      </RailButton>
    </nav>
  );
}
