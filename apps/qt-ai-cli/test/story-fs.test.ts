import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { emptyAiStoryConfig } from "@/lib/ai-story";
import {
  defaultSettings, listRawChapterIds, loadState, loadStoryConfig,
  saveState, saveStoryConfig, storyPaths, workFile,
} from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

describe("story-fs", () => {
  it("liệt kê chương theo thứ tự tự nhiên", () => {
    const root = makeStoryDir({ "10": "十", "2": "二", "1": "一" });
    expect(listRawChapterIds(storyPaths(root))).toEqual(["1", "2", "10"]);
  });

  it("save/load story atomic kèm .bak", () => {
    const root = makeStoryDir({});
    const paths = storyPaths(root);
    const config = emptyAiStoryConfig();
    saveStoryConfig(paths, config);                    // lần đầu chưa có .bak
    expect(existsSync(`${paths.storyJson}.bak`)).toBe(false);
    saveStoryConfig(paths, { ...config, name: "Truyện A" });
    expect(loadStoryConfig(paths).name).toBe("Truyện A");
    expect(JSON.parse(readFileSync(`${paths.storyJson}.bak`, "utf8")).name).toBe("");
  });

  it("story.json hỏng thì throw chứ không trả config rỗng", () => {
    const root = makeStoryDir({});
    const paths = storyPaths(root);
    writeFileSync(paths.storyJson, "{hỏng", "utf8");
    expect(() => loadStoryConfig(paths)).toThrow(/story\.json/);
  });

  it("state round-trip và validate", () => {
    const root = makeStoryDir({ "1": "一" });
    const paths = storyPaths(root);
    const state = {
      version: 1 as const,
      settings: defaultSettings(),
      chapters: { "1": { status: "queued" as const, reviewRound: 0, updatedAt: 1 } },
    };
    saveState(paths, state);
    expect(loadState(paths)).toEqual(state);
    writeFileSync(paths.stateJson, "[]", "utf8");
    expect(() => loadState(paths)).toThrow(/state\.json/);
  });

  it("đặt tên work files đúng quy ước", () => {
    const paths = storyPaths("/x");
    expect(workFile(paths, "0001", "prompt")).toBe("/x/work/0001.prompt.md");
    expect(workFile(paths, "0001", "glossary")).toBe("/x/work/0001.glossary.json");
  });
});
