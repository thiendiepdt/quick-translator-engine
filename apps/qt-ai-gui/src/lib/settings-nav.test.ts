import { describe, expect, it } from "vitest";

import { pickActiveSection } from "@/lib/settings-nav";

describe("pickActiveSection", () => {
  it("khối cuối đã chạm đầu khung (lệch ≤ 24px) là khối đang xem; chưa khối nào chạm thì khối đầu", () => {
    expect(pickActiveSection([0, 300, 900], false)).toBe(0);
    expect(pickActiveSection([-500, 10, 400], false)).toBe(1);
    expect(pickActiveSection([-900, -300, 24], false)).toBe(2);
    expect(pickActiveSection([50, 300], false)).toBe(0);
    expect(pickActiveSection([], false)).toBe(0);
  });

  it("cuộn tới đáy thì khối cuối sáng dù khối đó ngắn, không lên được đầu khung", () => {
    expect(pickActiveSection([-900, -300, 200], true)).toBe(2);
  });
});
