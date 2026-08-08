import { create } from "zustand";

export interface WorkspaceSummary {
  id: string;
  name: string;
}

interface WorkspaceCatalogState {
  workspaces: WorkspaceSummary[];
  activeWorkspaceId: string;
  addAndSelectWorkspace: (workspace: WorkspaceSummary) => void;
  selectWorkspace: (workspaceId: string) => void;
  removeWorkspace: (workspaceId: string) => void;
}

export const defaultWorkspace: WorkspaceSummary = {
  id: "default",
  name: "Mặc định",
};
export const workspaceCatalogStorageKey = "qt-web-workspaces-v1";
export const activeWorkspaceStorageKey = "qt-web-active-workspace-v1";

function readLocalStorage(key: string): string | null {
  try {
    return window.localStorage.getItem(key);
  } catch {
    return null;
  }
}

function writeLocalStorage(key: string, value: string): void {
  try {
    window.localStorage.setItem(key, value);
  } catch {
    // IndexedDB vẫn giữ dữ liệu workspace nếu trình duyệt chặn localStorage.
  }
}

function isWorkspaceSummary(value: unknown): value is WorkspaceSummary {
  if (!value || typeof value !== "object") return false;
  const candidate = value as Partial<WorkspaceSummary>;
  return (
    typeof candidate.id === "string" &&
    candidate.id.trim().length > 0 &&
    typeof candidate.name === "string" &&
    candidate.name.trim().length > 0
  );
}

export function readStoredWorkspaces(): WorkspaceSummary[] {
  const raw = readLocalStorage(workspaceCatalogStorageKey);
  let stored: unknown = [];
  if (raw) {
    try {
      stored = JSON.parse(raw);
    } catch {
      stored = [];
    }
  }

  const seen = new Set<string>([defaultWorkspace.id]);
  const workspaces = [defaultWorkspace];
  if (!Array.isArray(stored)) return workspaces;

  for (const candidate of stored) {
    if (!isWorkspaceSummary(candidate) || seen.has(candidate.id)) continue;
    seen.add(candidate.id);
    workspaces.push({ id: candidate.id, name: candidate.name.trim() });
  }
  return workspaces;
}

export function readStoredActiveWorkspaceId(
  workspaces = readStoredWorkspaces(),
): string {
  const stored = readLocalStorage(activeWorkspaceStorageKey);
  return stored && workspaces.some(({ id }) => id === stored)
    ? stored
    : defaultWorkspace.id;
}

function persistCatalog(
  workspaces: WorkspaceSummary[],
  activeWorkspaceId: string,
): void {
  writeLocalStorage(workspaceCatalogStorageKey, JSON.stringify(workspaces));
  writeLocalStorage(activeWorkspaceStorageKey, activeWorkspaceId);
}

const initialWorkspaces = readStoredWorkspaces();

export const useWorkspaceCatalogStore = create<WorkspaceCatalogState>((set) => ({
  workspaces: initialWorkspaces,
  activeWorkspaceId: readStoredActiveWorkspaceId(initialWorkspaces),
  addAndSelectWorkspace: (workspace) =>
    set((state) => {
      const workspaces = [...state.workspaces, workspace];
      persistCatalog(workspaces, workspace.id);
      return { workspaces, activeWorkspaceId: workspace.id };
    }),
  selectWorkspace: (workspaceId) =>
    set((state) => {
      if (!state.workspaces.some(({ id }) => id === workspaceId)) return state;
      writeLocalStorage(activeWorkspaceStorageKey, workspaceId);
      return { activeWorkspaceId: workspaceId };
    }),
  removeWorkspace: (workspaceId) =>
    set((state) => {
      if (workspaceId === defaultWorkspace.id) return state;
      const workspaces = state.workspaces.filter(({ id }) => id !== workspaceId);
      if (workspaces.length === state.workspaces.length) return state;
      // Xóa workspace đang dùng thì quay về mặc định.
      const activeWorkspaceId = state.activeWorkspaceId === workspaceId
        ? defaultWorkspace.id
        : state.activeWorkspaceId;
      persistCatalog(workspaces, activeWorkspaceId);
      return { workspaces, activeWorkspaceId };
    }),
}));

export function createWorkspaceId(): string {
  if (typeof globalThis.crypto?.randomUUID === "function") {
    return globalThis.crypto.randomUUID();
  }
  return `workspace-${Date.now().toString(36)}-${Math.random().toString(36).slice(2)}`;
}

export function normalizeWorkspaceName(name: string): string {
  const normalized = name.trim().replace(/\s+/g, " ");
  if (!normalized) throw new Error("Tên workspace không được để trống");
  if (normalized.length > 80) throw new Error("Tên workspace không được quá 80 ký tự");
  return normalized;
}

export function workspaceNameExists(name: string): boolean {
  const normalized = name.trim().replace(/\s+/g, " ").toLocaleLowerCase("vi-VN");
  return useWorkspaceCatalogStore
    .getState()
    .workspaces.some((workspace) => workspace.name.toLocaleLowerCase("vi-VN") === normalized);
}
