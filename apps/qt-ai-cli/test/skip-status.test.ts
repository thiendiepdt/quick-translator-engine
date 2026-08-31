import { describe, expect, it } from "vitest";
import { runInit } from "../src/commands/init.ts";
import { runNext } from "../src/commands/next.ts";
import { runSkip } from "../src/commands/skip.ts";
import { runStatus } from "../src/commands/status.ts";
import { loadState, storyPaths } from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

describe("qt-ai skip + status", () => {
  it("skip chương translating kèm lý do, next đi tiếp chương sau", () => {
    const root = makeStoryDir({ "0001": "第一章", "0002": "第二章" });
    runInit(root);
    runNext(root);
    runSkip(root, "0001", "model từ chối nội dung");
    const state = loadState(storyPaths(root));
    expect(state.chapters["0001"]?.status).toBe("skipped");
    expect(state.chapters["0001"]?.reason).toBe("model từ chối nội dung");
    expect(runNext(root).chapterId).toBe("0002");
  });

  it("skip đòi reason", () => {
    const root = makeStoryDir({ "0001": "第一章" });
    runInit(root);
    expect(() => runSkip(root, "0001", "  ")).toThrow(/reason|lý do/i);
  });

  it("status tổng hợp đủ trạng thái", () => {
    const root = makeStoryDir({ "0001": "第一章", "0002": "第二章" });
    runInit(root);
    runNext(root);
    runSkip(root, "0001", "thử");
    const report = runStatus(root);
    expect(report).toContain("queued: 1");
    expect(report).toContain("skipped: 1");
    expect(report).toContain("0001");
    expect(report).toContain("thử");
  });
});
