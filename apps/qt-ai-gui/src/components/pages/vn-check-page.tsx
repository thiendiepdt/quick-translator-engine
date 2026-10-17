import { VnCheckPanel } from "@/components/vn-check-panel";
import { selectCurrentRunning, useStoryStore } from "@/store/story";

/** Trang riêng trên rail cho tab Kiểm tra VN (trong Hồ sơ truyện khó tìm); cùng panel với mục trong Hồ sơ truyện. */
export function VnCheckPage() {
  const root = useStoryStore((s) => s.root);
  const setting = useStoryStore((s) => s.snapshot?.story.genre.setting);
  const running = useStoryStore(selectCurrentRunning);
  if (!root || !setting) return null;
  return (
    <div className="fine-scrollbar h-full min-h-0 overflow-y-auto">
      <div className="mx-auto flex max-w-4xl flex-col gap-4 px-6 py-6">
        <div>
          <h1 className="text-lg font-semibold">Kiểm tra nội dung nhạy cảm về Việt Nam</h1>
          <p className="text-xs text-muted-foreground">
            Những chỗ raw hoặc bản dịch nhắc tới Việt Nam cần xem lại. Bấm AI giải thích ở từng chỗ để AI dịch nhanh đoạn
            raw, diễn giải và gợi ý cách xử lý; đánh dấu Không sao hoặc Cần xử lý, hoặc thay thế ngay trong bản dịch. Chỉ
            cảnh báo, không chặn dịch hay export.
          </p>
        </div>
        <VnCheckPanel key={root} root={root} running={running} setting={setting} />
      </div>
    </div>
  );
}
