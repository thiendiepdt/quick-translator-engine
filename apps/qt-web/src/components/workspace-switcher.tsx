import {
  ArrowLeft,
  Check,
  ChevronDown,
  GitFork,
  LoaderCircle,
  PanelsTopLeft,
  Plus,
} from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

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
import { Label } from "@/components/ui/label";
import {
  createWorkspace,
  forkWorkspace,
  selectWorkspace,
} from "@/store/workspace-controller";
import {
  useWorkspaceCatalogStore,
  workspaceNameExists,
} from "@/store/workspace-catalog";

type CreationMode = "create" | "fork";

function availableForkName(baseName: string): string {
  const initial = `${baseName} (bản sao)`;
  if (!workspaceNameExists(initial)) return initial;
  let index = 2;
  while (workspaceNameExists(`${initial} ${index}`)) index += 1;
  return `${initial} ${index}`;
}

export function WorkspaceSwitcher() {
  const workspaces = useWorkspaceCatalogStore((state) => state.workspaces);
  const activeWorkspaceId = useWorkspaceCatalogStore(
    (state) => state.activeWorkspaceId,
  );
  const activeWorkspace =
    workspaces.find(({ id }) => id === activeWorkspaceId) ?? workspaces[0];
  const [open, setOpen] = useState(false);
  const [mode, setMode] = useState<CreationMode>();
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);

  const normalizedName = name.trim().replace(/\s+/g, " ");
  const duplicateName = normalizedName.length > 0 && workspaceNameExists(normalizedName);

  function changeOpen(nextOpen: boolean) {
    if (busy) return;
    setOpen(nextOpen);
    if (!nextOpen) {
      setMode(undefined);
      setName("");
    }
  }

  function startCreation(nextMode: CreationMode) {
    setMode(nextMode);
    setName(nextMode === "fork" ? availableForkName(activeWorkspace.name) : "");
  }

  async function switchTo(workspaceId: string) {
    if (workspaceId === activeWorkspaceId) {
      changeOpen(false);
      return;
    }
    setBusy(true);
    try {
      await selectWorkspace(workspaceId);
      const workspace = workspaces.find(({ id }) => id === workspaceId);
      toast.success(`Đã chuyển sang không gian làm việc “${workspace?.name ?? workspaceId}”`);
      setOpen(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không đổi được không gian làm việc");
    } finally {
      setBusy(false);
    }
  }

  async function submitCreation(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!mode || !normalizedName || duplicateName) return;

    setBusy(true);
    try {
      const workspace = mode === "fork"
        ? await forkWorkspace(normalizedName)
        : await createWorkspace(normalizedName);
      toast.success(
        mode === "fork"
          ? `Đã sao chép sang không gian làm việc “${workspace.name}”`
          : `Đã tạo không gian làm việc “${workspace.name}”`,
      );
      setOpen(false);
      setMode(undefined);
      setName("");
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không tạo được không gian làm việc");
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <div className="flex items-center gap-2">
        <span className="hidden whitespace-nowrap text-xs font-medium text-muted-foreground sm:inline">
          Không gian làm việc:
        </span>
        <Button
          type="button"
          variant="outline"
          className="max-w-40 px-2.5 sm:px-3"
          aria-label={`Không gian làm việc hiện tại: ${activeWorkspace.name}`}
          onClick={() => setOpen(true)}
        >
          <PanelsTopLeft />
          <span className="hidden max-w-24 truncate sm:inline">{activeWorkspace.name}</span>
          <ChevronDown className="hidden size-3.5 opacity-55 sm:block" />
        </Button>
      </div>

      <Dialog open={open} onOpenChange={changeOpen}>
        <DialogContent className="gap-0 sm:max-w-lg">
          {mode ? (
            <form onSubmit={(event) => void submitCreation(event)}>
              <DialogHeader className="border-b px-5 py-4 pr-12">
                <DialogTitle>
                  {mode === "fork"
                    ? "Sao chép không gian làm việc hiện tại"
                    : "Tạo không gian làm việc mới"}
                </DialogTitle>
                <DialogDescription>
                  {mode === "fork"
                    ? "Sao chép bộ từ điển và bộ nhớ tên hiện tại sang một không gian làm việc độc lập."
                    : "Không gian làm việc mới bắt đầu với bộ từ điển và bộ nhớ tên trống."}
                </DialogDescription>
              </DialogHeader>

              <div className="grid gap-2 px-5 py-5">
                <Label htmlFor="workspace-name">Tên không gian làm việc</Label>
                <Input
                  id="workspace-name"
                  autoFocus
                  maxLength={80}
                  value={name}
                  onChange={(event) => setName(event.target.value)}
                  placeholder="Ví dụ: Đấu Phá Thương Khung"
                  aria-invalid={duplicateName}
                />
                <p className={duplicateName ? "text-xs text-destructive" : "text-xs text-muted-foreground"}>
                  {duplicateName
                    ? "Tên không gian làm việc đã tồn tại."
                    : "Tối đa 80 ký tự; khoảng trắng thừa sẽ được chuẩn hóa."}
                </p>
              </div>

              <DialogFooter className="border-t px-5 py-4">
                <Button
                  type="button"
                  variant="ghost"
                  disabled={busy}
                  onClick={() => {
                    setMode(undefined);
                    setName("");
                  }}
                >
                  <ArrowLeft /> Quay lại
                </Button>
                <Button type="submit" disabled={busy || !normalizedName || duplicateName}>
                  {busy ? <LoaderCircle className="animate-spin" /> : mode === "fork" ? <GitFork /> : <Plus />}
                  {mode === "fork"
                    ? "Sao chép không gian làm việc"
                    : "Tạo không gian làm việc"}
                </Button>
              </DialogFooter>
            </form>
          ) : (
            <>
              <DialogHeader className="border-b px-5 py-4 pr-12">
                <DialogTitle>Không gian làm việc</DialogTitle>
                <DialogDescription>
                  Mỗi không gian làm việc có bộ từ điển và bộ nhớ tên riêng trên trình duyệt này.
                </DialogDescription>
              </DialogHeader>

              <div className="fine-scrollbar max-h-[min(52dvh,420px)] overflow-y-auto p-2">
                {workspaces.map((workspace) => {
                  const active = workspace.id === activeWorkspaceId;
                  return (
                    <button
                      key={workspace.id}
                      type="button"
                      disabled={busy}
                      aria-current={active ? "true" : undefined}
                      onClick={() => void switchTo(workspace.id)}
                      className="flex w-full items-center gap-3 rounded-md px-3 py-2.5 text-left text-sm transition-colors hover:bg-accent disabled:pointer-events-none disabled:opacity-60"
                    >
                      <span className="grid size-8 shrink-0 place-items-center rounded-md border bg-card">
                        {busy && !active ? <PanelsTopLeft /> : active ? <Check className="text-primary" /> : <PanelsTopLeft />}
                      </span>
                      <span className="min-w-0 flex-1 truncate font-medium">{workspace.name}</span>
                      {active ? <span className="text-[10px] font-semibold text-primary uppercase">Đang dùng</span> : null}
                    </button>
                  );
                })}
              </div>

              <DialogFooter className="border-t px-5 py-4 sm:justify-between">
                <Button type="button" variant="outline" disabled={busy} onClick={() => startCreation("fork")}>
                  <GitFork /> Sao chép hiện tại
                </Button>
                <Button type="button" disabled={busy} onClick={() => startCreation("create")}>
                  <Plus /> Tạo không gian làm việc
                </Button>
              </DialogFooter>
            </>
          )}
        </DialogContent>
      </Dialog>
    </>
  );
}
