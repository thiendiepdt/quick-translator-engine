import type { IdiomTable } from "@/lib/api";

/** remote: chỉ bảng chung; own: chỉ bảng của người dùng; override: người dùng sửa câu của bảng chung; off: tắt câu chung. */
export type IdiomRowKind = "remote" | "own" | "override" | "off";

export interface IdiomRow {
  key: string;
  /** Cách dịch đang áp (rỗng khi đã tắt). */
  value: string;
  remote?: string;
  kind: IdiomRowKind;
}

export const IDIOM_KIND_LABELS: Record<IdiomRowKind, string> = {
  remote: "Bảng chung",
  own: "Của bạn",
  override: "Đã sửa",
  off: "Đã tắt",
};

/** Câu riêng của người dùng trước (thứ tự thêm), rồi bảng chung theo thứ tự của nó. */
export function idiomRows(remote: IdiomTable, user: IdiomTable): IdiomRow[] {
  const own: IdiomRow[] = Object.entries(user)
    .filter(([key]) => !(key in remote))
    .map(([key, value]) => ({ key, value, kind: "own" }));
  const shared: IdiomRow[] = Object.entries(remote).map(([key, value]) => {
    const mine = user[key];
    if (mine === undefined) return { key, value, remote: value, kind: "remote" };
    return { key, value: mine, remote: value, kind: mine === "" ? "off" : "override" };
  });
  return [...own, ...shared];
}

/** Ghi cách dịch vào bảng người dùng; trùng bảng chung thì bỏ mục đè. */
export function setIdiom(user: IdiomTable, remote: IdiomTable, key: string, value: string): IdiomTable {
  const next = { ...user };
  if (remote[key] === value) delete next[key];
  else next[key] = value;
  return next;
}

export function removeIdiom(user: IdiomTable, key: string): IdiomTable {
  const next = { ...user };
  delete next[key];
  return next;
}

export function matchesIdiom(row: IdiomRow, query: string): boolean {
  const needle = query.trim().toLowerCase();
  if (!needle) return true;
  return row.key.includes(needle) || row.value.toLowerCase().includes(needle) || (row.remote ?? "").toLowerCase().includes(needle);
}
