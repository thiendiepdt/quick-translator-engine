import { Plus, Trash2 } from "lucide-react";
import { useFieldArray, useFormContext } from "react-hook-form";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { StoryFormValues } from "@/lib/story-form";

interface Props {
  name: `glossary.${keyof StoryFormValues["glossary"]}` | "signaturePhrases";
  label: string;
}

/** Bảng CN → VN thêm/xoá dòng cho một nhóm glossary (hoặc cụm đặc trưng của style). */
export function GlossaryEditor({ name, label }: Props) {
  const { control, register } = useFormContext<StoryFormValues>();
  const { fields, append, remove } = useFieldArray({ control, name });
  return (
    <fieldset className="flex flex-col gap-1">
      <div className="flex items-center justify-between">
        <legend className="text-sm font-medium">
          {label} <span className="text-xs text-muted-foreground">({fields.length})</span>
        </legend>
        <Button type="button" size="xs" variant="ghost" onClick={() => append({ source: "", target: "" })}>
          <Plus /> Thêm
        </Button>
      </div>
      {fields.map((field, index) => (
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
    </fieldset>
  );
}
