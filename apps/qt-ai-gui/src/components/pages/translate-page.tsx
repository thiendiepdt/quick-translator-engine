import { ChapterList } from "@/components/chapter-list";
import { ChapterReader } from "@/components/chapter-reader";
import { TranslateToolbar } from "@/components/translate-toolbar";
import { filterChapters } from "@/lib/chapters";
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
  if (!root || !snapshot) return null;
  const visible = filterChapters(snapshot.chapters, filter, query);
  const index = visible.findIndex((c) => c.id === selectedId);
  const row = index >= 0 ? visible[index] : snapshot.chapters.find((c) => c.id === selectedId);
  return (
    <div className="flex h-full flex-col">
      <TranslateToolbar />
      <div className="grid min-h-0 flex-1 grid-cols-[280px_1fr]">
        <aside className="min-h-0 border-r bg-card/50">
          <ChapterList
            rows={snapshot.chapters}
            filter={filter}
            query={query}
            selectedId={selectedId}
            onSelect={select}
            onFilter={setFilter}
            onQuery={setQuery}
          />
        </aside>
        <section className="min-h-0 min-w-0">
          {row ? (
            <ChapterReader
              root={root}
              row={row}
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
    </div>
  );
}
