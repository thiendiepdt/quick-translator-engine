import { beforeEach, describe, expect, it } from "vitest";

import type { DictionaryDefaults } from "@/lib/types";
import { dictionaryPayload, useWorkspaceStore } from "@/store/workspace";

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

let endpointIndex = 0;

beforeEach(() => {
  endpointIndex += 1;
  useWorkspaceStore
    .getState()
    .hydrateDictionaryDefaults(`/test-${endpointIndex}`, emptyDefaults);
  useWorkspaceStore.getState().clearNameMemory();
  useWorkspaceStore.getState().clearWorkspace();
});

describe("workspace dictionary semantics", () => {
  it("loads defaults, sends only changed files, and restores the engine version", () => {
    useWorkspaceStore.getState().hydrateDictionaryDefaults("/engine", {
      ...emptyDefaults,
      names: "萧炎=Tiêu Viêm\n药老=Dược Lão",
    });

    expect(useWorkspaceStore.getState().dictionaries.names.value).toContain("Tiêu Viêm");
    expect(dictionaryPayload(useWorkspaceStore.getState().dictionaries)).toBeUndefined();

    useWorkspaceStore.getState().setDictionaryValue("names", "");
    expect(dictionaryPayload(useWorkspaceStore.getState().dictionaries)).toEqual({ names: "" });

    useWorkspaceStore.getState().resetDictionary("names");
    expect(dictionaryPayload(useWorkspaceStore.getState().dictionaries)).toBeUndefined();
    expect(useWorkspaceStore.getState().dictionaries.names.value).toContain("Dược Lão");

    useWorkspaceStore
      .getState()
      .setDictionaryValue("names", "萧炎=Tiêu Viêm bản sửa");
    expect(dictionaryPayload(useWorkspaceStore.getState().dictionaries)).toEqual({
      names: "萧炎=Tiêu Viêm bản sửa",
    });
  });

  it("loads a fully mapped sample without persisting it", () => {
    useWorkspaceStore.getState().loadSample();
    const state = useWorkspaceStore.getState();
    expect(state.sourceText).toContain("萧炎");
    expect(state.response?.sourceRanges).toHaveLength(8);
    const payload = dictionaryPayload(state.dictionaries);
    expect(payload?.names).toContain("Tiêu Viêm");
    expect(payload?.pronouns).toBe("她=nàng");
  });

  it("persists accepted names into Names2 and removes them when rejected", () => {
    useWorkspaceStore.getState().acceptNameCandidate("萧炎", "Tiêu Viêm");
    let state = useWorkspaceStore.getState();
    expect(state.knownNames).toEqual({ 萧炎: "Tiêu Viêm" });
    expect(state.dictionaries.names2.value).toBe("萧炎=Tiêu Viêm");
    expect(dictionaryPayload(state.dictionaries)).toEqual({ names2: "萧炎=Tiêu Viêm" });

    useWorkspaceStore.getState().rejectNameCandidate("萧炎");
    state = useWorkspaceStore.getState();
    expect(state.knownNames).toEqual({});
    expect(state.rejectedNames).toContain("萧炎");
    expect(state.dictionaries.names2.value).toBe("");
    expect(dictionaryPayload(state.dictionaries)).toBeUndefined();
  });

  it("isolates accepted and rejected names by book memory id", () => {
    useWorkspaceStore.getState().acceptNameCandidate("萧炎", "Tiêu Viêm");
    useWorkspaceStore.getState().switchNameMemory("book-b");
    let state = useWorkspaceStore.getState();
    expect(state.knownNames).toEqual({});
    expect(state.dictionaries.names2.value).toBe("");

    useWorkspaceStore.getState().acceptNameCandidate("林动", "Lâm Động");
    useWorkspaceStore.getState().switchNameMemory("default");
    state = useWorkspaceStore.getState();
    expect(state.knownNames).toEqual({ 萧炎: "Tiêu Viêm" });
    expect(state.dictionaries.names2.value).toBe("萧炎=Tiêu Viêm");
    expect(state.dictionaries.names2.value).not.toContain("林动");
  });
});
