import { ImageOff, Loader2, RefreshCw } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { sourceLabel, type CoverStatus } from "@/hooks/use-book-info";

function statusText(status: CoverStatus, isDefault: boolean): string {
  switch (status.kind) {
    case "loading":
      return "Đang lấy tác giả và ảnh bìa từ link nguồn…";
    case "fetched":
      return status.source ? `Ảnh bìa lấy từ ${sourceLabel(status.source)}.` : "Dùng ảnh bìa đã lưu.";
    case "unsupported":
      return "Link nguồn không hỗ trợ (hoặc chưa có link) — dùng bìa mặc định có tên truyện.";
    case "no-cover":
      return `${sourceLabel(status.source)} không có ảnh bìa — dùng bìa mặc định.`;
    case "error":
      return `Lỗi: ${status.message}${isDefault ? " — tạm dùng bìa mặc định." : ""}`;
  }
}

export function EpubBookPanel({
  author,
  onAuthorChange,
  onAuthorCommit,
  status,
  busy,
  previewUrl,
  isDefaultCover,
  onRefetch,
}: {
  author: string;
  onAuthorChange: (value: string) => void;
  onAuthorCommit: () => void;
  status: CoverStatus;
  busy: boolean;
  previewUrl?: string;
  isDefaultCover: boolean;
  onRefetch: () => void;
}) {
  return (
    <section className="rounded-lg border bg-card p-5" aria-label="Thông tin sách">
      <h2 className="mb-4 text-sm font-semibold">Thông tin sách</h2>
      <div className="flex gap-5">
        <div className="flex aspect-[2/3] w-32 shrink-0 items-center justify-center overflow-hidden rounded-md border bg-muted">
          {previewUrl ? (
            <img src={previewUrl} alt={isDefaultCover ? "Bìa mặc định" : "Ảnh bìa"} className="h-full w-full object-cover" />
          ) : status.kind === "loading" ? (
            <Loader2 className="size-5 animate-spin text-muted-foreground" />
          ) : (
            <ImageOff className="size-5 text-muted-foreground" />
          )}
        </div>
        <div className="flex min-w-0 flex-1 flex-col gap-3">
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="epub-author">Tác giả</Label>
            <Input
              id="epub-author"
              value={author}
              onChange={(e) => onAuthorChange(e.target.value)}
              onBlur={onAuthorCommit}
              placeholder="Để trống nếu không rõ"
            />
          </div>
          <p className={status.kind === "error" ? "text-sm text-status-warning" : "text-sm text-muted-foreground"}>
            {statusText(status, isDefaultCover)}
          </p>
          <div>
            <Button size="sm" variant="outline" disabled={busy} onClick={onRefetch}>
              {busy ? <Loader2 className="animate-spin" /> : <RefreshCw />} Tải lại ảnh bìa
            </Button>
          </div>
        </div>
      </div>
    </section>
  );
}
