import {
  AlertTriangle,
  ChevronLeft,
  ChevronRight,
  Copy,
  FolderOpen,
  ListOrdered,
  ListPlus,
  Pencil,
  RotateCcw,
  Save,
  ShieldCheck,
  SkipForward,
  Trash2,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { toast } from "sonner";

import { LogPanel } from "@/components/log-panel";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useLineNumbers, useReadingWidth } from "@/hooks/use-reading-width";
import {
  chapterForceAccept,
  chapterPad,
  chapterRetry,
  chaptersDelete,
  chapterSkip,
  readChapter,
  saveChapterOutput,
  revealFolder,
  storySnapshot,
} from "@/lib/api";
import { describeDelete } from "@/lib/chapters";
import { copyText } from "@/lib/clipboard";
import {
  countChars,
  countWords,
  hasHan,
  isReadingWidth,
  numberLines,
  READING_WIDTH_LABELS,
  READING_WIDTHS,
  splitHan,
} from "@/lib/reading";
import { STATUS_LABELS, type ChapterRow, type ChapterStatus, type ChapterView } from "@/lib/types";
import { cn } from "@/lib/utils";
import { selectCurrentRunning, selectCurrentShort, useStoryStore } from "@/store/story";

const TAB_LABELS = { output: "bản dịch", draft: "nháp", review: "yêu cầu sửa", raw: "bản gốc" } as const;

const COUNT_FORMAT = new Intl.NumberFormat("vi-VN");
const RATIO_FORMAT = new Intl.NumberFormat("vi-VN", { minimumFractionDigits: 2, maximumFractionDigits: 2 });

/**
 * Độ dài hai bản cạnh thanh tab. Gốc tiếng Trung đếm theo ký tự (mỗi chữ Hán một chữ); bản dịch đếm theo chữ
 * (tiếng). Tooltip thêm số ký tự bản dịch và tỉ lệ ký tự dịch/gốc — con số harness so với ngưỡng "quá ngắn".
 */
function LengthStats({ raw, output }: { raw: string; output: string | null | undefined }) {
  const rawChars = useMemo(() => countChars(raw), [raw]);
  const translated = useMemo(
    () => (output == null ? undefined : { words: countWords(output), chars: countChars(output) }),
    [output],
  );
  const detail = [
    `Gốc: ${COUNT_FORMAT.format(rawChars)} ký tự, không tính khoảng trắng.`,
    translated
      ? `Bản dịch: ${COUNT_FORMAT.format(translated.words)} chữ (${COUNT_FORMAT.format(translated.chars)} ký tự).`
      : "",
    translated && rawChars > 0 ? `Tỉ lệ ký tự dịch/gốc: ${RATIO_FORMAT.format(translated.chars / rawChars)}.` : "",
  ]
    .filter(Boolean)
    .join(" ");
  return (
    <p className="text-xs text-muted-foreground tabular-nums" title={detail} aria-label="Độ dài chương">
      Gốc {COUNT_FORMAT.format(rawChars)} chữ
      {translated && <> · Dịch {COUNT_FORMAT.format(translated.words)} chữ</>}
    </p>
  );
}

/**
 * Văn bản vẽ theo từng dòng. `numbers`: đánh số đoạn ở lề trái — số nằm ngoài vùng chọn (không dính vào khi bôi
 * đen chép) và trùng nhãn [[n]] của harness, nên cảnh báo "[[12]] …" chỉ đúng dòng số 12 ở cả Bản dịch lẫn Gốc.
 * `flagHan` (bản dịch): dòng còn sót chữ Hán được tô nền, riêng các chữ Hán tô đậm hơn, kèm thanh báo liệt kê
 * số dòng ở đầu để nhảy tới — chương dài không phải cuộn dò.
 */
function ReadingText({ text, numbers, flagHan }: { text: string; numbers: boolean; flagHan: boolean }) {
  const lines = useMemo(() => numberLines(text), [text]);
  const body = useRef<HTMLDivElement>(null);
  const flagged = flagHan ? lines.filter((line) => line.n !== undefined && hasHan(line.text)) : [];
  return (
    <>
      {flagged.length > 0 && (
        <div
          role="alert"
          className="mb-4 flex flex-wrap items-center gap-1.5 rounded-md border border-status-error/40 bg-status-error/10 px-3 py-2 font-sans text-sm text-status-error"
        >
          <AlertTriangle className="size-4 shrink-0" aria-hidden />
          <span>{flagged.length} dòng còn sót chữ Hán:</span>
          {flagged.map((line) => (
            <button
              key={line.n}
              type="button"
              aria-label={`Tới dòng ${line.n}`}
              onClick={() => body.current?.querySelector(`[data-line="${line.n}"]`)?.scrollIntoView?.({ block: "center" })}
              className="rounded border border-status-error/40 px-1.5 font-mono text-xs tabular-nums hover:bg-status-error/15"
            >
              {line.n}
            </button>
          ))}
        </div>
      )}
      <div
        ref={body}
        // items-baseline: số ở lề (cỡ chữ nhỏ hơn) nằm cùng đường chân chữ với dòng đầu của đoạn.
        className={cn("grid items-baseline gap-x-4", numbers ? "grid-cols-[auto_minmax(0,1fr)]" : "grid-cols-1")}
        data-testid="reading-text"
      >
        {lines.map((line, index) => {
          const han = flagHan && hasHan(line.text);
          return (
            // Dòng trống giữ nguyên để khoảng cách đoạn giống hệt bản <pre>.
            <div key={index} className="contents">
              {numbers && (
                <span
                  aria-hidden
                  className={cn(
                    "text-right font-mono text-[0.7em] tabular-nums select-none",
                    han ? "font-semibold text-status-error" : "text-muted-foreground",
                  )}
                >
                  {line.n ?? ""}
                </span>
              )}
              <span
                data-line={line.n}
                data-han={han || undefined}
                className={cn("whitespace-pre-wrap", han && "rounded bg-status-error/10 outline outline-status-error/30")}
              >
                {line.text === "" ? (
                  <br />
                ) : han ? (
                  splitHan(line.text).map((part, at) =>
                    part.han ? (
                      <mark key={at} className="rounded bg-status-error/25 px-0.5 font-semibold text-status-error">
                        {part.text}
                      </mark>
                    ) : (
                      part.text
                    ),
                  )
                ) : (
                  line.text
                )}
              </span>
            </div>
          );
        })}
      </div>
    </>
  );
}

const STATUS_TONE: Record<ChapterStatus, string> = {
  queued: "bg-status-queued/30 text-foreground",
  translating: "bg-status-translating/15 text-status-translating",
  done: "bg-status-done/15 text-status-done",
  error: "bg-status-error/15 text-status-error",
  skipped: "bg-muted text-muted-foreground",
};

interface Props {
  root: string;
  row: ChapterRow;
  /** Số thứ tự 1-based trong danh sách đầy đủ, hiện cạnh mã. */
  ordinal?: number;
  hasPrev: boolean;
  hasNext: boolean;
  onPrev: () => void;
  onNext: () => void;
}

export function ChapterReader({ root, row, ordinal, hasPrev, hasNext, onPrev, onNext }: Props) {
  const running = useStoryStore(selectCurrentRunning);
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const threshold = useStoryStore(selectCurrentShort).threshold;
  const refreshShort = useStoryStore((s) => s.refreshShort);
  const [padding, setPadding] = useState(false);
  const { width, setWidth } = useReadingWidth();
  const { lineNumbers, setLineNumbers } = useLineNumbers();
  // View gắn key theo (chương, trạng thái, vòng soát): đổi chương → key lệch → hiện "Đang đọc…"
  // mà không cần reset state đồng bộ trong effect.
  // Số chữ khi còn là chương ngắn: "Chèn cho cả N chương" trên toolbar đổi out/ → key đổi → đọc lại bản đã chèn.
  const shortWords = useStoryStore((s) => selectCurrentShort(s).words.get(row.id));
  const viewKey = `${root}|${row.id}|${row.status}|${row.reviewRound}|${shortWords ?? "ok"}`;
  const [loaded, setLoaded] = useState<{ key: string; view: ChapterView } | undefined>();
  const view = loaded?.key === viewKey ? loaded.view : undefined;
  const [skipOpen, setSkipOpen] = useState(false);
  const [retryOpen, setRetryOpen] = useState(false);
  const [deleteOpen, setDeleteOpen] = useState(false);
  // Sửa tay bản dịch: mặc định chỉ đọc; trạng thái sửa gắn key view để đổi chương là thoát chế độ sửa.
  // Textarea KHÔNG controlled: React set .value mỗi phím làm WebKitGTK xoá undo stack → Ctrl+Z chết.
  // Nội dung sống trong DOM (ref), state chỉ giữ cờ "đã đổi" để bật nút Lưu.
  const [edit, setEdit] = useState<{ key: string; dirty: boolean } | undefined>();
  const editing = edit?.key === viewKey ? edit : undefined;
  const editBox = useRef<HTMLTextAreaElement>(null);
  const [saving, setSaving] = useState(false);
  const [reason, setReason] = useState("");

  useEffect(() => {
    let cancelled = false;
    readChapter(root, row.id)
      .then((v) => {
        if (!cancelled) setLoaded({ key: viewKey, view: v });
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : "Không đọc được chương"));
    return () => {
      cancelled = true;
    };
  }, [root, row.id, viewKey]);

  /** Chạy hành động rồi nạp lại snapshot; action trả chuỗi thì dùng làm thông báo thay cho `label`. */
  async function act(label: string, action: () => Promise<unknown>) {
    try {
      const message = await action();
      setSnapshot(await storySnapshot(root));
      void useStoryStore.getState().refreshProgressDrift(root);
      toast.success(typeof message === "string" ? message : label);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : `${label} thất bại`);
    }
  }

  // Chương nào cũng dịch lại được trừ chương đang chờ sẵn; chương done thì hỏi trước vì mất bản ở out/.
  async function copyTab(text: string) {
    try {
      await copyText(text);
      toast.success("Đã chép vào clipboard");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không chép được");
    }
  }

  /** Chèn văn mẫu (hoặc lặp nội dung, theo Cài đặt) sau dòng ❖❖❖ cho đủ ngưỡng chữ của nền tảng đăng truyện. */
  async function pad() {
    setPadding(true);
    try {
      const changed = await chapterPad(root, row.id);
      setLoaded({ key: viewKey, view: await readChapter(root, row.id) });
      void refreshShort(root);
      toast.success(changed ? `Đã chèn cho đủ ${threshold} chữ — phần chèn nằm sau dòng ❖❖❖` : "Chương đã đủ chữ, không chèn gì");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không chèn được");
    } finally {
      setPadding(false);
    }
  }

  async function saveEdit() {
    const text = editBox.current?.value;
    if (!editing || text == null) return;
    setSaving(true);
    try {
      await saveChapterOutput(root, row.id, text);
      const fresh = await readChapter(root, row.id);
      setLoaded({ key: viewKey, view: fresh });
      setEdit(undefined);
      toast.success("Đã lưu bản dịch");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được bản dịch");
    } finally {
      setSaving(false);
    }
  }

  const canRetry = row.status !== "queued";
  const retry = () => void act("Đã đưa về hàng đợi", () => chapterRetry(root, row.id));
  const canSkip = row.status !== "done";
  const canForce = row.status === "translating" && Boolean(view?.draft);
  const defaultTab = view?.output ? "output" : view?.draft ? "draft" : "raw";

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-wrap items-center gap-2 border-b px-5 py-3">
        {/* Chuyển chương luôn nằm trên đầu, không phải cuộn xuống cuối bài. */}
        <div className="flex items-center gap-0.5">
          <Button size="icon-sm" variant="ghost" aria-label="Chương trước" title="Chương trước" disabled={!hasPrev} onClick={onPrev}>
            <ChevronLeft />
          </Button>
          <Button size="icon-sm" variant="ghost" aria-label="Chương sau" title="Chương sau" disabled={!hasNext} onClick={onNext}>
            <ChevronRight />
          </Button>
        </div>
        {ordinal !== undefined && <span className="font-mono text-xs text-muted-foreground tabular-nums">#{ordinal}</span>}
        <span className="font-mono text-sm font-medium">{row.id}</span>
        <span className={cn("rounded-full px-2 py-0.5 text-xs font-medium", STATUS_TONE[row.status])}>
          {STATUS_LABELS[row.status]}
        </span>
        {row.reason && <span className="truncate text-xs text-muted-foreground">{row.reason}</span>}
        <div className="flex-1" />
        <Button
          size="sm"
          variant="outline"
          disabled={!canRetry || running}
          title={row.status === "done" ? "Dịch lại từ đầu; bản hiện có được giữ thành out/<id>.txt.bak" : undefined}
          onClick={() => (row.status === "done" ? setRetryOpen(true) : retry())}
        >
          <RotateCcw /> Dịch lại
        </Button>
        <Button size="sm" variant="outline" disabled={!canSkip || running} onClick={() => setSkipOpen(true)}>
          <SkipForward /> Bỏ qua
        </Button>
        <Button
          size="sm"
          variant="outline"
          disabled={!canForce || running}
          title="Chốt bản nháp hiện có dù check chưa đạt"
          onClick={() => void act("Đã chốt (force)", () => chapterForceAccept(root, row.id))}
        >
          <ShieldCheck /> Chốt --force
        </Button>
        <Button
          size="sm"
          variant="ghost"
          className="text-destructive hover:text-destructive"
          disabled={running}
          title="Xoá hẳn chương: xoá raw/ + work/, gỡ khỏi danh sách; bản dịch trong out/ giữ nguyên"
          onClick={() => setDeleteOpen(true)}
        >
          <Trash2 /> Xoá chương
        </Button>
        <Button size="sm" variant="ghost" onClick={() => void revealFolder(root)}>
          <FolderOpen /> Mở folder
        </Button>
      </div>
      {row.warnings.length > 0 && (
        <div className="border-b bg-status-warning/10 px-5 py-2 text-xs">
          <div className="mb-1 flex items-center gap-1.5 font-medium text-status-warning">
            <AlertTriangle className="size-3.5" /> {row.warnings.length} cảnh báo còn lại
          </div>
          <ul className="list-disc space-y-0.5 pl-5 text-foreground/80">
            {row.warnings.map((w) => (
              <li key={w}>{w}</li>
            ))}
          </ul>
        </div>
      )}
      {/* key đổi khi view tải xong để defaultValue tính lại (Bản dịch nếu có, không thì Gốc) */}
      <Tabs
        key={`${viewKey}|${view ? "loaded" : "loading"}`}
        defaultValue={defaultTab}
        className="flex min-h-0 flex-1 flex-col gap-0"
      >
        <div className="mx-5 mt-2 flex items-end justify-between gap-2 border-b">
          <TabsList variant="line" className="border-b-0">
            <TabsTrigger value="output" disabled={!view?.output}>
              Bản dịch
            </TabsTrigger>
            <TabsTrigger value="draft" disabled={!view?.draft}>
              Nháp
            </TabsTrigger>
            <TabsTrigger value="review" disabled={!view?.review}>
              Yêu cầu sửa
            </TabsTrigger>
            <TabsTrigger value="raw">Gốc</TabsTrigger>
            <TabsTrigger value="log">Log</TabsTrigger>
          </TabsList>
          <div className="mb-1 flex items-center gap-2">
            {view && !view.rawMissing && <LengthStats raw={view.raw} output={view.output} />}
            <Button
              size="sm"
              variant={lineNumbers ? "secondary" : "ghost"}
              aria-pressed={lineNumbers}
              title="Đánh số từng đoạn ở tab Bản dịch và Gốc — số trùng với nhãn [[n]] trong cảnh báo và yêu cầu sửa"
              onClick={() => void setLineNumbers(!lineNumbers)}
            >
              <ListOrdered /> Số dòng
            </Button>
            <Select value={width} onValueChange={(v) => isReadingWidth(v) && void setWidth(v)}>
              <SelectTrigger size="sm" className="w-32" aria-label="Chiều ngang văn bản">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {READING_WIDTHS.map((item) => (
                  <SelectItem key={item} value={item}>
                    {READING_WIDTH_LABELS[item]}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
        {(["output", "draft", "review", "raw"] as const).map((key) => (
          <TabsContent key={key} value={key} className="fine-scrollbar min-h-0 flex-1 overflow-y-auto">
            <article className="reading mx-auto px-6 py-8" data-width={width}>
              {view && (
                <div className="mb-4 flex items-center justify-end gap-2 font-sans text-sm">
                  {key === "output" && !editing && threshold > 0 && view.outputWords != null && view.outputWords < threshold && (
                    <>
                      <span className="mr-auto text-xs text-status-warning">
                        Chương ngắn: {view.outputWords}/{threshold} chữ — nền tảng đăng truyện có thể không nhận
                      </span>
                      <Button
                        variant="outline"
                        size="sm"
                        disabled={padding}
                        title="Chèn văn mẫu (hoặc lặp nội dung chương, theo Cài đặt) sau dòng ❖❖❖ cho đủ chữ"
                        onClick={() => void pad()}
                      >
                        <ListPlus /> Chèn cho đủ {threshold} chữ
                      </Button>
                    </>
                  )}
                  <Button
                    variant="ghost"
                    size="sm"
                    title="Chép toàn bộ nội dung tab này"
                    aria-label={`Chép ${TAB_LABELS[key]}`}
                    onClick={() => void copyTab(key === "raw" ? view.raw : (view[key] ?? ""))}
                  >
                    <Copy /> Chép
                  </Button>
                  {key === "output" && view.output != null && (editing ? (
                    <>
                      <Button variant="outline" size="sm" disabled={saving} onClick={() => setEdit(undefined)}>
                        Huỷ
                      </Button>
                      <Button size="sm" disabled={saving || !editing.dirty} onClick={() => void saveEdit()}>
                        <Save /> {saving ? "Đang lưu…" : "Lưu"}
                      </Button>
                    </>
                  ) : (
                    <Button
                      variant="outline"
                      size="sm"
                      disabled={running}
                      title="Sửa tay bản dịch, ghi đè out/<id>.txt"
                      onClick={() => setEdit({ key: viewKey, dirty: false })}
                    >
                      <Pencil /> Sửa
                    </Button>
                  ))}
                </div>
              )}
              {key === "output" && editing ? (
                <Textarea
                  ref={editBox}
                  aria-label="Bản dịch đang sửa"
                  defaultValue={view?.output ?? ""}
                  onChange={(e) => setEdit({ key: viewKey, dirty: e.target.value !== (view?.output ?? "") })}
                  onKeyDown={(e) => {
                    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
                      e.preventDefault();
                      if (editing.dirty && !saving) void saveEdit();
                    }
                  }}
                  disabled={saving}
                  className="min-h-[60vh] resize-y font-[inherit] text-[length:inherit] leading-[inherit]"
                />
              ) : view && key === "output" ? (
                // Bản dịch luôn vẽ theo dòng để tô được dòng sót chữ Hán, bật hay tắt số dòng cũng vậy.
                <ReadingText text={view.output ?? ""} numbers={lineNumbers} flagHan />
              ) : view && key === "raw" && lineNumbers && !view.rawMissing ? (
                <ReadingText text={view.raw} numbers flagHan={false} />
              ) : (
                <pre className="font-[inherit] whitespace-pre-wrap">
                  {view
                    ? key === "raw"
                      ? view.rawMissing
                        ? `File raw/${row.id}.txt đã mất — bấm Quét lại để gỡ chương khỏi danh sách (bản dịch trong out/ giữ nguyên).`
                        : view.raw
                      : (view[key] ?? "")
                    : "Đang đọc…"}
                </pre>
              )}
              <nav className="mt-10 flex items-center justify-between border-t pt-4 font-sans text-sm">
                <Button variant="ghost" size="sm" disabled={!hasPrev} onClick={onPrev}>
                  <ChevronLeft /> Chương trước
                </Button>
                <Button variant="ghost" size="sm" disabled={!hasNext} onClick={onNext}>
                  Chương sau <ChevronRight />
                </Button>
              </nav>
            </article>
          </TabsContent>
        ))}
        <TabsContent value="log" className="min-h-0 flex-1">
          <LogPanel />
        </TabsContent>
      </Tabs>
      <Dialog open={retryOpen} onOpenChange={setRetryOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Dịch lại chương {row.id}?</DialogTitle>
            <DialogDescription>
              Chương này đã dịch xong. Bản hiện có sẽ được đổi tên thành out/{row.id}.txt.bak (đè bản .bak cũ nếu
              có), chương về hàng đợi và dịch lại từ đầu.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setRetryOpen(false)}>
              Huỷ
            </Button>
            <Button
              onClick={() => {
                setRetryOpen(false);
                retry();
              }}
            >
              Dịch lại
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <Dialog open={skipOpen} onOpenChange={setSkipOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Bỏ qua chương {row.id}</DialogTitle>
          </DialogHeader>
          <Input
            value={reason}
            onChange={(e) => setReason(e.target.value)}
            placeholder="Lý do (vd. model từ chối nội dung)"
          />
          <DialogFooter>
            <Button variant="outline" onClick={() => setSkipOpen(false)}>
              Huỷ
            </Button>
            <Button
              disabled={!reason.trim()}
              onClick={() => {
                setSkipOpen(false);
                void act("Đã bỏ qua", () => chapterSkip(root, row.id, reason));
                setReason("");
              }}
            >
              Bỏ qua
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
      <Dialog open={deleteOpen} onOpenChange={setDeleteOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>
              Xoá chương {ordinal !== undefined ? `#${ordinal} ` : ""}
              {row.id}?
            </DialogTitle>
            <DialogDescription>
              File raw/{row.id}.txt và nháp trong work/ sẽ bị xoá, chương biến khỏi danh sách; không hoàn tác được.
              {row.status === "done" ? ` Bản dịch out/${row.id}.txt giữ nguyên trên đĩa.` : ""}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteOpen(false)}>
              Huỷ
            </Button>
            <Button
              variant="destructive"
              onClick={() => {
                setDeleteOpen(false);
                void act("Đã xoá chương", async () => {
                  const outcome = await chaptersDelete(root, [row.id]);
                  return describeDelete(outcome.removed.length, outcome.keptOutputs.length);
                });
              }}
            >
              <Trash2 /> Xoá
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
