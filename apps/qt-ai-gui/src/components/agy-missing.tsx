import { ExternalLink, FolderSearch, RefreshCw, TerminalSquare } from "lucide-react";
import type { ReactNode } from "react";

import { Button } from "@/components/ui/button";
import type { AgyStatus } from "@/lib/types";

interface Props {
  status: AgyStatus;
  onRetry: () => void;
  onPickPath: () => void;
}

const STEPS: ReactNode[] = [
  <>
    Mở PowerShell, chạy:
    <pre className="mt-2 rounded-md bg-muted p-3 font-mono text-xs">irm https://antigravity.google/cli/install.ps1 | iex</pre>
  </>,
  <>
    Mở terminal mới, gõ <code className="font-mono">agy</code> và đăng nhập Google theo hướng dẫn.
  </>,
  <>
    Bấm <strong>Kiểm tra lại</strong>.
  </>,
];

export function AgyMissing({ status, onRetry, onPickPath }: Props) {
  return (
    <main className="flex h-full items-center justify-center p-8">
      <section className="w-full max-w-xl rounded-xl border bg-card p-8 shadow-sm">
        <div className="mb-4 flex size-12 items-center justify-center rounded-lg bg-primary/10 text-primary">
          <TerminalSquare className="size-6" />
        </div>
        <h1 className="text-2xl font-semibold tracking-tight">Chưa thấy Antigravity CLI</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          App dịch bằng quota Antigravity của bạn qua lệnh <code className="font-mono">agy</code>. Cài một lần, đăng
          nhập Google, rồi quay lại đây.
        </p>
        <ol className="mt-6 space-y-4 text-sm">
          {STEPS.map((content, index) => (
            <li key={index} className="flex gap-3">
              <span className="flex size-6 shrink-0 items-center justify-center rounded-full bg-secondary text-xs font-semibold text-secondary-foreground">
                {index + 1}
              </span>
              <div className="min-w-0 flex-1">{content}</div>
            </li>
          ))}
        </ol>
        {status.message && (
          <p className="mt-4 rounded-md border border-destructive/30 bg-destructive/10 p-3 text-xs text-destructive">
            {status.message}
          </p>
        )}
        <div className="mt-6 flex flex-wrap gap-2">
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
      </section>
    </main>
  );
}
