import { useEffect } from "react";

/** Phần của Update (plugin-updater) mà luồng cập nhật cần — thu hẹp để test giả được. */
export interface UpdateHandle {
  version: string;
  /** Release notes (thân GitHub release = bullet của bản đó trong CHANGELOG.md), qua `notes` trong latest.json. */
  body?: string;
  downloadAndInstall: () => Promise<void>;
}

export interface UpdateCheckDeps {
  check: () => Promise<UpdateHandle | null>;
  ask: (text: string, options: { title: string; kind: "info" }) => Promise<boolean>;
  message: (text: string, options: { title: string }) => Promise<unknown>;
  relaunch: () => Promise<void>;
  warn: (...args: unknown[]) => void;
}

export type UpdateCheckResult = "none" | "declined" | "installed" | "failed";

export const UPDATE_TITLE = "Cập nhật VNCVT AI Translator";

const MAX_NOTES = 12;

/**
 * Câu hỏi cập nhật cho hộp thoại native (chỉ chữ thuần): bullet "- " của release notes thành "• ", bỏ ** và `,
 * tối đa 12 dòng. Không có bullet nào (bản cũ không ghi chú) thì giữ câu hỏi ngắn.
 */
export function updatePrompt(version: string, body?: string): string {
  const notes = (body ?? "")
    .split(/\r?\n/)
    .map((line) => /^\s*[-*] (.+)$/.exec(line)?.[1] ?? "")
    .map((note) => note.replace(/\*\*|`/g, "").trim())
    .filter(Boolean);
  if (notes.length === 0) return `Phiên bản mới ${version} đã có! Bạn có muốn cập nhật ngay không?`;
  const shown = notes.slice(0, MAX_NOTES).map((line) => `• ${line}`);
  if (notes.length > MAX_NOTES) shown.push("…");
  return `Phiên bản mới ${version} đã có!\n\nCó gì mới:\n${shown.join("\n")}\n\nBạn có muốn cập nhật ngay không?`;
}

/**
 * Luồng cập nhật y hệt novelkit: dò → hỏi hộp thoại native → tải & cài → báo → khởi động lại.
 * Lỗi ở bất kỳ bước nào (offline, endpoint chưa có release, chữ ký sai) chỉ warn, không chặn app.
 */
export async function runUpdateCheck(deps: UpdateCheckDeps): Promise<UpdateCheckResult> {
  try {
    const update = await deps.check();
    if (!update) return "none";
    const yes = await deps.ask(updatePrompt(update.version, update.body), { title: UPDATE_TITLE, kind: "info" });
    if (!yes) return "declined";
    await update.downloadAndInstall();
    await deps.message("Cập nhật hoàn tất. Ứng dụng sẽ khởi động lại.", { title: "Cập nhật thành công" });
    await deps.relaunch();
    return "installed";
  } catch (error) {
    deps.warn("Kiểm tra cập nhật thất bại:", error);
    return "failed";
  }
}

/** Deps thật, import động để vitest/jsdom không kéo `@tauri-apps/*` khi hook bị tắt. */
async function tauriDeps(): Promise<UpdateCheckDeps> {
  const [{ check }, { ask, message }, { relaunch }] = await Promise.all([
    import("@tauri-apps/plugin-updater"),
    import("@tauri-apps/plugin-dialog"),
    import("@tauri-apps/plugin-process"),
  ]);
  return { check, ask, message, relaunch, warn: console.warn };
}

/** Dò bản mới một lần khi app mở. Tắt ở dev server: không có bản cài nào để cập nhật. */
export function useUpdateCheck(enabled: boolean = !import.meta.env.DEV): void {
  useEffect(() => {
    if (!enabled) return;
    void tauriDeps().then(runUpdateCheck);
  }, [enabled]);
}
