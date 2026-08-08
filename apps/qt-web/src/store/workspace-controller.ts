import {
  createWorkspaceId,
  defaultWorkspace,
  normalizeWorkspaceName,
  type WorkspaceSummary,
  useWorkspaceCatalogStore,
  workspaceNameExists,
} from "@/store/workspace-catalog";
import {
  currentWorkspacePersistentState,
  emptyWorkspacePersistentState,
  serializeWorkspacePersistentState,
  useWorkspaceStore,
  type WorkspacePersistentState,
  workspaceStateStorage,
  workspaceStorageKeyFor,
} from "@/store/workspace";

function newWorkspaceSummary(name: string): WorkspaceSummary {
  const normalizedName = normalizeWorkspaceName(name);
  if (workspaceNameExists(normalizedName)) {
    throw new Error("Tên workspace đã tồn tại");
  }

  const existingIds = new Set(
    useWorkspaceCatalogStore.getState().workspaces.map(({ id }) => id),
  );
  let id = createWorkspaceId();
  while (existingIds.has(id)) id = createWorkspaceId();
  return { id, name: normalizedName };
}

async function activateWorkspaceData(
  workspaceId: string,
  initialState?: WorkspacePersistentState,
): Promise<void> {
  const storageKey = workspaceStorageKeyFor(workspaceId);
  const stored = await workspaceStateStorage.getItem(storageKey);
  if (initialState || stored === null) {
    await workspaceStateStorage.setItem(
      storageKey,
      serializeWorkspacePersistentState(
        initialState ?? emptyWorkspacePersistentState(),
      ),
    );
  }

  useWorkspaceStore.persist.setOptions({ name: storageKey });
  await useWorkspaceStore.persist.rehydrate();
  useWorkspaceStore.getState().clearWorkspace();
}

export async function selectWorkspace(workspaceId: string): Promise<void> {
  const catalog = useWorkspaceCatalogStore.getState();
  if (workspaceId === catalog.activeWorkspaceId) return;
  if (!catalog.workspaces.some(({ id }) => id === workspaceId)) {
    throw new Error("Workspace không tồn tại");
  }

  await activateWorkspaceData(workspaceId);
  useWorkspaceCatalogStore.getState().selectWorkspace(workspaceId);
}

export async function createWorkspace(name: string): Promise<WorkspaceSummary> {
  const workspace = newWorkspaceSummary(name);
  await activateWorkspaceData(workspace.id, emptyWorkspacePersistentState());
  useWorkspaceCatalogStore.getState().addAndSelectWorkspace(workspace);
  return workspace;
}

export async function forkWorkspace(name: string): Promise<WorkspaceSummary> {
  const workspace = newWorkspaceSummary(name);
  const currentState = currentWorkspacePersistentState();
  await activateWorkspaceData(workspace.id, currentState);
  useWorkspaceCatalogStore.getState().addAndSelectWorkspace(workspace);
  return workspace;
}

export async function deleteWorkspace(workspaceId: string): Promise<void> {
  const catalog = useWorkspaceCatalogStore.getState();
  if (workspaceId === defaultWorkspace.id) {
    throw new Error("Không thể xóa không gian làm việc mặc định");
  }
  if (!catalog.workspaces.some(({ id }) => id === workspaceId)) {
    throw new Error("Workspace không tồn tại");
  }

  // Đang đứng trong workspace bị xóa thì nạp dữ liệu mặc định trước khi gỡ
  // khỏi catalog, tránh khoảnh khắc store trỏ vào storage key đã xóa.
  if (catalog.activeWorkspaceId === workspaceId) {
    await activateWorkspaceData(defaultWorkspace.id);
  }
  useWorkspaceCatalogStore.getState().removeWorkspace(workspaceId);
  await workspaceStateStorage.removeItem(workspaceStorageKeyFor(workspaceId));
}
