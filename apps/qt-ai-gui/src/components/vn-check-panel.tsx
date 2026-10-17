import { AlertTriangle, Check, LoaderCircle, RefreshCw, Sparkles, Wrench } from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
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

/** Kết quả "AI giải thích" của một chỗ: dịch nhanh (khi chưa có bản dịch), diễn giải, gợi ý xử lý. */
function AiExplanation({ ai }: { ai: NonNullable<VnHitView["ai"]> }) {
  return (
    <div className="mt-2 flex flex-col gap-1.5 rounded-md border bg-muted/40 p-2.5 text-sm" aria-label="Giải thích của AI">
      {ai.translation && (
        <div>
          <p className="text-xs font-medium text-muted-foreground">Dịch nhanh</p>
          {ai.translation.split("\n").map((line, i) => (
            <p key={i}>{line}</p>
          ))}
        </div>
      )}
      <div>
        <p className="text-xs font-medium text-muted-foreground">Diễn giải</p>
        <p>{ai.reason}</p>
      </div>
      {ai.suggestion && (
        <div>
          <p className="text-xs font-medium text-muted-foreground">Gợi ý</p>
          <p>{ai.suggestion}</p>
        </div>
      )}
      <p className="text-xs text-muted-foreground">{ai.model}</p>
    </div>
  );
}

/** Tab "Kiểm tra VN": chỗ nhạy cảm về Việt Nam trong raw/bản dịch, xác minh, AI giải thích từng chỗ, thay thế. */
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
  // Chỗ đang chờ AI giải thích (mỗi block một nút, bấm được nhiều chỗ cùng lúc).
  const [explaining, setExplaining] = useState<string[]>([]);
  const [scanning, setScanning] = useState(false);
  const [scanError, setScanError] = useState<string>();
  const [replaceOpen, setReplaceOpen] = useState(false);
  const mounted = useRef(true);
  const loadSeq = useRef(0);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  // Chỉ phản hồi của lần tải mới nhất được áp dụng; chọn/mở rộng cắt về những id còn tồn tại.
  const reload = useCallback(async () => {
    const seq = ++loadSeq.current;
    const next = await vnReport(root);
    if (!mounted.current || seq !== loadSeq.current) return;
    const ids = new Set(next.map((hit) => hit.id));
    setHits(next);
    setSelected((list) => (list.every((id) => ids.has(id)) ? list : list.filter((id) => ids.has(id))));
    setExpanded((list) => (list.every((id) => ids.has(id)) ? list : list.filter((id) => ids.has(id))));
  }, [root]);

  const scan = useCallback(async () => {
    await vnRefresh(root);
    await reload();
  }, [root, reload]);

  useEffect(() => {
    void scan().catch((error) => {
      if (mounted.current) setScanError(error instanceof Error ? error.message : "Không quét được");
    });
  }, [scan]);

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

  const selectedSet = useMemo(() => new Set(selected), [selected]);
  const expandedSet = useMemo(() => new Set(expanded), [expanded]);
  // Chỉ những chỗ vừa được chọn vừa đang hiện mới được gửi đi (chỗ bị lọc ẩn thì không).
  const chosen = useMemo(() => shown.filter((hit) => selectedSet.has(hit.id)), [shown, selectedSet]);
  const allShownSelected = shown.length > 0 && chosen.length === shown.length;

  async function mark(ids: string[], next: VnStatus) {
    try {
      await vnSetStatus(root, ids, next);
      await reload();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không đặt được trạng thái");
    }
  }

  async function retry() {
    setScanError(undefined);
    try {
      await scan();
    } catch (error) {
      if (mounted.current) setScanError(error instanceof Error ? error.message : "Không quét được");
    }
  }

  async function rescan() {
    setScanning(true);
    try {
      await scan();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không quét được");
    } finally {
      if (mounted.current) setScanning(false);
    }
  }

  /** AI dịch nhanh + diễn giải + gợi ý cho một chỗ; kết quả lưu ở vn-check.json nên mở lại vẫn thấy. */
  async function explain(id: string) {
    setExplaining((list) => [...list, id]);
    try {
      const outcome = await vnAiVerify(root, [id]);
      if (outcome.error) toast.error(`AI giải thích: ${outcome.error}`);
      else if (outcome.verified === 0) toast.error("AI không trả kết quả hợp lệ — thử lại");
      await reload();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "AI giải thích thất bại");
    } finally {
      if (mounted.current) setExplaining((list) => list.filter((item) => item !== id));
    }
  }

  if (scanError && !hits) {
    return (
      <div className="flex items-center gap-2 text-sm">
        <p className="text-destructive">Không quét được: {scanError}</p>
        <Button type="button" size="sm" variant="secondary" onClick={() => void retry()}>
          Thử lại
        </Button>
      </div>
    );
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
        <Button type="button" size="sm" variant="ghost" disabled={scanning} onClick={() => void rescan()}>
          <RefreshCw /> Quét lại
        </Button>
        <Button type="button" size="sm" disabled={running || chosen.length === 0} onClick={() => setReplaceOpen(true)}>
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
          const open = expandedSet.has(hit.id);
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
                  checked={selectedSet.has(hit.id)}
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
                  variant="ghost"
                  disabled={running || explaining.includes(hit.id)}
                  title={running ? "Đang có phiên dịch chạy — dừng phiên trước" : "AI dịch nhanh đoạn này, giải thích có nhạy cảm không và gợi ý cách xử lý"}
                  aria-label={`AI giải thích ${hit.id}`}
                  onClick={() => void explain(hit.id)}
                >
                  {explaining.includes(hit.id) ? <LoaderCircle className="animate-spin" /> : <Sparkles />}
                  {explaining.includes(hit.id) ? "AI đang xem…" : hit.ai ? "Giải thích lại" : "AI giải thích"}
                </Button>
                <Button
                  type="button"
                  size="xs"
                  variant="link"
                  aria-label={`Mở chương ${hit.chapter}`}
                  onClick={() => {
                    select(hit.chapter);
                    setPage("translate");
                  }}
                >
                  Mở chương
                </Button>
                <Button
                  type="button"
                  size="xs"
                  variant={hit.status === "ok" ? "secondary" : "ghost"}
                  aria-label={`Không sao ${hit.id}`}
                  aria-pressed={hit.status === "ok"}
                  onClick={() => void mark([hit.id], "ok")}
                >
                  <Check /> Không sao
                </Button>
                <Button
                  type="button"
                  size="xs"
                  variant={hit.status === "action" ? "destructive" : "ghost"}
                  aria-label={`Cần xử lý ${hit.id}`}
                  aria-pressed={hit.status === "action"}
                  onClick={() => void mark([hit.id], "action")}
                >
                  Cần xử lý
                </Button>
              </div>
              {hit.ai && <AiExplanation ai={hit.ai} />}
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
                aria-label={`${open ? "Thu gọn" : "Xem đoạn trước/sau"} ${hit.id}`}
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
        hits={chosen}
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
