import { existsSync, writeFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { runInit } from "../src/commands/init.ts";
import { runNext } from "../src/commands/next.ts";
import { runRetry } from "../src/commands/retry.ts";
import { runSkip } from "../src/commands/skip.ts";
import { loadState, saveState, storyPaths, workFile } from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

describe("qt-ai retry", () => {
  it("chương error về queued, reviewRound 0, xoá reason + work files; next phát lại nó", () => {
    const root = makeStoryDir({ "0001": "第一章", "0002": "第二章" });
    runInit(root);
    const paths = storyPaths(root);
    const state = loadState(paths);
    state.chapters["0001"] = {
      status: "error", reviewRound: 2, reason: "Quá 2 vòng review", updatedAt: 1,
    };
    saveState(paths, state);
    writeFileSync(workFile(paths, "0001", "draft"), "[[1]] nháp cũ", "utf8");

    runRetry(root, "0001");
    const after = loadState(paths).chapters["0001"]!;
    expect(after.status).toBe("queued");
    expect(after.reviewRound).toBe(0);
    expect(after.reason).toBeUndefined();
    expect(existsSync(workFile(paths, "0001", "draft"))).toBe(false);
    expect(runNext(root).chapterId).toBe("0001"); // phát lại đúng chương vừa retry
  });

  it("chương skipped cũng retry được", () => {
    const root = makeStoryDir({ "0001": "第一章" });
    runInit(root);
    runNext(root);
    runSkip(root, "0001", "model từ chối");
    runRetry(root, "0001");
    expect(loadState(storyPaths(root)).chapters["0001"]?.status).toBe("queued");
  });

  it("chặn done/queued/translating với message rõ ràng", () => {
    const root = makeStoryDir({ "0001": "第一章" });
    runInit(root);
    expect(() => runRetry(root, "0001")).toThrow(/queued sẵn/);
    runNext(root);
    expect(() => runRetry(root, "0001")).toThrow(/translating/);
    expect(() => runRetry(root, "9999")).toThrow(/Không có chương/);
  });
});
