import { CloudDownload, KeyRound, ListTree, LoaderCircle, LogIn, Puzzle, Square, TriangleAlert } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { ExtConfigDialog } from "@/components/ext-config-dialog";
import { EXTENSIONS_CARD_ID } from "@/components/ext-manager";
import { ExtField } from "@/components/ext-fields";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import {
  extLogin,
  onRawProgress,
  rawDownload,
  rawDownloadCancel,
  rawInfo,
  rawSave,
  rawToc,
  rescanStory,
  type RawInfo,
  type RawMode,
  type RawSummary,
  type RawToc,
  type RawTocRow,
} from "@/lib/api";
import { authStatus, changedValues, initialDisplay, storyValues, type FieldDisplay } from "@/lib/ext";
import { RAW_STATUS_LABELS, renderName, validateTemplate } from "@/lib/raw";
import { useStoryStore } from "@/store/story";

const MAX_LOG = 200;

interface Progress {
  done: number;
  total: number;
}

const message = (error: unknown) => (error instanceof Error ? error.message : String(error));

/** Trang Tải raw: link tải raw, extension khớp link, mẫu tên file, mục lục, tải chương qua extension. */
export function RawPage() {
  const root = useStoryStore((s) => s.root);
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const setPage = useStoryStore((s) => s.setPage);
  const [info, setInfo] = useState<RawInfo>();
  const [url, setUrl] = useState("");
  const [extension, setExtension] = useState("");
  const [template, setTemplate] = useState("");
  const [toc, setToc] = useState<RawToc>();
  const [anchor, setAnchor] = useState("");
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState<"toc" | "download" | "save" | "login" | undefined>();
  const [progress, setProgress] = useState<Progress>();
  const [log, setLog] = useState<string[]>([]);
  const [summary, setSummary] = useState<RawSummary>();
  const [storyDisplays, setStoryDisplays] = useState<Record<string, FieldDisplay>>({});
  const [configOpen, setConfigOpen] = useState(false);

  function applyInfo(next: RawInfo) {
    setInfo(next);
    setUrl(next.rawUrl);
    setExtension(next.extensionChoice || next.extension?.id || "");
    setTemplate(next.fileTemplate);
    setStoryDisplays({});
  }

  useEffect(() => {
    if (!root) return;
    rawInfo(root).then(applyInfo, (error: unknown) => toast.error(message(error)));
  }, [root]);

  useEffect(() => {
    const stop = onRawProgress((event) => {
      if (event.root !== root) return;
      if (event.kind === "toc") setProgress({ done: 0, total: event.total });
      if (event.kind === "log") setLog((lines) => [...lines, event.message].slice(-MAX_LOG));
      if (event.kind === "chapter") {
        setProgress({ done: event.done, total: event.total });
        const label = event.status === "failed" ? `Lỗi: ${event.message}` : event.status === "locked" ? "Khoá" : event.message;
        setLog((lines) => [...lines, `#${event.index} ${event.title} — ${label}`].slice(-MAX_LOG));
        if (event.status === "saved" || event.status === "short") {
          const file = event.message.split(" ")[0] ?? null;
          setToc((current) =>
            current && {
              ...current,
              rows: current.rows.map((row) => (row.index === event.index ? { ...row, status: "have", file } : row)),
            },
          );
        }
      }
    });
    return () => {
      void stop.then((unlisten) => unlisten());
    };
  }, [root]);

  const templateError = template.trim() ? validateTemplate(template) : null;
  const storyChanged = !!info && Object.keys(changedValues(info.storySettings, storyDisplays)).length > 0;
  const dirty =
    !!info && (url.trim() !== info.rawUrl || extension !== (info.extensionChoice || info.extension?.id || "") || template.trim() !== info.fileTemplate || storyChanged);
  const activeTemplate = template.trim() || info?.effectiveTemplate || "";
  const preview = useMemo(() => {
    if (templateError || !activeTemplate) return null;
    const next = toc?.rows.find((row) => row.status !== "have");
    return renderName(activeTemplate, { index: next?.index ?? 1, id: next?.id ?? "7445623259162542654", title: next?.title ?? "第1章 起" });
  }, [activeTemplate, templateError, toc]);

  const newRows = useMemo(() => (toc ? toc.rows.filter((row) => row.index > (toc.lastIndex ?? 0) && !row.locked && row.status !== "have") : []), [toc]);
  const anchorNumber = Number(anchor) > 0 ? Number(anchor) : undefined;

  async function save() {
    if (!root || templateError) return;
    setBusy("save");
    try {
      const storySettings = storyChanged && info?.extConfig ? { extension: info.extConfig.id, values: storyValues(info.storySettings, storyDisplays) } : undefined;
      applyInfo(await rawSave(root, { rawUrl: url, extension, fileTemplate: template, storySettings }));
      setToc(undefined);
      toast.success("Đã lưu cài đặt tải raw");
    } catch (error) {
      toast.error(message(error));
    } finally {
      setBusy(undefined);
    }
  }

  async function login() {
    if (!root || !info?.extConfig) return;
    setBusy("login");
    try {
      const result = await extLogin(root, info.extConfig.id);
      setInfo((current) => current && { ...current, extConfig: result.config });
      if (result.hasLogin) toast.success("Đã đăng nhập");
      else toast.success("Extension này không có bước đăng nhập — auth được dùng thẳng khi tải");
    } catch (error) {
      toast.error(message(error));
    } finally {
      setBusy(undefined);
    }
  }

  async function loadToc() {
    if (!root) return;
    setBusy("toc");
    setSummary(undefined);
    try {
      const view = await rawToc(root, anchorNumber);
      setToc(view);
      setSelected(new Set());
      if (view.lastIndex && !anchor) setAnchor(String(view.lastIndex));
    } catch (error) {
      toast.error(message(error));
    } finally {
      setBusy(undefined);
    }
  }

  async function run(mode: RawMode, ids: string[]) {
    if (!root) return;
    setBusy("download");
    setSummary(undefined);
    setLog([]);
    setProgress(undefined);
    try {
      const result = await rawDownload(root, mode, ids, anchorNumber);
      setSummary(result);
      setSelected(new Set());
      setSnapshot(await rescanStory(root));
      toast.success(result.cancelled ? `Đã huỷ — tải được ${result.saved.length} chương` : `Đã tải ${result.saved.length} chương`);
    } catch (error) {
      toast.error(message(error));
    } finally {
      setBusy(undefined);
    }
  }

  function openExtensions() {
    setPage("settings");
    requestAnimationFrame(() => document.getElementById(EXTENSIONS_CARD_ID)?.scrollIntoView({ block: "start" }));
  }

  function toggle(row: RawTocRow) {
    setSelected((current) => {
      const next = new Set(current);
      if (next.has(row.id)) next.delete(row.id);
      else next.add(row.id);
      return next;
    });
  }

  if (!info) {
    return <div className="p-8 text-sm text-muted-foreground">Đang đọc cài đặt tải raw…</div>;
  }

  const noExtension = info.candidates.length === 0;
  const running = busy === "download" || busy === "toc";

  return (
    <div className="fine-scrollbar h-full min-h-0 overflow-y-auto">
      <div className="mx-auto flex max-w-3xl flex-col gap-6 px-8 py-8">
        <header>
          <h1 className="text-2xl font-semibold tracking-tight">Tải raw</h1>
          <p className="text-sm text-muted-foreground">
            Tải chương raw từ trang nguồn qua extension. Chương tải về vào thư mục <code>raw/</code> theo mẫu tên file rồi vào
            hàng đợi dịch.
          </p>
        </header>

        <section className="flex flex-col gap-4 rounded-lg border bg-card p-5">
          <div className="flex flex-col gap-1.5">
            <Label htmlFor="raw-url">Link tải raw</Label>
            <Input id="raw-url" value={url} onChange={(e) => setUrl(e.target.value)} placeholder="https://…" />
            {info.rawUrlIsDefault && url.trim() === info.rawUrl && (
              <p className="text-xs text-muted-foreground">Đang theo link nguồn của truyện.</p>
            )}
          </div>

          <div className="flex flex-col gap-1.5">
            <Label>Extension</Label>
            {noExtension ? (
              <div className="rounded-md border border-dashed p-3 text-sm text-muted-foreground">
                <p>
                  Chưa có extension nào khớp link này ({info.installed} extension đang bật).
                </p>
                <Button variant="link" className="h-auto p-0" onClick={openExtensions}>
                  <Puzzle /> Cài extension ở Cài đặt → Extension tải raw
                </Button>
              </div>
            ) : info.candidates.length === 1 ? (
              <ExtensionLine ext={info.candidates[0]} />
            ) : (
              <Select value={extension} onValueChange={setExtension}>
                <SelectTrigger aria-label="Chọn extension" className="w-full">
                  <SelectValue placeholder={`Có ${info.candidates.length} extension khớp link — chọn một`} />
                </SelectTrigger>
                <SelectContent>
                  {info.candidates.map((ext) => (
                    <SelectItem key={ext.id} value={ext.id}>
                      {ext.name} {ext.version} — {ext.developer.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}
            {info.loadErrors.length > 0 && (
              <ul className="text-xs text-destructive">
                {info.loadErrors.map((line) => (
                  <li key={line}>Không nạp được {line}</li>
                ))}
              </ul>
            )}
            {info.extConfig && <AuthLine config={info.extConfig} busy={!!busy} onLogin={() => void login()} onOpen={() => setConfigOpen(true)} />}
          </div>

          {info.storySettings.length > 0 && (
            <div className="flex flex-col gap-3">
              <p className="text-sm font-medium">Cài đặt của truyện</p>
              {info.storySettings.map((field) => (
                <ExtField
                  key={field.key}
                  idPrefix="raw-story"
                  field={field}
                  value={storyDisplays[field.key] ?? initialDisplay(field)}
                  onChange={(value) => setStoryDisplays((current) => ({ ...current, [field.key]: value }))}
                />
              ))}
            </div>
          )}

          <div className="flex flex-col gap-1.5">
            <Label htmlFor="raw-template">Mẫu tên file</Label>
            <Input
              id="raw-template"
              value={template}
              onChange={(e) => setTemplate(e.target.value)}
              placeholder={info.effectiveTemplate}
              className="font-mono"
              aria-invalid={!!templateError}
            />
            {templateError ? (
              <p className="text-xs text-destructive">{templateError}</p>
            ) : (
              <p className="text-xs text-muted-foreground">
                Để trống là dùng <code>{info.effectiveTemplate}</code>
                {info.guessedTemplate && info.guessedTemplate === info.effectiveTemplate && !info.fileTemplate ? " (đoán theo tên file đang có)" : ""}.
                {preview && (
                  <>
                    {" "}
                    Chương kế tiếp: <code>{preview}.txt</code>
                  </>
                )}
              </p>
            )}
            {info.guessedTemplate && info.guessedTemplate !== activeTemplate && (
              <Button variant="outline" size="sm" className="self-start" onClick={() => setTemplate(info.guessedTemplate ?? "")}>
                Dùng mẫu theo tên cũ: <code>{info.guessedTemplate}</code>
              </Button>
            )}
          </div>

          <div className="flex justify-end">
            <Button onClick={() => void save()} disabled={!dirty || !!templateError || !!busy}>
              Lưu
            </Button>
          </div>
        </section>

        <section className="flex flex-col gap-4">
          <div className="flex flex-wrap items-center gap-2">
            <Button variant="outline" onClick={() => void loadToc()} disabled={noExtension || dirty || !!busy}>
              {busy === "toc" ? <LoaderCircle className="animate-spin" /> : <ListTree />} Tải mục lục
            </Button>
            <Button onClick={() => void run("new", [])} disabled={noExtension || dirty || !!busy || (!!toc && newRows.length === 0)}>
              <CloudDownload /> Tải chương mới{toc ? ` (${newRows.length})` : ""}
            </Button>
            <Button variant="outline" onClick={() => void run("selected", [...selected])} disabled={!!busy || selected.size === 0}>
              Tải đã chọn ({selected.size})
            </Button>
            {running && root && (
              <Button variant="ghost" onClick={() => void rawDownloadCancel(root)}>
                <Square /> Huỷ
              </Button>
            )}
            {dirty && <span className="text-xs text-muted-foreground">Lưu cài đặt trước khi tải.</span>}
          </div>

          {progress && busy === "download" && (
            <div className="flex flex-col gap-1" aria-label="Tiến độ tải">
              <div className="h-2 overflow-hidden rounded bg-muted">
                <div className="h-full bg-primary transition-all" style={{ width: `${progress.total ? (100 * progress.done) / progress.total : 0}%` }} />
              </div>
              <span className="text-xs text-muted-foreground">
                {progress.done}/{progress.total} chương
              </span>
            </div>
          )}

          {summary && <SummaryBox summary={summary} />}

          {toc && toc.mismatches.length > 0 && (
            <div role="alert" className="flex flex-col gap-2 rounded-lg border border-destructive/30 bg-destructive/10 p-4 text-sm">
              <p className="flex items-center gap-2 font-medium">
                <TriangleAlert className="size-4" /> Tiêu đề raw đang có lệch mục lục — kiểm lại chương cuối đã có trước khi tải
              </p>
              <ul className="text-xs">
                {toc.mismatches.map((m) => (
                  <li key={m.index}>
                    Mục {m.index}: <strong>{m.tocTitle}</strong> ↔ file {m.fileId}: <strong>{m.fileTitle || "(trống)"}</strong>
                  </li>
                ))}
              </ul>
              <div className="flex items-end gap-2">
                <div className="flex flex-col gap-1">
                  <Label htmlFor="raw-anchor">Chương cuối đã có là mục số</Label>
                  <Input id="raw-anchor" inputMode="numeric" className="w-28 font-mono" value={anchor} onChange={(e) => setAnchor(e.target.value)} />
                </div>
                <Button variant="outline" onClick={() => void loadToc()} disabled={!!busy}>
                  Khớp lại
                </Button>
              </div>
            </div>
          )}

          {toc && !toc.nextSortsAfter && toc.nextName && (
            <p role="alert" className="text-sm text-destructive">
              File mới <code>{toc.nextName}.txt</code> sẽ bị xếp trước file raw cuối đang có — đổi mẫu tên file cho khớp kiểu tên cũ.
            </p>
          )}

          {toc && (
            <div className="rounded-lg border">
              <p className="border-b px-4 py-2 text-sm text-muted-foreground">
                {toc.rows.length} chương · đã có {toc.rows.filter((r) => r.status === "have").length} · khoá {toc.rows.filter((r) => r.locked).length}
                {toc.by === "order" ? " · khớp chương cũ theo thứ tự" : ""}
              </p>
              <ul className="divide-y text-sm">
                {toc.rows.map((row) => (
                  <li key={row.id} aria-label={`Mục ${row.index}`} className="flex items-center gap-3 px-4 py-1.5">
                    <input type="checkbox" aria-label={`Chọn ${row.title}`} checked={selected.has(row.id)} onChange={() => toggle(row)} />
                    <span className="w-12 shrink-0 text-right font-mono text-xs text-muted-foreground">{row.index}</span>
                    <span className="min-w-0 flex-1 truncate">{row.title}</span>
                    <Badge variant={row.status === "have" ? "secondary" : row.status === "locked" ? "outline" : "default"}>{RAW_STATUS_LABELS[row.status]}</Badge>
                    <Button
                      variant="ghost"
                      size="sm"
                      aria-label={`${row.status === "have" ? "Tải lại" : "Tải"} chương ${row.index}`}
                      disabled={!!busy}
                      onClick={() => void run("single", [row.id])}
                    >
                      {row.status === "have" ? "Tải lại" : "Tải"}
                    </Button>
                  </li>
                ))}
              </ul>
            </div>
          )}

          {log.length > 0 && (
            <pre aria-label="Log tải raw" className="fine-scrollbar max-h-56 overflow-y-auto rounded-md bg-muted p-3 text-xs">
              {log.join("\n")}
            </pre>
          )}
        </section>
      </div>
      {info.extConfig && (
        <ExtConfigDialog
          config={info.extConfig}
          open={configOpen}
          onOpenChange={setConfigOpen}
          onChange={(config) => setInfo((current) => current && { ...current, extConfig: config })}
        />
      )}
    </div>
  );
}

/** Trạng thái auth của extension đang dùng + nút Đăng nhập, Auth và cài đặt. */
function AuthLine({ config, busy, onLogin, onOpen }: { config: NonNullable<RawInfo["extConfig"]>; busy: boolean; onLogin: () => void; onOpen: () => void }) {
  const missing = !!config.auth?.required && !config.auth.scheme;
  return (
    <div className="flex flex-wrap items-center gap-2 text-sm">
      <span aria-label="Trạng thái auth" className={missing ? "text-destructive" : "text-muted-foreground"}>
        {authStatus(config)}
      </span>
      {config.auth && (
        <Button variant="outline" size="sm" disabled={busy || missing} onClick={onLogin}>
          <LogIn /> Đăng nhập
        </Button>
      )}
      <Button variant="ghost" size="sm" onClick={onOpen}>
        <KeyRound /> Auth và cài đặt…
      </Button>
    </div>
  );
}

function ExtensionLine({ ext }: { ext: RawInfo["candidates"][number] }) {
  return (
    <div className="flex flex-col rounded-md border p-3 text-sm">
      <span className="font-medium">
        {ext.name} <span className="text-muted-foreground">{ext.version}</span> {ext.dev && <Badge variant="outline">Dev</Badge>}
      </span>
      <span className="text-xs text-muted-foreground">
        {ext.description} · {ext.developer.url ? <a href={ext.developer.url}>{ext.developer.name}</a> : ext.developer.name}
        {ext.license ? ` · ${ext.license}` : ""} · gọi tới {ext.hosts.join(", ")}
      </span>
    </div>
  );
}

function SummaryBox({ summary }: { summary: RawSummary }) {
  return (
    <div aria-label="Kết quả tải" className="flex flex-col gap-1 rounded-lg border p-4 text-sm">
      <p>
        {summary.cancelled ? "Đã huỷ. " : ""}Tải được <strong>{summary.saved.length}</strong> chương
        {summary.short.length > 0 && <> · {summary.short.length} chương ngắn (vẫn ghi): {summary.short.join(", ")}</>}
        {summary.locked.length > 0 && <> · khoá: {summary.locked.join(", ")}</>}
      </p>
      {summary.failed.length > 0 && (
        <ul className="text-xs text-destructive">
          {summary.failed.map((f) => (
            <li key={f.index}>
              #{f.index} {f.title}: {f.reason}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
