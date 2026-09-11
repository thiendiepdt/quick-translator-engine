import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { runExport } from "../src/commands/export.ts";
import { runInit } from "../src/commands/init.ts";
import { loadState, saveState, storyPaths } from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

/** Dựng truyện 4 chương: 1, 2, 4 done (có out/), 3 skipped. */
function storyWithOutputs(): string {
  const root = makeStoryDir({ "0001": "一", "0002": "二", "0003": "三", "0004": "四" });
  runInit(root);
  const paths = storyPaths(root);
  mkdirSync(paths.outDir, { recursive: true });
  const state = loadState(paths);
  for (const [id, text] of [["0001", "Chương một.\n"], ["0002", "Chương hai.\n\n"], ["0004", "Chương bốn."]] as const) {
    writeFileSync(join(paths.outDir, `${id}.txt`), text, "utf8");
    state.chapters[id] = { status: "done", reviewRound: 0, updatedAt: 1 };
  }
  state.chapters["0003"] = { status: "skipped", reviewRound: 0, reason: "thử", updatedAt: 1 };
  saveState(paths, state);
  return root;
}

describe("qt-ai export", () => {
  it("mặc định gộp từ chương done đầu tới cuối, cách nhau đúng 1 dòng trống, báo chương hổng", () => {
    const root = storyWithOutputs();
    const result = runExport(root);
    expect(result.ids).toEqual(["0001", "0002", "0004"]);
    expect(result.gaps).toEqual(["0003"]);
    expect(result.outPath).toBe(join(root, "export", "0001-0004.txt"));
    expect(readFileSync(result.outPath, "utf8")).toBe("Chương một.\n\nChương hai.\n\nChương bốn.\n");
  });

  it("--from/--to lọc theo thứ tự tự nhiên, --out đặt file ra chỗ khác", () => {
    const root = storyWithOutputs();
    const out = join(root, "custom", "tap1.txt");
    const result = runExport(root, { from: "0002", to: "0004", out });
    expect(result.ids).toEqual(["0002", "0004"]);
    expect(existsSync(out)).toBe(true);
    expect(readFileSync(out, "utf8")).toBe("Chương hai.\n\nChương bốn.\n");
  });

  it("báo lỗi rõ khi chưa có chương done, khoảng ngược, hoặc id lạ", () => {
    const empty = makeStoryDir({ "0001": "一" });
    runInit(empty);
    expect(() => runExport(empty)).toThrow(/done/);
    const root = storyWithOutputs();
    expect(() => runExport(root, { from: "0004", to: "0001" })).toThrow(/ngược/);
    expect(() => runExport(root, { from: "9999" })).toThrow(/9999/);
  });
});
