import { ChevronRight } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { CHANGELOG_ENTRIES, entriesSince, inlineTokens, releasedEntries, type ChangelogEntry } from "@/lib/changelog";
import { cn } from "@/lib/utils";

const COLLAPSED_COUNT = 5;

/** 2026-10-13 → 13/10/2026. */
function formatDate(date: string): string {
  const [year, month, day] = date.split("-");
  return `${day}/${month}/${year}`;
}

/** Một bullet changelog: chỉ **đậm** và `code`, dựng bằng phần tử React (chữ còn lại luôn là chữ, không phải HTML). */
export function InlineText({ text }: { text: string }) {
  return (
    <>
      {inlineTokens(text).map((token, i) =>
        token.kind === "bold" ? (
          <strong key={i} className="font-semibold">
            {token.text}
          </strong>
        ) : token.kind === "code" ? (
          <code key={i} className="rounded bg-muted px-1 py-0.5 font-mono text-[0.85em]">
            {token.text}
          </code>
        ) : (
          <span key={i}>{token.text}</span>
        ),
      )}
    </>
  );
}

function Bullets({ items }: { items: string[] }) {
  return (
    <ul className="flex list-disc flex-col gap-1.5 pl-5 text-sm leading-relaxed marker:text-muted-foreground">
      {items.map((item, i) => (
        <li key={i}>
          <InlineText text={item} />
        </li>
      ))}
    </ul>
  );
}

/** Danh sách các bản đã phát hành tới bản đang dùng, mới nhất trước: bản đang dùng mở sẵn, bản cũ bấm để mở. */
export function ChangelogList({
  currentVersion,
  entries = CHANGELOG_ENTRIES,
  initialShowAll = false,
}: {
  currentVersion: string;
  entries?: readonly ChangelogEntry[];
  /** true = hiện đủ mọi bản ngay (người dùng vừa bấm "Xem tất cả" ở nơi khác). */
  initialShowAll?: boolean;
}) {
  const released = releasedEntries(currentVersion, entries);
  const [expanded, setExpanded] = useState<ReadonlySet<string>>(() => new Set([currentVersion]));
  const [showAll, setShowAll] = useState(initialShowAll);
  const visible = showAll ? released : released.slice(0, COLLAPSED_COUNT);

  function toggle(version: string) {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(version)) next.delete(version);
      else next.add(version);
      return next;
    });
  }

  if (released.length === 0) return <p className="text-sm text-muted-foreground">Chưa có ghi chú cho bản này.</p>;
  return (
    <div className="flex flex-col gap-1">
      {visible.map((entry) => {
        const open = expanded.has(entry.version);
        return (
          <div key={entry.version} className="flex flex-col">
            <button
              type="button"
              aria-expanded={open}
              onClick={() => toggle(entry.version)}
              className="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-accent"
            >
              <ChevronRight className={cn("size-4 shrink-0 text-muted-foreground transition-transform", open && "rotate-90")} />
              <span className="font-medium">Bản {entry.version}</span>
              <span className="text-xs text-muted-foreground tabular-nums">{formatDate(entry.date)}</span>
              {entry.version === currentVersion && (
                <span className="rounded-full bg-primary/10 px-2 py-0.5 text-xs font-medium text-primary">Đang dùng</span>
              )}
            </button>
            {open && (
              <div className="pt-1 pr-2 pb-3 pl-8">
                <Bullets items={entry.items} />
              </div>
            )}
          </div>
        );
      })}
      {released.length > COLLAPSED_COUNT && (
        <Button type="button" variant="link" size="sm" className="w-fit px-2" onClick={() => setShowAll((v) => !v)}>
          {showAll ? "Thu gọn" : `Xem tất cả (${released.length} bản)`}
        </Button>
      )}
    </div>
  );
}

const DIALOG_MAX_ENTRIES = 5;

interface WhatsNewDialogProps {
  /** Bản vừa cập nhật lên; undefined = đóng. */
  version: string | undefined;
  /** Bản đã thấy lần trước: hiện mọi bản sau nó tới `version`. null/không truyền = chỉ `version`. */
  since?: string | null;
  onClose: () => void;
  /** Có thì "Xem tất cả" chuyển sang Cài đặt; không (màn chọn truyện chưa có Cài đặt) thì mở danh sách ngay trong hộp thoại. */
  onShowAll?: () => void;
  entries?: readonly ChangelogEntry[];
}

/** Hộp thoại "Có gì mới" hiện một lần ngay sau khi app được cập nhật lên bản mới (gồm cả các bản đã bỏ qua). */
export function WhatsNewDialog({ version, since = null, onClose, onShowAll, entries = CHANGELOG_ENTRIES }: WhatsNewDialogProps) {
  const [expandedList, setExpandedList] = useState(false);
  const range = version ? entriesSince(version, since, entries).slice(0, DIALOG_MAX_ENTRIES) : [];
  if (!version || range.length === 0) return null;
  return (
    <Dialog open onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Có gì mới ở bản {version}</DialogTitle>
          <DialogDescription>
            {since ? `Các thay đổi kể từ bản ${since}.` : `App vừa được cập nhật (${formatDate(range[0].date)}).`}
          </DialogDescription>
        </DialogHeader>
        <div className="fine-scrollbar -mx-6 min-h-0 flex-1 overflow-y-auto px-6">
          {expandedList ? (
            <ChangelogList currentVersion={version} entries={entries} initialShowAll />
          ) : range.length === 1 ? (
            <Bullets items={range[0].items} />
          ) : (
            <div className="flex flex-col gap-4">
              {range.map((entry) => (
                <section key={entry.version} className="flex flex-col gap-1.5">
                  <h3 className="flex items-baseline gap-2 text-sm font-semibold">
                    Bản {entry.version}
                    <span className="text-xs font-normal text-muted-foreground tabular-nums">{formatDate(entry.date)}</span>
                  </h3>
                  <Bullets items={entry.items} />
                </section>
              ))}
            </div>
          )}
        </div>
        <DialogFooter className="sm:items-center sm:justify-between">
          {expandedList ? (
            <span />
          ) : (
            <Button type="button" variant="link" size="sm" className="px-0" onClick={() => (onShowAll ? onShowAll() : setExpandedList(true))}>
              Xem tất cả
            </Button>
          )}
          <Button type="button" onClick={onClose}>
            Đã hiểu
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
