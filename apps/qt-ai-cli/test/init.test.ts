import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { runInit } from "../src/commands/init.ts";
import { loadState, loadStoryConfig, storyPaths } from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

describe("qt-ai init", () => {
  it("dựng story.json rỗng, state queued cho từng chương, copy template", () => {
    const root = makeStoryDir({ "0001": "第一章", "0002": "第二章" });
    runInit(root);
    const paths = storyPaths(root);
    expect(loadStoryConfig(paths).glossary.names).toEqual({});
    const state = loadState(paths);
    expect(Object.keys(state.chapters)).toEqual(["0001", "0002"]);
    expect(state.chapters["0001"]?.status).toBe("queued");
    expect(state.settings.maxReviewRounds).toBe(2);
    const agents = readFileSync(join(root, "AGENTS.md"), "utf8");
    expect(agents).not.toContain("{{QT_AI}}");
    expect(agents).toContain(root);
    expect(existsSync(join(root, ".agent/workflows/translate.md"))).toBe(true);
  });

  it("idempotent: giữ state cũ, thêm chương mới, không đè AGENTS.md", () => {
    const root = makeStoryDir({ "0001": "第一章" });
    runInit(root);
    const paths = storyPaths(root);
    const state = loadState(paths);
    state.chapters["0001"] = { status: "done", reviewRound: 1, updatedAt: 5 };
    writeFileSync(paths.stateJson, JSON.stringify(state), "utf8");
    writeFileSync(join(root, "AGENTS.md"), "tự sửa", "utf8");
    writeFileSync(join(paths.rawDir, "0002.txt"), "第二章", "utf8");
    runInit(root);
    const after = loadState(paths);
    expect(after.chapters["0001"]?.status).toBe("done");
    expect(after.chapters["0002"]?.status).toBe("queued");
    expect(readFileSync(join(root, "AGENTS.md"), "utf8")).toBe("tự sửa");
  });

  it("template đầy đủ: có vòng lặp translate và luật vệ sinh context", () => {
    const root = makeStoryDir({ "0001": "第一章" });
    runInit(root);
    const translate = readFileSync(join(root, ".agent/workflows/translate.md"), "utf8");
    expect(translate).toContain("next");
    expect(translate).toContain("check");
    expect(translate).toContain("accept");
    expect(translate).toContain("skip");
    const agents = readFileSync(join(root, "AGENTS.md"), "utf8");
    expect(agents).toContain("không đọc lại out/");
    expect(agents).toContain("chương/phiên");
  });
});
