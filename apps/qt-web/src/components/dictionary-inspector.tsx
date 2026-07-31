import {
  AlertCircle,
  FileUp,
  Info,
  LoaderCircle,
  Maximize2,
  PencilLine,
  RotateCcw,
  Trash2,
} from "lucide-react";
import { useMemo, useRef, useState } from "react";
import { toast } from "sonner";

import { DictionaryEditorDialog } from "@/components/dictionary-editor-dialog";
import { EngineOptions } from "@/components/engine-options";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { getDictionaryDocumentStats } from "@/lib/dictionary-document";
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
  const [editorOpen, setEditorOpen] = useState(false);

  const definition = dictionaryDefinitions.find(({ key }) => key === activeDictionary);
  const activeDraft = dictionaries[activeDictionary];
  const dictionaryStats = useMemo(
    () => getDictionaryDocumentStats(activeDraft.value, activeDictionary),
    [activeDictionary, activeDraft.value],
  );
  if (!definition) return null;
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
    // Trong sheet, chiều cao phải do flex quyết định. Trước đây dùng
    // h-[calc(100dvh-5rem)] cứng nên đổi chiều cao header là panel hụt đáy,
    // dòng ghi chú đè lên hàng cuối và không cuộn tới được.
    <div className={cn("flex h-full min-h-0 flex-col bg-card", mobile && "min-h-0 flex-1")}>
      {/* Đã bỏ eyebrow "REQUEST INSPECTOR" và đoạn giải thích QT2025: tiêu đề
          đã nói đủ, còn ý nghĩa chấm xanh nằm trong tooltip của chính chấm đó. */}
      <div className="shrink-0 border-b px-4 py-3">
        <div className="flex items-center justify-between gap-3">
          <h2 className="text-base font-semibold tracking-tight">Từ điển &amp; engine</h2>
          <Badge
            variant={touchedCount > 0 ? "default" : "secondary"}
            title="Số bộ từ điển đã sửa, lưu cục bộ và sẽ được gửi kèm request"
          >
            {touchedCount}/8
          </Badge>
        </div>
        {defaultsStatus === "loading" ? (
          <p className="mt-1.5 flex items-center gap-1.5 text-xs text-muted-foreground">
            <LoaderCircle className="size-3 animate-spin" /> Đang tải từ điển mặc định…
          </p>
        ) : null}
        {defaultsStatus === "error" ? (
          <div className="mt-2 flex items-center gap-2 rounded-md border border-destructive/25 bg-destructive/5 p-2 text-[11px] text-destructive">
            <AlertCircle className="size-3.5 shrink-0" />
            <span className="min-w-0 flex-1 truncate">{defaultsError ?? "Không tải được từ điển mặc định"}</span>
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
                      defaultsReady && draft.touched && "bg-ok",
                    )}
                    title={draft.touched ? "Đã sửa · lưu cục bộ · sẽ gửi bản này" : "Dùng mặc định từ server"}
                    aria-label={draft.touched ? "Đã sửa, lưu cục bộ và sẽ gửi bản này" : "Dùng mặc định từ server"}
                  />
                </button>
              );
            })}
          </div>

          <div className="mt-4">
            <div className="mb-2">
              <div className="flex items-baseline justify-between gap-3">
                <h3 className="min-w-0 truncate text-sm font-semibold">{definition.label}</h3>
                {/* Tên file + số record gộp một dòng thay vì xuống hai dòng. */}
                <span className="shrink-0 font-mono text-[10px] text-muted-foreground">
                  {dictionaryStats.recordCount.toLocaleString("vi-VN")}
                  {dictionaryStats.rawCount > 0
                    ? ` · ${dictionaryStats.rawCount.toLocaleString("vi-VN")} raw`
                    : ""}
                </span>
              </div>
              <p className="mt-0.5 text-[11px] leading-4 text-muted-foreground">
                <span className="font-mono">{definition.filename}</span> · {definition.description}
              </p>
            </div>
            {/* Nút này trước đây mô tả chính nó bằng 8 chữ ở dòng thứ hai. */}
            <button
              type="button"
              disabled={!defaultsReady}
              title="Tìm kiếm, phân trang, sửa inline, thêm và xóa record"
              className="group flex w-full items-center gap-2.5 rounded-lg border bg-background p-2.5 text-left transition-colors hover:border-primary/35 hover:bg-primary/5 disabled:pointer-events-none disabled:opacity-55"
              onClick={() => setEditorOpen(true)}
            >
              <span className="grid size-8 shrink-0 place-items-center rounded-md bg-primary/10 text-primary">
                <PencilLine className="size-4" />
              </span>
              <span className="min-w-0 flex-1 text-xs font-semibold">Sửa records</span>
              <Maximize2 className="size-4 shrink-0 text-muted-foreground transition-colors group-hover:text-primary" />
            </button>
            <div className="mt-2 flex flex-wrap items-center gap-1">
              <Button type="button" variant="ghost" size="xs" disabled={!defaultsReady} onClick={() => fileInput.current?.click()}>
                <FileUp /> Nạp .txt
              </Button>
              <Button type="button" variant="ghost" size="xs" title="Gửi một bộ rỗng thay cho bản mặc định" disabled={!defaultsReady} onClick={() => setDictionaryValue(activeDictionary, "")}>
                <Trash2 /> Tập rỗng
              </Button>
              <Button type="button" variant="ghost" size="xs" title="Xóa bản sửa cục bộ và quay lại mặc định từ server" disabled={!defaultsReady || !activeDraft.touched} onClick={() => resetDictionary(activeDictionary)}>
                <RotateCcw /> Khôi phục
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

      {/* Đoạn giải thích Lambda dài hai dòng đã rút thành một dòng + tooltip. */}
      <div
        className="flex shrink-0 items-center gap-1.5 border-t bg-muted/35 px-4 py-2 text-[11px] text-muted-foreground"
        title="VietPhrase.txt và ChinesePhienAmWords.txt được nhúng trong Lambda, không thể ghi đè từ web. Chỉ gửi được các bản vá entry lẻ."
      >
        <Info className="size-3.5 shrink-0" />
        VietPhrase &amp; Phiên Âm cố định trong engine
      </div>

      <DictionaryEditorDialog
        open={editorOpen}
        onOpenChange={setEditorOpen}
        definition={definition}
        value={activeDraft.value}
        onSave={(content) => setDictionaryValue(activeDictionary, content)}
      />
    </div>
  );
}
