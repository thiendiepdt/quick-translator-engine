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
      <h3 id="engine-options-title" className="mb-2 font-mono text-[10px] font-semibold tracking-[0.16em] text-muted-foreground uppercase">
        Engine options
      </h3>
      <OptionSwitch name="pretty" label="Pretty output" />
      <OptionSwitch name="wrap" label="Wrap phrase" />
      <OptionSwitch name="prioritizedName" label="Prioritized Name" />
      <div className="flex min-h-10 items-center justify-between border-b border-border/70 py-2">
        <div>
          <div className="text-xs font-medium">Range mapping</div>
          <div className="mt-0.5 text-[10px] text-muted-foreground">Luôn bật để đối chiếu hai chiều</div>
        </div>
        <Switch checked disabled aria-label="Range mapping luôn bật" />
      </div>
      <div className="grid min-h-12 grid-cols-[1fr_92px] items-center gap-3 border-b border-border/70 py-2">
        <Label htmlFor="scanRange" className="text-xs font-medium">Scan Range</Label>
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
      <div className="grid min-h-12 grid-cols-[1fr_92px] items-center gap-3 py-2">
        <Label htmlFor="translationAlgorithm" className="text-xs font-medium">Algorithm</Label>
        <Controller
          name="translationAlgorithm"
          control={control}
          render={({ field }) => (
            <Select value={String(field.value)} onValueChange={(value) => field.onChange(Number(value))}>
              <SelectTrigger id="translationAlgorithm" className="h-8 w-full font-mono text-xs">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="0">0</SelectItem>
                <SelectItem value="1">1</SelectItem>
                <SelectItem value="2">2</SelectItem>
              </SelectContent>
            </Select>
          )}
        />
      </div>
    </section>
  );
}
