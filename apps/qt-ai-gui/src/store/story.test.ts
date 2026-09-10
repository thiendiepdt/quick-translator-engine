import { beforeEach, describe, expect, it } from "vitest";

import { storySnapshotSchema } from "@/lib/schema";
import type { StorySnapshot } from "@/lib/types";
import {
  isRunning,
  runningRoots,
  selectCurrentLogs,
  selectCurrentProgress,
  selectCurrentSession,
  sessionOf,
  useStoryStore,
} from "@/store/story";

const A = "D:\\lib\\a";
const B = "D:\\lib\\b";

function snapshotFor(root: string, sessionRunning = false): StorySnapshot {
  return storySnapshotSchema.parse({
    root,
    chapters: [],
    counts: { total: 0, queued: 0, translating: 0, done: 0, error: 0, skipped: 0, withWarnings: 0 },
    settings: { minLengthRatio: 0.75, maxReviewRounds: 3, chaptersPerSession: 10 },
    story: {
      name: "",
      sourceUrl: "",
      protagonist: "",
      summary: "",
      genre: { setting: "ancient" as const, names: "han" as const },
      glossary: { names: {}, places: {}, items: {}, creatures: {}, skills: {}, common: {}, signature_phrases: {} },
      style: { voice: "", toneRules: [], signaturePhrases: {}, avoid: [] },
      customPrompt: "",
      checkRules: [],
      autoGlossaryLog: [],
      autoGlossary: "inherit" as const,
    },
    sessionRunning,
  });
}

beforeEach(() => {
  useStoryStore.setState({
    sessions: {},
    progress: {},
    logs: {},
    roots: {},
    names: {},
    opened: [],
    root: undefined,
    snapshot: undefined,
  });
});

describe("applySessionEvent theo truyện", () => {
  it("started/progress/stopped của hai truyện không lẫn nhau; khoá đường dẫn bỏ hoa thường", () => {
    const store = useStoryStore.getState();
    store.applySessionEvent(A, { type: "started", session_no: 2 });
    store.applySessionEvent("d:/lib/b/", { type: "started", session_no: 1 });
    store.applySessionEvent(A, {
      type: "progress",
      done: 3,
      queued: 1,
      translating: 1,
      error: 0,
      skipped: 0,
      warnings_count: 1,
      current: "0005",
    });
    store.applySessionEvent(B, { type: "stopped", kind: "agy_failed", code: 3 });
    const state = useStoryStore.getState();
    expect(sessionOf(state, A)).toEqual({ status: "running", sessionNo: 2 });
    expect(sessionOf(state, "D:\\LIB\\B")).toEqual({ status: "stopped", reason: { kind: "agy_failed", code: 3 } });
    expect(state.progress["d:\\lib\\a"]?.current).toBe("0005");
    expect(state.progress["d:\\lib\\b"]).toBeUndefined();
    expect(isRunning(state, A) && !isRunning(state, B)).toBe(true);
    expect(runningRoots(state)).toEqual([A]);
  });

  it("agy_log nối vào ring buffer 2000 dòng riêng từng truyện; clearLogs chỉ xoá truyện đó", () => {
    for (let i = 0; i < 2100; i += 1) {
      useStoryStore.getState().applySessionEvent(A, { type: "agy_log", line: `l${i}`, stream: "stdout" });
    }
    useStoryStore.getState().applySessionEvent(B, { type: "agy_log", line: "b0", stream: "stderr" });
    const logsA = useStoryStore.getState().logs["d:\\lib\\a"] ?? [];
    expect(logsA).toHaveLength(2000);
    expect(logsA[0]?.line).toBe("l100");
    expect(logsA[1999]?.seq).toBeGreaterThan(logsA[0]?.seq ?? Number.MAX_SAFE_INTEGER);
    expect(useStoryStore.getState().logs["d:\\lib\\b"]).toHaveLength(1);
    useStoryStore.getState().clearLogs(A);
    expect(useStoryStore.getState().logs["d:\\lib\\a"]).toHaveLength(0);
    expect(useStoryStore.getState().logs["d:\\lib\\b"]).toHaveLength(1);
  });

  it("selector hiện tại đọc theo truyện đang mở, trả tham chiếu ổn định khi rỗng", () => {
    useStoryStore.getState().applySessionEvent(A, { type: "started", session_no: 1 });
    useStoryStore.getState().openStory(snapshotFor(B));
    const s1 = useStoryStore.getState();
    expect(selectCurrentSession(s1)).toEqual({ status: "idle" });
    expect(selectCurrentProgress(s1)).toBeUndefined();
    expect(selectCurrentLogs(s1)).toBe(selectCurrentLogs(useStoryStore.getState()));
    // Rust xác nhận đang chạy → giữ nguyên sessionNo đã nhận từ event, không reset về 0.
    useStoryStore.getState().openStory(snapshotFor(A, true));
    expect(selectCurrentSession(useStoryStore.getState())).toEqual({ status: "running", sessionNo: 1 });
  });
});

describe("openStory/closeStory", () => {
  it("đổi màn hình, reset chọn, giữ phiên của truyện khác; sessionRunning từ Rust đồng bộ trạng thái", () => {
    useStoryStore.getState().applySessionEvent(B, { type: "started", session_no: 4 });
    useStoryStore.getState().openStory(snapshotFor(A, true));
    let state = useStoryStore.getState();
    expect(state.screen).toBe("workbench");
    expect(state.root).toBe(A);
    expect(sessionOf(state, A)).toEqual({ status: "running", sessionNo: 0 }); // Rust bảo đang chạy → running
    expect(sessionOf(state, B)).toEqual({ status: "running", sessionNo: 4 }); // truyện khác không bị đụng

    useStoryStore.getState().closeStory();
    state = useStoryStore.getState();
    expect(state.screen).toBe("picker");
    expect(state.snapshot).toBeUndefined();
    expect(runningRoots(state).sort()).toEqual([A, B]); // đóng truyện không xoá phiên

    // Rust bảo không chạy nữa (phiên kết thúc lúc app không nghe) → về idle.
    useStoryStore.getState().openStory(snapshotFor(A, false));
    expect(sessionOf(useStoryStore.getState(), A)).toEqual({ status: "idle" });
  });

  it("switchStory giữ trang đang xem và ghi tên truyện vào names", () => {
    useStoryStore.getState().openStory({ ...snapshotFor(A), story: { ...snapshotFor(A).story, name: "Alpha" } });
    useStoryStore.getState().setPage("story");
    useStoryStore.getState().switchStory({ ...snapshotFor(B), story: { ...snapshotFor(B).story, name: "Bravo" } });
    const state = useStoryStore.getState();
    expect(state.root).toBe(B);
    expect(state.page).toBe("story");
    expect(state.names).toEqual({ "d:\\lib\\a": "Alpha", "d:\\lib\\b": "Bravo" });
    expect(state.opened).toEqual([B, A]); // mới mở nhất đứng đầu
    useStoryStore.getState().switchStory(snapshotFor("d:/lib/a/"));
    expect(useStoryStore.getState().opened).toEqual(["d:/lib/a/", B]); // cùng truyện (khác chữ) không nhân đôi
  });

  it("page mặc định translate, openStory reset về translate, setPage đổi", () => {
    useStoryStore.getState().setPage("settings");
    expect(useStoryStore.getState().page).toBe("settings");
    useStoryStore.getState().setSearchQuery("00");
    useStoryStore.getState().openStory(snapshotFor("D:\\t"));
    expect(useStoryStore.getState().page).toBe("translate");
    expect(useStoryStore.getState().searchQuery).toBe("");
  });

  it("openStory đẩy root lên đầu config.recent như touch_recent bên Rust (khử trùng, tối đa 10)", () => {
    const snapshot = snapshotFor("D:\\moi");
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
      maxParallel: 2,
      recent: ["D:\\a", "D:\\moi", ...Array.from({ length: 9 }, (_, i) => `D:\\cu${i}`)],
      libraryRoot: null,
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

describe("baseVersion", () => {
  it("bumpBaseVersion tăng để hook defaults nạp lại", () => {
    const before = useStoryStore.getState().baseVersion;
    useStoryStore.getState().bumpBaseVersion();
    expect(useStoryStore.getState().baseVersion).toBe(before + 1);
  });
});
