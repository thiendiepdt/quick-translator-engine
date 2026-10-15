import { AlertTriangle, Check, RefreshCw, Sparkles, Wrench } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { VnReplaceDialog } from "@/components/vn-replace-dialog";
import { vnAiVerify, vnRefresh, vnReport, vnSetStatus } from "@/lib/api";
import type { GenreSetting, VnHitView, VnStatus } from "@/lib/types";
import { VN_GROUP_LABELS, VN_LABEL_TEXT } from "@/lib/types";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

interface Props {
  root: string;
  running: boolean;
  setting: GenreSetting;
}

const SELECT_CLASS = "h-8 rounded-md border border-input bg-background px-2 text-sm";
type StatusFilter = "open" | VnStatus | "all";

/** Tô chữ khớp trong đoạn (lần đầu tiên). */
function Highlight({ text, matched }: { text: string; matched: string }) {
  const at = matched ? text.indexOf(matched) : -1;
  if (at < 0) return <>{text}</>;
  return (
    <>
      {text.slice(0, at)}
      <mark className="rounded bg-status-warning/30 px-0.5">{matched}</mark>
      {text.slice(at + matched.length)}
    </>
  );
}

/** Tab "Kiểm tra VN": chỗ nhạy cảm về Việt Nam trong raw/bản dịch, xác minh, AI sơ loại, thay thế. */
export function VnCheckPanel({ root, running, setting }: Props) {
  const select = useStoryStore((s) => s.select);
  const setPage = useStoryStore((s) => s.setPage);
  const [hits, setHits] = useState<VnHitView[]>();
  const [status, setStatus] = useState<StatusFilter>("open");
  const [group, setGroup] = useState("all");
  const [level, setLevel] = useState("all");
  const [aiLabel, setAiLabel] = useState("all");
  const [order, setOrder] = useState<"chapter" | "entry">("chapter");
  const [selected, setSelected] = useState<string[]>([]);
  const [expanded, setExpanded] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);
  const [replaceOpen, setReplaceOpen] = useState(false);

  const reload = useCallback(async () => setHits(await vnReport(root)), [root]);

  useEffect(() => {
    let alive = true;
    void (async () => {
      try {
        await vnRefresh(root);
        if (alive) await reload();
      } catch (error) {
        toast.error(error instanceof Error ? error.message : "Không quét được");
      }
    })();
    return () => {
      alive = false;
    };
  }, [root, reload]);

  const shown = useMemo(() => {
    const list = (hits ?? []).filter(
      (hit) =>
        (status === "all" || (status === "open" ? hit.status !== "ok" : hit.status === status)) &&
        (group === "all" || hit.group === group) &&
        (level === "all" || hit.level === level) &&
        (aiLabel === "all" || (aiLabel === "none" ? !hit.ai : hit.ai?.label === aiLabel)),
    );
    // sort ổn định: trong cùng mục vẫn giữ thứ tự chương.
    return order === "entry" ? [...list].sort((a, b) => a.entry.localeCompare(b.entry)) : list;
  }, [hits, status, group, level, aiLabel, order]);

  const allShownSelected = shown.length > 0 && shown.every((hit) => selected.includes(hit.id));

  async function mark(ids: string[], next: VnStatus) {
    await vnSetStatus(root, ids, next);
    await reload();
  }

  async function verify() {
    setBusy(true);
    try {
      const outcome = await vnAiVerify(root, selected);
      if (outcome.error) toast.error(`AI xét: ${outcome.error}`);
      else toast.success(`AI đã xét ${outcome.verified} chỗ${outcome.failed ? `, ${outcome.failed} chỗ lỗi — thử lại` : ""}`);
      // Các lô trước có thể đã lưu kể cả khi có lỗi, nên luôn tải lại.
      await reload();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "AI xét thất bại");
    } finally {
      setBusy(false);
    }
  }

  if (!hits) return <p className="text-sm text-muted-foreground">Đang quét…</p>;

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-wrap items-center gap-2">
        <select aria-label="Trạng thái" className={SELECT_CLASS} value={status} onChange={(e) => setStatus(e.target.value as StatusFilter)}>
          <option value="open">Chưa xét + cần xử lý</option>
          <option value="pending">Chưa xét</option>
          <option value="action">Cần xử lý</option>
          <option value="ok">Không sao</option>
          <option value="all">Tất cả</option>
        </select>
        <select aria-label="Nhóm" className={SELECT_CLASS} value={group} onChange={(e) => setGroup(e.target.value)}>
          <option value="all">Mọi nhóm</option>
          {Object.entries(VN_GROUP_LABELS).map(([id, label]) => (
            <option key={id} value={id}>
              {label}
            </option>
          ))}
        </select>
        <select aria-label="Mức" className={SELECT_CLASS} value={level} onChange={(e) => setLevel(e.target.value)}>
          <option value="all">Mọi mức</option>
          <option value="mietthi">Miệt thị</option>
          <option value="canxem">Cần xem</option>
        </select>
        <select aria-label="Nhãn AI" className={SELECT_CLASS} value={aiLabel} onChange={(e) => setAiLabel(e.target.value)}>
          <option value="all">Mọi nhãn AI</option>
          <option value="none">Chưa xét AI</option>
          {Object.entries(VN_LABEL_TEXT).map(([id, label]) => (
            <option key={id} value={id}>
              {label}
            </option>
          ))}
        </select>
        <select aria-label="Xếp theo" className={SELECT_CLASS} value={order} onChange={(e) => setOrder(e.target.value as "chapter" | "entry")}>
          <option value="chapter">Theo chương</option>
          <option value="entry">Theo mục từ điển</option>
        </select>
        <span className="text-xs text-muted-foreground">
          {shown.length} / {hits.length} chỗ
        </span>
        <div className="flex-1" />
        <Button type="button" size="sm" variant="ghost" onClick={() => void vnRefresh(root).then(reload)}>
          <RefreshCw /> Quét lại
        </Button>
        <Button type="button" size="sm" variant="secondary" disabled={busy || selected.length === 0} onClick={() => void verify()}>
          <Sparkles /> AI xét {selected.length} chỗ
        </Button>
        <Button type="button" size="sm" disabled={running || selected.length === 0} onClick={() => setReplaceOpen(true)}>
          <Wrench /> Thay thế…
        </Button>
      </div>
      <label className="flex items-center gap-2 text-sm">
        <input
          type="checkbox"
          aria-label="Chọn tất cả đang lọc"
          checked={allShownSelected}
          onChange={(e) => {
            const ids = shown.map((hit) => hit.id);
            setSelected((list) => (e.target.checked ? [...new Set([...list, ...ids])] : list.filter((id) => !ids.includes(id))));
          }}
        />
        Chọn tất cả đang lọc
      </label>
      {shown.length === 0 && <p className="py-6 text-center text-sm text-muted-foreground">Không có chỗ nào.</p>}
      <ul className="flex flex-col gap-2">
        {shown.map((hit, index) => {
          const open = expanded.includes(hit.id);
          const range = (list: string[]) => (open ? list : list.slice(hit.focus, hit.focus + 1));
          const header = order === "entry" && (index === 0 || shown[index - 1].entry !== hit.entry);
          return (
            <li key={hit.id} className="rounded-lg border bg-card p-3">
              {header && (
                <p data-testid="vn-entry-header" className="mb-2 font-mono text-xs font-semibold text-muted-foreground">
                  {hit.entry}
                </p>
              )}
              <div className="flex flex-wrap items-center gap-2 text-sm">
                <input
                  type="checkbox"
                  aria-label={`Chọn ${hit.id}`}
                  checked={selected.includes(hit.id)}
                  onChange={(e) => setSelected((s) => (e.target.checked ? [...s, hit.id] : s.filter((id) => id !== hit.id)))}
                />
                <span data-testid="vn-hit-chapter" className="font-mono">
                  {hit.chapter}
                </span>
                <span className="text-xs text-muted-foreground">
                  đoạn {hit.paragraph} · {VN_GROUP_LABELS[hit.group]} · {hit.side === "raw" ? "raw (chưa dịch)" : "bản dịch"}
                  {hit.count > 1 ? ` · ×${hit.count}` : ""}
                </span>
                {hit.level === "mietthi" && (
                  <span className="flex items-center gap-1 rounded-full bg-destructive/15 px-2 text-xs text-destructive">
                    <AlertTriangle className="size-3" /> Miệt thị
                  </span>
                )}
                {hit.ai && (
                  <span className={cn("rounded-full px-2 text-xs", hit.ai.label === "xucpham" ? "bg-destructive/15 text-destructive" : "bg-muted")}>
                    AI: {VN_LABEL_TEXT[hit.ai.label]}
                  </span>
                )}
                <div className="flex-1" />
                <Button
                  type="button"
                  size="xs"
                  variant="link"
                  onClick={() => {
                    select(hit.chapter);
                    setPage("translate");
                  }}
                >
                  Mở chương
                </Button>
                <Button type="button" size="xs" variant={hit.status === "ok" ? "secondary" : "ghost"} onClick={() => void mark([hit.id], "ok")}>
                  <Check /> Không sao
                </Button>
                <Button
                  type="button"
                  size="xs"
                  variant={hit.status === "action" ? "destructive" : "ghost"}
                  onClick={() => void mark([hit.id], "action")}
                >
                  Cần xử lý
                </Button>
              </div>
              {hit.ai && <p className="mt-1 text-xs text-muted-foreground">{hit.ai.reason}</p>}
              <div className="mt-2 grid gap-2 text-sm md:grid-cols-2">
                <div className="flex flex-col gap-1 font-serif">
                  {range(hit.raw).map((text, i) => (
                    <p key={i}>
                      <Highlight text={text} matched={hit.side === "raw" ? hit.matched : ""} />
                    </p>
                  ))}
                </div>
                <div className="flex flex-col gap-1">
                  {range(hit.out).map((text, i) => (
                    <p key={i}>
                      <Highlight text={text} matched={hit.side === "out" ? hit.matched : ""} />
                    </p>
                  ))}
                </div>
              </div>
              <Button
                type="button"
                size="xs"
                variant="link"
                onClick={() => setExpanded((list) => (open ? list.filter((id) => id !== hit.id) : [...list, hit.id]))}
              >
                {open ? "Thu gọn" : "Xem đoạn trước/sau"}
              </Button>
            </li>
          );
        })}
      </ul>
      <VnReplaceDialog
        root={root}
        setting={setting}
        hits={hits.filter((hit) => selected.includes(hit.id))}
        allHits={hits}
        open={replaceOpen}
        onOpenChange={setReplaceOpen}
        onDone={() => {
          setSelected([]);
          void reload();
        }}
      />
    </div>
  );
}
