import { describe, expect, it } from "vitest";

import { idiomRows, matchesIdiom, removeIdiom, setIdiom } from "@/lib/idioms";

const remote = { 打草惊蛇: "đánh rắn động cỏ", 九死一生: "thập tử nhất sinh", 对牛弹琴: "đàn gảy tai trâu" };

describe("bảng thành ngữ", () => {
  it("câu riêng lên đầu, câu chung giữ thứ tự; sửa = đè, rỗng = tắt", () => {
    const user = { 打草惊蛇: "rút dây động rừng", 九死一生: "", 三十年河东: "ba mươi năm hà đông" };
    expect(idiomRows(remote, user).map((row) => [row.key, row.kind, row.value])).toEqual([
      ["三十年河东", "own", "ba mươi năm hà đông"],
      ["打草惊蛇", "override", "rút dây động rừng"],
      ["九死一生", "off", ""],
      ["对牛弹琴", "remote", "đàn gảy tai trâu"],
    ]);
  });

  it("sửa về đúng bảng chung thì bỏ mục đè; xoá mục người dùng", () => {
    const user = setIdiom({}, remote, "打草惊蛇", "rút dây động rừng");
    expect(user).toEqual({ 打草惊蛇: "rút dây động rừng" });
    expect(setIdiom(user, remote, "打草惊蛇", "đánh rắn động cỏ")).toEqual({});
    expect(removeIdiom({ a: "x", b: "y" }, "a")).toEqual({ b: "y" });
  });

  it("tìm theo chữ Hán, cách dịch đang áp hoặc cách dịch của bảng chung", () => {
    const row = idiomRows({ 打草惊蛇: "đánh rắn động cỏ" }, { 打草惊蛇: "rút dây động rừng" })[0];
    expect(matchesIdiom(row, "惊蛇")).toBe(true);
    expect(matchesIdiom(row, "RÚT DÂY")).toBe(true);
    expect(matchesIdiom(row, "đánh rắn")).toBe(true);
    expect(matchesIdiom(row, "tai trâu")).toBe(false);
  });
});
