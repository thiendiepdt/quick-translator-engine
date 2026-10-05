import { AlertTriangle, Save, Search, Undo2 } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { titlesApply, titlesSurvey } from "@/lib/api";
import type { TitleSurvey } from "@/lib/types";
import { cn } from "@/lib/utils";

interface Props {
  root: string;
  /** Phiên dịch đang chạy: xem được, không áp dụng được (phiên đang ghi out/ và titles.json). */
  running: boolean;
}

/** Bảng hiện từng khúc chừng này dòng — truyện chia phần có thể có cả trăm tên lặp. */
const PAGE = 40;

/**
 * Tên tiêu đề chương lặp lại (truyện chia phần: "清宫炮灰小宫女1…完"). App chốt bản dịch của mỗi tên ngay lần
 * đầu chốt chương và ép các chương sau dùng lại; bảng này để xem tên nào đang bị dịch nhiều kiểu, sửa bản dịch,
 * hoặc điền trước cho phần chưa dịch. Lưu = ghi `titles.json` + viết lại dòng tiêu đề của các chương đã dịch.
 * Dữ liệu ở file riêng nên có nút Lưu riêng, như bảng nhân vật.
 */
export function TitlesPanel({ root, running }: Props) {
  const [survey, setSurvey] = useState<TitleSurvey>();
  const [error, setError] = useState("");
  /** Bản dịch người dùng đang gõ, theo tên gốc; chỉ chứa dòng đã đụng tới. */
  const [edits, setEdits] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState(false);
  const [query, setQuery] = useState("");
  const [onlySplit, setOnlySplit] = useState(false);
  const [limit, setLimit] = useState(PAGE);

  useEffect(() => {
    let alive = true;
    void titlesSurvey(root)
      .then((next) => alive && setSurvey(next))
      .catch((cause: unknown) => alive && setError(cause instanceof Error ? cause.message : "Không đọc được tiêu đề chương"));
    return () => {
      alive = false;
    };
  }, [root]);

  const groups = useMemo(() => survey?.groups ?? [], [survey]);
  const changed = useMemo(
    () =>
      Object.fromEntries(
        groups
          .filter((group) => edits[group.source] !== undefined && edits[group.source]?.trim() !== group.target)
          .map((group) => [group.source, edits[group.source]?.trim() ?? ""]),
      ),
    [groups, edits],
  );
  if (!survey) return <p className="text-sm text-muted-foreground">{error || "Đang đọc tiêu đề các chương…"}</p>;
  if (groups.length === 0) {
    return (
      <p className="text-sm text-muted-foreground">
        Truyện này không có tên tiêu đề nào lặp lại ở từ hai chương trở lên, nên không có gì cần chốt.
      </p>
    );
  }

  const split = groups.filter((group) => group.variants.length > 1).length;
  const pending = groups.reduce((sum, group) => sum + group.pending, 0);
  const edited = Object.keys(changed).length;
  // Ô để trống = chưa chốt; không gửi lên để khỏi xoá bản đang dùng.
  const targets = Object.fromEntries(Object.entries(changed).filter(([, target]) => target !== ""));
  const needle = query.trim().toLowerCase();
  const rows = groups.filter((group) => {
    if (onlySplit && group.variants.length <= 1 && group.pending === 0) return false;
    return !needle || group.source.toLowerCase().includes(needle) || group.target.toLowerCase().includes(needle);
  });

  async function apply() {
    setBusy(true);
    try {
      const result = await titlesApply(root, targets);
      setSurvey(result.survey);
      setEdits({});
      toast.success(
        result.changed > 0
          ? `Đã chốt tiêu đề và sửa dòng tiêu đề của ${result.changed} chương`
          : "Đã chốt tiêu đề; không chương nào cần sửa",
      );
    } catch (cause) {
      toast.error(cause instanceof Error ? cause.message : "Không áp dụng được tiêu đề");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm text-muted-foreground">
        Tên tiêu đề lặp ở nhiều chương (truyện chia phần). Lúc chốt chương, app dùng lại đúng bản dịch đã chốt của tên
        đó và viết đuôi thống nhất: số trần thành “1”, số trong ngoặc thành “(1)”, 完 thành “(hết)”. Tên tác giả viết
        khác chữ là tên khác, không gộp.
      </p>
      <p role="status" className={cn("text-sm", split > 0 || pending > 0 ? "text-status-warning" : "text-muted-foreground")}>
        {groups.length} tên lặp · {split} tên đang bị dịch nhiều kiểu · {pending} dòng tiêu đề sẽ được sửa khi lưu
        {survey.unparsed > 0 ? ` · ${survey.unparsed} chương có dòng tiêu đề không tách được, để nguyên` : ""}
      </p>
      <div className="flex flex-wrap items-center gap-2">
        <div className="relative min-w-52 flex-1">
          <Search className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input
            value={query}
            onChange={(event) => {
              setQuery(event.target.value);
              setLimit(PAGE);
            }}
            placeholder="Tìm theo tên gốc hoặc bản dịch…"
            aria-label="Tìm tiêu đề"
            className="pl-8"
          />
        </div>
        <label className="flex items-center gap-2 text-sm">
          <input type="checkbox" checked={onlySplit} onChange={(event) => setOnlySplit(event.target.checked)} />
          Chỉ tên đang lệch
        </label>
        <Button type="button" variant="ghost" size="sm" disabled={busy || edited === 0} onClick={() => setEdits({})}>
          <Undo2 /> Hoàn tác
        </Button>
        <Button
          type="button"
          size="sm"
          disabled={busy || running || (edited === 0 && pending === 0)}
          title={
            running
              ? "Dừng phiên dịch trước khi sửa tiêu đề"
              : "Ghi bản dịch đã chốt và viết lại dòng tiêu đề của các chương đã dịch cho khớp"
          }
          onClick={() => void apply()}
        >
          <Save /> Lưu &amp; áp dụng
        </Button>
      </div>
      <ul className="flex flex-col gap-2" aria-label="Tên tiêu đề lặp">
        {rows.slice(0, limit).map((group) => {
          const value = edits[group.source] ?? group.target;
          return (
            <li key={group.source} className="rounded-md border bg-card p-3">
              <div className="flex flex-wrap items-center gap-2">
                <span className="min-w-40 flex-1 font-medium">{group.source}</span>
                <span className="text-xs tabular-nums text-muted-foreground">
                  {group.translated}/{group.chapters} chương đã dịch
                  {group.pending > 0 ? ` · sửa ${group.pending}` : ""}
                </span>
              </div>
              <Input
                value={value}
                onChange={(event) => setEdits((current) => ({ ...current, [group.source]: event.target.value }))}
                placeholder="Chưa dịch — điền trước nếu muốn chốt sẵn"
                aria-label={`Bản dịch của ${group.source}`}
                className="mt-2"
              />
              {group.variants.length > 1 && (
                <div className="mt-2 flex flex-wrap items-center gap-1.5 text-xs">
                  <AlertTriangle className="size-3.5 text-status-warning" aria-hidden />
                  <span className="text-status-warning">Đang có {group.variants.length} kiểu dịch:</span>
                  {group.variants.map((variant) => (
                    <button
                      key={variant.text}
                      type="button"
                      title="Dùng kiểu này"
                      onClick={() => setEdits((current) => ({ ...current, [group.source]: variant.text }))}
                      className={cn(
                        "rounded border px-1.5 py-0.5 hover:bg-accent",
                        variant.text === value.trim() && "border-primary text-primary",
                      )}
                    >
                      {variant.text} <span className="tabular-nums text-muted-foreground">×{variant.count}</span>
                    </button>
                  ))}
                </div>
              )}
              {!group.settled && group.target && group.variants.length <= 1 && (
                <p className="mt-1 text-xs text-muted-foreground">Chưa chốt — đang lấy theo bản dịch hiện có.</p>
              )}
            </li>
          );
        })}
      </ul>
      {rows.length === 0 && <p className="text-sm text-muted-foreground">Không có tên nào khớp.</p>}
      {rows.length > limit && (
        <Button type="button" variant="outline" size="sm" className="self-center" onClick={() => setLimit((v) => v + PAGE)}>
          Xem thêm ({rows.length - limit})
        </Button>
      )}
    </div>
  );
}
