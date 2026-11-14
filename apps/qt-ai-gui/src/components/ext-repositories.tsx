import { KeyRound, LoaderCircle, Plus, RefreshCw, Trash2 } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { toast } from "sonner";

import { AuthForm } from "@/components/ext-config-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  repoAdd,
  repoAuth,
  repoClearAuth,
  repoList,
  repoRefresh,
  repoRemove,
  repoSaveAuth,
  type ExtListItem,
  type RepoAuth,
  type RepoEntry,
  type RepoView,
} from "@/lib/api";
import { compareVersions } from "@/lib/ext";

const message = (error: unknown) => (error instanceof Error ? error.message : String(error));

export interface RepoTarget {
  repoId: string;
  entry: RepoEntry;
}

interface Props {
  installed: ExtListItem[];
  busy: boolean;
  onInstall: (target: RepoTarget) => Promise<void>;
  onUpdateAll: (targets: RepoTarget[]) => Promise<void>;
}

type Status = "missing" | "have" | "update";

function formatTime(seconds: number): string {
  if (!seconds) return "chưa";
  const d = new Date(seconds * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${pad(d.getHours())}:${pad(d.getMinutes())} ${pad(d.getDate())}/${pad(d.getMonth() + 1)}`;
}

/** Kho extension: thêm kho bằng link, làm mới, API key chung, gỡ; danh sách extension có trong kho để cài / cập nhật. */
export function ExtRepositories({ installed, busy, onInstall, onUpdateAll }: Props) {
  const [repos, setRepos] = useState<RepoView[]>();
  const [url, setUrl] = useState("");
  const [filter, setFilter] = useState("");
  const [working, setWorking] = useState(false);
  const [auth, setAuth] = useState<RepoAuth>();
  const [authVersion, setAuthVersion] = useState(0);

  useEffect(() => {
    repoList().then(setRepos, (error: unknown) => toast.error(message(error)));
  }, []);

  async function run<T>(task: () => Promise<T>): Promise<T | undefined> {
    setWorking(true);
    try {
      return await task();
    } catch (error) {
      toast.error(message(error));
      return undefined;
    } finally {
      setWorking(false);
    }
  }

  async function add() {
    const next = await run(() => repoAdd(url.trim()));
    if (!next) return;
    setRepos(next);
    setUrl("");
    toast.success("Đã thêm kho");
  }

  async function refresh(id?: string) {
    const result = await run(() => repoRefresh(id));
    if (!result) return;
    setRepos(result.repositories);
    if (result.errors.length) result.errors.forEach((e) => toast.error(`Không làm mới được ${e}`));
    else toast.success(id ? "Đã làm mới kho" : "Đã làm mới mọi kho");
  }

  async function remove(repo: RepoView) {
    if (!window.confirm(`Gỡ kho ${repo.name}? Xoá cả API key chung của kho. Extension đã cài từ kho giữ nguyên nhưng không còn cập nhật theo kho.`)) return;
    const next = await run(() => repoRemove(repo.id));
    if (next) setRepos(next);
  }

  async function openAuth(repo: RepoView) {
    const found = await run(() => repoAuth(repo.id));
    if (found) setAuth(found);
  }

  async function saveAuth(task: () => Promise<RepoAuth>, done: string) {
    const next = await run(task);
    if (!next) return;
    setAuth(next);
    setAuthVersion((v) => v + 1);
    toast.success(done);
  }

  const rows = useMemo(() => {
    const needle = filter.trim().toLowerCase();
    return (repos ?? []).flatMap((repo) =>
      repo.extensions
        .filter((e) => !needle || [e.name, e.id, e.description, ...e.hosts].some((t) => t.toLowerCase().includes(needle)))
        .map((entry) => {
          const have = installed.find((i) => i.id === entry.id);
          const status: Status = !have ? "missing" : compareVersions(entry.version, have.version) > 0 ? "update" : "have";
          return { repo, entry, have, status };
        }),
    );
  }, [repos, installed, filter]);
  const updates = rows.filter((r) => r.status === "update");
  const disabled = busy || working;

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-end gap-2">
        <div className="flex flex-1 flex-col gap-1.5">
          <Label htmlFor="repo-url">Link kho (index.json, https)</Label>
          <Input id="repo-url" value={url} onChange={(e) => setUrl(e.target.value)} placeholder="https://…/index.json" />
        </div>
        <Button variant="outline" onClick={() => void add()} disabled={disabled || !url.trim().startsWith("https://")}>
          {working ? <LoaderCircle className="animate-spin" /> : <Plus />} Thêm kho
        </Button>
      </div>

      {repos && repos.length === 0 && <p className="text-sm text-muted-foreground">Chưa thêm kho nào.</p>}
      <ul className="flex flex-col gap-2">
        {repos?.map((repo) => (
          <li key={repo.id} aria-label={`Kho ${repo.name}`} className="flex flex-col gap-1 rounded-md border p-3 text-sm">
            <div className="flex flex-wrap items-center gap-2">
              <span className="font-medium">{repo.name}</span>
              <span className="text-xs text-muted-foreground">
                {repo.extensions.length} extension · {repo.maintainer.url ? <a href={repo.maintainer.url}>{repo.maintainer.name}</a> : repo.maintainer.name} · làm mới {formatTime(repo.refreshedAt)}
              </span>
            </div>
            {repo.description && <p className="text-muted-foreground">{repo.description}</p>}
            <p className="break-all text-xs text-muted-foreground">{repo.url}</p>
            {repo.error && <p className="text-xs text-destructive">{repo.error}</p>}
            {repo.warnings.length > 0 && (
              <ul className="text-xs text-destructive">
                {repo.warnings.map((w) => (
                  <li key={w}>{w}</li>
                ))}
              </ul>
            )}
            <div className="flex flex-wrap gap-2">
              <Button variant="outline" size="sm" disabled={disabled} onClick={() => void refresh(repo.id)}>
                <RefreshCw /> Làm mới
              </Button>
              {repo.hasAuth && (
                <Button variant="outline" size="sm" disabled={disabled} onClick={() => void openAuth(repo)}>
                  <KeyRound /> API key của kho…
                </Button>
              )}
              <Button variant="ghost" size="sm" disabled={disabled} onClick={() => void remove(repo)}>
                <Trash2 /> Gỡ kho
              </Button>
            </div>
          </li>
        ))}
      </ul>

      {rows.length > 0 || filter ? (
        <section aria-label="Có trong kho" className="flex flex-col gap-2">
          <div className="flex flex-wrap items-center gap-2">
            <p className="text-sm font-medium">Có trong kho</p>
            <Input aria-label="Lọc extension trong kho" value={filter} onChange={(e) => setFilter(e.target.value)} placeholder="Lọc theo tên, domain…" className="h-8 max-w-56" />
            {repos && repos.length > 1 && (
              <Button variant="outline" size="sm" disabled={disabled} onClick={() => void refresh()}>
                <RefreshCw /> Làm mới mọi kho
              </Button>
            )}
            {updates.length > 0 && (
              <Button size="sm" disabled={disabled} onClick={() => void onUpdateAll(updates.map((r) => ({ repoId: r.repo.id, entry: r.entry })))}>
                Cập nhật hết ({updates.length})
              </Button>
            )}
          </div>
          <ul className="flex flex-col gap-2">
            {rows.map(({ repo, entry, have, status }) => (
              <li key={`${repo.id}/${entry.id}`} aria-label={`Trong kho: ${entry.name}`} className="flex items-start gap-3 rounded-md border p-3 text-sm">
                {entry.icon ? <img src={entry.icon} alt="" className="size-8 rounded" /> : <div className="size-8 rounded bg-muted" />}
                <div className="min-w-0 flex-1">
                  <p className="flex flex-wrap items-center gap-2 font-medium">
                    {entry.name} <span className="font-normal text-muted-foreground">{entry.version}</span>
                    {status === "have" && <Badge variant="secondary">Đã cài</Badge>}
                    {status === "update" && <Badge>Có bản mới (đang {have?.version})</Badge>}
                    {entry.sharedAuth && <Badge variant="outline">API key của kho</Badge>}
                  </p>
                  {entry.description && <p className="text-muted-foreground">{entry.description}</p>}
                  <p className="text-xs text-muted-foreground">
                    {entry.developer?.name ?? ""}
                    {entry.hosts.length ? ` · gọi tới ${entry.hosts.join(", ")}` : ""}
                    {entry.permissions.includes("webview") ? " · mở cửa sổ trình duyệt" : ""}
                    {repos && repos.length > 1 ? ` · kho ${repo.name}` : ""}
                  </p>
                </div>
                {status !== "have" && (
                  <Button size="sm" variant={status === "update" ? "default" : "outline"} disabled={disabled} onClick={() => void onInstall({ repoId: repo.id, entry })}>
                    {status === "update" ? "Cập nhật" : "Cài"}
                  </Button>
                )}
              </li>
            ))}
          </ul>
        </section>
      ) : null}

      <Dialog open={!!auth} onOpenChange={(open) => !open && setAuth(undefined)}>
        <DialogContent className="sm:max-w-lg">
          {auth && (
            <>
              <DialogHeader>
                <DialogTitle>API key của kho — {auth.name}</DialogTitle>
                <DialogDescription>Nhập một lần, mọi extension của kho được kho cho dùng chung đều nhận key này. Key lưu trong Credential Manager của Windows.</DialogDescription>
              </DialogHeader>
              {auth.secretStore !== "keyring" && (
                <p role="alert" className="text-xs text-destructive">
                  Máy này không có kho khoá — key đang lưu trong file ở thư mục dữ liệu của app.
                </p>
              )}
              <AuthForm
                key={authVersion}
                auth={auth.auth}
                busy={disabled}
                onSave={(scheme, values) => void saveAuth(() => repoSaveAuth(auth.id, scheme, values), "Đã lưu API key của kho")}
                onClear={() => void saveAuth(() => repoClearAuth(auth.id), "Đã xoá API key của kho")}
              />
            </>
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
