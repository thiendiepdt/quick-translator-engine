import { Pencil, Plus, Trash2 } from "lucide-react";
import { useFieldArray, useFormContext } from "react-hook-form";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import type { StoryFormValues } from "@/lib/story-form";
import type { StoryDefaults } from "@/lib/types";

type Rule = StoryFormValues["checkRules"][number];

function defaultRows(defaults: StoryDefaults | undefined): Rule[] {
  return (defaults?.checkRules ?? []).map((rule) => ({ pattern: rule.pattern, flags: rule.flags ?? "", message: rule.message }));
}

/**
 * Trống trong story.json = dùng bộ mặc định của hệ: hiện bộ đó (chỉ đọc) để người dùng thấy đang
 * kiểm gì; "Sửa bộ mặc định" sao chép ra thành rule riêng để chỉnh; "Về mặc định" xoá bản riêng.
 */
export function CheckRulesEditor({ defaults }: { defaults: StoryDefaults | undefined }) {
  const { control, register } = useFormContext<StoryFormValues>();
  const { fields, append, remove, replace } = useFieldArray({ control, name: "checkRules" });
  const usingDefaults = fields.length === 0;
  const rows = usingDefaults ? defaultRows(defaults) : undefined;
  return (
    <fieldset className="flex flex-col gap-1">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <legend className="text-sm font-medium">
          Rule kiểm tra{" "}
          <span className="rounded-full bg-muted px-1.5 text-xs font-normal text-muted-foreground">
            {usingDefaults ? `mặc định · ${rows?.length ?? "…"}` : `riêng · ${fields.length}`}
          </span>
          <span className="ml-2 text-xs text-muted-foreground">(rule CJK còn sót luôn chạy)</span>
        </legend>
        <div className="flex gap-1">
          {usingDefaults ? (
            <Button type="button" size="xs" variant="secondary" disabled={!defaults} onClick={() => replace(defaultRows(defaults))}>
              <Pencil /> Sửa bộ mặc định
            </Button>
          ) : (
            <Button type="button" size="xs" variant="ghost" onClick={() => replace([])}>
              Về mặc định
            </Button>
          )}
          <Button
            type="button"
            size="xs"
            variant="ghost"
            onClick={() => {
              if (usingDefaults) replace([...defaultRows(defaults), { pattern: "", flags: "", message: "" }]);
              else append({ pattern: "", flags: "", message: "" });
            }}
          >
            <Plus /> Thêm
          </Button>
        </div>
      </div>
      {rows?.map((rule, index) => (
        <div key={index} className="grid grid-cols-[2fr_60px_2fr] gap-1 text-xs text-muted-foreground">
          <code className="truncate rounded bg-muted px-2 py-1 font-mono" title={rule.pattern}>
            {rule.pattern}
          </code>
          <code className="rounded bg-muted px-2 py-1 font-mono">{rule.flags}</code>
          <span className="truncate px-2 py-1" title={rule.message}>
            {rule.message}
          </span>
        </div>
      ))}
      {!usingDefaults &&
        fields.map((field, index) => (
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
