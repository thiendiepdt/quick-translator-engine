import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";
import { z } from "zod";

import { GlossaryBackfillDialog } from "@/components/glossary-backfill-dialog";
import { Button } from "@/components/ui/button";
import { glossaryPendingList, glossaryPendingResolve } from "@/lib/api";
import { backfillHintSchema, glossaryPendingSchema } from "@/lib/schema";

type Pending = z.infer<typeof glossaryPendingSchema>;
type Hint = z.infer<typeof backfillHintSchema>;

const KIND_LABELS: Record<Pending["kind"], string> = {
  homophone: "khác một chữ đồng âm — có thể tác giả gõ nhầm",
  anagram: "cùng các chữ, đảo thứ tự — có thể tác giả gõ nhầm",
  dotSegment: "một đoạn của tên có · mà cách dịch lệch",
  prefix: "cùng chữ Hán nhưng hai mục đọc khác âm",
  oldUsage: "mục mới khác cách dịch ở các chương đã dịch trước",
};

interface Props {
  root: string;
  running: boolean;
  /** Form có thay đổi chưa lưu: gộp sẽ nạp lại hồ sơ và làm mất chúng, nên khoá nút. */
  dirty: boolean;
  /** story.json vừa đổi trên đĩa — nạp lại snapshot. */
  onStoryChanged: () => void;
}

/**
 * Khối "Chờ xác nhận" ở tab Glossary: mục tự học nghi trùng mục cũ. Mục mới đã vào glossary như trước; ở đây người
 * dùng chọn cùng một (dùng tên cũ / tên mới, rồi Rà chương cũ) hay hai thứ khác nhau.
 */
export function GlossaryPendingPanel({ root, running, dirty, onStoryChanged }: Props) {
  const [items, setItems] = useState<Pending[]>([]);
  const [hint, setHint] = useState<Hint>();
  const [busy, setBusy] = useState(false);

  const reload = useCallback(() => {
    void glossaryPendingList(root)
      .then(setItems)
      .catch(() => setItems([]));
  }, [root]);
  useEffect(reload, [reload]);

  if (items.length === 0 && !hint) return null;
  const locked = running || dirty || busy;

  /** Mục mới trái cách dịch cũ: bỏ câu hỏi rồi mở Rà chương cũ cho chính mục đó (người dùng gõ cụm cũ cần thay). */
  async function backfillOld(item: Pending) {
    await answer(item, "dismiss");
    setHint({ key: item.source, from: "", to: item.target });
  }

  async function answer(item: Pending, choice: "keepOld" | "useNew" | "dismiss") {
    setBusy(true);
    try {
      const result = await glossaryPendingResolve(root, item.source, choice);
      reload();
      if (result) {
        onStoryChanged();
        setHint(result);
      }
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="space-y-2 rounded-md border border-status-warning/40 bg-status-warning/5 p-3">
      {items.length > 0 && (
        <>
          <p className="text-sm font-medium">Chờ xác nhận ({items.length})</p>
          <p className="text-xs text-muted-foreground">
            Mục mới tự học nghi trùng mục cũ. Mục mới đã dùng trong bản dịch; chọn cùng một thì glossary đổi theo và mở Rà
            chương cũ.
            {running && " Đang có phiên dịch chạy — dừng trước khi trả lời."}
            {!running && dirty && " Lưu thay đổi trên form trước khi trả lời."}
          </p>
          <ul className="space-y-2">
            {items.map((item) => {
              const mergeable = item.kind === "homophone" || item.kind === "anagram";
              if (item.kind === "oldUsage") {
                return (
                  <li key={item.source} className="space-y-1 rounded border bg-background p-2 text-sm">
                    <div>
                      <span className="font-medium">{item.source}</span> “{item.target}” <span className="text-muted-foreground">(mới, ch.{item.chapter})</span>
                    </div>
                    <div className="text-xs text-muted-foreground">
                      {item.similarCount} / {item.sourceCount} đoạn đã dịch có {item.source} không dùng “{item.target}”
                    </div>
                    <div className="flex flex-wrap gap-2">
                      <Button type="button" size="sm" variant="outline" disabled={locked} onClick={() => void backfillOld(item)}>
                        Rà chương cũ
                      </Button>
                      <Button type="button" size="sm" variant="ghost" disabled={locked} onClick={() => void answer(item, "dismiss")}>
                        Đã xem
                      </Button>
                    </div>
                  </li>
                );
              }
              return (
                <li key={item.source} className="space-y-1 rounded border bg-background p-2 text-sm">
                  <div>
                    <span className="font-medium">{item.source}</span> “{item.target}” <span className="text-muted-foreground">(ch.{item.chapter}, raw {item.sourceCount} lần)</span>
                    {" ⟷ "}
                    <span className="font-medium">{item.similarSource}</span> “{item.similarTarget}”{" "}
                    <span className="text-muted-foreground">(raw {item.similarCount} lần)</span>
                  </div>
                  <div className="text-xs text-muted-foreground">{KIND_LABELS[item.kind]}</div>
                  <div className="flex flex-wrap gap-2">
                    {mergeable && (
                      <>
                        <Button type="button" size="sm" variant="outline" disabled={locked} onClick={() => void answer(item, "keepOld")}>
                          Cùng một — dùng “{item.similarTarget}”
                        </Button>
                        <Button type="button" size="sm" variant="outline" disabled={locked} onClick={() => void answer(item, "useNew")}>
                          Cùng một — dùng “{item.target}”
                        </Button>
                      </>
                    )}
                    <Button type="button" size="sm" variant="ghost" disabled={locked} onClick={() => void answer(item, "dismiss")}>
                      {mergeable ? "Hai thứ khác nhau" : "Đã xem (sửa tay ở bảng bên dưới nếu cần)"}
                    </Button>
                  </div>
                </li>
              );
            })}
          </ul>
        </>
      )}
      {hint && (
        <GlossaryBackfillDialog
          open
          onOpenChange={(open) => {
            if (!open) setHint(undefined);
          }}
          root={root}
          source={hint.key}
          target={hint.to}
          initialFrom={hint.from}
          running={running}
        />
      )}
    </div>
  );
}
