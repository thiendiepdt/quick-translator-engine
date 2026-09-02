import { LoaderCircle, Sparkles } from "lucide-react";
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
import { ScrollArea } from "@/components/ui/scroll-area";
import { aiFillStory } from "@/lib/api";
import { diffStoryConfig, type DiffLine } from "@/lib/story-form";
import type { AiFillResult, StoryConfig } from "@/lib/types";

interface Props {
  root: string;
  initialName: string;
  initialUrl: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onApply: (after: StoryConfig) => void;
}

export function AiFillDialog({ root, initialName, initialUrl, open, onOpenChange, onApply }: Props) {
  const [name, setName] = useState(initialName);
  const [url, setUrl] = useState(initialUrl);
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<AiFillResult | undefined>();
  const [diff, setDiff] = useState<DiffLine[]>([]);

  async function run() {
    setRunning(true);
    setResult(undefined);
    try {
      const outcome = await aiFillStory(root, name.trim(), url.trim());
      setResult(outcome);
      setDiff(diffStoryConfig(outcome.before, outcome.after));
      if (outcome.exitCode !== 0) toast.error(`agy thoát mã ${outcome.exitCode} — xem log bên dưới`);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "AI điền thất bại");
    } finally {
      setRunning(false);
    }
  }

  return (
    <Dialog
      open={open}
      onOpenChange={(value) => {
        if (!running) onOpenChange(value);
      }}
    >
      <DialogContent className="max-w-3xl">
        <DialogHeader>
          <DialogTitle>AI điền hồ sơ truyện</DialogTitle>
          <DialogDescription>
            agy sẽ tra web theo tên + link, đọc vài chương đầu rồi đề xuất hồ sơ. Không ghi gì cho tới khi bạn bấm
            Áp dụng.
          </DialogDescription>
        </DialogHeader>
        <div className="grid grid-cols-2 gap-3">
          <div>
            <Label htmlFor="fill-name">Tên truyện tiếng Việt</Label>
            <Input id="fill-name" value={name} onChange={(e) => setName(e.target.value)} disabled={running} />
          </div>
          <div>
            <Label htmlFor="fill-url">Link truyện tiếng Trung</Label>
            <Input
              id="fill-url"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://www.qidian.com/book/…"
              disabled={running}
            />
          </div>
        </div>
        {result && (
          <ScrollArea className="max-h-80 rounded border">
            {diff.length === 0 ? (
              <p className="p-3 text-sm text-muted-foreground">Không có thay đổi nào.</p>
            ) : (
              <table className="w-full text-xs">
                <thead>
                  <tr className="bg-muted">
                    <th className="p-2 text-left">Field</th>
                    <th className="p-2 text-left">Trước</th>
                    <th className="p-2 text-left">Sau</th>
                  </tr>
                </thead>
                <tbody>
                  {diff.map((line) => (
                    <tr key={line.field} className="border-t align-top">
                      <td className="p-2 font-mono">{line.field}</td>
                      <td className="p-2 whitespace-pre-wrap text-muted-foreground">{line.before}</td>
                      <td className="p-2 whitespace-pre-wrap">{line.after}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
            {result.log.length > 0 && (
              <pre className="border-t bg-zinc-950 p-2 font-mono text-[11px] text-zinc-200">
                {result.log.slice(-40).join("\n")}
              </pre>
            )}
          </ScrollArea>
        )}
        <DialogFooter>
          <Button variant="outline" disabled={running} onClick={() => onOpenChange(false)}>
            Bỏ
          </Button>
          <Button variant="secondary" disabled={running || !name.trim()} onClick={() => void run()}>
            {running ? (
              <>
                <LoaderCircle className="animate-spin" /> Đang chạy agy…
              </>
            ) : (
              <>
                <Sparkles /> {result ? "Chạy lại" : "Chạy AI điền"}
              </>
            )}
          </Button>
          <Button
            disabled={!result || diff.length === 0 || running}
            onClick={() => {
              if (result) onApply(result.after);
            }}
          >
            Áp dụng
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
