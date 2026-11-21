import { Download } from "lucide-react";

import type { RecentSummary } from "@/lib/types";
import { cn } from "@/lib/utils";

export type StorySource = NonNullable<RecentSummary["source"]>;

function chipTitle(source: StorySource | undefined, inApp: boolean): string {
  if (!source) return "Đã đánh dấu tải raw trong app nhưng truyện chưa có link tải raw";
  if (inApp) {
    return source.extension
      ? `Tải raw trong app qua extension ${source.label} — ${source.url}`
      : `Đã đánh dấu tải raw trong app nhưng chưa có extension cho link này — ${source.url}`;
  }
  return source.extension
    ? `Extension ${source.label} khớp link; truyện chưa đánh dấu tải raw trong app — ${source.url}`
    : `Chưa có extension cho nguồn này — ${source.url}`;
}

/**
 * Nhãn nguồn của truyện: tên extension khớp link tải raw, không thì tên miền. Nền màu chủ đạo + dấu tải chỉ khi người
 * dùng đánh dấu truyện tải raw trong app (trang Tải raw) — có extension khớp chưa chắc truyện tải trong app, có truyện
 * lấy raw miễn phí bên ngoài. Đánh dấu mà chưa có link thì chip "Tải trong app". Không có gì để hiện thì không vẽ.
 */
export function SourceChip({ source, inApp = false, className }: { source?: StorySource; inApp?: boolean; className?: string }) {
  if (!source && !inApp) return null;
  return (
    <span
      title={chipTitle(source, inApp)}
      data-in-app={inApp || undefined}
      className={cn(
        "max-w-48 shrink-0 truncate rounded-full px-2 py-0.5 text-[11px] font-medium",
        inApp ? "bg-primary/10 text-primary" : "border text-muted-foreground",
        className,
      )}
    >
      {/* Biểu tượng nằm inline (không flex) để tên dài vẫn cắt bằng dấu … */}
      {inApp && <Download className="mr-0.5 inline size-3 align-[-2px]" aria-hidden />}
      {source?.label ?? "Tải trong app"}
    </span>
  );
}
