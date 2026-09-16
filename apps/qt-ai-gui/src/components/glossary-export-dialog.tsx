import { ChevronDown, ChevronRight, Copy, Save, Search } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";

import { Choice } from "@/components/choice";
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
import { Textarea } from "@/components/ui/textarea";
import { pickSaveFile, writeTextFile } from "@/lib/api";
import { copyText } from "@/lib/clipboard";
import {
  defaultSelection,
  NAMES_EXPORT_KEYS,
  NAMES_FILE_NAMES,
  planNamesExport,
  renderNames,
  type NamesExportKey,
  type NamesExportRow,
  type NamesFileName,
} from "@/lib/names-export";
import type { StoryFormValues } from "@/lib/story-form";
import { GLOSSARY_LABELS } from "@/lib/types";
import { cn } from "@/lib/utils";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Glossary đang hiển thị ở form (kể cả sửa chưa lưu). */
  glossary: StoryFormValues["glossary"];
}

const FILE_LABELS: Record<NamesFileName, string> = {
  "Names.txt": "Names.txt",
  "Names2.txt": "Names2.txt",
};

/** Checkbox tổng của nhóm: checked/indeterminate theo số dòng hợp lệ đã tick. */
function GroupCheckbox({
  label,
  total,
  picked,
  onChange,
}: {
  label: string;
  total: number;
  picked: number;
  onChange: (checked: boolean) => void;
}) {
  const ref = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (ref.current) ref.current.indeterminate = picked > 0 && picked < total;
  }, [picked, total]);
  return (
    <input
      ref={ref}
      type="checkbox"
      aria-label={label}
      className="size-4 accent-primary"
      disabled={total === 0}
      checked={total > 0 && picked === total}
      onChange={(e) => onChange(e.target.checked)}
    />
  );
}

/**
 * Chọn mục glossary rồi chép/lưu ra `Hán=Việt` mỗi dòng cho Names.txt / Names2.txt của bản convert.
 * Thân dialog chỉ mount khi mở nên mỗi lần mở tick lại mặc định theo glossary hiện tại, nhóm gọn hết.
 */
export function GlossaryExportDialog({ open, onOpenChange, glossary }: Props) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      {open && <ExportBody glossary={glossary} />}
    </Dialog>
  );
}

/** Quy tắc bỏ dòng và mặc định tick nằm ở lib/names-export.ts. */
function ExportBody({ glossary }: Pick<Props, "glossary">) {
  const rows = useMemo(() => planNamesExport(glossary), [glossary]);
  const [selected, setSelected] = useState<Set<string>>(() => defaultSelection(rows));
  const [query, setQuery] = useState("");
  const [fileName, setFileName] = useState<NamesFileName>("Names.txt");
  const [expanded, setExpanded] = useState<Set<NamesExportKey>>(new Set());
  const [busy, setBusy] = useState(false);
  const [written, setWritten] = useState<string | undefined>();

  const text = renderNames(rows, selected);
  const picked = rows.filter((r) => !r.skip && selected.has(r.id)).length;
  const skipped = rows.filter((r) => Boolean(r.skip)).length;
  const needle = query.trim().toLowerCase();
  const byGroup = useMemo(() => {
    const map = new Map<NamesExportKey, NamesExportRow[]>();
    for (const key of NAMES_EXPORT_KEYS) map.set(key, []);
    for (const row of rows) map.get(row.group)?.push(row);
    return map;
  }, [rows]);

  function toggleRow(id: string, checked: boolean) {
    setSelected((prev) => {
      const next = new Set(prev);
      if (checked) next.add(id);
      else next.delete(id);
      return next;
    });
  }

  function toggleGroup(key: NamesExportKey, checked: boolean) {
    setSelected((prev) => {
      const next = new Set(prev);
      for (const row of byGroup.get(key) ?? []) {
        if (row.skip) continue;
        if (checked) next.add(row.id);
        else next.delete(row.id);
      }
      return next;
    });
  }

  function toggleExpanded(key: NamesExportKey) {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  }

  async function copy() {
    try {
      await copyText(text);
      toast.success(`Đã chép ${picked} dòng`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không chép được");
    }
  }

  async function saveFile() {
    setBusy(true);
    try {
      const out = await pickSaveFile(fileName, `Lưu ${fileName}`);
      if (!out) return;
      await writeTextFile(out, renderNames(rows, selected, { bom: true }));
      setWritten(out);
      toast.success(`Đã ghi ${picked} dòng`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Ghi file thất bại");
    } finally {
      setBusy(false);
    }
  }

  return (
    <DialogContent className="flex max-h-[85vh] w-[min(96vw,64rem)] max-w-none flex-col">
      <DialogHeader>
        <DialogTitle>Export glossary ra Names.txt</DialogTitle>
        <DialogDescription>
          Mỗi dòng <code className="font-mono">Hán=Việt</code> cho từ điển tên của bản convert. Xưng hô theo cặp không
          export; dòng có dấu <code className="font-mono">=</code>, thiếu một cột hay trùng Hán bị bỏ (QT chỉ giữ dòng
          đầu).
        </DialogDescription>
      </DialogHeader>
      <div className="grid min-h-0 flex-1 grid-cols-2 gap-4">
        <div className="flex min-h-0 flex-col gap-2">
          <div className="relative">
            <Search className="pointer-events-none absolute top-1/2 left-2 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              className="pl-8"
              placeholder="Tìm Hán hoặc Việt…"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
            />
          </div>
          <div className="fine-scrollbar min-h-0 flex-1 overflow-y-auto rounded-md border">
            {NAMES_EXPORT_KEYS.map((key) => {
              const all = byGroup.get(key) ?? [];
              const valid = all.filter((r) => !r.skip);
              const pickedHere = valid.filter((r) => selected.has(r.id)).length;
              const visible = needle
                ? all.filter((r) => r.source.toLowerCase().includes(needle) || r.target.toLowerCase().includes(needle))
                : all;
              // Mặc định gọn hết (nhóm dài hàng trăm dòng); đang tìm thì dòng khớp hiện ở mọi nhóm.
              const isOpen = expanded.has(key) || Boolean(needle);
              return (
                <section key={key} className="border-b last:border-b-0">
                  <div
                    className={cn(
                      "flex items-center gap-2 px-2 py-1.5 text-sm",
                      all.length === 0 && "text-muted-foreground",
                    )}
                  >
                    <GroupCheckbox
                      label={GLOSSARY_LABELS[key]}
                      total={valid.length}
                      picked={pickedHere}
                      onChange={(checked) => toggleGroup(key, checked)}
                    />
                    <button
                      type="button"
                      className="flex flex-1 items-center gap-1 text-left font-medium"
                      aria-expanded={isOpen}
                      onClick={() => toggleExpanded(key)}
                    >
                      {isOpen ? <ChevronDown className="size-4" /> : <ChevronRight className="size-4" />}
                      {GLOSSARY_LABELS[key]}
                    </button>
                    <span className="text-xs text-muted-foreground tabular-nums">
                      {pickedHere}/{valid.length}
                    </span>
                  </div>
                  {isOpen &&
                    visible.map((row) => (
                      <label
                        key={row.id}
                        className={cn(
                          "flex items-center gap-2 px-2 py-1 pl-8 text-sm",
                          row.skip && "text-muted-foreground",
                        )}
                      >
                        <input
                          type="checkbox"
                          aria-label={`${row.source} = ${row.target}`}
                          className="size-4 accent-primary"
                          disabled={Boolean(row.skip)}
                          checked={!row.skip && selected.has(row.id)}
                          onChange={(e) => toggleRow(row.id, e.target.checked)}
                        />
                        <span className={cn("min-w-0 flex-1 truncate", row.skip && "line-through")}>
                          <span>{row.source}</span>
                          <span className="text-muted-foreground"> = </span>
                          <span>{row.target}</span>
                        </span>
                        {row.skip && <span className="shrink-0 text-xs">{row.skip}</span>}
                      </label>
                    ))}
                </section>
              );
            })}
          </div>
        </div>
        <div className="flex min-h-0 flex-col gap-2">
          <p className="text-sm">
            Sẽ ghi <strong className="tabular-nums">{picked}</strong> dòng · bỏ{" "}
            <strong className="tabular-nums">{skipped}</strong> dòng
          </p>
          <Textarea
            aria-label="Xem trước"
            readOnly
            value={text}
            className="min-h-0 flex-1 resize-none font-mono text-xs"
          />
          {written && (
            <p className="text-xs text-muted-foreground">
              Đã ghi <code className="font-mono break-all">{written}</code>
            </p>
          )}
        </div>
      </div>
      <DialogFooter className="items-center sm:justify-between">
        <Choice
          label="Tên file"
          value={fileName}
          options={NAMES_FILE_NAMES}
          labels={FILE_LABELS}
          onChange={setFileName}
        />
        <div className="flex gap-2">
          <Button variant="outline" disabled={picked === 0 || busy} onClick={() => void copy()}>
            <Copy /> Chép
          </Button>
          <Button disabled={picked === 0 || busy} onClick={() => void saveFile()}>
            <Save /> Lưu…
          </Button>
        </div>
      </DialogFooter>
    </DialogContent>
  );
}
