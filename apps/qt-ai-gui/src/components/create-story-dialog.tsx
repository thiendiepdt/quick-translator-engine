import { FolderPlus, LoaderCircle } from "lucide-react";
import { useEffect, useState } from "react";
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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { createStory, slugifyName } from "@/lib/api";
import type { StorySnapshot } from "@/lib/types";

interface Props {
  libraryRoot: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated: (snapshot: StorySnapshot) => void;
}

const SLUG_PATTERN = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;

/** Tạo `<thư viện>/<slug>/raw/` + init rồi mở. Slug tự sinh từ tên cho tới khi người dùng sửa tay. */
export function CreateStoryDialog({ libraryRoot, open, onOpenChange, onCreated }: Props) {
  const [name, setName] = useState("");
  const [slug, setSlug] = useState("");
  const [slugTouched, setSlugTouched] = useState(false);
  const [url, setUrl] = useState("");
  const [busy, setBusy] = useState(false);

  function reset() {
    setName("");
    setSlug("");
    setSlugTouched(false);
    setUrl("");
  }

  function close() {
    reset();
    onOpenChange(false);
  }

  useEffect(() => {
    if (slugTouched) return;
    let cancelled = false;
    (name.trim() ? slugifyName(name) : Promise.resolve(""))
      .then((value) => {
        if (!cancelled) setSlug(value);
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, [name, slugTouched]);

  const slugValid = SLUG_PATTERN.test(slug);
  const canSubmit = name.trim().length > 0 && slugValid && !busy;
  const separator = libraryRoot.includes("\\") ? "\\" : "/";
  const target = `${libraryRoot.replace(/[\\/]+$/, "")}${separator}${slug || "…"}`;

  async function submit() {
    if (!canSubmit) return;
    setBusy(true);
    try {
      const snapshot = await createStory(name.trim(), slug, url.trim());
      toast.success(`Đã tạo ${snapshot.root} — thả file .txt vào để nạp chương`);
      reset();
      onCreated(snapshot);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không tạo được truyện");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(value) => {
        if (busy) return;
        if (value) onOpenChange(true);
        else close();
      }}
    >
      <DialogContent>
        <form
          className="contents"
          onSubmit={(e) => {
            e.preventDefault();
            void submit();
          }}
        >
          <DialogHeader>
            <DialogTitle>Tạo truyện mới</DialogTitle>
            <DialogDescription>
              Tạo folder truyện trong thư viện và khởi tạo. Chương thêm sau bằng cách thả file <code>.txt</code>{" "}
              vào app hoặc copy vào <code className="font-mono">raw/</code>.
            </DialogDescription>
          </DialogHeader>
          <div className="flex flex-col gap-3">
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="create-name">Tên truyện tiếng Việt</Label>
              <Input
                id="create-name"
                value={name}
                autoFocus
                onChange={(e) => setName(e.target.value)}
                placeholder="Ta Tuyệt Thế Chị Dâu"
                disabled={busy}
              />
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="create-slug">Tên folder</Label>
              <Input
                id="create-slug"
                value={slug}
                onChange={(e) => {
                  setSlugTouched(true);
                  setSlug(e.target.value);
                }}
                className="font-mono text-xs"
                aria-invalid={slug.length > 0 && !slugValid}
                disabled={busy}
              />
              <p className="truncate font-mono text-xs text-muted-foreground" title={target}>
                {target}
              </p>
              {slug.length > 0 && !slugValid && (
                <p className="text-xs text-destructive">Chỉ dùng a-z, 0-9 và dấu gạch nối.</p>
              )}
            </div>
            <div className="flex flex-col gap-1.5">
              <Label htmlFor="create-url">Link truyện tiếng Trung (tuỳ chọn)</Label>
              <Input
                id="create-url"
                value={url}
                onChange={(e) => setUrl(e.target.value)}
                placeholder="https://www.qidian.com/book/…"
                disabled={busy}
              />
            </div>
          </div>
          <DialogFooter>
            <Button type="button" variant="outline" disabled={busy} onClick={close}>
              Bỏ
            </Button>
            <Button type="submit" disabled={!canSubmit}>
              {busy ? <LoaderCircle className="animate-spin" /> : <FolderPlus />} Tạo và mở
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
