/**
 * IndexedDB mặc định là best-effort: thiếu disk hoặc origin lâu không dùng là
 * trình duyệt tự dọn — người dùng mất trắng công dịch. `storage.persist()`
 * chuyển origin sang chế độ persistent: chỉ người dùng chủ động xóa được.
 * Trình duyệt có quyền từ chối (Chrome cấp theo mức độ dùng site / cài PWA),
 * nên trạng thái trả về để UI cảnh báo và khuyên backup khi bị từ chối.
 */

export type PersistentStorageStatus =
  | "persisted"
  | "granted"
  | "denied"
  | "unsupported";

export async function ensurePersistentStorage(): Promise<PersistentStorageStatus> {
  const storage: StorageManager | undefined = navigator.storage;
  if (
    typeof storage?.persisted !== "function" ||
    typeof storage.persist !== "function"
  ) {
    return "unsupported";
  }
  try {
    if (await storage.persisted()) return "persisted";
    return (await storage.persist()) ? "granted" : "denied";
  } catch {
    return "unsupported";
  }
}
