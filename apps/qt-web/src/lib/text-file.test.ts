import { describe, expect, it } from "vitest";

import { decodeChapterBytes, looksLikeTextFile, readChapterFile } from "@/lib/text-file";

describe("looksLikeTextFile", () => {
  it("accepts text MIME types and falls back to the extension", () => {
    expect(looksLikeTextFile(new File([""], "a.txt", { type: "text/plain" }))).toBe(true);
    expect(looksLikeTextFile(new File([""], "a.txt", { type: "" }))).toBe(true);
    expect(looksLikeTextFile(new File([""], "a.exe", { type: "" }))).toBe(false);
    expect(looksLikeTextFile(new File([""], "a.png", { type: "image/png" }))).toBe(false);
  });
});

describe("decodeChapterBytes", () => {
  it("decodes UTF-8 with and without BOM", () => {
    expect(decodeChapterBytes(new TextEncoder().encode("萧炎好"))).toBe("萧炎好");
    const bom = new Uint8Array([0xef, 0xbb, 0xbf, ...new TextEncoder().encode("好")]);
    expect(decodeChapterBytes(bom)).toBe("好");
  });

  it("falls back to GB18030 for GBK novel files", () => {
    // "你好" mã hóa GBK: C4 E3 BA C3 — không phải UTF-8 hợp lệ.
    expect(decodeChapterBytes(new Uint8Array([0xc4, 0xe3, 0xba, 0xc3]))).toBe("你好");
  });

  it("honors UTF-16 byte order marks", () => {
    // "好" = U+597D → LE: 7D 59, BE: 59 7D.
    expect(decodeChapterBytes(new Uint8Array([0xff, 0xfe, 0x7d, 0x59]))).toBe("好");
    expect(decodeChapterBytes(new Uint8Array([0xfe, 0xff, 0x59, 0x7d]))).toBe("好");
  });
});

describe("readChapterFile", () => {
  it("reads a GBK file into the right text", async () => {
    const file = new File([new Uint8Array([0xc4, 0xe3, 0xba, 0xc3])], "chuong.txt", {
      type: "text/plain",
    });
    await expect(readChapterFile(file)).resolves.toBe("你好");
  });

  it("rejects files over the size limit", async () => {
    const file = new File([new Uint8Array(4 * 1024 * 1024 + 1)], "novel.txt", {
      type: "text/plain",
    });
    await expect(readChapterFile(file)).rejects.toThrow("File quá 4 MB");
  });
});
