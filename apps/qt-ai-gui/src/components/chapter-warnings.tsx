import { AlertTriangle, ChevronDown, ChevronUp } from "lucide-react";
import { useMemo, useState } from "react";

import { Button } from "@/components/ui/button";
import { groupWarnings } from "@/lib/chapter-warnings";

/** Số đoạn hiện sẵn mỗi luật; còn lại sau nút "+N". */
const SHOWN_HITS = 20;

/**
 * Khung cảnh báo còn lại của chương, gom theo luật: mỗi luật một dòng kèm số đoạn dính luật (rê chuột xem câu trích,
 * bấm nhảy tới đoạn ở tab Bản dịch). Danh sách cao tối đa 1/4 màn hình rồi cuộn trong, nên chương vài trăm cảnh báo
 * không che nội dung. `onJump` undefined (chưa có bản dịch) thì số không bấm được.
 */
export function ChapterWarnings({
  warnings,
  collapsed,
  onCollapsedChange,
  onJump,
}: {
  warnings: readonly string[];
  collapsed: boolean;
  onCollapsedChange: (collapsed: boolean) => void;
  onJump?: (n: number) => void;
}) {
  const groups = useMemo(() => groupWarnings(warnings), [warnings]);
  // Luật đã bấm "+N": gắn với danh sách cảnh báo nên sang chương khác thì về như cũ.
  const [expanded, setExpanded] = useState<{ warnings: readonly string[]; rules: ReadonlySet<string> }>();
  const open = expanded?.warnings === warnings ? expanded.rules : new Set<string>();

  return (
    <section aria-label="Cảnh báo còn lại" className="border-b bg-status-warning/10 px-5 py-2 text-xs">
      <div className="flex items-center gap-1.5 font-medium text-status-warning">
        <AlertTriangle className="size-3.5" />
        <span>
          {warnings.length} cảnh báo còn lại · {groups.length} luật
        </span>
        <Button
          type="button"
          variant="ghost"
          size="xs"
          className="ml-auto text-muted-foreground"
          aria-expanded={!collapsed}
          onClick={() => onCollapsedChange(!collapsed)}
        >
          {collapsed ? <ChevronDown /> : <ChevronUp />} {collapsed ? "Mở ra" : "Thu gọn"}
        </Button>
      </div>
      {!collapsed && (
        <ul className="fine-scrollbar mt-1 max-h-[25vh] list-disc space-y-1.5 overflow-y-auto pl-5 text-foreground/80">
          {groups.map((group) => {
            const all = open.has(group.rule);
            const hits = all ? group.hits : group.hits.slice(0, SHOWN_HITS);
            const hidden = group.hits.length - hits.length;
            return (
              <li key={group.rule}>
                <span>
                  {group.rule}
                  {group.hits.length > 0 && <span className="text-muted-foreground"> — {group.hits.length} đoạn</span>}
                </span>
                {hits.length > 0 && (
                  <div className="mt-1 flex flex-wrap gap-1">
                    {hits.map((hit, index) => (
                      <button
                        key={`${hit.n}-${index}`}
                        type="button"
                        aria-label={`Tới đoạn ${hit.n}`}
                        title={hit.excerpt}
                        disabled={!onJump}
                        onClick={() => onJump?.(hit.n)}
                        className="rounded border border-status-warning/40 px-1.5 font-mono tabular-nums hover:bg-status-warning/15 disabled:cursor-default disabled:opacity-60"
                      >
                        {hit.n}
                      </button>
                    ))}
                    {hidden > 0 && (
                      <button
                        type="button"
                        title={`Hiện hết ${group.hits.length} đoạn`}
                        onClick={() => setExpanded({ warnings, rules: new Set([...open, group.rule]) })}
                        className="rounded px-1.5 font-medium text-status-warning hover:bg-status-warning/15"
                      >
                        +{hidden}
                      </button>
                    )}
                  </div>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </section>
  );
}
