import type { GenreSetting, VnHitView } from "@/lib/types";

// Bản tạm để tab Kiểm tra VN biên dịch được; Task 9 viết thật.
export function VnReplaceDialog(_props: {
  root: string;
  setting: GenreSetting;
  hits: VnHitView[];
  allHits: VnHitView[];
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onDone: () => void;
}) {
  return null;
}
