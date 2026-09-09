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
        genre: { setting: "ancient" as const, names: "han" as const },
        glossary: { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {}, addressing: {} },
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

  it("page mặc định translate, openStory reset về translate, setPage đổi", () => {
    useStoryStore.getState().setPage("settings");
    expect(useStoryStore.getState().page).toBe("settings");
    useStoryStore.getState().setSearchQuery("00");
    useStoryStore.getState().openStory({
      root: "D:\\t",
      chapters: [],
      counts: { total: 0, queued: 0, translating: 0, done: 0, error: 0, skipped: 0, withWarnings: 0 },
      settings: { minLengthRatio: 0.75, maxReviewRounds: 3, chaptersPerSession: 10 },
      story: {
        name: "",
        sourceUrl: "",
        protagonist: "",
        summary: "",
        genre: { setting: "ancient", names: "han" },
        glossary: { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {}, addressing: {} },
        style: { voice: "", toneRules: [], signaturePhrases: {}, avoid: [] },
        customPrompt: "",
        checkRules: [],
        autoGlossaryLog: [],
        autoGlossary: "inherit" as const,
      },
      sessionRunning: false,
    });
    expect(useStoryStore.getState().page).toBe("translate");
    expect(useStoryStore.getState().searchQuery).toBe("");
  });

  it("openStory đẩy root lên đầu config.recent như touch_recent bên Rust (khử trùng, tối đa 10)", () => {
    const snapshot = {
      root: "D:\\moi",
      chapters: [],
      counts: { total: 0, queued: 0, translating: 0, done: 0, error: 0, skipped: 0, withWarnings: 0 },
      settings: { minLengthRatio: 0.75, maxReviewRounds: 3, chaptersPerSession: 10 },
      story: {
        name: "",
        sourceUrl: "",
        protagonist: "",
        summary: "",
        genre: { setting: "ancient" as const, names: "han" as const },
        glossary: { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {}, addressing: {} },
        style: { voice: "", toneRules: [], signaturePhrases: {}, avoid: [] },
        customPrompt: "",
        checkRules: [],
        autoGlossaryLog: [],
        autoGlossary: "inherit" as const,
      },
      sessionRunning: false,
    };
    const config = {
      engine: "agy" as const,
      api: {
        provider: "gemini" as const,
        gemini: { apiKey: "", model: "", baseUrl: "" },
        openai: { apiKey: "", model: "", baseUrl: "" },
        thinking: true,
        reasoningEffort: "high" as const,
      },
      agyPath: null,
      model: null,
      maxSessions: 50,
      recent: ["D:\\a", "D:\\moi", ...Array.from({ length: 9 }, (_, i) => `D:\\cu${i}`)],
      palette: "editorial",
      themeMode: "system",
      readingWidth: "normal",
    };
    useStoryStore.getState().setConfig(config);
    useStoryStore.getState().openStory(snapshot);
    const recent = useStoryStore.getState().config?.recent ?? [];
    expect(recent[0]).toBe("D:\\moi");
    expect(recent.filter((r) => r === "D:\\moi")).toHaveLength(1);
    expect(recent).toHaveLength(10);
    expect(recent[1]).toBe("D:\\a");
    // Chưa nạp config (probe chưa xong) thì không được tự bịa config.
    useStoryStore.setState({ config: undefined });
    useStoryStore.getState().openStory(snapshot);
    expect(useStoryStore.getState().config).toBeUndefined();
  });
});
