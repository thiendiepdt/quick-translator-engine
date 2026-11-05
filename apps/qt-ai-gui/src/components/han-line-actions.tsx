import { useEffect, useState } from "react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { hanVietName } from "@/lib/api";
import { hanRuns, hasHan, replaceHan, spokenReading } from "@/lib/reading";

const BUTTON_CLASS = "h-6 px-2 text-xs";

interface Props {
  /** Số dòng ở lề, để phân biệt nút Thay cả dòng giữa các dòng. */
  n: number;
  line: string;
  disabled: boolean;
  onReplace: (line: string) => void;
}

/**
 * Nút thay chữ Hán bằng phiên âm, hiện sẵn ngay dưới dòng còn sót. Âm tra bảng ChinesePhienAmWords qua `han_viet_name`;
 * chữ không có trong bảng bị trả nguyên, nên cụm vẫn còn chữ Hán coi như chưa có âm và không có nút.
 */
export function HanLineActions({ n, line, disabled, onReplace }: Props) {
  const [loaded, setLoaded] = useState<{ line: string; readings: ReadonlyMap<string, string> }>();

  useEffect(() => {
    let cancelled = false;
    const texts = [...new Set(hanRuns(line).map((run) => run.text))];
    Promise.all(texts.map(async (text) => [text, await hanVietName(text)] as const))
      .then((pairs) => {
        if (!cancelled) setLoaded({ line, readings: new Map(pairs.filter(([, reading]) => !hasHan(reading))) });
      })
      .catch((error: unknown) => toast.error(error instanceof Error ? error.message : "Không tra được phiên âm"));
    return () => {
      cancelled = true;
    };
  }, [line]);

  const readings = loaded?.line === line ? loaded.readings : undefined;
  if (!readings) return null;
  // Cụm lặp trong dòng chỉ cần một nút: bấm là thay mọi chỗ, nhãn theo chỗ xuất hiện đầu.
  const seen = new Set<string>();
  const choices = hanRuns(line).flatMap((run) => {
    const reading = readings.get(run.text);
    if (reading === undefined || seen.has(run.text)) return [];
    seen.add(run.text);
    return [{ text: run.text, spoken: spokenReading(run, reading) }];
  });
  if (choices.length === 0) return null;

  return (
    <div className="flex flex-wrap items-center gap-1 pt-1 pb-2 font-sans select-none">
      {choices.map(({ text, spoken }) => (
        <Button
          key={text}
          type="button"
          variant="outline"
          size="sm"
          className={BUTTON_CLASS}
          disabled={disabled}
          title="Thay bằng phiên âm Hán-Việt, ghi thẳng vào bản dịch"
          aria-label={`Thay ${text} bằng ${spoken}`}
          onClick={() => onReplace(replaceHan(line, readings, text))}
        >
          {text} → {spoken}
        </Button>
      ))}
      {choices.length > 1 && (
        <Button
          type="button"
          variant="ghost"
          size="sm"
          className={BUTTON_CLASS}
          disabled={disabled}
          title="Thay mọi cụm chữ Hán trong dòng này bằng phiên âm Hán-Việt"
          aria-label={`Thay cả dòng ${n}`}
          onClick={() => onReplace(replaceHan(line, readings))}
        >
          Thay cả dòng
        </Button>
      )}
    </div>
  );
}
