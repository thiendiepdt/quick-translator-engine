import { Plus, Search, Trash2 } from "lucide-react";
import { useState } from "react";
import { useFieldArray, useFormContext, useWatch } from "react-hook-form";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { StoryFormValues } from "@/lib/story-form";

interface Props {
  name: `glossary.${keyof StoryFormValues["glossary"]}` | "signaturePhrases";
  label: string;
}

/** Bảng CN → VN thêm/xoá dòng, có ô tìm nhanh lọc theo cả hai cột (index giữ nguyên để register đúng dòng). */
export function GlossaryEditor({ name, label }: Props) {
  const { control, register } = useFormContext<StoryFormValues>();
  const { fields, append, remove } = useFieldArray({ control, name });
  const values = useWatch({ control, name }) ?? [];
  const [query, setQuery] = useState("");
  const needle = query.trim().toLowerCase();
  const visible = fields
    .map((field, index) => ({ field, index }))
    .filter(({ index }) => {
      if (!needle) return true;
      const pair = values[index];
      return Boolean(
        pair && (pair.source.toLowerCase().includes(needle) || pair.target.toLowerCase().includes(needle)),
      );
    });
  return (
    <fieldset className="rounded-lg border bg-card p-4">
      <div className="mb-3 flex flex-wrap items-center gap-2">
        <legend className="text-sm font-semibold">
          {label}{" "}
          <span className="ml-1 rounded-full bg-muted px-1.5 text-xs font-normal text-muted-foreground">
            {fields.length}
          </span>
        </legend>
        <div className="flex-1" />
        {fields.length > 8 && (
          <div className="relative">
            <Search className="pointer-events-none absolute top-1/2 left-2 size-3.5 -translate-y-1/2 text-muted-foreground" />
            <Input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Tìm…"
              aria-label={`Tìm trong ${label}`}
              className="h-7 w-40 pl-7 text-xs"
            />
          </div>
        )}
        <Button
          type="button"
          size="xs"
          variant="secondary"
          onClick={() => {
            setQuery("");
            append({ source: "", target: "" });
          }}
        >
          <Plus /> Thêm
        </Button>
      </div>
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
      </div>
    </fieldset>
  );
}
