import { RotateCcw, Save } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { paddingGet, paddingReset, paddingSave, type PaddingSettings } from "@/lib/api";
import { PADDING_MODES } from "@/lib/schema";

const MODE_LABELS: Record<PaddingSettings["mode"], string> = {
  off: "Tắt — chỉ cảnh báo",
  filler: "Chèn văn mẫu",
  repeat: "Lặp lại nội dung chương",
};

/**
 * Chương ngắn cho nền tảng đăng truyện: ngưỡng chữ, cách bù tự động lúc chốt chương, văn mẫu. Lưu base/padding.json
 * nên áp mọi truyện, cả phiên agy. Nút thủ công trong khung đọc chương dùng văn mẫu khi đang tắt.
 */
/** id thẻ Chương ngắn trong Cài đặt — toolbar cuộn tới khi gợi ý bật tự chèn. */
export const PADDING_CARD_ID = "padding-settings";

export function PaddingSettingsEditor() {
  const [saved, setSaved] = useState<PaddingSettings>();
  const [draft, setDraft] = useState<PaddingSettings>();
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let alive = true;
    paddingGet()
      .then((settings) => {
        if (!alive) return;
        setSaved(settings);
        setDraft(settings);
      })
      .catch((error: unknown) => alive && toast.error(error instanceof Error ? error.message : "Không đọc được cài đặt chương ngắn"));
    return () => {
      alive = false;
    };
  }, []);

  if (!draft || !saved) return <p className="text-sm text-muted-foreground">Đang đọc…</p>;
  const current = draft;
  const dirty = JSON.stringify(current) !== JSON.stringify(saved);
  const valid = Number.isInteger(current.threshold) && current.threshold > 0;

  async function run(action: () => Promise<PaddingSettings>, message: string) {
    setBusy(true);
    try {
      const next = await action();
      setSaved(next);
      setDraft(next);
      toast.success(message);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không lưu được");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="grid gap-4 sm:grid-cols-[1fr_12rem]">
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="padding-mode">Khi chốt chương ngắn</Label>
          <Select
            value={current.mode}
            onValueChange={(mode) => setDraft({ ...current, mode: mode as PaddingSettings["mode"] })}
          >
            <SelectTrigger id="padding-mode" aria-label="Khi chốt chương ngắn" className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {PADDING_MODES.map((mode) => (
                <SelectItem key={mode} value={mode}>
                  {MODE_LABELS[mode]}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="padding-threshold">Ngưỡng (chữ)</Label>
          <Input
            id="padding-threshold"
            type="number"
            min={1}
            value={Number.isNaN(current.threshold) ? "" : current.threshold}
            onChange={(e) => setDraft({ ...current, threshold: e.target.valueAsNumber })}
          />
        </div>
      </div>
      <div className="flex flex-col gap-1.5">
        <Label htmlFor="padding-filler">Văn mẫu</Label>
        <Textarea
          id="padding-filler"
          value={current.filler}
          onChange={(e) => setDraft({ ...current, filler: e.target.value })}
          className="field-sizing-fixed h-64 resize-y text-sm"
        />
        <p className="text-xs text-muted-foreground">
          Mỗi đoạn cách nhau một dòng trống. App chèn từng đoạn nguyên vẹn theo thứ tự tới khi đủ ngưỡng, đoạn cuối là câu
          kết luôn chèn sau cùng. Phần chèn nằm cuối chương, bọc giữa hai dòng <code>❖❖❖❖❖❖❖❖❖❖</code> (gộp chương thì tách ra được). Chữ đếm như nền tảng: không
          tính dòng tên chương.
        </p>
      </div>
      <div className="flex items-center gap-2">
        <Button
          type="button"
          variant="ghost"
          size="sm"
          disabled={busy}
          onClick={() => {
            if (window.confirm("Về mặc định: tắt, 500 chữ, văn mẫu gốc?")) void run(paddingReset, "Đã về mặc định");
          }}
        >
          <RotateCcw /> Về mặc định
        </Button>
        <div className="flex-1" />
        {!valid && <span className="text-xs text-destructive">Ngưỡng phải là số nguyên lớn hơn 0</span>}
        <Button
          type="button"
          size="sm"
          disabled={busy || !dirty || !valid}
          onClick={() => void run(() => paddingSave(current), "Đã lưu cài đặt chương ngắn")}
        >
          <Save /> Lưu
        </Button>
      </div>
    </div>
  );
}
