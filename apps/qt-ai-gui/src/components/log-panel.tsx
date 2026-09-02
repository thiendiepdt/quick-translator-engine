import { Eraser } from "lucide-react";
import { useEffect, useRef } from "react";

import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

export function LogPanel() {
  const logs = useStoryStore((s) => s.logs);
  const clear = useStoryStore((s) => s.clearLogs);
  const bottom = useRef<HTMLDivElement>(null);
  useEffect(() => {
    bottom.current?.scrollIntoView({ block: "end" });
  }, [logs.length]);
  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between border-b p-2">
        <span className="text-xs text-muted-foreground">{logs.length} dòng log agy</span>
        <Button size="xs" variant="ghost" onClick={clear}>
          <Eraser /> Xoá
        </Button>
      </div>
      <div className="flex-1 overflow-auto bg-zinc-950 p-2 font-mono text-xs text-zinc-100">
        {logs.map((entry) => (
          <div key={entry.seq} className={cn("whitespace-pre-wrap", entry.stream === "stderr" && "text-amber-300")}>
            {entry.line}
          </div>
        ))}
        <div ref={bottom} />
      </div>
    </div>
  );
}
