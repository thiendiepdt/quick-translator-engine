import { cn } from "@/lib/utils";

/** Mã chương ứng với số thứ tự vừa gõ, để người dùng chắc mình chọn đúng. Ẩn khi ô trống. */
export function ChapterRefHint({ text, id }: { text: string; id: string }) {
  if (!text.trim()) return null;
  return (
    <p className={cn("truncate font-mono text-[11px]", id ? "text-muted-foreground" : "text-destructive")} title={id}>
      {id ? `→ ${id}` : "không có chương này"}
    </p>
  );
}
