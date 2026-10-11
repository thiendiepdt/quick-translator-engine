import { describe, expect, it } from "vitest";

import { chapterLabel, latestAddressing } from "@/lib/cast-addressing";
import { castSchema } from "@/lib/schema";

describe("cast-addressing", () => {
  it("nhãn chương lấy số thứ tự đầu id", () => {
    expect(chapterLabel("0150_7412272672542835262")).toBe("150");
    expect(chapterLabel("10021482_1040_第一千零四十章")).toBe("1040");
    expect(chapterLabel("chuong-0392")).toBe("392");
    expect(chapterLabel("0000_x")).toBe("0");
    expect(chapterLabel("prologue")).toBe("prologue");
  });

  it("lấy mốc muộn nhất của mỗi cặp, bỏ cặp chưa có mốc", () => {
    const cast = castSchema.parse({
      addressing: {
        "莫衡→贺静昭": { changes: [{ from: "0300", target: "chồng–vợ" }, { from: "0150", target: "anh–em" }] },
        "宋时安→贺静昭": { pinned: true, changes: [] },
      },
    });
    expect(latestAddressing(cast)).toEqual({ "莫衡→贺静昭": { target: "chồng–vợ", from: "0300" } });
  });
});
