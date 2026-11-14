import { useEffect } from "react";
import { toast } from "sonner";

import { extAutoUpdate } from "@/lib/api";
import { invalidateRawChecks } from "@/store/raw-check";

/** Mỗi lần mở app chỉ chạy một lần (StrictMode gọi effect hai lần, đổi config cũng không chạy lại). */
let started = false;

/** Cho test. */
export function resetExtAutoUpdate(): void {
  started = false;
}

const names = (items: { name: string; to: string }[]) => items.map((item) => `${item.name} ${item.to}`).join(", ");

/**
 * Mở app thì tự cập nhật extension tải raw lên bản mới (chạy nền, không chặn giao diện). Bản xin thêm domain / quyền
 * không tự cài — báo để người dùng vào Cài đặt đồng ý. Mất mạng / kho lỗi thì im lặng, giữ bản đang cài.
 * `enabled`: theo ô "Tự cập nhật extension khi mở app"; `undefined` = chưa đọc xong cấu hình.
 */
export function useExtAutoUpdate(enabled: boolean | undefined): void {
  useEffect(() => {
    if (!enabled || started) return;
    started = true;
    extAutoUpdate().then(
      (report) => {
        if (report.updated.length) {
          invalidateRawChecks();
          toast.success(`Đã cập nhật extension: ${names(report.updated)}`);
        }
        if (report.needsConsent.length) {
          toast.message(`Có bản mới cần bạn đồng ý quyền: ${names(report.needsConsent)}`, {
            description: "Bản mới xin thêm domain hoặc quyền nên không tự cài — vào Cài đặt → Extension tải raw → Cập nhật.",
            duration: 15_000,
          });
        }
        if (report.failed.length) console.warn("Tự cập nhật extension:", report.failed);
      },
      (error: unknown) => console.warn("Tự cập nhật extension không chạy được:", error),
    );
  }, [enabled]);
}
