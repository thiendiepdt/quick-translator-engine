import { useCallback, useEffect, useState } from "react";

import { appVersion } from "@/lib/app-version";
import { entryFor } from "@/lib/changelog";

export const LAST_SEEN_KEY = "qt-ai-gui:last-seen-version";

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
    // Không lưu được thì lần sau coi như chạy lần đầu (không hiện) — không đáng chặn app.
  }
}

/**
 * "Có gì mới" sau khi cập nhật: version đang chạy khác version đã thấy lần trước thì trả `version` để mở hộp thoại.
 * Lần đầu chạy chỉ lưu im lặng. Lưu ngay khi phát hiện để mỗi bản chỉ hiện một lần, kể cả khi tắt app không bấm gì.
 */
export function useWhatsNew(): { version: string | undefined; dismiss: () => void } {
  const [version, setVersion] = useState<string>();
  useEffect(() => {
    let alive = true;
    void appVersion().then((current) => {
      // Lượt effect đã huỷ (StrictMode chạy effect hai lần) không được ghi trước, kẻo lượt sau đọc lại thấy "đã xem".
      if (!alive) return;
      const lastSeen = readLastSeen();
      writeLastSeen(current);
      if (lastSeen && lastSeen !== current && entryFor(current)?.items.length) setVersion(current);
    });
    return () => {
      alive = false;
    };
  }, []);
  const dismiss = useCallback(() => setVersion(undefined), []);
  return { version, dismiss };
}
