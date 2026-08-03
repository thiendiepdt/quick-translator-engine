import {
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  ClipboardPaste,
  Database,
  FileUp,
  Plus,
  Save,
  Search,
  Trash2,
} from "lucide-react";
import {
  useCallback,
  useDeferredValue,
  useMemo,
  useRef,
  useState,
} from "react";
import { toast } from "sonner";

import { Badge } from "@/components/ui/badge";
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
import { useTextFileImport, type TextImportSource } from "@/hooks/use-text-file-import";
import {
  appendDictionaryText,
  parseDictionaryDocument,
  serializeDictionaryDocument,
  type DictionaryRecord,
} from "@/lib/dictionary-document";
import { paginationItems } from "@/lib/pagination";
import type { DictionaryDefinition } from "@/lib/types";
import { cn } from "@/lib/utils";

const PAGE_SIZE = 100;

interface DictionaryEditorDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  definition: DictionaryDefinition;
  value: string;
  onSave: (value: string) => void;
}

interface RecordRowProps {
  record: DictionaryRecord;
  valueOnly: boolean;
  highlightQuery: string;
  onDraft: (record: DictionaryRecord) => void;
  onCommit: (id: string) => void;
  onDelete: (id: string) => void;
}

interface HighlightedInputProps {
  ariaLabel: string;
  lang?: string;
  value: string;
  highlightQuery: string;
  onChange: (value: string) => void;
  onCommit: () => void;
}

function HighlightedText({
  value,
  query,
}: {
  value: string;
  query: string;
}) {
  const normalizedValue = value.toLocaleLowerCase("vi");
  const normalizedQuery = query.toLocaleLowerCase("vi");
  const parts = [];
  let cursor = 0;

  while (cursor < value.length) {
    const matchIndex = normalizedValue.indexOf(normalizedQuery, cursor);
    if (matchIndex === -1) break;
    if (matchIndex > cursor) {
      parts.push(value.slice(cursor, matchIndex));
    }
    const matchEnd = matchIndex + normalizedQuery.length;
    parts.push(
      <mark
        key={`${matchIndex}-${matchEnd}`}
        className="rounded-[2px] bg-highlight text-highlight-foreground"
      >
        {value.slice(matchIndex, matchEnd)}
      </mark>,
    );
    cursor = matchEnd;
  }

  if (cursor < value.length) parts.push(value.slice(cursor));
  return parts;
}

function HighlightedInput({
  ariaLabel,
  lang,
  value,
  highlightQuery,
  onChange,
  onCommit,
}: HighlightedInputProps) {
  const [focused, setFocused] = useState(false);
  const query = highlightQuery.trim();
  const hasHighlight =
    query.length > 0 &&
    value.toLocaleLowerCase("vi").includes(query.toLocaleLowerCase("vi"));

  return (
    <div className="relative min-w-0">
      {hasHighlight && !focused ? (
        <div
          aria-hidden="true"
          lang={lang}
          className="pointer-events-none absolute inset-0 z-10 flex items-center overflow-hidden px-3 py-1 font-mono text-xs whitespace-pre"
        >
          <span className="truncate">
            <HighlightedText value={value} query={query} />
          </span>
        </div>
      ) : null}
      <Input
        aria-label={ariaLabel}
        lang={lang}
        className={cn(
          "h-8 min-w-0 bg-field font-mono text-xs shadow-none focus-visible:bg-card",
          hasHighlight && !focused && "text-transparent",
        )}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        onFocus={() => setFocused(true)}
        onBlur={() => {
          setFocused(false);
          onCommit();
        }}
      />
    </div>
  );
}

function RecordRow({
  record,
  valueOnly,
  highlightQuery,
  onDraft,
  onCommit,
  onDelete,
}: RecordRowProps) {
  const [key, setKey] = useState(record.key);
  const [value, setValue] = useState(record.value);
  const raw = record.kind === "raw";

  function updateKey(nextKey: string) {
    setKey(nextKey);
    onDraft({ ...record, key: nextKey, value });
  }

  function updateValue(nextValue: string) {
    setValue(nextValue);
    onDraft({ ...record, key, value: nextValue });
  }

  return (
    <div
      className={cn(
        "grid min-w-[720px] items-center gap-2 border-b bg-card px-3 py-1.5 transition-colors hover:bg-accent/45",
        valueOnly || raw
          ? "grid-cols-[58px_minmax(0,1fr)_36px]"
          : "grid-cols-[58px_minmax(220px,0.8fr)_minmax(320px,1.4fr)_36px]",
      )}
    >
      <span className="truncate border-r border-border/70 pr-2 text-center font-mono text-[10px] text-muted-foreground">
        {record.lineNumber ?? "Mới"}
      </span>
      {valueOnly || raw ? (
        <div className="flex min-w-0 items-center gap-2">
          {raw ? <Badge variant="outline" className="shrink-0 text-[9px]">THÔ</Badge> : null}
          <HighlightedInput
            ariaLabel={`${raw ? "Dòng thô" : "Cụm"} ${record.lineNumber ?? "mới"}`}
            value={value}
            highlightQuery={highlightQuery}
            onChange={updateValue}
            onCommit={() => onCommit(record.id)}
          />
        </div>
      ) : (
        <>
          <HighlightedInput
            lang="zh-Hans"
            ariaLabel={`Khóa dòng ${record.lineNumber ?? "mới"}`}
            value={key}
            highlightQuery={highlightQuery}
            onChange={updateKey}
            onCommit={() => onCommit(record.id)}
          />
          <HighlightedInput
            ariaLabel={`Giá trị dòng ${record.lineNumber ?? "mới"}`}
            value={value}
            highlightQuery={highlightQuery}
            onChange={updateValue}
            onCommit={() => onCommit(record.id)}
          />
        </>
      )}
      <Button
        type="button"
        variant="ghost"
        size="icon-xs"
        className="text-muted-foreground hover:text-destructive"
        aria-label={`Xóa dòng ${record.lineNumber ?? "mới"}`}
        onClick={() => onDelete(record.id)}
      >
        <Trash2 />
      </Button>
    </div>
  );
}

function DictionaryEditorSession({
  definition,
  value,
  onSave,
  onOpenChange,
  onDirtyChange,
  onImport,
  onEmpty,
}: Omit<DictionaryEditorDialogProps, "open"> & {
  onDirtyChange: (dirty: boolean) => void;
  onImport: (content: string, source: TextImportSource, previous: string) => void;
  onEmpty: (previous: string) => void;
}) {
  const [document] = useState(() => parseDictionaryDocument(value, definition.key));
  const pendingEditsRef = useRef(new Map<string, DictionaryRecord>());
  const [edits, setEdits] = useState(new Map<string, DictionaryRecord>());
  const [deleted, setDeleted] = useState(new Set<string>());
  const nextAddedId = useRef(1);
  const [added, setAdded] = useState<DictionaryRecord[]>([]);
  const [dirty, setDirty] = useState(false);
  const [query, setQuery] = useState("");
  const deferredQuery = useDeferredValue(query.trim().toLocaleLowerCase("vi"));
  const [page, setPage] = useState(0);
  const [adding, setAdding] = useState(false);
  const [newKey, setNewKey] = useState("");
  const [newValue, setNewValue] = useState("");
  const importInputRef = useRef<HTMLInputElement>(null);

  // Nhập file / kéo thả / dán clipboard THÊM vào cuối nội dung đang mở.
  // Serialize phiên hiện tại trước khi nối nên chỉnh sửa đang dở không mất —
  // toàn bộ được lưu cùng phần vừa nhập, và hoàn tác quay về đúng bản trước
  // khi nhập (vẫn giữ chỉnh sửa).
  const { dropActive, dropHandlers, importFile, importClipboard } = useTextFileImport(
    (text, source) => {
      const base = serializeSession();
      onImport(appendDictionaryText(base, text), source, base);
    },
  );

  const draftRecord = useCallback((record: DictionaryRecord) => {
    pendingEditsRef.current.set(record.id, record);
    setDirty(true);
    onDirtyChange(true);
  }, [onDirtyChange]);

  const commitRecord = useCallback((id: string) => {
    const record = pendingEditsRef.current.get(id);
    if (!record) return;
    setEdits((current) => {
      const next = new Map(current);
      next.set(id, record);
      return next;
    });
  }, []);

  const deleteRecord = useCallback((id: string) => {
    setDeleted((current) => {
      const next = new Set(current);
      next.add(id);
      return next;
    });
    setDirty(true);
    onDirtyChange(true);
  }, [onDirtyChange]);

  const effectiveRecords = useMemo(
    () =>
      [...added, ...document.records]
        .filter((record) => !deleted.has(record.id))
        .map((record) => edits.get(record.id) ?? record),
    [added, deleted, document.records, edits],
  );

  const filteredRecords = useMemo(() => {
    if (!deferredQuery) return effectiveRecords;
    return effectiveRecords.filter((record) =>
      `${record.key}\n${record.value}`.toLocaleLowerCase("vi").includes(deferredQuery),
    );
  }, [deferredQuery, effectiveRecords]);

  const pageCount = Math.max(1, Math.ceil(filteredRecords.length / PAGE_SIZE));
  const safePage = Math.min(page, pageCount - 1);
  const visibleRecords = filteredRecords.slice(
    safePage * PAGE_SIZE,
    (safePage + 1) * PAGE_SIZE,
  );
  const rawCount = effectiveRecords.filter(({ kind }) => kind === "raw").length;
  const recordCount = effectiveRecords.length - rawCount;

  function addRecord() {
    if (document.valueOnly ? !newValue.trim() : !newKey.trim()) {
      toast.error(document.valueOnly ? "Nhập nội dung cụm" : "Khóa không được để trống");
      return;
    }
    const record: DictionaryRecord = {
      id: `added-${nextAddedId.current}`,
      kind: "entry",
      key: document.valueOnly ? "" : newKey,
      value: newValue,
    };
    nextAddedId.current += 1;
    setAdded((records) => [record, ...records]);
    setDirty(true);
    onDirtyChange(true);
    setNewKey("");
    setNewValue("");
    setAdding(false);
    setQuery("");
    setPage(0);
  }

  /** Nội dung hiện tại của phiên, gộp cả các chỉnh sửa chưa lưu. */
  function serializeSession(): string {
    const finalEdits = new Map(edits);
    for (const [id, record] of pendingEditsRef.current) {
      finalEdits.set(id, record);
    }
    return serializeDictionaryDocument(document, {
      edits: finalEdits,
      deleted,
      added,
    });
  }

  function saveChanges() {
    onSave(serializeSession());
    onDirtyChange(false);
    toast.success(`Đã cập nhật ${definition.filename}`);
    onOpenChange(false);
  }

  return (
    <DialogContent className="h-[92dvh]">
      <div className="shrink-0 border-b px-6 py-4 pr-14">
        <DialogHeader>
          <div className="flex flex-wrap items-center gap-2">
            <Database className="size-4 text-primary" />
            <DialogTitle>{definition.label}</DialogTitle>
            <Badge variant="secondary">{definition.filename}</Badge>
            {dirty ? <Badge>Chưa lưu</Badge> : null}
          </div>
          <DialogDescription>
            {definition.description} Chỉ các dòng đang hiển thị được render để giữ editor
            mượt với file lớn.
          </DialogDescription>
        </DialogHeader>
      </div>

      <div className="flex shrink-0 flex-col gap-3 border-b bg-muted/60 px-6 py-3 lg:flex-row lg:items-center">
        <div className="relative min-w-0 flex-1">
          <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={query}
            onChange={(event) => {
              setQuery(event.target.value);
              setPage(0);
            }}
            className="h-10 bg-card pl-9"
            placeholder={document.valueOnly ? "Tìm nội dung cụm…" : "Tìm theo key hoặc value…"}
            autoFocus
          />
        </div>
        <div className="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
          <span>{recordCount.toLocaleString("vi")} bản ghi</span>
          {rawCount > 0 ? <span>· {rawCount.toLocaleString("vi")} dòng thô</span> : null}
          <input
            ref={importInputRef}
            type="file"
            accept=".txt,.md,text/plain"
            hidden
            onChange={(event) => {
              const file = event.target.files?.[0];
              if (file) void importFile(file);
              event.target.value = "";
            }}
          />
          <Button
            type="button"
            size="sm"
            variant="outline"
            title="Thêm nội dung file .txt vào cuối — hoặc kéo thả file vào danh sách"
            onClick={() => importInputRef.current?.click()}
          >
            <FileUp /> Nhập file
          </Button>
          <Button
            type="button"
            size="sm"
            variant="outline"
            title="Thêm văn bản đang có trong clipboard vào cuối"
            onClick={() => void importClipboard()}
          >
            <ClipboardPaste /> Dán
          </Button>
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="text-destructive hover:text-destructive"
            title="Xóa toàn bộ bản ghi — gửi một bộ rỗng thay cho bản mặc định"
            disabled={effectiveRecords.length === 0}
            onClick={() => onEmpty(serializeSession())}
          >
            <Trash2 /> Đặt rỗng
          </Button>
          <Button
            type="button"
            size="sm"
            variant={adding ? "secondary" : "outline"}
            className={cn(
              !adding &&
                "border-primary/45 bg-primary/10 text-primary hover:border-primary/60 hover:bg-primary/18 hover:text-primary",
            )}
            onClick={() => setAdding((current) => !current)}
          >
            <Plus /> Thêm bản ghi
          </Button>
        </div>
      </div>

      {adding ? (
        <div className="shrink-0 border-b bg-primary/5 px-6 py-3">
          <div className={cn("grid gap-2", document.valueOnly ? "grid-cols-[1fr_auto]" : "grid-cols-[minmax(180px,0.8fr)_minmax(240px,1.4fr)_auto]")}>
            {!document.valueOnly ? (
              <Input
                value={newKey}
                onChange={(event) => setNewKey(event.target.value)}
                placeholder="Khóa tiếng Trung"
                className="bg-card font-mono text-xs"
              />
            ) : null}
            <Input
              value={newValue}
              onChange={(event) => setNewValue(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") addRecord();
              }}
              placeholder={document.valueOnly ? "Nội dung cụm" : "Giá trị tiếng Việt"}
              className="bg-card font-mono text-xs"
            />
            <Button type="button" onClick={addRecord}><Plus /> Thêm</Button>
          </div>
        </div>
      ) : null}

      <div className="relative min-h-0 flex-1" {...dropHandlers}>
        {dropActive ? (
          <div className="pointer-events-none absolute inset-0 z-20 grid place-items-center border-2 border-dashed border-primary/60 bg-primary/10 text-sm font-medium text-primary backdrop-blur-[1px]">
            Thả file .txt để thêm vào cuối {definition.filename}
          </div>
        ) : null}
        <div className="h-full overflow-auto bg-card">
        <div
          className={cn(
            "sticky top-0 z-10 grid min-w-[720px] gap-2 border-b bg-secondary px-3 py-2 font-mono text-[10px] font-semibold tracking-wide text-secondary-foreground uppercase",
            document.valueOnly
              ? "grid-cols-[58px_minmax(0,1fr)_36px]"
              : "grid-cols-[58px_minmax(220px,0.8fr)_minmax(320px,1.4fr)_36px]",
          )}
        >
          <span className="text-center">Dòng</span>
          {document.valueOnly ? (
            <span>Nội dung</span>
          ) : (
            <>
              <span>Khóa</span>
              <span>Giá trị</span>
            </>
          )}
          <span />
        </div>

        {visibleRecords.length > 0 ? (
          visibleRecords.map((record) => (
            <RecordRow
              key={record.id}
              record={record}
              valueOnly={document.valueOnly}
              highlightQuery={query}
              onDraft={draftRecord}
              onCommit={commitRecord}
              onDelete={deleteRecord}
            />
          ))
        ) : (
          <div className="grid h-48 place-items-center text-sm text-muted-foreground">
            Không tìm thấy bản ghi phù hợp.
          </div>
        )}
        </div>
      </div>

      <div className="flex shrink-0 flex-col gap-3 border-t bg-card px-6 py-3 xl:flex-row xl:items-center xl:justify-between">
        <div className="text-xs text-muted-foreground">
          {filteredRecords.length.toLocaleString("vi")} kết quả · tối đa {PAGE_SIZE} dòng/trang
        </div>
        {/* Nhảy thẳng theo số trang; màn hẹp chỉ còn icon «‹›» cho gọn. */}
        <nav className="flex flex-wrap items-center gap-1" aria-label="Phân trang bản ghi">
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={safePage === 0}
            aria-label="Trang đầu"
            onClick={() => setPage(0)}
          >
            <ChevronsLeft /> <span className="hidden lg:inline">Đầu</span>
          </Button>
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={safePage === 0}
            aria-label="Trang trước"
            onClick={() => setPage(Math.max(0, safePage - 1))}
          >
            <ChevronLeft /> <span className="hidden lg:inline">Trước</span>
          </Button>
          {paginationItems(safePage, pageCount).map((item, index) =>
            item === "ellipsis" ? (
              <span
                key={`ellipsis-${index}`}
                aria-hidden="true"
                className="px-1 font-mono text-xs text-muted-foreground"
              >
                …
              </span>
            ) : (
              <Button
                key={item}
                type="button"
                variant={item === safePage ? "default" : "ghost"}
                size="sm"
                className="min-w-8 px-2 font-mono"
                aria-label={`Trang ${item + 1}`}
                aria-current={item === safePage ? "page" : undefined}
                onClick={() => setPage(item)}
              >
                {(item + 1).toLocaleString("vi")}
              </Button>
            ),
          )}
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={safePage >= pageCount - 1}
            aria-label="Trang sau"
            onClick={() => setPage(Math.min(pageCount - 1, safePage + 1))}
          >
            <span className="hidden lg:inline">Sau</span> <ChevronRight />
          </Button>
          <Button
            type="button"
            variant="outline"
            size="sm"
            disabled={safePage >= pageCount - 1}
            aria-label="Trang cuối"
            onClick={() => setPage(pageCount - 1)}
          >
            <span className="hidden lg:inline">Cuối</span> <ChevronsRight />
          </Button>
        </nav>
      </div>

      <DialogFooter className="shrink-0 border-t px-6 py-4">
        <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
          Hủy
        </Button>
        <Button type="button" disabled={!dirty} onClick={saveChanges}>
          <Save /> Lưu thay đổi
        </Button>
      </DialogFooter>
    </DialogContent>
  );
}

export function DictionaryEditorDialog({
  open,
  onOpenChange,
  definition,
  value,
  onSave,
}: DictionaryEditorDialogProps) {
  const dirtyRef = useRef(false);
  // Session parse nội dung một lần lúc mount; nhập file/dán thay cả nội dung
  // nên bump key để dựng lại phiên mới trên văn bản vừa nhập.
  const [sessionKey, setSessionKey] = useState(0);

  function handleOpenChange(nextOpen: boolean) {
    if (
      !nextOpen &&
      dirtyRef.current &&
      !window.confirm("Bỏ các thay đổi chưa lưu trong dictionary này?")
    ) {
      return;
    }
    if (!nextOpen) dirtyRef.current = false;
    onOpenChange(nextOpen);
  }

  // `previous` do session đưa lên = nội dung phiên NGAY TRƯỚC khi nối (đã gộp
  // chỉnh sửa đang dở), nên hoàn tác chỉ gỡ phần vừa nhập, không mất chỉnh sửa.
  function handleImport(content: string, source: TextImportSource, previous: string) {
    onSave(content);
    dirtyRef.current = false;
    setSessionKey((key) => key + 1);
    toast.success(
      source.kind === "file"
        ? `Đã thêm ${source.name} vào cuối ${definition.filename}`
        : `Đã thêm nội dung dán vào cuối ${definition.filename}`,
      {
        action: {
          label: "Hoàn tác",
          onClick: () => {
            onSave(previous);
            setSessionKey((key) => key + 1);
          },
        },
      },
    );
  }

  function handleEmpty(previous: string) {
    onSave("");
    dirtyRef.current = false;
    setSessionKey((key) => key + 1);
    toast.success(`Đã đặt rỗng ${definition.filename}`, {
      description: "Request sẽ gửi một bộ rỗng thay cho bản mặc định.",
      action: {
        label: "Hoàn tác",
        onClick: () => {
          onSave(previous);
          setSessionKey((key) => key + 1);
        },
      },
    });
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      {open ? (
        <DictionaryEditorSession
          key={sessionKey}
          definition={definition}
          value={value}
          onSave={onSave}
          onOpenChange={handleOpenChange}
          onDirtyChange={(dirty) => {
            dirtyRef.current = dirty;
          }}
          onImport={handleImport}
          onEmpty={handleEmpty}
        />
      ) : null}
    </Dialog>
  );
}
