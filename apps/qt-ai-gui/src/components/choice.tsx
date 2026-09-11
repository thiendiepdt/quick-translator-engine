import { cn } from "@/lib/utils";

/** Ô chọn kiểu radio giống chọn chế độ sáng/tối — dùng ở Cài đặt và các dialog Bản mặc định. */
export function Choice<T extends string>({
  label,
  value,
  options,
  labels,
  onChange,
  disabled,
}: {
  label: string;
  value: T;
  options: readonly T[];
  labels: Record<T, string>;
  onChange: (value: T) => void;
  disabled?: boolean;
}) {
  return (
    <div role="radiogroup" aria-label={label} className="inline-flex w-fit rounded-md border p-0.5">
      {options.map((option) => (
        <button
          key={option}
          type="button"
          role="radio"
          aria-checked={value === option}
          disabled={disabled}
          onClick={() => onChange(option)}
          className={cn(
            "rounded-[calc(var(--radius)-2px)] px-3 py-1.5 text-sm disabled:opacity-50",
            value === option ? "bg-primary text-primary-foreground" : "text-muted-foreground hover:bg-accent",
          )}
        >
          {labels[option]}
        </button>
      ))}
    </div>
  );
}
