import { existsSync, readFileSync, rmSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { runInit } from "../src/commands/init.ts";
import { runNext } from "../src/commands/next.ts";
import {
  loadState, saveStoryConfig, loadStoryConfig, storyPaths, workFile,
} from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

const RAW = "赵静文抬头。\n\n方寸之间。";

function initStory(chapters: Record<string, string>): string {
  const root = makeStoryDir(chapters);
  runInit(root);
  return root;
}

describe("qt-ai next", () => {
  it("phát chương queued đầu tiên, prompt đủ 3 phần, state → translating", () => {
    const root = initStory({ "0001": RAW, "0002": "第二章" });
    const paths = storyPaths(root);
    const config = loadStoryConfig(paths);
    config.glossary.names["赵静文"] = "Triệu Tĩnh Văn";
    config.glossary.names["不出现"] = "Không Xuất Hiện";
    saveStoryConfig(paths, config);

    const result = runNext(root);
    expect(result.chapterId).toBe("0001");
    const prompt = readFileSync(result.promptPath, "utf8");
    expect(prompt).toContain("dịch giả tiểu thuyết Trung Quốc");   // base prompt
    expect(prompt).toContain("Triệu Tĩnh Văn");                     // glossary đã lọc theo chương
    expect(prompt).not.toContain("Không Xuất Hiện");                // entry không có trong chương bị lọc
    expect(prompt).toContain("[[1]] 赵静文抬头。");                  // payload gắn nhãn
    expect(prompt).toContain("0001.draft.md");                      // chỉ dẫn ghi draft
    expect(prompt).toContain("0001.glossary.json");                 // chỉ dẫn đề xuất glossary
    expect(loadState(paths).chapters["0001"]?.status).toBe("translating");
  });

  it("từ chối phát chương mới khi còn chương translating", () => {
    const root = initStory({ "0001": RAW, "0002": "第二章" });
    runNext(root);
    expect(() => runNext(root)).toThrow(/0001/);
  });

  it("chương translating mất work/prompt.md (session chết) thì next phát lại đúng chương đó", () => {
    const root = initStory({ "0001": RAW, "0002": "第二章" });
    const paths = storyPaths(root);
    runNext(root); // 0001 → translating, ghi work/0001.prompt.md
    const promptPath = workFile(paths, "0001", "prompt");
    expect(existsSync(promptPath)).toBe(true);
    rmSync(promptPath); // giả lập session chết, mất file work/

    const result = runNext(root);
    expect(result.chapterId).toBe("0001");
    expect(existsSync(result.promptPath)).toBe(true);
    const prompt = readFileSync(result.promptPath, "utf8");
    expect(prompt).toContain("[[1]] 赵静文抬头。");
    expect(loadState(paths).chapters["0001"]?.status).toBe("translating");
    expect(loadState(paths).chapters["0001"]?.reviewRound).toBe(0);
  });

  it("hết queued thì báo", () => {
    const root = initStory({});
    expect(() => runNext(root)).toThrow(/không còn chương/i);
  });
});
