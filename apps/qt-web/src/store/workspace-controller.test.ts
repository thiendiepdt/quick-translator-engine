import { afterEach, beforeEach, describe, expect, it } from "vitest";

import {
  dictionaryUpdateKeys,
  type DictionaryDefaults,
  type LocalDictionaryEntries,
} from "@/lib/types";
import {
  createWorkspace,
  forkWorkspace,
  selectWorkspace,
} from "@/store/workspace-controller";
import {
  activeWorkspaceStorageKey,
  defaultWorkspace,
  readStoredActiveWorkspaceId,
  useWorkspaceCatalogStore,
} from "@/store/workspace-catalog";
import { useWorkspaceStore } from "@/store/workspace";

const emptyDefaults: DictionaryDefaults = {
  names: "",
  names2: "",
  luatNhan: "",
  pronouns: "",
  danhTu: "",
  hoNguoi: "",
  hauTu: "",
  ignoredChinesePhrases: "",
};

let testIndex = 0;

beforeEach(async () => {
  testIndex += 1;
  await selectWorkspace(defaultWorkspace.id);
  useWorkspaceStore.setState({
    knownNames: {},
    rejectedNames: [],
    rangePinEnabled: true,
    dictionaryOverrides: {},
    localDictionaryEntries: Object.fromEntries(
      dictionaryUpdateKeys.map((key) => [key, {}]),
    ) as LocalDictionaryEntries,
  });
  useWorkspaceStore
    .getState()
    .hydrateDictionaryDefaults(`/workspace-test-${testIndex}`, emptyDefaults);
  useWorkspaceStore.getState().clearWorkspace();
});

afterEach(async () => {
  await selectWorkspace(defaultWorkspace.id);
});

describe("workspace lifecycle", () => {
  it("provides the default workspace and isolates newly created workspaces", async () => {
    useWorkspaceStore.getState().acceptNameCandidate("萧炎", "Tiêu Viêm");
    useWorkspaceStore
      .getState()
      .saveLocalDictionaryEntries("vietPhrase", { 看着: "quan sát" });

    const created = await createWorkspace(`Truyện mới ${testIndex}`);

    expect(useWorkspaceCatalogStore.getState().activeWorkspaceId).toBe(created.id);
    expect(localStorage.getItem(activeWorkspaceStorageKey)).toBe(created.id);
    expect(readStoredActiveWorkspaceId()).toBe(created.id);
    expect(useWorkspaceStore.getState().knownNames).toEqual({});
    expect(useWorkspaceStore.getState().localDictionaryEntries.vietPhrase).toEqual({});

    useWorkspaceStore.getState().acceptNameCandidate("林动", "Lâm Động");
    await selectWorkspace(defaultWorkspace.id);
    expect(useWorkspaceStore.getState().knownNames).toEqual({ 萧炎: "Tiêu Viêm" });
    expect(useWorkspaceStore.getState().localDictionaryEntries.vietPhrase).toEqual({
      看着: "quan sát",
    });

    await selectWorkspace(created.id);
    expect(useWorkspaceStore.getState().knownNames).toEqual({ 林动: "Lâm Động" });
  });

  it("forks persistent data and keeps later changes independent", async () => {
    useWorkspaceStore.getState().acceptNameCandidate("萧炎", "Tiêu Viêm");
    useWorkspaceStore.getState().setRangePinEnabled(false);
    useWorkspaceStore
      .getState()
      .saveLocalDictionaryEntries("names", { 药老: "Dược Lão" });

    const forked = await forkWorkspace(`Bản fork ${testIndex}`);

    expect(useWorkspaceCatalogStore.getState().activeWorkspaceId).toBe(forked.id);
    expect(useWorkspaceStore.getState().knownNames).toEqual({ 萧炎: "Tiêu Viêm" });
    expect(useWorkspaceStore.getState().rangePinEnabled).toBe(false);
    expect(useWorkspaceStore.getState().localDictionaryEntries.names).toEqual({
      药老: "Dược Lão",
    });

    useWorkspaceStore.getState().acceptNameCandidate("林动", "Lâm Động");
    await selectWorkspace(defaultWorkspace.id);
    expect(useWorkspaceStore.getState().knownNames).toEqual({ 萧炎: "Tiêu Viêm" });
  });
});
