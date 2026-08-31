import { readFileSync, writeFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { buildAiTranslationSystemPrompt } from "@/lib/ai-translation";
import { runAccept } from "../src/commands/accept.ts";
import { runCheck } from "../src/commands/check.ts";
import { runInit } from "../src/commands/init.ts";
import { runNext } from "../src/commands/next.ts";
import { runStatus } from "../src/commands/status.ts";
import { loadStoryConfig, readRawChapter, storyPaths, workFile } from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

const CH1 = "赵静文抬头看向远方的高塔。\n\n她沉默了很久没有说话。";
const CH2 = "第二天早上，赵静文和他们一起出发了。";
const DRAFTS: Record<string, string> = {
  "0001":
    "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn về phía tòa tháp cao nơi xa.\n\n[[2]] Nàng im lặng hồi lâu không nói lời nào.",
  "0002": "[[1]] Sáng sớm hôm sau, Triệu Tĩnh Văn cùng bọn họ lên đường xuất phát.",
};

describe("e2e: hai chương liên tiếp, glossary học từ chương 1 lọt vào prompt chương 2", () => {
  it("chạy trọn vòng next→draft→check→accept cho cả truyện", () => {
    const root = makeStoryDir({ "0001": CH1, "0002": CH2 });
    runInit(root);
    const paths = storyPaths(root);

    for (const id of ["0001", "0002"]) {
      const next = runNext(root);
      expect(next.chapterId).toBe(id);
      if (id === "0002") {
        // glossary học được từ chương 1 (赵静文 → Triệu Tĩnh Văn, merge lúc accept
        // 0001 ở vòng lặp trước) phải lọt vào prompt của chương 2, vì CH2 cũng
        // nhắc tới 赵静文 nên filterTranslationGlossaryForSource không lọc bỏ.
        const prompt2 = readFileSync(next.promptPath, "utf8");
        expect(prompt2).toContain("Triệu Tĩnh Văn");
      }
      writeFileSync(workFile(paths, id, "draft"), DRAFTS[id]!, "utf8");
      if (id === "0001") {
        writeFileSync(
          workFile(paths, id, "glossary"),
          JSON.stringify({ entries: [{ source: "赵静文", target: "Triệu Tĩnh Văn", category: "names" }] }),
          "utf8",
        );
      }
      expect(runCheck(root, id).pass).toBe(true);
      runAccept(root, id);
    }

    expect(runStatus(root)).toContain("done: 2");
    expect(readFileSync(`${paths.outDir}/0001.md`, "utf8")).toContain("Triệu Tĩnh Văn");
    expect(loadStoryConfig(paths).glossary.names["赵静文"]).toBe("Triệu Tĩnh Văn");
    expect(() => runNext(root)).toThrow(/không còn chương/i);
  });

  it("prompt parity: prompt CLI lắp = buildAiTranslationSystemPrompt của web với cùng config", () => {
    const root = makeStoryDir({ "0001": CH1 });
    runInit(root);
    const paths = storyPaths(root);
    const next = runNext(root);
    const cliPrompt = readFileSync(next.promptPath, "utf8");
    const webPrompt = buildAiTranslationSystemPrompt(
      {}, loadStoryConfig(paths), readRawChapter(paths, "0001"),
    );
    expect(cliPrompt.startsWith(webPrompt)).toBe(true); // phần system y hệt web, CLI chỉ nối thêm payload + chỉ dẫn
  });
});
