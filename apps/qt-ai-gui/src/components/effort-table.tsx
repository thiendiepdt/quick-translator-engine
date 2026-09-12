import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { API_STEPS, type ApiStep } from "@/lib/schema";
import { API_STEP_LABELS } from "@/lib/types";

/** Radix Select không nhận value rỗng — "" (không gửi tham số) đi qua sentinel này trong UI. */
const DEFAULT_SENTINEL = "__default";

interface Props {
  /** Tiền tố id/aria để hai bảng (Gemini, OpenAI) không trùng. */
  idPrefix: string;
  values: Record<ApiStep, string>;
  /** Danh sách mức hợp lệ của provider; "" = mặc định model. */
  options: readonly string[];
  hint: string;
  disabled?: boolean;
  onChange: (step: ApiStep, value: string) => void;
}

/** Bảng "Mức nghĩ theo bước": mỗi bước gọi model một Select. */
export function EffortTable({ idPrefix, values, options, hint, disabled, onChange }: Props) {
  return (
    <div className="flex flex-col gap-2 rounded-md bg-background/60 px-3 py-2">
      <p className="text-xs font-medium">Mức nghĩ theo bước</p>
      {API_STEPS.map((step) => {
        const id = `${idPrefix}-effort-${step}`;
        return (
          <div key={step} className="flex items-center justify-between gap-2">
            <Label htmlFor={id} className="text-xs font-normal">
              {API_STEP_LABELS[step]}
            </Label>
            <Select
              value={values[step] === "" ? DEFAULT_SENTINEL : values[step]}
              onValueChange={(v) => onChange(step, v === DEFAULT_SENTINEL ? "" : v)}
              disabled={disabled}
            >
              <SelectTrigger id={id} size="sm" className="w-40" aria-label={`Mức nghĩ ${API_STEP_LABELS[step]}`}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {options.map((option) => (
                  <SelectItem key={option || DEFAULT_SENTINEL} value={option || DEFAULT_SENTINEL}>
                    {option || "Mặc định model"}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        );
      })}
      <p className="text-xs text-muted-foreground">{hint}</p>
    </div>
  );
}
