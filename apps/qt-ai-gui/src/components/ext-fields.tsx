import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import type { ExtSetting } from "@/lib/api";
import { defaultHint, type FieldDisplay } from "@/lib/ext";

interface ExtFieldProps {
  field: ExtSetting;
  value: FieldDisplay;
  onChange: (value: FieldDisplay) => void;
  /** Tiền tố id ô nhập, để hai form trên cùng trang không trùng id. */
  idPrefix: string;
}

/** Một ô nhập vẽ từ trường cài đặt / auth trong manifest của extension. */
export function ExtField({ field, value, onChange, idPrefix }: ExtFieldProps) {
  const id = `${idPrefix}-${field.key}`;
  const hint = defaultHint(field);

  if (field.kind === "boolean") {
    return (
      <div className="flex items-center gap-2">
        <Switch id={id} checked={value === true} onCheckedChange={(checked) => onChange(checked)} />
        <Label htmlFor={id}>{field.label}</Label>
      </div>
    );
  }

  if (field.kind === "select") {
    return (
      <div className="flex flex-col gap-1.5">
        <Label htmlFor={id}>{field.label}</Label>
        <Select value={String(value)} onValueChange={onChange}>
          <SelectTrigger id={id} aria-label={field.label} className="w-full">
            <SelectValue placeholder={hint || "Chọn…"} />
          </SelectTrigger>
          <SelectContent>
            {field.choices.map((choice) => (
              <SelectItem key={choice} value={choice}>
                {choice}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    );
  }

  const secret = field.kind === "secret";
  return (
    <div className="flex flex-col gap-1.5">
      <Label htmlFor={id}>{field.label}</Label>
      <Input
        id={id}
        type={secret ? "password" : "text"}
        inputMode={field.kind === "number" ? "decimal" : undefined}
        autoComplete={secret ? "new-password" : "off"}
        value={String(value)}
        onChange={(e) => onChange(e.target.value)}
        placeholder={secret ? (field.filled ? "Đã lưu — để trống để giữ" : "") : hint}
        className={field.kind === "number" ? "font-mono" : undefined}
      />
    </div>
  );
}
