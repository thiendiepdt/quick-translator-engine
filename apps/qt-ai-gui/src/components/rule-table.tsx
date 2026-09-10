import { Plus, Trash2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";

export interface RuleRow {
  pattern: string;
  flags: string;
  message: string;
}

interface Props {
  rows: RuleRow[];
  onChange: (rows: RuleRow[]) => void;
  /** Chỉ hiện (bộ mặc định chưa sao chép ra), không có ô nhập. */
  readOnly?: boolean;
}

/** Bảng rule regex thuần props — dùng cho rule riêng của truyện lẫn base rule ở Cài đặt. */
export function RuleTable({ rows, onChange, readOnly }: Props) {
  if (readOnly) {
    return (
      <div className="flex flex-col gap-1">
        {rows.map((rule, index) => (
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
      </div>
    );
  }
  const update = (index: number, patch: Partial<RuleRow>) =>
    onChange(rows.map((row, i) => (i === index ? { ...row, ...patch } : row)));
  return (
    <div className="flex flex-col gap-1">
      {rows.map((rule, index) => (
        <div key={index} className="grid grid-cols-[2fr_60px_2fr_auto] gap-1">
          <Input
            value={rule.pattern}
            onChange={(e) => update(index, { pattern: e.target.value })}
            placeholder="regex (cú pháp JS)"
            aria-label={`Regex ${index + 1}`}
            className="h-8 font-mono"
          />
          <Input
            value={rule.flags}
            onChange={(e) => update(index, { flags: e.target.value })}
            placeholder="i"
            aria-label={`Flags ${index + 1}`}
            className="h-8 font-mono"
          />
          <Input
            value={rule.message}
            onChange={(e) => update(index, { message: e.target.value })}
            placeholder="Mô tả / cách sửa"
            aria-label={`Mô tả rule ${index + 1}`}
            className="h-8"
          />
          <Button
            type="button"
            size="icon-sm"
            variant="ghost"
            aria-label="Xoá rule"
            onClick={() => onChange(rows.filter((_, i) => i !== index))}
          >
            <Trash2 />
          </Button>
        </div>
      ))}
      <Button
        type="button"
        size="xs"
        variant="ghost"
        className="w-fit"
        onClick={() => onChange([...rows, { pattern: "", flags: "", message: "" }])}
      >
        <Plus /> Thêm
      </Button>
    </div>
  );
}
