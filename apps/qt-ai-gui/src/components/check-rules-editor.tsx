import { Plus, Trash2 } from "lucide-react";
import { useFieldArray, useFormContext } from "react-hook-form";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { StoryFormValues } from "@/lib/story-form";

export function CheckRulesEditor() {
  const { control, register } = useFormContext<StoryFormValues>();
  const { fields, append, remove, replace } = useFieldArray({ control, name: "checkRules" });
  return (
    <fieldset className="flex flex-col gap-1">
      <div className="flex items-center justify-between">
        <legend className="text-sm font-medium">
          Rule kiểm tra riêng{" "}
          <span className="text-xs text-muted-foreground">(trống = dùng bộ mặc định; rule CJK luôn chạy)</span>
        </legend>
        <div className="flex gap-1">
          <Button type="button" size="xs" variant="ghost" onClick={() => replace([])}>
            Về mặc định
          </Button>
          <Button
            type="button"
            size="xs"
            variant="ghost"
            onClick={() => append({ pattern: "", flags: "", message: "" })}
          >
            <Plus /> Thêm
          </Button>
        </div>
      </div>
      {fields.map((field, index) => (
        <div key={field.id} className="grid grid-cols-[2fr_60px_2fr_auto] gap-1">
          <Input
            {...register(`checkRules.${index}.pattern`)}
            placeholder="regex (cú pháp JS)"
            aria-label={`Regex ${index + 1}`}
            className="h-8 font-mono"
          />
          <Input
            {...register(`checkRules.${index}.flags`)}
            placeholder="i"
            aria-label={`Flags ${index + 1}`}
            className="h-8 font-mono"
          />
          <Input
            {...register(`checkRules.${index}.message`)}
            placeholder="Mô tả / cách sửa"
            aria-label={`Mô tả rule ${index + 1}`}
            className="h-8"
          />
          <Button type="button" size="icon-sm" variant="ghost" aria-label="Xoá rule" onClick={() => remove(index)}>
            <Trash2 />
          </Button>
        </div>
      ))}
    </fieldset>
  );
}
