import { FileDown, FolderOpen } from "lucide-react";
import { useCallback } from "react";
import { toast } from "sonner";

import { ChapterList } from "@/components/chapter-list";
import { ChapterReader } from "@/components/chapter-reader";
import { TranslateToolbar } from "@/components/translate-toolbar";
import { Button } from "@/components/ui/button";
import { describeImport, useChapterDrop } from "@/hooks/use-chapter-drop";
import { revealFolder } from "@/lib/api";
import { filterChapters } from "@/lib/chapters";
import type { ImportOutcome } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

export function TranslatePage() {
  const root = useStoryStore((s) => s.root);
  const snapshot = useStoryStore((s) => s.snapshot);
  const selectedId = useStoryStore((s) => s.selectedId);
  const filter = useStoryStore((s) => s.statusFilter);
  const query = useStoryStore((s) => s.searchQuery);
  const select = useStoryStore((s) => s.select);
  const setFilter = useStoryStore((s) => s.setStatusFilter);
  const setQuery = useStoryStore((s) => s.setSearchQuery);
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const onImported = useCallback(
    (outcome: ImportOutcome) => {
      setSnapshot(outcome.snapshot);
      const message = describeImport(outcome);
      if (outcome.added.length > 0) toast.success(message);
      else toast.message(message);
    },
    [setSnapshot],
  );
  const onImportError = useCallback((message: string) => toast.error(message), []);
  const dragging = useChapterDrop(root, onImported, onImportError);
  if (!root || !snapshot) return null;
  const visible = filterChapters(snapshot.chapters, filter, query);
  const index = visible.findIndex((c) => c.id === selectedId);
  const row = index >= 0 ? visible[index] : snapshot.chapters.find((c) => c.id === selectedId);
  const empty = snapshot.counts.total === 0;
  return (
    <div className="relative flex h-full flex-col">
      <TranslateToolbar />
      <div className="grid min-h-0 flex-1 grid-cols-[280px_1fr]">
        <aside className="min-h-0 border-r bg-card/50">
          {empty ? (
            <div className="flex h-full flex-col items-center justify-center gap-3 p-4 text-center text-sm text-muted-foreground">
              <FileDown className="size-8 text-muted-foreground/60" />
              <p className="font-medium text-foreground">Chưa có chương</p>
              <p>
                Thả file <code className="font-mono">.txt</code> vào cửa sổ này, hoặc copy vào{" "}
                <code className="font-mono">raw/</code> rồi bấm Quét lại.
              </p>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => void revealFolder(`${root}${root.includes("\\") ? "\\" : "/"}raw`)}
              >
                <FolderOpen /> Mở folder raw/
              </Button>
            </div>
          ) : (
            <ChapterList
              rows={snapshot.chapters}
              filter={filter}
              query={query}
              selectedId={selectedId}
              onSelect={select}
              onFilter={setFilter}
              onQuery={setQuery}
            />
          )}
        </aside>
        <section className="min-h-0 min-w-0">
          {row ? (
            <ChapterReader
              root={root}
              row={row}
              ordinal={snapshot.chapters.findIndex((c) => c.id === row.id) + 1}
              hasPrev={index > 0}
              hasNext={index >= 0 && index < visible.length - 1}
              onPrev={() => {
                const prev = visible[index - 1];
                if (prev) select(prev.id);
              }}
              onNext={() => {
                const next = visible[index + 1];
                if (next) select(next.id);
              }}
            />
          ) : (
            <div className="flex h-full flex-col items-center justify-center gap-2 text-center text-sm text-muted-foreground">
              <p className="text-base font-medium text-foreground">Chọn một chương bên trái</p>
              <p>Bản dịch, bản nháp, bản gốc và log agy sẽ hiện ở đây.</p>
            </div>
          )}
        </section>
      </div>
      <div
        aria-hidden={!dragging}
        className={cn(
          "pointer-events-none absolute inset-0 z-20 flex items-center justify-center bg-background/80 transition-opacity",
          dragging ? "opacity-100" : "opacity-0",
        )}
      >
        <div className="rounded-xl border-2 border-dashed border-primary bg-card px-8 py-6 text-center shadow-lg">
          <FileDown className="mx-auto size-8 text-primary" />
          <p className="mt-2 font-medium">Thả để thêm vào raw/</p>
          <p className="text-xs text-muted-foreground">Nhận file .txt hoặc folder chứa .txt; file trùng tên bị bỏ qua.</p>
        </div>
      </div>
    </div>
  );
}
