import { History } from "lucide-react";
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
import { storyReset } from "@/lib/api";
import { useStoryStore } from "@/store/story";

interface Props {
  root: string;
  /** Tổng số chương và số chương đã dịch xong — để người dùng thấy quy mô trước khi bấm. */
  total: number;
  done: number;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** Gọi sau khi reset xong — trang cha nạp lại những gì đọc từ file ngoài snapshot (bảng nhân vật). */
  onReset?: () => void;
}

/**
 * Reset truyện về như lúc vừa tạo để dịch lại từ đầu sau khi app đổi lớn: hồ sơ chỉ còn tên + link, glossary
 * và bảng nhân vật xoá, mọi chương về hàng đợi. File chương gốc, bản dịch cũ và bản xuất còn nguyên.
 */
export function ResetStoryDialog({ root, total, done, open, onOpenChange, onReset }: Props) {
  const setSnapshot = useStoryStore((s) => s.setSnapshot);
  const [busy, setBusy] = useState(false);

  async function run() {
    setBusy(true);
    try {
      setSnapshot(await storyReset(root));
      onReset?.();
      toast.success(`Đã reset truyện — ${total} chương về hàng đợi. Điền lại hồ sơ rồi bấm Bắt đầu để dịch.`);
      onOpenChange(false);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "Không reset được truyện");
    } finally {
      setBusy(false);
    }
  }

  return (
    <Dialog open={open} onOpenChange={(value) => !busy && onOpenChange(value)}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Reset truyện</DialogTitle>
          <DialogDescription>
            Đưa truyện về như lúc vừa tạo để dịch lại từ đầu bằng bản app mới.
          </DialogDescription>
        </DialogHeader>
        <div className="flex flex-col gap-2 rounded-md bg-muted p-3 text-sm">
          <p>
            <strong>Xoá:</strong> hồ sơ truyện trừ tên và link (nhân vật chính, tóm tắt, thể loại, style, glossary, bảng
            nhân vật, rule và prompt riêng), nháp đang dịch dở trong work/.
          </p>
          <p>
            <strong>Về hàng đợi:</strong> cả <span className="tabular-nums">{total}</span> chương, trong đó{" "}
            <span className="tabular-nums">{done}</span> chương đã dịch xong.
          </p>
          <p>
            <strong>Giữ:</strong> raw/, out/ và export/ giữ nguyên, cài đặt dịch của truyện cũng vậy. Bản dịch cũ trong
            out/ chỉ bị ghi đè khi chương đó được dịch lại.
          </p>
          <p>
            <strong>Ghi lại:</strong> AGENTS.md và workflow của agent ghi lại theo template mới, kể cả file đã sửa tay.
          </p>
          <p className="text-muted-foreground">
            Bản trước khi reset được lưu dạng .bak trong thư mục truyện: story.json.bak, state.json.bak, cast.json.bak,
            và .bak của AGENTS.md / workflow nào từng sửa tay.
          </p>
        </div>
        <DialogFooter>
          <Button variant="outline" disabled={busy} onClick={() => onOpenChange(false)}>
            Huỷ
          </Button>
          <Button variant="destructive" disabled={busy} onClick={() => void run()}>
            <History /> Reset truyện
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
