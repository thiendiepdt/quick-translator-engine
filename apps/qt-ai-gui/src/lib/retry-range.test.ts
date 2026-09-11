import { describe, expect, it } from "vitest";

import { previewRetryRange } from "@/lib/retry-range";
import type { ChapterRow } from "@/lib/types";

const row = (id: string, status: ChapterRow["status"]): ChapterRow => ({ id, status, reviewRound: 0, reason: null, warnings: [] });
const chapters = [row("0001", "done"), row("0002", "error"), row("0003", "queued"), row("0004", "done")];

describe("previewRetryRange", () => {
  it("đếm chương sẽ dịch lại, số done mất bản cũ, số queued bỏ qua; rỗng = toàn bộ", () => {
    const part = previewRetryRange(chapters, "0001", "0003");
    expect(part.targets.map((c) => c.id)).toEqual(["0001", "0002"]);
    expect(part.done).toBe(1);
    expect(part.queued).toBe(1);
    const all = previewRetryRange(chapters, "", "");
    expect(all.targets.map((c) => c.id)).toEqual(["0001", "0002", "0004"]);
    expect(all.done).toBe(2);
  });

  it("mã lạ hoặc khoảng ngược thì không hợp lệ", () => {
    expect(previewRetryRange(chapters, "0009", "").valid).toBe(false);
    expect(previewRetryRange(chapters, "0004", "0001").valid).toBe(false);
  });
});
