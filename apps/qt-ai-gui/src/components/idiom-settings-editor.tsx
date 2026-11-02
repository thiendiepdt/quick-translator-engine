import { Plus, RefreshCw, RotateCcw, Save, Search } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { idiomsGet, idiomsRefresh, idiomsSaveUser, type IdiomsInfo, type IdiomTable } from "@/lib/api";
import { IDIOM_KIND_LABELS, idiomRows, matchesIdiom, removeIdiom, setIdiom, type IdiomRow } from "@/lib/idioms";
import { clockLabel } from "@/lib/types";
import { cn } from "@/lib/utils";

/** Bảng dài cả nghìn câu: chỉ vẽ chừng này dòng, còn lại lọc bằng ô tìm. */
const MAX_ROWS = 200;

/**
 * Bảng thành ngữ: bảng chung tải từ GitHub mỗi lần mở app (chỉ xem, sửa = đè), bảng của người dùng đè theo khoá.
 * Chương có câu nào thì AI bắt buộc dịch đúng cách ghi; bản dịch không theo thì cảnh báo khi chốt chương.
 */
export function IdiomSettingsEditor() {
  const [info, setInfo] = useState<IdiomsInfo>();
  const [draft, setDraft] = useState<IdiomTable>({});
  const [query, setQuery] = useState("");
  const [newKey, setNewKey] = useState("");
  const [newValue, setNewValue] = useState("");
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let alive = true;
    idiomsGet()
      .then((next) => {
        if (!alive) return;
        setInfo(next);
        setDraft(next.user);
      })
      .catch((error: unknown) => alive && toast.error(error instanceof Error ? error.message : "Không đọc được bảng thành ngữ"));
    return () => {
      alive = false;
    };
  }, []);

  const rows = useMemo(() => (info ? idiomRows(info.remote, draft) : []), [info, draft]);
  const shown = useMemo(() => rows.filter((row) => matchesIdiom(row, query)), [rows, query]);

  if (!info) return <p className="text-sm text-muted-foreground">Đang đọc…</p>;
  const remote = info.remote;
  const dirty = JSON.stringify(draft) !== JSON.stringify(info.user);
  const remoteCount = Object.keys(remote).length;

  async function run(action: () => Promise<IdiomsInfo>, message?: (next: IdiomsInfo) => string | undefined) {
    setBusy(true);
    try {
      const next = await action();
      setInfo(next);
      setDraft(next.user);
      const text = message?.(next);
      if (text) toast.success(text);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được");
    } finally {
      setBusy(false);
    }
  }

  function refresh() {
    if (dirty && !window.confirm("Tải lại bảng chung sẽ bỏ các thay đổi chưa lưu của bạn. Tiếp tục?")) return;
    void run(idiomsRefresh, (next) => (next.error ? undefined : `Đã tải bảng chung: ${Object.keys(next.remote).length} câu`));
  }

  function add() {
    const key = newKey.trim();
    const value = newValue.trim();
    if (!key || !value) return;
    setDraft(setIdiom(draft, remote, key, value));
    setNewKey("");
    setNewValue("");
  }

  function action(row: IdiomRow) {
    switch (row.kind) {
      case "remote":
        return { label: "Tắt", run: () => setDraft({ ...draft, [row.key]: "" }) };
      case "own":
        return { label: "Xoá", run: () => setDraft(removeIdiom(draft, row.key)) };
      case "override":
      case "off":
        return { label: "Về bảng chung", run: () => setDraft(removeIdiom(draft, row.key)) };
    }
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center gap-2 text-sm">
        <span>
          Bảng chung: <span className="font-medium tabular-nums">{remoteCount}</span> câu ·{" "}
          {info.source === "cache" && info.fetchedAt !== null
            ? `tải lúc ${clockLabel(info.fetchedAt)}`
            : "bản đóng sẵn trong app (chưa tải được bản mới)"}
        </span>
        <Button type="button" variant="outline" size="sm" className="h-7" disabled={busy} onClick={refresh}>
          <RefreshCw className={cn(busy && "animate-spin")} /> Tải lại
        </Button>
      </div>
      {info.error && <p className="text-xs text-destructive">Lần tải gần nhất lỗi: {info.error} — đang dùng bản cũ.</p>}
      <div className="flex flex-wrap items-end gap-2">
        <Input
          aria-label="Thành ngữ chữ Hán"
          placeholder="Thành ngữ chữ Hán, vd 打草惊蛇"
          value={newKey}
          onChange={(e) => setNewKey(e.target.value)}
          className="w-56 font-mono text-sm"
        />
        <Input
          aria-label="Cách dịch"
          placeholder="Cách dịch — nhiều cách ngăn bằng /"
          value={newValue}
          onChange={(e) => setNewValue(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && add()}
          className="min-w-64 flex-1 text-sm"
        />
        <Button type="button" size="sm" variant="outline" disabled={!newKey.trim() || !newValue.trim()} onClick={add}>
          <Plus /> Thêm
        </Button>
      </div>
      <div className="relative">
        <Search className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" aria-hidden />
        <Input
          aria-label="Tìm thành ngữ"
          placeholder={`Tìm trong ${rows.length} câu (chữ Hán hoặc cách dịch)`}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="pl-8 text-sm"
        />
      </div>
      <ul className="flex max-h-96 flex-col divide-y overflow-y-auto rounded-md border fine-scrollbar">
        {shown.slice(0, MAX_ROWS).map((row) => {
          const act = action(row);
          return (
            <li key={row.key} aria-label={row.key} className="flex items-center gap-2 px-2 py-1.5">
              <span className="w-48 shrink-0 truncate font-mono text-sm" title={row.key}>
                {row.key}
              </span>
              <Input
                aria-label={`Cách dịch ${row.key}`}
                value={row.value}
                placeholder={row.kind === "off" ? `Đã tắt — bảng chung: ${row.remote ?? ""}` : undefined}
                onChange={(e) => setDraft(setIdiom(draft, remote, row.key, e.target.value))}
                className="h-8 flex-1 text-sm"
              />
              <Badge variant={row.kind === "remote" ? "outline" : "secondary"} className="w-24 shrink-0 justify-center">
                {IDIOM_KIND_LABELS[row.kind]}
              </Badge>
              <Button type="button" size="sm" variant="ghost" className="h-7 w-28 shrink-0" onClick={act.run}>
                {act.label}
              </Button>
            </li>
          );
        })}
        {shown.length === 0 && <li className="px-2 py-3 text-sm text-muted-foreground">Không có câu nào khớp.</li>}
        {shown.length > MAX_ROWS && (
          <li className="px-2 py-2 text-xs text-muted-foreground">
            Còn {shown.length - MAX_ROWS} câu nữa — gõ vào ô tìm để lọc.
          </li>
        )}
      </ul>
      <div className="flex items-center gap-2">
        <span className="text-xs text-muted-foreground">
          {Object.keys(draft).length} câu của bạn (thêm, sửa hoặc tắt câu của bảng chung)
        </span>
        <div className="flex-1" />
        <Button type="button" variant="ghost" size="sm" disabled={busy || !dirty} onClick={() => setDraft(info.user)}>
          <RotateCcw /> Huỷ thay đổi
        </Button>
        <Button
          type="button"
          size="sm"
          disabled={busy || !dirty}
          onClick={() => void run(() => idiomsSaveUser(draft), () => "Đã lưu bảng thành ngữ của bạn")}
        >
          <Save /> Lưu
        </Button>
      </div>
    </div>
  );
}
