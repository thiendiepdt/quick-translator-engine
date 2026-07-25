import { Save, Trash2 } from "lucide-react";
import { FormEvent, useMemo, useState } from "react";
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
import type {
  DictionaryUpdateKey,
  LocalDictionaryEntries,
} from "@/lib/types";

export interface DictionaryUpdateSelection {
  source: string;
  target: string;
}

interface DictionaryUpdateDialogProps {
  open: boolean;
  dictionaryKey?: DictionaryUpdateKey;
  selection?: DictionaryUpdateSelection;
  localEntries: LocalDictionaryEntries;
  onOpenChange: (open: boolean) => void;
  onSave: (
    key: DictionaryUpdateKey,
    entries: Record<string, string>,
    previousKeys: string[],
  ) => void;
  onRemove: (key: DictionaryUpdateKey, previousKeys: string[]) => void;
}

const labels: Record<DictionaryUpdateKey, string> = {
  vietPhrase: "VietPhrase",
  names: "Name (chính)",
  names2: "Name (phụ)",
  chinesePhienAmWords: "Phiên Âm",
  danhTu: "Danh Từ",
  hauTu: "Hậu Từ",
  hoNguoi: "Họ Người",
  luatNhan: "Luật Nhân",
};

function selectedKeys(
  dictionaryKey: DictionaryUpdateKey,
  source: string,
): string[] {
  if (dictionaryKey === "chinesePhienAmWords") {
    return Array.from(source.replace(/\s+/g, ""));
  }
  return [source.trim()];
}

function initialValue(
  dictionaryKey: DictionaryUpdateKey,
  selection: DictionaryUpdateSelection,
  localEntries: LocalDictionaryEntries,
): string {
  const keys = selectedKeys(dictionaryKey, selection.source);
  const saved = keys.map((key) => localEntries[dictionaryKey][key]);
  if (saved.length > 0 && saved.every((value) => value !== undefined)) {
    return saved.join(" ");
  }
  return selection.target.trim();
}

export function DictionaryUpdateDialog({
  open,
  dictionaryKey,
  selection,
  localEntries,
  onOpenChange,
  onSave,
  onRemove,
}: DictionaryUpdateDialogProps) {
  const [source, setSource] = useState(() => selection?.source.trim() ?? "");
  const [target, setTarget] = useState(() =>
    dictionaryKey && selection
      ? initialValue(dictionaryKey, selection, localEntries)
      : "",
  );

  const previousKeys = useMemo(
    () =>
      dictionaryKey && selection
        ? selectedKeys(dictionaryKey, selection.source)
        : [],
    [dictionaryKey, selection],
  );
  const hasSavedEntry = Boolean(
    dictionaryKey &&
      previousKeys.some((key) => localEntries[dictionaryKey][key] !== undefined),
  );

  if (!dictionaryKey || !selection) return null;
  const activeDictionaryKey = dictionaryKey;

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const normalizedSource = source.trim();
    const normalizedTarget = target.trim();
    if (!normalizedSource || !normalizedTarget) {
      toast.error("Key tiếng Trung và nghĩa tiếng Việt không được để trống");
      return;
    }

    if (activeDictionaryKey === "chinesePhienAmWords") {
      const characters = Array.from(normalizedSource.replace(/\s+/g, ""));
      const readings = normalizedTarget.split(/\s+/).filter(Boolean);
      if (characters.length !== readings.length) {
        toast.error("Phiên Âm cần đúng một âm đọc cho mỗi chữ Hán");
        return;
      }
      onSave(
        activeDictionaryKey,
        Object.fromEntries(
          characters.map((character, index) => [character, readings[index]]),
        ),
        previousKeys,
      );
    } else {
      if (
        activeDictionaryKey === "luatNhan" &&
        !normalizedSource.includes("{n}") &&
        !normalizedSource.includes("{s}")
      ) {
        toast.error("Luật Nhân cần chứa placeholder {n} hoặc {s}");
        return;
      }
      onSave(
        activeDictionaryKey,
        { [normalizedSource]: normalizedTarget },
        previousKeys,
      );
    }
    toast.success(`Đã lưu ${labels[activeDictionaryKey]} vào local`);
    onOpenChange(false);
  }

  function remove() {
    onRemove(activeDictionaryKey, previousKeys);
    toast.success(`Đã xóa bản cập nhật ${labels[activeDictionaryKey]} khỏi local`);
    onOpenChange(false);
  }

  const fixed =
    dictionaryKey === "vietPhrase" ||
    dictionaryKey === "chinesePhienAmWords";

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="w-[min(94vw,560px)]">
        <DialogHeader className="border-b px-6 py-5">
          <DialogTitle>Update {labels[dictionaryKey]}</DialogTitle>
          <DialogDescription>
            {fixed
              ? "Entry được lưu cục bộ và phủ lên base ở mỗi lần chạy engine. File từ điển gốc không bị thay đổi."
              : "Entry được lưu cục bộ, ghép vào dictionary draft và dùng ở lần dịch tiếp theo."}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={submit}>
          <div className="grid gap-5 px-6 py-5">
            <div className="grid gap-2">
              <Label htmlFor="dictionary-update-source">
                {dictionaryKey === "luatNhan" ? "Rule" : "Tiếng Trung"}
              </Label>
              <Input
                id="dictionary-update-source"
                value={source}
                onChange={(event) => setSource(event.target.value)}
                autoFocus
              />
              {dictionaryKey === "luatNhan" ? (
                <p className="text-xs text-muted-foreground">
                  Ví dụ: <code>在{"{n}"}身后</code> hoặc{" "}
                  <code>百分之{"{s}"}</code>.
                </p>
              ) : null}
            </div>

            <div className="grid gap-2">
              <Label htmlFor="dictionary-update-target">
                {dictionaryKey === "chinesePhienAmWords"
                  ? "Âm đọc, cách nhau bằng dấu cách"
                  : "Tiếng Việt"}
              </Label>
              <Input
                id="dictionary-update-target"
                value={target}
                onChange={(event) => setTarget(event.target.value)}
              />
            </div>
          </div>

          <DialogFooter className="border-t px-6 py-4">
            {hasSavedEntry ? (
              <Button
                type="button"
                variant="ghost"
                className="mr-auto text-destructive hover:text-destructive"
                onClick={remove}
              >
                <Trash2 />
                Xóa patch
              </Button>
            ) : null}
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              Hủy
            </Button>
            <Button type="submit">
              <Save />
              Lưu local
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
