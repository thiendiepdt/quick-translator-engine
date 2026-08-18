import { describe, expect, it } from "vitest";

import { buildZip, crc32 } from "@/lib/zip";

function readU32(bytes: Uint8Array, offset: number): number {
  return (
    (bytes[offset] | (bytes[offset + 1] << 8) | (bytes[offset + 2] << 16)) +
    bytes[offset + 3] * 0x1000000
  );
}

function readU16(bytes: Uint8Array, offset: number): number {
  return bytes[offset] | (bytes[offset + 1] << 8);
}

describe("crc32", () => {
  it("matches the standard test vector", () => {
    expect(crc32(new TextEncoder().encode("abc"))).toBe(0x352441c2);
    expect(crc32(new Uint8Array())).toBe(0);
  });
});

describe("buildZip", () => {
  it("produces a stored zip whose entries round-trip", () => {
    const bytes = buildZip([
      { name: "chuong-1.txt", content: "Chấn Lôi Tử nhìn về Thái Thanh Sơn.\n" },
      { name: "chương 2.txt", content: "第二章" },
    ]);

    // Local header đầu tiên.
    expect(readU32(bytes, 0)).toBe(0x04034b50);
    // Method 0 (store) — content nằm nguyên văn sau header + tên.
    expect(readU16(bytes, 8)).toBe(0);
    const name1 = new TextEncoder().encode("chuong-1.txt");
    const content1 = new TextEncoder().encode("Chấn Lôi Tử nhìn về Thái Thanh Sơn.\n");
    expect(readU16(bytes, 26)).toBe(name1.length);
    expect([...bytes.slice(30, 30 + name1.length)]).toEqual([...name1]);
    expect([...bytes.slice(30 + name1.length, 30 + name1.length + content1.length)]).toEqual([...content1]);
    // CRC của entry khớp nội dung.
    expect(readU32(bytes, 14)).toBe(crc32(content1));

    // EOCD ở cuối: đúng 2 entry.
    const eocd = bytes.length - 22;
    expect(readU32(bytes, eocd)).toBe(0x06054b50);
    expect(readU16(bytes, eocd + 10)).toBe(2);
  });

  it("is deterministic for the same input", () => {
    const files = [{ name: "a.txt", content: "x" }];
    expect(buildZip(files)).toEqual(buildZip(files));
  });
});
