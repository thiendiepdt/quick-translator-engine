import { ArrowDownUp, FolderOpen, LoaderCircle } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { rawFiles, revealFolder, type RawFiles, type RawToc } from "@/lib/api";
import { STATUS_LABELS } from "@/lib/types";
import { useStoryStore } from "@/store/story";

const message = (error: unknown) => (error instanceof Error ? error.message : String(error));

/** Khung chờ của một danh sách (mục lục, raw trong máy): vòng quay + vài dòng giả. */
export function ListSkeleton({ label }: { label: string }) {
  return (
    <div role="status" aria-label={label} className="rounded-lg border">
      <div className="flex items-center gap-2 border-b px-4 py-2 text-sm text-muted-foreground">
        <LoaderCircle className="size-4 animate-spin" /> {label}
      </div>
      <ul className="divide-y">
        {[0, 1, 2, 3, 4, 5].map((i) => (
          <li key={i} className="flex items-center gap-3 px-4 py-2">
            <div className="size-5 shrink-0 animate-pulse rounded bg-muted" />
            <div className="h-3 w-10 shrink-0 animate-pulse rounded bg-muted" />
            <div className="h-3 flex-1 animate-pulse rounded bg-muted" style={{ maxWidth: `${70 - i * 7}%` }} />
          </li>
        ))}
      </ul>
    </div>
  );
}

interface Props {
  root: string;
  /** Mục lục nguồn đã tải (nếu có) — để ghi file nào khớp mục số mấy. */
  toc?: RawToc;
  /** Đổi số này để đọc lại (tải xong chương, dọn raw). */
  version: number;
  newestFirst: boolean;
  onToggleOrder: () => void;
}

/** Tab "Raw trong máy": file chương đang có trong raw/, tiêu đề, số chữ, khớp mục nào, trạng thái dịch. */
export function RawLocalFiles({ root, toc, version, newestFirst, onToggleOrder }: Props) {
  const chapters = useStoryStore((s) => s.snapshot?.chapters);
  const select = useStoryStore((s) => s.select);
  const setPage = useStoryStore((s) => s.setPage);
  const [view, setView] = useState<RawFiles>();
  const [query, setQuery] = useState("");

  useEffect(() => {
    let alive = true;
    rawFiles(root).then(
      (next) => alive && setView(next),
      (error: unknown) => alive && toast.error(message(error)),
    );
    return () => {
      alive = false;
    };
  }, [root, version]);

  const tocIndex = useMemo(() => new Map((toc?.rows ?? []).flatMap((row) => (row.file ? [[row.file, row.index] as const] : []))), [toc]);
  const status = useMemo(() => new Map((chapters ?? []).map((c) => [c.id, c.status])), [chapters]);

  const rows = useMemo(() => {
    if (!view) return [];
    const q = query.trim().toLowerCase();
    const found = q ? view.files.filter((f) => f.id.toLowerCase().includes(q) || f.title.toLowerCase().includes(q)) : view.files;
    return newestFirst ? [...found].reverse() : found;
  }, [view, query, newestFirst]);

  if (!view) return <ListSkeleton label="Đang đọc raw trong máy…" />;

  const totalChars = view.files.reduce((sum, f) => sum + (f.chars ?? 0), 0);
  const unreadable = view.files.filter((f) => f.chars === null).length;

  return (
    <div className="rounded-lg border">
      <div className="flex flex-wrap items-center gap-x-3 gap-y-2 border-b px-4 py-2 text-sm text-muted-foreground">
        <span>
          {view.files.length} file · {totalChars.toLocaleString("vi-VN")} chữ
          {unreadable > 0 && <span className="text-destructive"> · {unreadable} file không đọc được</span>}
        </span>
        <span className="ml-auto flex items-center gap-1">
          <Button variant="ghost" size="sm" onClick={() => void revealFolder(view.dir)}>
            <FolderOpen /> Mở thư mục raw/
          </Button>
          <Button
            variant="ghost"
            size="sm"
            aria-label={newestFirst ? "Đang xếp mới đến cũ — bấm để xếp cũ đến mới" : "Đang xếp cũ đến mới — bấm để xếp mới đến cũ"}
            onClick={onToggleOrder}
          >
            <ArrowDownUp /> {newestFirst ? "Mới → cũ" : "Cũ → mới"}
          </Button>
        </span>
      </div>
      {view.files.length > 0 && (
        <div className="border-b px-4 py-2">
          <Input aria-label="Lọc raw theo tên file / tiêu đề" placeholder="Lọc theo tên file / tiêu đề…" className="h-8" value={query} onChange={(e) => setQuery(e.target.value)} />
        </div>
      )}
      {view.files.length === 0 ? (
        <p className="px-4 py-6 text-center text-sm text-muted-foreground">raw/ chưa có file chương nào.</p>
      ) : rows.length === 0 ? (
        <p className="px-4 py-6 text-center text-sm text-muted-foreground">Không có file nào khớp “{query.trim()}”.</p>
      ) : (
        <ul className="divide-y text-sm">
          {rows.map((file) => {
            const index = tocIndex.get(file.id);
            const state = status.get(file.id);
            return (
              <li key={file.id} aria-label={`File ${file.id}`} className="flex items-center gap-3 px-4 py-1.5">
                <span className="w-40 shrink-0 truncate font-mono text-xs text-muted-foreground" title={`${file.id}.txt`}>
                  {file.id}
                </span>
                <span className="min-w-0 flex-1 truncate" title={file.title}>
                  {file.title || <span className="text-muted-foreground">(trống)</span>}
                </span>
                {index !== undefined && <span className="shrink-0 font-mono text-xs text-muted-foreground">mục {index}</span>}
                <span className="w-20 shrink-0 text-right text-xs text-muted-foreground">
                  {file.chars === null ? <span className="text-destructive">không đọc được</span> : `${file.chars.toLocaleString("vi-VN")} chữ`}
                </span>
                {state && (
                  <Badge variant={state === "done" ? "secondary" : "outline"} className="shrink-0">
                    {STATUS_LABELS[state]}
                  </Badge>
                )}
                <Button
                  variant="ghost"
                  size="sm"
                  aria-label={`Xem ${file.id} ở trang Dịch`}
                  disabled={!state}
                  title={state ? undefined : "Chưa có trong hàng đợi dịch — bấm Quét raw/ ở trang Dịch"}
                  onClick={() => {
                    select(file.id);
                    setPage("translate");
                  }}
                >
                  Xem
                </Button>
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
