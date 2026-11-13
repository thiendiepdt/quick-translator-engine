import { describe, expect, it } from "vitest";

import { renderName, validateTemplate } from "@/lib/raw";

describe("mẫu tên file raw", () => {
  it("kiểm mẫu cùng luật Rust", () => {
    expect(validateTemplate("{chuong:4}_{chapter-id}")).toBeNull();
    expect(validateTemplate("chuong-{chuong}")).toBeNull();
    expect(validateTemplate("{chapterTitle}")).toMatch(/\{chuong\}/);
    expect(validateTemplate("{chuong}_{foo}")).toMatch(/\{foo\}/);
    expect(validateTemplate("{chuong:0}")).not.toBeNull();
    expect(validateTemplate(" ")).not.toBeNull();
  });

  it("render: đệm số, bỏ ký tự cấm, cắt 120 ký tự", () => {
    expect(renderName("{chuong:4}_{chapter-id}", { index: 5, id: "7445", title: "" })).toBe("0005_7445");
    expect(renderName("{chuong:2}", { index: 123, id: "", title: "" })).toBe("123");
    expect(renderName("{chuong:4} {chapterTitle}", { index: 1, id: "", title: "第1章 何以?:*  为家 " })).toBe("0001 第1章 何以 为家");
    expect(renderName("{chuong:4}_{chapterTitle}...", { index: 1, id: "", title: "a" })).toBe("0001_a");
    expect(Array.from(renderName("{chuong:4}_{chapterTitle}", { index: 1, id: "", title: "长".repeat(300) }))).toHaveLength(120);
  });
});
