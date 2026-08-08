import { beforeEach, describe, expect, it } from "vitest";

import { emptyAiStoryConfig } from "@/lib/ai-story";
import {
  dictionaryUpdateKeys,
  type DictionaryDefaults,
  type LocalDictionaryEntries,
} from "@/lib/types";
import {
  dictionaryPatchPayload,
  dictionaryPayload,
  useWorkspaceStore,
  workspaceStateStorage,
  workspaceStorageKey,
} from "@/store/workspace";

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

async function persistedWorkspace(): Promise<string> {
  return (await workspaceStateStorage.getItem(workspaceStorageKey)) ?? "";
}

beforeEach(() => {
  endpointIndex += 1;
  useWorkspaceStore.setState({
    dictionaryOverrides: {},
    localDictionaryEntries: Object.fromEntries(
      dictionaryUpdateKeys.map((key) => [key, {}]),
    ) as LocalDictionaryEntries,
  });
  useWorkspaceStore
    .getState()
    .hydrateDictionaryDefaults(`/test-${endpointIndex}`, emptyDefaults);
  useWorkspaceStore.getState().clearNameMemory();
  useWorkspaceStore.getState().clearWorkspace();
  useWorkspaceStore.getState().clearAiTranslationChapters();
  useWorkspaceStore.getState().updateAiStory(emptyAiStoryConfig());
});

describe("AI story workspace persistence", () => {
  it("stores sorted chapters, results and story config in IndexedDB", async () => {
    useWorkspaceStore.getState().updateAiStory({
      name: "Đấu Phá Thương Khung",
      protagonist: "Tiêu Viêm",
    });
    useWorkspaceStore.getState().importAiTranslationChapters([
      { filename: "chuong-10.txt", source: "第十章" },
      { filename: "chuong-2.txt", source: "第二章" },
    ]);

    const state = useWorkspaceStore.getState();
    expect(state.aiTranslationChapters.map((chapter) => chapter.filename)).toEqual([
      "chuong-2.txt",
      "chuong-10.txt",
    ]);
    const activeId = state.activeAiTranslationChapterId;
    expect(activeId).toBeTruthy();
    state.updateAiTranslationChapter(activeId!, {
      status: "done",
      output: "Chương hai",
      reviewRound: 1,
    });

    const stored = await persistedWorkspace();
    expect(stored).toContain("Đấu Phá Thương Khung");
    expect(stored).toContain("chuong-10.txt");
    expect(stored).toContain("Chương hai");
  });

  it("migrates v0 state and resumes an interrupted chapter as queued", async () => {
    await workspaceStateStorage.setItem(
      workspaceStorageKey,
      JSON.stringify({
        state: {
          knownNames: {},
          rejectedNames: [],
          rangePinEnabled: true,
          localDictionaryEntries: Object.fromEntries(
            dictionaryUpdateKeys.map((key) => [key, {}]),
          ),
          dictionaryOverrides: {},
          aiStory: { name: "Truyện đang dịch" },
          aiTranslationChapters: [{
            id: "chapter-recover",
            filename: "chuong-1.txt",
            source: "原文",
            output: "Bản dịch đang dở",
            thinking: "",
            violations: [],
            status: "translating",
            reviewRound: 0,
            updatedAt: 1,
          }],
          activeAiTranslationChapterId: "chapter-recover",
        },
        version: 0,
      }),
    );

    await useWorkspaceStore.persist.rehydrate();
    const state = useWorkspaceStore.getState();
    expect(state.aiStory.name).toBe("Truyện đang dịch");
    expect(state.aiTranslationChapters[0]).toMatchObject({
      status: "queued",
      output: "Bản dịch đang dở",
    });
    expect(state.aiTranslationOutput).toBe("Bản dịch đang dở");
  });
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

  it("persists only dictionary overrides created after server hydration", async () => {
    const serverDefault = "萧炎=Tiêu Viêm từ server";
    useWorkspaceStore.getState().hydrateDictionaryDefaults("/override", {
      ...emptyDefaults,
      names: serverDefault,
    });

    expect(useWorkspaceStore.getState().dictionaryOverrides).toEqual({});
    expect(await persistedWorkspace()).not.toContain(serverDefault);

    const customValue = "萧炎=Tiêu Viêm tùy chỉnh";
    useWorkspaceStore.getState().setDictionaryValue("names", customValue);
    expect(useWorkspaceStore.getState().dictionaryOverrides).toEqual({
      names: customValue,
    });
    expect(await persistedWorkspace()).toContain(customValue);

    useWorkspaceStore.getState().clearWorkspace();
    expect(useWorkspaceStore.getState().dictionaries.names.value).toBe(customValue);

    const nextServerDefault = "萧炎=Tiêu Viêm server mới";
    useWorkspaceStore.getState().hydrateDictionaryDefaults("/override-next", {
      ...emptyDefaults,
      names: nextServerDefault,
    });
    expect(useWorkspaceStore.getState().dictionaries.names).toMatchObject({
      value: customValue,
      defaultValue: nextServerDefault,
      touched: true,
    });

    useWorkspaceStore.getState().resetDictionary("names");
    expect(useWorkspaceStore.getState().dictionaries.names.value).toBe(nextServerDefault);
    expect(useWorkspaceStore.getState().dictionaryOverrides).toEqual({});
    expect(await persistedWorkspace()).not.toContain(customValue);
  });

  it("does not persist dictionary drafts before defaults are ready", async () => {
    useWorkspaceStore.setState({
      dictionaryDefaultsEndpoint: undefined,
      dictionaryOverrides: {},
    });

    useWorkspaceStore.getState().setDictionaryValue("names", "draft-before-server");

    expect(useWorkspaceStore.getState().dictionaryOverrides).toEqual({});
    expect(await persistedWorkspace()).not.toContain("draft-before-server");
  });

  it("keeps compact record patches layered above a full dictionary override", () => {
    useWorkspaceStore.getState().hydrateDictionaryDefaults("/layered", {
      ...emptyDefaults,
      names: "萧炎=Server",
    });
    useWorkspaceStore.getState().setDictionaryValue("names", "萧炎=Override");
    useWorkspaceStore
      .getState()
      .saveLocalDictionaryEntries("names", { 萧炎: "Patch" });

    expect(useWorkspaceStore.getState().dictionaries.names.value).toBe("萧炎=Patch");

    useWorkspaceStore.getState().removeLocalDictionaryEntries("names", ["萧炎"]);
    expect(useWorkspaceStore.getState().dictionaries.names.value).toBe(
      "萧炎=Override",
    );
    expect(useWorkspaceStore.getState().dictionaryOverrides.names).toBe(
      "萧炎=Override",
    );
  });

  it("reapplies IndexedDB data when async hydration finishes after server defaults", async () => {
    const serverDefault = "萧炎=Server";
    useWorkspaceStore.getState().hydrateDictionaryDefaults("/async-hydration", {
      ...emptyDefaults,
      names: serverDefault,
    });
    const localDictionaryEntries = Object.fromEntries(
      dictionaryUpdateKeys.map((key) => [key, {}]),
    ) as LocalDictionaryEntries;
    localDictionaryEntries.names = { 萧炎: "Patch" };

    await workspaceStateStorage.setItem(
      workspaceStorageKey,
      JSON.stringify({
        state: {
          dictionaryOverrides: { names: "萧炎=Override" },
          localDictionaryEntries,
        },
        version: 0,
      }),
    );
    await useWorkspaceStore.persist.rehydrate();

    expect(useWorkspaceStore.getState().dictionaries.names).toMatchObject({
      value: "萧炎=Patch",
      defaultValue: serverDefault,
      touched: true,
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

  it("undoes an accepted name without rejecting it", () => {
    useWorkspaceStore.getState().acceptNameCandidate("萧炎", "Tiêu Viêm");
    useWorkspaceStore.getState().undoAcceptedNameCandidate("萧炎");

    const state = useWorkspaceStore.getState();
    expect(state.knownNames).toEqual({});
    expect(state.rejectedNames).not.toContain("萧炎");
    expect(state.dictionaries.names2.value).toBe("");
  });

  it("restores a rejected name to the review queue", () => {
    useWorkspaceStore.getState().rejectNameCandidate("萧炎");
    expect(useWorkspaceStore.getState().rejectedNames).toContain("萧炎");

    useWorkspaceStore.getState().restoreRejectedNameCandidate("萧炎");
    const state = useWorkspaceStore.getState();
    expect(state.rejectedNames).not.toContain("萧炎");
    expect(state.knownNames).toEqual({});
    expect(state.dictionaries.names2.value).toBe("");
  });

  it("restores all rejected names in one update", () => {
    useWorkspaceStore.getState().rejectNameCandidate("萧炎");
    useWorkspaceStore.getState().rejectNameCandidate("药老");
    expect(useWorkspaceStore.getState().rejectedNames).toEqual(["萧炎", "药老"]);

    useWorkspaceStore.getState().restoreAllRejectedNameCandidates();
    const state = useWorkspaceStore.getState();
    expect(state.rejectedNames).toEqual([]);
  });

  it("persists compact VietPhrase and Phiên Âm patches", async () => {
    const store = useWorkspaceStore.getState();
    store.saveLocalDictionaryEntries("vietPhrase", { 看着: "quan sát" });
    store.saveLocalDictionaryEntries("chinesePhienAmWords", {
      看: "khán",
      着: "trứ",
    });

    const entries = useWorkspaceStore.getState().localDictionaryEntries;
    expect(dictionaryPatchPayload(entries)).toEqual({
      vietPhrase: { 看着: "quan sát" },
      chinesePhienAmWords: { 看: "khán", 着: "trứ" },
    });
    expect(await persistedWorkspace()).toContain(
      '"vietPhrase":{"看着":"quan sát"}',
    );
  });

  it("keeps local editable entries across hydration and clear workspace", () => {
    useWorkspaceStore.getState().hydrateDictionaryDefaults("/local-entry", {
      ...emptyDefaults,
      names: "萧炎=Tiêu Viêm",
    });
    useWorkspaceStore
      .getState()
      .saveLocalDictionaryEntries("names", { 萧炎: "Tiêu Viêm mới" });

    useWorkspaceStore.getState().clearWorkspace();
    let state = useWorkspaceStore.getState();
    expect(state.dictionaries.names.value).toBe("萧炎=Tiêu Viêm mới");
    expect(dictionaryPayload(state.dictionaries)).toEqual({
      names: "萧炎=Tiêu Viêm mới",
    });

    useWorkspaceStore
      .getState()
      .hydrateDictionaryDefaults("/local-entry-next", {
        ...emptyDefaults,
        names: "萧炎=Tiêu Viêm",
      });
    state = useWorkspaceStore.getState();
    expect(state.dictionaries.names.value).toBe("萧炎=Tiêu Viêm mới");

    state.removeLocalDictionaryEntries("names", ["萧炎"]);
    state = useWorkspaceStore.getState();
    expect(state.dictionaries.names.value).toBe("萧炎=Tiêu Viêm");
    expect(dictionaryPayload(state.dictionaries)).toBeUndefined();
  });
});
