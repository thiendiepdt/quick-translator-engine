import { AlertCircle, FileUp, LoaderCircle, RotateCcw, Trash2 } from "lucide-react";
import { useRef } from "react";
import { toast } from "sonner";

import { EngineOptions } from "@/components/engine-options";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { Textarea } from "@/components/ui/textarea";
import { dictionaryDefinitions } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useWorkspaceStore } from "@/store/workspace";

const MAX_DICTIONARY_FILE_BYTES = 4 * 1024 * 1024;

interface DictionaryInspectorProps {
  mobile?: boolean;
  defaultsStatus: "loading" | "ready" | "error";
  defaultsError?: string;
  onRetry: () => void;
}

export function DictionaryInspector({
  mobile = false,
  defaultsStatus,
  defaultsError,
  onRetry,
}: DictionaryInspectorProps) {
  const activeDictionary = useWorkspaceStore((state) => state.activeDictionary);
  const dictionaries = useWorkspaceStore((state) => state.dictionaries);
  const setActiveDictionary = useWorkspaceStore((state) => state.setActiveDictionary);
  const setDictionaryValue = useWorkspaceStore((state) => state.setDictionaryValue);
  const resetDictionary = useWorkspaceStore((state) => state.resetDictionary);
  const fileInput = useRef<HTMLInputElement>(null);

  const definition = dictionaryDefinitions.find(({ key }) => key === activeDictionary);
  if (!definition) return null;
  const activeDraft = dictionaries[activeDictionary];
  const touchedCount = Object.values(dictionaries).filter(({ touched }) => touched).length;
  const defaultsReady = defaultsStatus === "ready";

  async function importFile(file: File | undefined) {
    if (!file) return;
    if (file.size > MAX_DICTIONARY_FILE_BYTES) {
      toast.error("File từ điển vượt quá 4 MiB");
      return;
    }
    const content = await file.text();
    setDictionaryValue(activeDictionary, content);
    toast.success(`Đã nạp ${file.name}`);
  }

  return (
    <div className={cn("flex h-full min-h-0 flex-col bg-card", mobile && "h-[calc(100dvh-5rem)]")}>
      <div className="shrink-0 border-b px-5 py-4">
        <div className="flex items-center justify-between gap-3">
          <div>
            <p className="font-mono text-[10px] font-semibold tracking-[0.14em] text-primary uppercase">Request inspector</p>
            <h2 className="mt-1 text-lg font-semibold tracking-tight">Từ điển & engine</h2>
          </div>
          <Badge variant={touchedCount > 0 ? "default" : "secondary"}>{touchedCount}/8 custom</Badge>
        </div>
        <p className="mt-2 text-xs leading-5 text-muted-foreground">
          {defaultsStatus === "ready"
            ? "Đã tải bản mặc định QT2025. Chấm xanh là file đã sửa và sẽ được gửi."
            : defaultsStatus === "error"
              ? "Không tải được từ điển mặc định từ engine."
              : "Đang tải các file mặc định từ engine…"}
        </p>
        {defaultsStatus === "error" ? (
          <div className="mt-3 flex items-center gap-2 rounded-md border border-destructive/25 bg-destructive/5 p-2 text-[10px] text-destructive">
            <AlertCircle className="size-3.5 shrink-0" />
            <span className="min-w-0 flex-1 truncate">{defaultsError ?? "Request thất bại"}</span>
            <Button type="button" variant="outline" size="xs" onClick={onRetry}>Thử lại</Button>
          </div>
        ) : null}
      </div>

      <ScrollArea className="min-h-0 flex-1">
        <div className="p-4">
          <div className="grid grid-cols-2 gap-1.5" aria-label="Danh sách từ điển tùy chỉnh">
            {dictionaryDefinitions.map((item) => {
              const draft = dictionaries[item.key];
              return (
                <button
                  key={item.key}
                  type="button"
                  disabled={!defaultsReady}
                  className={cn(
                    "flex min-h-9 items-center justify-between rounded-md border px-2.5 text-left text-[11px] font-semibold transition-colors",
                    item.key === activeDictionary
                      ? "border-primary/45 bg-primary/8 text-primary"
                      : "bg-background hover:bg-accent",
                  )}
                  onClick={() => setActiveDictionary(item.key)}
                >
                  <span className="truncate">{item.shortLabel}</span>
                  <span
                    className={cn(
                      "ml-2 size-1.5 shrink-0 rounded-full bg-border",
                      defaultsReady && draft.touched && "bg-emerald-600",
                    )}
                    aria-label={draft.touched ? "Sẽ gửi bản đã sửa" : "Dùng mặc định QT2025"}
                  />
                </button>
              );
            })}
          </div>

          <div className="mt-5">
            <div className="mb-2 flex items-end justify-between gap-3">
              <div>
                <h3 className="text-xs font-semibold">{definition.label}</h3>
                <p className="mt-0.5 text-[10px] leading-4 text-muted-foreground">{definition.description}</p>
              </div>
              <span className="shrink-0 text-right font-mono text-[9px] text-muted-foreground">
                {definition.filename}<br />{activeDraft.value.length.toLocaleString("vi-VN")} ký tự
              </span>
            </div>
            <Textarea
              aria-label={`Nội dung ${definition.label}`}
              value={activeDraft.value}
              onChange={(event) => setDictionaryValue(activeDictionary, event.target.value)}
              disabled={!defaultsReady}
              placeholder={defaultsStatus === "loading" ? "Đang tải bản mặc định QT2025…" : "Mỗi dòng theo định dạng của file QT tương ứng…"}
              className="h-32 resize-none bg-background font-mono text-[11px] leading-5"
            />
            <div className="mt-2 flex flex-wrap items-center gap-1">
              <Button type="button" variant="ghost" size="xs" disabled={!defaultsReady} onClick={() => fileInput.current?.click()}>
                <FileUp /> Nạp .txt
              </Button>
              <Button type="button" variant="ghost" size="xs" disabled={!defaultsReady} onClick={() => setDictionaryValue(activeDictionary, "")}>
                <Trash2 /> Dùng tập rỗng
              </Button>
              <Button type="button" variant="ghost" size="xs" disabled={!defaultsReady || !activeDraft.touched} onClick={() => resetDictionary(activeDictionary)}>
                <RotateCcw /> Khôi phục QT2025
              </Button>
              <input
                ref={fileInput}
                type="file"
                accept=".txt,text/plain"
                hidden
                onChange={(event) => {
                  void importFile(event.target.files?.[0]);
                  event.target.value = "";
                }}
              />
            </div>
          </div>

          <Separator className="my-5" />
          <EngineOptions />
        </div>
      </ScrollArea>

      <div className="shrink-0 border-t bg-muted/35 px-5 py-3 text-[10px] leading-4 text-muted-foreground">
        {defaultsStatus === "loading" ? <LoaderCircle className="mr-1 inline size-3 animate-spin" /> : null}
        <strong className="text-foreground/75">Cố định:</strong> VietPhrase.txt và ChinesePhienAmWords.txt được nhúng trong Lambda, không thể ghi đè từ web.
      </div>
    </div>
  );
}
