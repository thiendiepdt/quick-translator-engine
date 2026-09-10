import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { runInit } from "../src/commands/init.ts";
import { runNext } from "../src/commands/next.ts";
import { retryBackupPath, runRetry } from "../src/commands/retry.ts";
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

  it("translating (phiên chết, chương kẹt) cũng về queued; chặn queued và chương lạ", () => {
    const root = makeStoryDir({ "0001": "第一章" });
    runInit(root);
    expect(() => runRetry(root, "0001")).toThrow(/queued sẵn/);
    runNext(root);
    runRetry(root, "0001");
    expect(loadState(storyPaths(root)).chapters["0001"]?.status).toBe("queued");
    expect(() => runRetry(root, "9999")).toThrow(/Không có chương/);
  });

  it("chương done: out/<id>.txt đổi thành .bak (đè bak cũ), về queued, next phát lại", () => {
    const root = makeStoryDir({ "0001": "第一章" });
    runInit(root);
    const paths = storyPaths(root);
    const state = loadState(paths);
    state.chapters["0001"] = { status: "done", reviewRound: 1, updatedAt: 1 };
    saveState(paths, state);
    const out = join(paths.outDir, "0001.txt");
    writeFileSync(out, "bản dịch cũ", "utf8");

    runRetry(root, "0001");
    expect(loadState(paths).chapters["0001"]?.status).toBe("queued");
    expect(existsSync(out)).toBe(false);
    expect(readFileSync(retryBackupPath(paths, "0001"), "utf8")).toBe("bản dịch cũ");
    expect(runNext(root).chapterId).toBe("0001");

    const again = loadState(paths);
    again.chapters["0001"] = { status: "done", reviewRound: 0, updatedAt: 2 };
    saveState(paths, again);
    writeFileSync(out, "bản dịch mới", "utf8");
    runRetry(root, "0001");
    expect(readFileSync(retryBackupPath(paths, "0001"), "utf8")).toBe("bản dịch mới");
    expect(readdirSync(paths.outDir)).toEqual(["0001.txt.bak"]);
  });
});
