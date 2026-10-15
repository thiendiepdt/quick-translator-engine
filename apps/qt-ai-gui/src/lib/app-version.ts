import { getVersion } from "@tauri-apps/api/app";
import { useEffect, useState } from "react";

// Chỉ lấy trường version (Vite tree-shake JSON theo named import, không nhúng cả package.json).
import { version as packageVersion } from "../../package.json";

/** set-version ghi cùng một version vào package.json và tauri.conf.json, nên đây là dự phòng đúng khi ngoài Tauri. */
export const PACKAGE_VERSION: string = packageVersion;

/** Version của bản đang chạy: Tauri getVersion (tauri.conf.json); dev server/test không có IPC thì lấy package.json. */
export async function appVersion(): Promise<string> {
  try {
    return await getVersion();
  } catch {
    return PACKAGE_VERSION;
  }
}

/** undefined cho tới khi đọc xong (một nhịp IPC). */
export function useAppVersion(): string | undefined {
  const [version, setVersion] = useState<string>();
  useEffect(() => {
    let alive = true;
    void appVersion().then((value) => alive && setVersion(value));
    return () => {
      alive = false;
    };
  }, []);
  return version;
}
