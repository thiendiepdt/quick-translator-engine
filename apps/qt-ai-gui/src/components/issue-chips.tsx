import { cn } from "@/lib/utils";

/**
 * Chip "Lỗi N" (đỏ) / "Bỏ qua N" (vàng) cạnh tên truyện ở màn chọn truyện và hộp Chuyển truyện: chương lỗi hay bị
 * bỏ qua không tự được dịch lại, phải mở truyện ra xử lý. Không có thì không vẽ gì.
 */
export function IssueChips({ error, skipped, className }: { error: number; skipped: number; className?: string }) {
  const chip = cn("shrink-0 rounded-full px-2 py-0.5 text-[11px] font-medium", className);
  return (
    <>
      {error > 0 && (
        <span
          title={`${error} chương lỗi — mở truyện, lọc Lỗi để dịch lại hoặc Chốt --force`}
          className={cn(chip, "bg-status-error/15 text-status-error")}
        >
          Lỗi {error}
        </span>
      )}
      {skipped > 0 && (
        <span
          title={`${skipped} chương bị bỏ qua — mở truyện, lọc Bỏ qua để dịch lại`}
          className={cn(chip, "bg-status-warning/15 text-status-warning")}
        >
          Bỏ qua {skipped}
        </span>
      )}
    </>
  );
}
