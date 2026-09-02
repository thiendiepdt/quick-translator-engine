import { ExternalLink, FolderSearch, RefreshCw } from "lucide-react";

import { Button } from "@/components/ui/button";
import type { AgyStatus } from "@/lib/types";

interface Props {
  status: AgyStatus;
  onRetry: () => void;
  onPickPath: () => void;
}

export function AgyMissing({ status, onRetry, onPickPath }: Props) {
  return (
    <main className="mx-auto flex max-w-xl flex-col gap-4 p-8">
      <h1 className="text-xl font-semibold">Chưa thấy Antigravity CLI (agy)</h1>
      <p className="text-sm text-muted-foreground">
        App dịch bằng quota Antigravity của bạn thông qua lệnh <code>agy</code>. Cài một lần, đăng nhập Google,
        rồi quay lại đây.
      </p>
      <ol className="list-decimal space-y-2 pl-5 text-sm">
        <li>
          Mở PowerShell, chạy:
          <pre className="mt-1 rounded bg-muted p-2 text-xs">irm https://antigravity.google/cli/install.ps1 | iex</pre>
        </li>
        <li>
          Mở terminal mới, gõ <code>agy</code> và đăng nhập Google theo hướng dẫn.
        </li>
        <li>
          Bấm <strong>Kiểm tra lại</strong>.
        </li>
      </ol>
      {status.message && (
        <p className="rounded bg-destructive/10 p-2 text-xs text-destructive">{status.message}</p>
      )}
      <div className="flex gap-2">
        <Button onClick={onRetry}>
          <RefreshCw /> Kiểm tra lại
        </Button>
        <Button variant="outline" onClick={onPickPath}>
          <FolderSearch /> Chọn file agy tay
        </Button>
        <Button variant="ghost" asChild>
          <a href="https://antigravity.google/docs/cli/install" target="_blank" rel="noreferrer">
            <ExternalLink /> Hướng dẫn
          </a>
        </Button>
      </div>
    </main>
  );
}
