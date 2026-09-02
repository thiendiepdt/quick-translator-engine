import { toast } from "sonner";

import { ChapterPanel } from "@/components/chapter-panel";
import { ChapterTable } from "@/components/chapter-table";
import { LogPanel } from "@/components/log-panel";
import { ProgressHeader } from "@/components/progress-header";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useStoryStore } from "@/store/story";

export function Workbench() {
  const root = useStoryStore((s) => s.root);
  const snapshot = useStoryStore((s) => s.snapshot);
  const selectedId = useStoryStore((s) => s.selectedId);
  const filter = useStoryStore((s) => s.statusFilter);
  const select = useStoryStore((s) => s.select);
  const setFilter = useStoryStore((s) => s.setStatusFilter);
  if (!root || !snapshot) return null;
  const row = snapshot.chapters.find((c) => c.id === selectedId);
  const soon = () => toast.message("Sắp có");
  return (
    <div className="flex h-screen flex-col">
      <ProgressHeader onOpenStory={soon} onOpenSettings={soon} onOpenExport={soon} />
      <div className="grid min-h-0 flex-1 grid-cols-[360px_1fr]">
        <aside className="min-h-0 border-r">
          <ChapterTable
            rows={snapshot.chapters}
            filter={filter}
            selectedId={selectedId}
            onSelect={select}
            onFilter={setFilter}
          />
        </aside>
        <main className="min-h-0">
          <Tabs defaultValue="chapter" className="flex h-full flex-col">
            <TabsList className="mx-2 mt-2 w-fit">
              <TabsTrigger value="chapter">Chương</TabsTrigger>
              <TabsTrigger value="log">Log agy</TabsTrigger>
            </TabsList>
            <TabsContent value="chapter" className="min-h-0 flex-1">
              {row ? (
                <ChapterPanel root={root} row={row} />
              ) : (
                <p className="p-6 text-sm text-muted-foreground">Chọn một chương bên trái.</p>
              )}
            </TabsContent>
            <TabsContent value="log" className="min-h-0 flex-1">
              <LogPanel />
            </TabsContent>
          </Tabs>
        </main>
      </div>
    </div>
  );
}
