import { useCallback, useEffect, useRef, useState } from "react";

import { appVersion } from "@/lib/app-version";
import { compareVersions, entriesSince } from "@/lib/changelog";

export const LAST_SEEN_KEY = "qt-ai-gui:last-seen-version";

const VERSION = /^\d+\.\d+\.\d+$/;

function readLastSeen(): string | null {
  try {
    return localStorage.getItem(LAST_SEEN_KEY);
  } catch {
    return null;
  }
}

function writeLastSeen(version: string): void {
  try {
    localStorage.setItem(LAST_SEEN_KEY, version);
  } catch {
    // Không lưu được thì lần sau lại dựa vào dấu hiệu đã dùng app — không đáng chặn app.
  }
}

/**
 * Quyết định lúc mở app. `since`: undefined = không mở; string = vừa lên từ bản đó; null = người dùng cũ chưa từng
 * lưu khoá (bản đầu có tính năng này) nên không rõ bản trước → chỉ hiện bản đang dùng. `store` = bản cao nhất đã thấy
 * (hạ bản rồi lên lại không hiện lại ghi chú đã xem).
 */
export function decideWhatsNew(lastSeen: string | null, current: string, hasPriorUse: boolean): { since: string | null | undefined; store: string } {
  if (!lastSeen || !VERSION.test(lastSeen)) return { since: hasPriorUse ? null : undefined, store: current };
  return compareVersions(current, lastSeen) > 0 ? { since: lastSeen, store: current } : { since: undefined, store: lastSeen };
}

/**
 * "Có gì mới" sau khi cập nhật. `hasPriorUse` (đã từng dùng app, tính từ config) — undefined khi config chưa nạp thì
 * chờ; quyết đúng một lần mỗi lần mở app. Lưu ngay khi quyết để mỗi bản chỉ hiện một lần kể cả khi tắt app không bấm gì.
 */
export function useWhatsNew(hasPriorUse: boolean | undefined): {
  version: string | undefined;
  since: string | null;
  dismiss: () => void;
} {
  const [state, setState] = useState<{ version: string; since: string | null }>();
  const decided = useRef(false);
  useEffect(() => {
    if (hasPriorUse === undefined || decided.current) return;
    let alive = true;
    void appVersion().then((current) => {
      // Lượt effect đã huỷ (StrictMode, hoặc config đổi giữa chừng) không được ghi trước, kẻo lượt sau thấy "đã xem".
      if (!alive || decided.current) return;
      decided.current = true;
      const { since, store } = decideWhatsNew(readLastSeen(), current, hasPriorUse);
      writeLastSeen(store);
      if (since !== undefined && entriesSince(current, since).some((entry) => entry.items.length > 0)) {
        setState({ version: current, since });
      }
    });
    return () => {
      alive = false;
    };
  }, [hasPriorUse]);
  const dismiss = useCallback(() => setState(undefined), []);
  return { version: state?.version, since: state?.since ?? null, dismiss };
}
