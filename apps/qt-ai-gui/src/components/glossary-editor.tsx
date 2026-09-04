import { ChevronDown, ChevronRight, FileText, Plus, Search, Trash2 } from "lucide-react";
import { useState } from "react";
import { useFieldArray, useFormContext, useWatch } from "react-hook-form";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { pairsToText, textToPairs, type StoryFormValues } from "@/lib/story-form";

interface Props {
  name: `glossary.${keyof StoryFormValues["glossary"]}` | "signaturePhrases";
  label: string;
}

/** Nhóm nhiều hơn chừng này mục thì thu gọn sẵn để trang không dài vô tận. */
const COLLAPSE_THRESHOLD = 20;
/** Bảng hiện từng khúc chừng này dòng — hàng trăm ô input cùng lúc là nguyên nhân lag. */
const PAGE = 50;

/**
 * Bảng CN → VN thêm/xoá dòng, tìm nhanh theo cả hai cột, hiện theo khúc 50 dòng. Nhóm dài có
 * chế độ "Sửa dạng văn bản" (mỗi dòng `Hán=Việt`) để sửa hàng loạt rồi Áp dụng một lần.
 */
export function GlossaryEditor({ name, label }: Props) {
  const { control, register } = useFormContext<StoryFormValues>();
  const { fields, append, remove, replace } = useFieldArray({ control, name });
  const values = useWatch({ control, name }) ?? [];
  const [query, setQuery] = useState("");
  const [open, setOpen] = useState(fields.length <= COLLAPSE_THRESHOLD);
  const [limit, setLimit] = useState(PAGE);
  const [text, setText] = useState<string | undefined>();
  const needle = query.trim().toLowerCase();
  const matching = fields
    .map((field, index) => ({ field, index }))
    .filter(({ index }) => {
      if (!needle) return true;
      const pair = values[index];
      return Boolean(
        pair && (pair.source.toLowerCase().includes(needle) || pair.target.toLowerCase().includes(needle)),
      );
    });
  const visible = matching.slice(0, limit);
  const editingText = text !== undefined;

  return (
    <fieldset className="rounded-lg border bg-card p-4">
      <div className="mb-3 flex flex-wrap items-center gap-2">
        <button
          type="button"
          className="flex items-center gap-1 text-sm font-semibold"
          aria-expanded={open}
          onClick={() => setOpen((o) => !o)}
        >
          {open ? <ChevronDown className="size-4" /> : <ChevronRight className="size-4" />}
          <legend className="contents">{label}</legend>
          <span className="ml-1 rounded-full bg-muted px-1.5 text-xs font-normal text-muted-foreground">
            {fields.length}
          </span>
        </button>
        <div className="flex-1" />
        {open && !editingText && fields.length > 8 && (
          <div className="relative">
            <Search className="pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-muted-foreground" />
            <Input
              value={query}
              onChange={(e) => {
                setQuery(e.target.value);
                setLimit(PAGE);
              }}
              placeholder="Tìm…"
              aria-label={`Tìm trong ${label}`}
              className="h-7 w-40 pl-7 text-xs"
            />
          </div>
        )}
        {open && !editingText && (
          <Button
            type="button"
            size="xs"
            variant="ghost"
            title="Sửa hàng loạt: mỗi dòng Hán=Việt"
            onClick={() => setText(pairsToText(values.length ? values : fields))}
          >
            <FileText /> Sửa dạng văn bản
          </Button>
        )}
        {open && !editingText && (
          <Button
            type="button"
            size="xs"
            variant="secondary"
            onClick={() => {
              setQuery("");
              setLimit(Math.max(PAGE, fields.length + 1));
              append({ source: "", target: "" });
            }}
          >
            <Plus /> Thêm
          </Button>
        )}
      </div>
      {open && editingText && (
        <div className="flex flex-col gap-2">
          <Textarea
            value={text}
            onChange={(e) => setText(e.target.value)}
            aria-label={`${label} dạng văn bản`}
            rows={Math.min(24, Math.max(6, fields.length + 2))}
            className="font-mono text-xs"
            placeholder="赵静文=Triệu Tĩnh Văn"
            spellCheck={false}
          />
          <div className="flex items-center gap-2">
            <p className="flex-1 text-xs text-muted-foreground">Mỗi dòng một mục dạng Hán=Việt; dòng trống bị bỏ.</p>
            <Button type="button" size="xs" variant="ghost" onClick={() => setText(undefined)}>
              Huỷ
            </Button>
            <Button
              type="button"
              size="xs"
              onClick={() => {
                replace(textToPairs(text));
                setText(undefined);
                setQuery("");
                setLimit(PAGE);
              }}
            >
              Áp dụng
            </Button>
          </div>
        </div>
      )}
      {open && !editingText && (
        <div className="flex flex-col gap-1">
          {visible.length === 0 && (
            <p className="py-2 text-xs text-muted-foreground">{fields.length === 0 ? "Chưa có mục nào." : "Không khớp."}</p>
          )}
          {visible.map(({ field, index }) => (
            <div key={field.id} className="grid grid-cols-[1fr_1fr_auto] gap-1">
              <Input
                {...register(`${name}.${index}.source`)}
                placeholder="Hán tự"
                aria-label={`${label} CN ${index + 1}`}
                className="h-8 font-mono"
              />
              <Input
                {...register(`${name}.${index}.target`)}
                placeholder="Tiếng Việt"
                aria-label={`${label} VN ${index + 1}`}
                className="h-8"
              />
              <Button type="button" size="icon-sm" variant="ghost" aria-label="Xoá dòng" onClick={() => remove(index)}>
                <Trash2 />
              </Button>
            </div>
          ))}
          {matching.length > visible.length && (
            <Button type="button" size="xs" variant="ghost" className="self-start" onClick={() => setLimit((l) => l + PAGE)}>
              Hiện thêm {Math.min(PAGE, matching.length - visible.length)} / còn {matching.length - visible.length}
            </Button>
          )}
        </div>
      )}
    </fieldset>
  );
}
