import { TriangleAlert } from "lucide-react";
import { useEffect, useState } from "react";

import { Button } from "@/components/ui/button";
import { storyPov, type PovReport } from "@/lib/api";
import { toiPovWarning } from "@/lib/story-pov";
import { cn } from "@/lib/utils";
import { useStoryStore } from "@/store/story";

/**
 * Dòng cảnh báo ở màn dịch khi truyện chọn "Ngôi một — người kể xưng tôi" (chỉ có hiệu lực ở truyện hiện đại, như
 * prompt): luôn hiện để người dùng biết lời kể sẽ ra "tôi", kèm kết quả đọc thử raw để bắt chỗ chọn nhầm.
 */
export function PovToiWarning() {
  const root = useStoryStore((s) => s.root);
  const genre = useStoryStore((s) => s.snapshot?.story.genre);
  const setPage = useStoryStore((s) => s.setPage);
  const active = !!root && genre?.pov === "first-toi" && genre.setting === "modern";
  // Gắn root để chuyển truyện không hiện nhầm kết quả của truyện trước trong lúc đọc raw truyện mới.
  const [loaded, setLoaded] = useState<{ root: string; report: PovReport }>();

  useEffect(() => {
    if (!active || !root) return;
    let alive = true;
    storyPov(root).then(
      (report) => {
        if (alive) setLoaded({ root, report });
      },
      () => undefined,
    );
    return () => {
      alive = false;
    };
  }, [active, root]);

  if (!active) return null;
  const warning = toiPovWarning(loaded?.root === root ? loaded.report : undefined);
  return (
    <div role="alert" aria-label="Cảnh báo ngôi kể" className="mt-2 flex flex-wrap items-start gap-x-2 gap-y-1 text-xs text-status-warning">
      <TriangleAlert className="mt-0.5 size-3.5 shrink-0" />
      <span className="min-w-0 flex-1">
        {warning.lead}
        {warning.raw && (
          <>
            {" "}
            <span className={cn(warning.mismatch && "font-medium text-status-error")}>{warning.raw}</span>
          </>
        )}
      </span>
      <Button type="button" variant="outline" size="sm" className="h-6 shrink-0 px-2 text-xs" onClick={() => setPage("story")}>
        Đổi ở Hồ sơ truyện…
      </Button>
    </div>
  );
}
