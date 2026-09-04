import { Eraser } from "lucide-react";
import { useEffect, useRef } from "react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

export function LogPanel() {
  const logs = useStoryStore((s) => s.logs);
  const clear = useStoryStore((s) => s.clearLogs);
  const engine = useStoryStore((s) => s.config?.engine ?? "agy");
  const source = engine === "api" ? "API" : "agy";
  const bottom = useRef<HTMLDivElement>(null);
  useEffect(() => {
    bottom.current?.scrollIntoView({ block: "end" });
  }, [logs.length]);
  return (
    <div className="flex h-full flex-col bg-log text-log-foreground">
      <div className="flex items-center justify-between border-b border-log-foreground/10 px-3 py-1.5">
        <span className="font-mono text-xs opacity-70">{logs.length} dòng log {source}</span>
        <Button
          size="xs"
          variant="ghost"
          onClick={clear}
          className="text-log-foreground hover:bg-log-foreground/10 hover:text-log-foreground"
        >
          <Eraser /> Xoá
        </Button>
      </div>
      <div className="fine-scrollbar flex-1 overflow-auto p-3 font-mono text-xs leading-relaxed">
        {logs.length === 0 && <p className="opacity-60">Chưa có log. Bấm Bắt đầu dịch để {source} chạy.</p>}
        {logs.map((entry) => (
          <div key={entry.seq} className={cn("whitespace-pre-wrap", entry.stream === "stderr" && "text-status-warning")}>
            {entry.line}
          </div>
        ))}
        <div ref={bottom} />
      </div>
    </div>
  );
}
