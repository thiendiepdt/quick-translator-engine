import type { RecentSummary } from "@/lib/types";
import { cn } from "@/lib/utils";

export type StorySource = NonNullable<RecentSummary["source"]>;

/** Nhãn nguồn của truyện: tên extension sẽ tải raw (màu chủ đạo), chưa có extension thì tên miền (viền nhạt). */
export function SourceChip({ source, className }: { source: StorySource; className?: string }) {
  return (
    <span
      title={source.extension ? `Tải raw qua extension ${source.label} — ${source.url}` : `Chưa có extension cho nguồn này — ${source.url}`}
      className={cn(
        "max-w-48 shrink-0 truncate rounded-full px-2 py-0.5 text-[11px] font-medium",
        source.extension ? "bg-primary/10 text-primary" : "border text-muted-foreground",
        className,
      )}
    >
      {source.label}
    </span>
  );
}
