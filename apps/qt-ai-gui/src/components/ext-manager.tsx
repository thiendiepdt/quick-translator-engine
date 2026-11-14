import { getCurrentWebview } from "@tauri-apps/api/webview";
import { FileArchive, FolderCode, KeyRound, Play, RefreshCw, Trash2 } from "lucide-react";
import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";

import { ExtConfigDialog } from "@/components/ext-config-dialog";
import { ExtPermissionDialog } from "@/components/ext-permission-dialog";
import { ExtTestDialog } from "@/components/ext-test-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  extAddDev,
  extCheckUpdate,
  extConfig,
  extInspectFile,
  extInspectUrl,
  extInstall,
  extList,
  extSetEnabled,
  extUninstall,
  pickFolder,
  pickZipFile,
  type ExtConfig,
  type ExtInspection,
  type ExtListItem,
} from "@/lib/api";

const message = (error: unknown) => (error instanceof Error ? error.message : String(error));

/** id của Card "Extension tải raw" trong Cài đặt (trang Tải raw dẫn tới đây). */
export const EXTENSIONS_CARD_ID = "extensions";

function sourceLabel(item: ExtListItem): string {
  switch (item.source.kind) {
    case "file":
      return `Cài từ file ${item.source.path}`;
    case "link":
      return `Cài từ link ${item.source.url}`;
    case "dev":
      return `Thư mục dev ${item.source.dir}`;
    case "manual":
      return "Đặt tay vào thư mục extension";
  }
}

/** Cài đặt → Extension tải raw: cài từ file zip / link, xin quyền, bật/tắt, cập nhật, auth, chạy thử, gỡ. */
export function ExtensionManager() {
  const [items, setItems] = useState<ExtListItem[]>();
  const [link, setLink] = useState("");
  const [pending, setPending] = useState<ExtInspection>();
  const [busy, setBusy] = useState(false);
  const [config, setConfig] = useState<ExtConfig>();
  const [testing, setTesting] = useState<ExtListItem>();
  const [dragging, setDragging] = useState(false);

  useEffect(() => {
    extList().then(setItems, (error: unknown) => toast.error(message(error)));
  }, []);

  const run = useCallback(async <T,>(task: () => Promise<T>): Promise<T | undefined> => {
    setBusy(true);
    try {
      return await task();
    } catch (error) {
      toast.error(message(error));
      return undefined;
    } finally {
      setBusy(false);
    }
  }, []);

  const inspectFile = useCallback(
    async (path: string) => {
      const found = await run(() => extInspectFile(path));
      if (found) setPending(found);
    },
    [run],
  );

  // Kéo thả file .zip vào cửa sổ khi đang ở trang này.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;
    let listening: ReturnType<ReturnType<typeof getCurrentWebview>["onDragDropEvent"]>;
    try {
      listening = getCurrentWebview().onDragDropEvent((event) => {
        const payload = event.payload;
        if (payload.type === "enter" || payload.type === "over") setDragging(true);
        else if (payload.type === "leave") setDragging(false);
        else if (payload.type === "drop") {
          setDragging(false);
          const zip = payload.paths.find((p) => p.toLowerCase().endsWith(".zip"));
          if (zip) void inspectFile(zip);
          else toast.error("Thả file .zip của extension.");
        }
      });
    } catch {
      return; // ngoài cửa sổ Tauri (test): không có kéo thả
    }
    listening
      .then((stop) => {
        if (disposed) stop();
        else unlisten = stop;
      })
      .catch(() => undefined);
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [inspectFile]);

  async function fromFile() {
    const path = await pickZipFile();
    if (path) await inspectFile(path);
  }

  async function fromLink() {
    const found = await run(() => extInspectUrl(link.trim()));
    if (found) setPending(found);
  }

  async function confirmInstall() {
    if (!pending) return;
    const target = pending;
    const next = await run(() => extInstall(target.sha256));
    if (!next) return;
    setItems(next);
    setPending(undefined);
    setLink("");
    toast.success(target.currentVersion ? `Đã cập nhật ${target.name} lên ${target.version}` : `Đã cài ${target.name} ${target.version}`);
  }

  async function addDev() {
    const dir = await pickFolder("Chọn thư mục extension đang viết");
    if (!dir) return;
    const next = await run(() => extAddDev(dir));
    if (next) {
      setItems(next);
      toast.success("Đã nạp thư mục dev — sửa file là lượt tải sau dùng bản mới");
    }
  }

  async function checkUpdate(item: ExtListItem) {
    const found = await run(() => extCheckUpdate(item.id));
    if (found === undefined) return;
    if (found === null) {
      toast.success(`${item.name} đang là bản mới nhất`);
    } else if (found.needsConsent) {
      setPending(found);
    } else {
      const next = await run(() => extInstall(found.sha256));
      if (next) {
        setItems(next);
        toast.success(`Đã cập nhật ${item.name} lên ${found.version}`);
      }
    }
  }

  async function uninstall(item: ExtListItem) {
    if (!window.confirm(`Gỡ ${item.name}? Xoá cả auth, cài đặt, cache, phiên đăng nhập của extension này. Raw đã tải giữ nguyên.`)) return;
    const next = await run(() => extUninstall(item.id));
    if (next) {
      setItems(next);
      toast.success(`Đã gỡ ${item.name}`);
    }
  }

  async function toggle(item: ExtListItem, enabled: boolean) {
    const next = await run(() => extSetEnabled(item.id, enabled));
    if (next) setItems(next);
  }

  async function openConfig(item: ExtListItem) {
    const found = await run(() => extConfig(item.id));
    if (found) setConfig(found);
  }

  return (
    <div className="flex flex-col gap-4">
      <div className={`flex flex-col gap-3 rounded-md border border-dashed p-3 ${dragging ? "bg-muted" : ""}`}>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" onClick={() => void fromFile()} disabled={busy}>
            <FileArchive /> Cài từ file…
          </Button>
          <Button variant="outline" onClick={() => void addDev()} disabled={busy}>
            <FolderCode /> Nạp thư mục (developer)…
          </Button>
        </div>
        <div className="flex items-end gap-2">
          <div className="flex flex-1 flex-col gap-1.5">
            <Label htmlFor="ext-link">Link gói (.zip, https)</Label>
            <Input id="ext-link" value={link} onChange={(e) => setLink(e.target.value)} placeholder="https://github.com/…/releases/download/…/extension.zip" />
          </div>
          <Button onClick={() => void fromLink()} disabled={busy || !link.trim().startsWith("https://")}>
            Cài
          </Button>
        </div>
        <p className="text-xs text-muted-foreground">Hoặc kéo thả file .zip vào đây.</p>
      </div>

      {items && items.length === 0 && <p className="text-sm text-muted-foreground">Chưa cài extension nào.</p>}
      <ul className="flex flex-col gap-3">
        {items?.map((item) => (
          <li key={item.id} aria-label={item.name} className="flex flex-col gap-2 rounded-md border p-3 text-sm">
            <div className="flex items-start gap-3">
              {item.icon ? <img src={item.icon} alt="" className="size-8 rounded" /> : <div className="size-8 rounded bg-muted" />}
              <div className="min-w-0 flex-1">
                <p className="flex flex-wrap items-center gap-2 font-medium">
                  {item.name} <span className="font-normal text-muted-foreground">{item.version}</span>
                  {item.dev && <Badge variant="outline">Dev</Badge>}
                  {!item.enabled && <Badge variant="secondary">Tắt</Badge>}
                </p>
                {item.description && <p className="text-muted-foreground">{item.description}</p>}
                {item.developer && (
                  <p className="text-xs text-muted-foreground">
                    {item.developer.url ? <a href={item.developer.url}>{item.developer.name}</a> : item.developer.name}
                    {item.developer.email ? ` · ${item.developer.email}` : ""}
                    {item.license ? ` · ${item.license}` : ""}
                    {item.homepage ? (
                      <>
                        {" · "}
                        <a href={item.homepage}>trang chủ</a>
                      </>
                    ) : null}
                  </p>
                )}
                <p className="text-xs text-muted-foreground">
                  Gọi tới {item.hosts.join(", ") || "—"}
                  {item.permissions.includes("webview") ? " · mở cửa sổ trình duyệt" : ""}
                  {item.authRequired ? " · cần auth" : ""}
                </p>
                <p className="break-all text-xs text-muted-foreground">{sourceLabel(item)}</p>
                {item.error && <p className="text-xs text-destructive">Nạp lỗi: {item.error}</p>}
              </div>
              <Switch aria-label={`Bật ${item.name}`} checked={item.enabled} disabled={busy} onCheckedChange={(checked) => void toggle(item, checked)} />
            </div>
            <div className="flex flex-wrap gap-2">
              {item.source.kind === "link" && (
                <Button variant="outline" size="sm" disabled={busy} onClick={() => void checkUpdate(item)}>
                  <RefreshCw /> Kiểm tra cập nhật
                </Button>
              )}
              {item.configurable && !item.error && (
                <Button variant="outline" size="sm" disabled={busy} onClick={() => void openConfig(item)}>
                  <KeyRound /> Auth và cài đặt…
                </Button>
              )}
              {item.dev && !item.error && (
                <Button variant="outline" size="sm" disabled={busy} onClick={() => setTesting(item)}>
                  <Play /> Chạy thử…
                </Button>
              )}
              <Button variant="ghost" size="sm" disabled={busy} onClick={() => void uninstall(item)}>
                <Trash2 /> Gỡ
              </Button>
            </div>
          </li>
        ))}
      </ul>

      <ExtPermissionDialog inspection={pending} busy={busy} onCancel={() => setPending(undefined)} onConfirm={() => void confirmInstall()} />
      {config && <ExtConfigDialog config={config} open={!!config} onOpenChange={(open) => !open && setConfig(undefined)} onChange={setConfig} />}
      {testing && <ExtTestDialog ext={testing} open={!!testing} onOpenChange={(open) => !open && setTesting(undefined)} />}
    </div>
  );
}
