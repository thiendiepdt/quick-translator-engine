import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { runAccept } from "../src/commands/accept.ts";
import { runInit } from "../src/commands/init.ts";
import { runNext } from "../src/commands/next.ts";
import { loadState, loadStoryConfig, saveStoryConfig, storyPaths, workFile } from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

const RAW = "赵静文抬头看向远方的高塔。\n\n她沉默了很久没有说话。";
const GOOD_DRAFT =
  "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn về phía tòa tháp cao nơi xa.\n\n[[2]] Nàng im lặng hồi lâu không nói lời nào.";

function readyStory(draft: string, glossaryJson?: string): string {
  const root = makeStoryDir({ "0001": RAW });
  runInit(root);
  runNext(root);
  const paths = storyPaths(root);
  writeFileSync(workFile(paths, "0001", "draft"), draft, "utf8");
  if (glossaryJson !== undefined) {
    writeFileSync(workFile(paths, "0001", "glossary"), glossaryJson, "utf8");
  }
  return root;
}

describe("qt-ai accept", () => {
  it("ghi out sạch nhãn, merge glossary hợp lệ, dọn work, state done", () => {
    const root = readyStory(
      GOOD_DRAFT,
      JSON.stringify({
        entries: [
          { source: "赵静文", target: "Triệu Tĩnh Văn", category: "names" },
          { source: "不在raw", target: "Bịa", category: "names" },          // bị sanitize loại
          { source: "高塔", target: "không có trong dịch", category: "places" }, // bị loại
        ],
      }),
    );
    const result = runAccept(root, "0001");
    const paths = storyPaths(root);
    const out = readFileSync(result.outPath, "utf8");
    expect(out).toContain("Triệu Tĩnh Văn ngẩng đầu");
    expect(out).not.toContain("[[");
    expect(result.addedGlossary).toBe(1);
    const story = loadStoryConfig(paths);
    expect(story.glossary.names["赵静文"]).toBe("Triệu Tĩnh Văn");
    expect(story.autoGlossaryLog).toHaveLength(1);
    expect(story.autoGlossaryLog[0]?.chapter).toBe("0001");
    expect(existsSync(workFile(paths, "0001", "draft"))).toBe(false);
    expect(loadState(paths).chapters["0001"]?.status).toBe("done");
  });

  it("check fail thì từ chối, force thì cho qua", () => {
    const root = readyStory("[[1]] Còn 高塔 sót.");
    expect(() => runAccept(root, "0001")).toThrow(/check/i);
    expect(loadState(storyPaths(root)).chapters["0001"]?.status).not.toBe("done");
    const forced = runAccept(root, "0001", { force: true });
    expect(existsSync(forced.outPath)).toBe(true);
  });

  it("không glossary mới thì KHÔNG ghi đè story.json (giữ field lạ do loadStoryConfig normalize mất)", () => {
    const root = readyStory(GOOD_DRAFT); // không có glossary.json → addedGlossary sẽ = 0
    const paths = storyPaths(root);
    const withExtraField =
      `${JSON.stringify({ ...JSON.parse(readFileSync(paths.storyJson, "utf8")), __unknownField: "seed lạ" }, null, 2)}\n`;
    writeFileSync(paths.storyJson, withExtraField, "utf8");

    const result = runAccept(root, "0001");

    expect(result.addedGlossary).toBe(0);
    const after = readFileSync(paths.storyJson, "utf8");
    expect(after).toBe(withExtraField); // bytes y hệt — không bị saveStoryConfig ghi đè
    expect(after).toContain("__unknownField");
  });

  it("autoGlossary off thì không merge nhưng vẫn accept", () => {
    const root = readyStory(
      GOOD_DRAFT,
      JSON.stringify({ entries: [{ source: "赵静文", target: "Triệu Tĩnh Văn", category: "names" }] }),
    );
    const paths = storyPaths(root);
    const config = loadStoryConfig(paths);
    config.autoGlossary = "off";
    saveStoryConfig(paths, config);
    const result = runAccept(root, "0001");
    expect(result.addedGlossary).toBe(0);
    expect(loadStoryConfig(paths).glossary.names).toEqual({});
  });
});
