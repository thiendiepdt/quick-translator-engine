import { beforeEach, describe, expect, it } from "vitest";

import { dictionaryPayload, useWorkspaceStore } from "@/store/workspace";

beforeEach(() => {
  useWorkspaceStore.getState().clearWorkspace();
});

describe("workspace dictionary semantics", () => {
  it("omits untouched dictionaries and preserves an explicit empty override", () => {
    expect(dictionaryPayload(useWorkspaceStore.getState().dictionaries)).toBeUndefined();

    useWorkspaceStore.getState().setDictionaryValue("names", "");
    expect(dictionaryPayload(useWorkspaceStore.getState().dictionaries)).toEqual({ names: "" });

    useWorkspaceStore.getState().resetDictionary("names");
    expect(dictionaryPayload(useWorkspaceStore.getState().dictionaries)).toBeUndefined();
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
});
