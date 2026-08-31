import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { runCheck } from "../src/commands/check.ts";
import { runInit } from "../src/commands/init.ts";
import { runNext } from "../src/commands/next.ts";
import { loadState, storyPaths, workFile } from "../src/story-fs.ts";
import { makeStoryDir } from "./helpers.ts";

const RAW = "赵静文抬头看向远方的高塔。\n\n她沉默了很久没有说话。";

function storyWithDraft(draft: string): string {
  const root = makeStoryDir({ "0001": RAW });
  runInit(root);
  runNext(root);
  writeFileSync(workFile(storyPaths(root), "0001", "draft"), draft, "utf8");
  return root;
}

describe("qt-ai check", () => {
  it("pass khi đủ đoạn, sạch rule, đủ dài", () => {
    const root = storyWithDraft(
      "[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn về phía tòa tháp cao nơi xa.\n\n[[2]] Nàng im lặng hồi lâu không nói lời nào.",
    );
    const result = runCheck(root, "0001");
    expect(result.pass).toBe(true);
    expect(result.missing).toEqual([]);
    expect(result.violations).toEqual([]);
    expect(loadState(storyPaths(root)).chapters["0001"]?.status).toBe("translating");
    const report = JSON.parse(readFileSync(workFile(storyPaths(root), "0001", "check"), "utf8"));
    expect(report.pass).toBe(true);
  });

  it("bắt thiếu đoạn + vi phạm rule, sinh review.md, tăng reviewRound", () => {
    // [[2]] thiếu; [[1]] chứa Hán tự sót (rule bắt buộc) → 2 loại lỗi cùng lúc
    const root = storyWithDraft("[[1]] Triệu Tĩnh Văn ngẩng đầu nhìn 高塔 nơi xa.");
    const result = runCheck(root, "0001");
    expect(result.pass).toBe(false);
    expect(result.missing).toEqual([2]);
    expect(result.violations.some((v) => v.message.includes("CJK"))).toBe(true);
    expect(result.reviewPath && existsSync(result.reviewPath)).toBe(true);
    const review = readFileSync(result.reviewPath!, "utf8");
    expect(review).toContain("[[2]] 她沉默了很久没有说话。");   // repair payload đoạn thiếu
    expect(review).toContain("CJK");                            // danh sách vi phạm
    expect(review).toMatch(/\[\[1\]\][^\n]*CJK/);                // vi phạm gắn đúng nhãn đoạn [[1]]
    expect(review).not.toContain("Chỉ xuất toàn bộ text đã soát"); // không còn nhồi prompt review cũ (F1)
    expect(review).toContain("0001.draft.md");                  // chỉ agent sửa tại chỗ trong draft
    expect(review).toMatch(/GIỮ NGUYÊN.*nhãn/i);                 // giữ nhãn [[n]], không viết lại đoạn khác
    expect(loadState(storyPaths(root)).chapters["0001"]?.reviewRound).toBe(1);
  });

  it("dịch quá ngắn thì fail theo minLengthRatio", () => {
    const root = storyWithDraft("[[1]] Nàng nhìn.\n\n[[2]] Nàng im.");
    const result = runCheck(root, "0001");
    expect(result.pass).toBe(false);
    expect(result.ratio).toBeLessThan(0.75);
  });

  it("quá maxReviewRounds thì chuyển error", () => {
    const root = storyWithDraft("[[1]] 高塔");
    runCheck(root, "0001"); // round 1
    runCheck(root, "0001"); // round 2
    const result = runCheck(root, "0001"); // hết lượt
    expect(result.escalatedToError).toBe(true);
    expect(loadState(storyPaths(root)).chapters["0001"]?.status).toBe("error");
  });

  it("draft mất sạch nhãn → coi như thiếu toàn bộ", () => {
    const root = storyWithDraft("Bản dịch không có nhãn nào cả.");
    const result = runCheck(root, "0001");
    expect(result.missing).toEqual([1, 2]);
  });
});
