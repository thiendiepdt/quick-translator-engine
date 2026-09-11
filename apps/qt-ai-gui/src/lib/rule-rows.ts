import type { RuleRow } from "@/components/rule-table";
import type { CheckRule } from "@/lib/types";

/** CheckRule (Rust, flags tuỳ chọn) → dòng bảng (flags luôn là chuỗi). */
export function rowsOf(rules: CheckRule[] | undefined): RuleRow[] {
  return (rules ?? []).map((r) => ({ pattern: r.pattern, flags: r.flags ?? "", message: r.message }));
}

/** Bỏ dòng trống pattern; flags trống không gửi (khớp `skip_serializing_if` bên Rust). */
export function rulesOf(rows: RuleRow[]): CheckRule[] {
  return rows
    .filter((r) => r.pattern.trim())
    .map((r) =>
      r.flags.trim()
        ? { pattern: r.pattern, flags: r.flags.trim(), message: r.message }
        : { pattern: r.pattern, message: r.message },
    );
}
