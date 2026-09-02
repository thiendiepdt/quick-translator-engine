import { beforeEach, describe, expect, it } from "vitest";

import { useStoryStore } from "@/store/story";

beforeEach(() => {
  useStoryStore.setState({ session: { status: "idle" }, logs: [], progress: undefined });
});

describe("applySessionEvent", () => {
  it("started → running kèm số phiên", () => {
    useStoryStore.getState().applySessionEvent({ type: "started", session_no: 2 });
    expect(useStoryStore.getState().session).toEqual({ status: "running", sessionNo: 2 });
  });

  it("progress cập nhật, agy_log nối vào ring buffer 2000 dòng", () => {
    const store = useStoryStore.getState();
    store.applySessionEvent({
      type: "progress",
      done: 3,
      queued: 1,
      translating: 1,
      error: 0,
      skipped: 0,
      warnings_count: 1,
      current: "0005",
    });
    expect(useStoryStore.getState().progress?.current).toBe("0005");
    for (let i = 0; i < 2100; i += 1) {
      useStoryStore.getState().applySessionEvent({ type: "agy_log", line: `l${i}`, stream: "stdout" });
    }
    const logs = useStoryStore.getState().logs;
    expect(logs).toHaveLength(2000);
    expect(logs[0]?.line).toBe("l100");
    expect(logs[1999]?.seq).toBeGreaterThan(logs[0]?.seq ?? Number.MAX_SAFE_INTEGER);
  });

  it("stopped → stopped kèm lý do", () => {
    useStoryStore.getState().applySessionEvent({ type: "stopped", kind: "agy_failed", code: 3 });
    expect(useStoryStore.getState().session).toEqual({
      status: "stopped",
      reason: { kind: "agy_failed", code: 3 },
    });
  });

  it("openStory/closeStory đổi màn hình và reset chọn", () => {
    const snapshot = {
      root: "D:\\t",
      chapters: [],
      counts: { total: 0, queued: 0, translating: 0, done: 0, error: 0, skipped: 0, withWarnings: 0 },
      settings: { minLengthRatio: 0.75, maxReviewRounds: 3, chaptersPerSession: 10 },
      story: {
        name: "",
        sourceUrl: "",
        protagonist: "",
        summary: "",
        glossary: { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {} },
        style: { voice: "", toneRules: [], signaturePhrases: {}, avoid: [] },
        customPrompt: "",
        checkRules: [],
        autoGlossaryLog: [],
        autoGlossary: "inherit" as const,
      },
      sessionRunning: true,
    };
    useStoryStore.getState().openStory(snapshot);
    const state = useStoryStore.getState();
    expect(state.screen).toBe("workbench");
    expect(state.root).toBe("D:\\t");
    expect(state.session.status).toBe("running"); // sessionRunning từ Rust → running
    useStoryStore.getState().closeStory();
    expect(useStoryStore.getState().screen).toBe("picker");
    expect(useStoryStore.getState().snapshot).toBeUndefined();
  });
});
