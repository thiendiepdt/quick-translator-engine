import { Pencil, Plus } from "lucide-react";
import { useFieldArray, useFormContext, useWatch } from "react-hook-form";

import { RuleTable, type RuleRow } from "@/components/rule-table";
import { Button } from "@/components/ui/button";
import type { StoryFormValues } from "@/lib/story-form";
import type { StoryDefaults } from "@/lib/types";

function defaultRows(defaults: StoryDefaults | undefined): RuleRow[] {
  return (defaults?.checkRules ?? []).map((rule) => ({ pattern: rule.pattern, flags: rule.flags ?? "", message: rule.message }));
}

/**
 * Trống trong story.json = dùng bộ mặc định của app: hiện bộ đó (chỉ đọc) để người dùng thấy đang kiểm gì;
 * "Sửa bộ mặc định" sao chép ra thành rule riêng để chỉnh; "Về mặc định" xoá bản riêng. Bộ mặc định có thể
 * là bản người dùng đã sửa ở Cài đặt → Bản mặc định (nhãn "đã sửa").
 */
export function CheckRulesEditor({ defaults }: { defaults: StoryDefaults | undefined }) {
  const { control } = useFormContext<StoryFormValues>();
  const { fields, replace } = useFieldArray({ control, name: "checkRules" });
  const values = useWatch({ control, name: "checkRules" }) ?? [];
  const usingDefaults = fields.length === 0;
  const rows = usingDefaults ? defaultRows(defaults) : values;
  const edited = defaults?.rulesSource === "file";
  return (
    <fieldset className="flex flex-col gap-1">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <legend className="text-sm font-medium">
          Rule kiểm tra{" "}
          <span className="rounded-full bg-muted px-1.5 text-xs font-normal text-muted-foreground">
            {usingDefaults ? `mặc định${edited ? " (đã sửa)" : ""} · ${defaults ? rows.length : "…"}` : `riêng · ${fields.length}`}
          </span>
          <span className="ml-2 text-xs text-muted-foreground">(rule CJK còn sót luôn chạy)</span>
        </legend>
        <div className="flex gap-1">
          {usingDefaults ? (
            <>
              <Button type="button" size="xs" variant="secondary" disabled={!defaults} onClick={() => replace(defaultRows(defaults))}>
                <Pencil /> Sửa bộ mặc định
              </Button>
              <Button
                type="button"
                size="xs"
                variant="ghost"
                disabled={!defaults}
                onClick={() => replace([...defaultRows(defaults), { pattern: "", flags: "", message: "" }])}
              >
                <Plus /> Thêm
              </Button>
            </>
          ) : (
            <Button type="button" size="xs" variant="ghost" onClick={() => replace([])}>
              Về mặc định
            </Button>
          )}
        </div>
      </div>
      <RuleTable rows={rows} readOnly={usingDefaults} onChange={(next) => replace(next)} />
    </fieldset>
  );
}
