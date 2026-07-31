import { Controller, useFormContext } from "react-hook-form";

import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import type { TranslationOptionsValues } from "@/lib/schema";

const translationAlgorithmOptions = [
  {
    value: "0",
    selectedLabel: "Nghiêm ngặt",
    label: "Cụm dài nhất · nghiêm ngặt",
  },
  {
    value: "1",
    selectedLabel: "Mặc định",
    label: "Khớp đầu tiên tại vị trí · mặc định",
  },
  {
    value: "2",
    selectedLabel: "Linh hoạt",
    label: "Cụm dài nhất · linh hoạt",
  },
] as const;

function OptionSwitch({ name, label }: { name: "pretty" | "wrap" | "prioritizedName"; label: string }) {
  const { control } = useFormContext<TranslationOptionsValues>();
  return (
    <Controller
      name={name}
      control={control}
      render={({ field }) => (
        <div className="flex min-h-10 items-center justify-between border-b border-border/70 py-2">
          <Label htmlFor={name} className="text-xs font-medium">{label}</Label>
          <Switch id={name} checked={field.value} onCheckedChange={field.onChange} />
        </div>
      )}
    />
  );
}

export function EngineOptions() {
  const {
    control,
    register,
    formState: { errors },
  } = useFormContext<TranslationOptionsValues>();

  return (
    <section aria-labelledby="engine-options-title">
      <h3 id="engine-options-title" className="mb-1 text-sm font-semibold">
        Bộ máy dịch
      </h3>
      <OptionSwitch name="pretty" label="Đầu ra dễ đọc" />
      <OptionSwitch name="wrap" label="Ngắt dòng theo cụm từ" />
      <OptionSwitch name="prioritizedName" label="Ưu tiên tên riêng" />
      {/* Dòng giải thích đã chuyển vào tooltip: công tắc này khoá cứng, người
          dùng không có quyết định nào để đưa ra ở đây. */}
      <div
        className="flex min-h-10 items-center justify-between border-b border-border/70 py-2"
        title="Luôn bật để đối chiếu hai chiều giữa nguyên văn và bản dịch"
      >
        <div className="text-xs font-medium">Ghép phạm vi</div>
        <Switch checked disabled aria-label="Ghép phạm vi luôn bật" />
      </div>
      <div className="grid min-h-12 grid-cols-[1fr_92px] items-center gap-3 border-b border-border/70 py-2">
        <Label htmlFor="scanRange" className="text-xs font-medium">Phạm vi quét</Label>
        <Input
          id="scanRange"
          type="number"
          min={1}
          max={100}
          aria-invalid={Boolean(errors.scanRange)}
          className="h-8 font-mono text-xs"
          {...register("scanRange", { valueAsNumber: true })}
        />
        {errors.scanRange ? <p className="col-span-2 text-[10px] text-destructive">Giá trị từ 1 đến 100.</p> : null}
      </div>
      <div className="grid min-h-12 grid-cols-[72px_minmax(0,1fr)] items-center gap-3 py-2">
        <Label htmlFor="translationAlgorithm" className="text-xs font-medium">Thuật toán</Label>
        <Controller
          name="translationAlgorithm"
          control={control}
          render={({ field }) => (
            <Select value={String(field.value)} onValueChange={(value) => field.onChange(Number(value))}>
              <SelectTrigger id="translationAlgorithm" className="h-8 w-full text-xs">
                <SelectValue>
                  {translationAlgorithmOptions.find(({ value }) => value === String(field.value))?.selectedLabel}
                </SelectValue>
              </SelectTrigger>
              <SelectContent>
                {translationAlgorithmOptions.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          )}
        />
      </div>
    </section>
  );
}
