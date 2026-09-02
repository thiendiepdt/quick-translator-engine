import { FolderOpen, History, Sparkles } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ApiError, initStory, openStory, pickFolder } from "@/lib/api";
import { useStoryStore } from "@/store/story";

export function StoryPicker() {
  const recent = useStoryStore((s) => s.config?.recent ?? []);
  const open = useStoryStore((s) => s.openStory);
  const [pendingInit, setPendingInit] = useState<string | undefined>();
  const [busy, setBusy] = useState(false);

  async function tryOpen(root: string) {
    setBusy(true);
    try {
      open(await openStory(root));
    } catch (error) {
      if (error instanceof ApiError && error.kind === "story_not_found") {
        setPendingInit(root); // chưa có state.json → hỏi khởi tạo
      } else {
        toast.error(error instanceof Error ? error.message : "Không mở được truyện");
      }
    } finally {
      setBusy(false);
    }
  }

  async function confirmInit() {
    if (!pendingInit) return;
    setBusy(true);
    try {
      open(await initStory(pendingInit));
      toast.success("Đã khởi tạo folder truyện");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không khởi tạo được");
    } finally {
      setBusy(false);
      setPendingInit(undefined);
    }
  }

  async function pickAndOpen() {
    const root = await pickFolder("Chọn folder truyện");
    if (root) await tryOpen(root);
  }

  return (
    <main className="mx-auto flex max-w-2xl flex-col gap-6 p-8">
      <header>
        <h1 className="text-2xl font-semibold">QT AI Translator</h1>
        <p className="text-sm text-muted-foreground">
          Chọn folder truyện có thư mục <code>raw/</code> chứa các chương <code>.txt</code>.
        </p>
      </header>
      <Button size="lg" disabled={busy} onClick={() => void pickAndOpen()}>
        <FolderOpen /> Mở folder truyện
      </Button>
      {recent.length > 0 && (
        <section className="flex flex-col gap-2">
          <h2 className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
            <History className="size-4" /> Mở gần đây
          </h2>
          <ul className="divide-y rounded-md border">
            {recent.map((root) => (
              <li key={root}>
                <button
                  type="button"
                  disabled={busy}
                  className="w-full px-3 py-2 text-left text-sm hover:bg-accent disabled:opacity-50"
                  onClick={() => void tryOpen(root)}
                >
                  {root}
                </button>
              </li>
            ))}
          </ul>
        </section>
      )}
      <Dialog
        open={pendingInit !== undefined}
        onOpenChange={(value) => {
          if (!value) setPendingInit(undefined);
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Khởi tạo folder truyện?</DialogTitle>
            <DialogDescription>
              <code>{pendingInit}</code> chưa có <code>state.json</code>. Khởi tạo sẽ tạo <code>story.json</code>,{" "}
              <code>state.json</code>, <code>AGENTS.md</code> và đưa mọi chương trong <code>raw/</code> vào hàng
              đợi. Không đụng file gốc.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setPendingInit(undefined)}>
              Bỏ
            </Button>
            <Button disabled={busy} onClick={() => void confirmInit()}>
              <Sparkles /> Khởi tạo
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </main>
  );
}
