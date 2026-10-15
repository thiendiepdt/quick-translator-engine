import { describe, expect, it } from "vitest";

import { CHANGELOG_ENTRIES, compareVersions, entriesSince, entryFor, inlineTokens, parseChangelog, releasedEntries } from "@/lib/changelog";

const SAMPLE = `# Có gì mới

## Chưa phát hành
- Thêm **EPUB**.

## 1.0.10 — 2026-09-19
- Mười.

## 1.0.9 — 2026-09-18
- Chín.
ghi chú lạc

## 1.0.2 — 2026-09-11
- Hai.
- Hai b.
`;

describe("parseChangelog", () => {
  it("cùng ngữ nghĩa với scripts/changelog.mjs: Chưa phát hành có version/date null, bỏ dòng không phải bullet", () => {
    expect(parseChangelog(SAMPLE)).toEqual([
      { version: null, date: null, items: ["Thêm **EPUB**."] },
      { version: "1.0.10", date: "2026-09-19", items: ["Mười."] },
      { version: "1.0.9", date: "2026-09-18", items: ["Chín."] },
      { version: "1.0.2", date: "2026-09-11", items: ["Hai.", "Hai b."] },
    ]);
    expect(parseChangelog(SAMPLE.replace(/\n/g, "\r\n"))).toEqual(parseChangelog(SAMPLE));
  });

  it("đọc được CHANGELOG.md thật (nhúng qua ?raw), có bản 1.0.21", () => {
    const entry = CHANGELOG_ENTRIES.find((e) => e.version === "1.0.21");
    expect(entry?.date).toBe("2026-10-13");
    expect(entry?.items.length).toBeGreaterThan(0);
    expect(entryFor("1.0.0")?.items.length).toBeGreaterThan(0);
  });
});

describe("compareVersions", () => {
  it("so theo số, không theo chữ", () => {
    expect(compareVersions("1.0.10", "1.0.9")).toBeGreaterThan(0);
    expect(compareVersions("1.0.9", "1.0.10")).toBeLessThan(0);
    expect(compareVersions("2.0.0", "1.99.99")).toBeGreaterThan(0);
    expect(compareVersions("1.0.21", "1.0.21")).toBe(0);
  });
});

describe("releasedEntries / entryFor", () => {
  const entries = parseChangelog(SAMPLE);

  it("chỉ các bản ≤ bản đang dùng, bỏ Chưa phát hành, mới nhất trước", () => {
    expect(releasedEntries("1.0.9", entries).map((e) => e.version)).toEqual(["1.0.9", "1.0.2"]);
    expect(releasedEntries("1.0.10", entries).map((e) => e.version)).toEqual(["1.0.10", "1.0.9", "1.0.2"]);
    expect(releasedEntries("1.0.5", entries).map((e) => e.version)).toEqual(["1.0.2"]);
  });

  it("entryFor tìm đúng bản, không có thì undefined; không bao giờ trả Chưa phát hành", () => {
    expect(entryFor("1.0.9", entries)?.items).toEqual(["Chín."]);
    expect(entryFor("9.9.9", entries)).toBeUndefined();
  });
});

describe("entriesSince", () => {
  const entries = parseChangelog(SAMPLE);

  it("các bản sau `since` tới bản đang dùng, mới nhất trước", () => {
    expect(entriesSince("1.0.10", "1.0.2", entries).map((e) => e.version)).toEqual(["1.0.10", "1.0.9"]);
    expect(entriesSince("1.0.10", "1.0.9", entries).map((e) => e.version)).toEqual(["1.0.10"]);
    expect(entriesSince("1.0.9", "1.0.10", entries)).toEqual([]);
  });

  it("since null (cập nhật từ bản không rõ) thì chỉ bản đang dùng", () => {
    expect(entriesSince("1.0.10", null, entries).map((e) => e.version)).toEqual(["1.0.10"]);
    expect(entriesSince("9.9.9", null, entries)).toEqual([]);
  });
});

describe("inlineTokens", () => {
  it("tách **đậm** và `code`, phần còn lại là chữ thường", () => {
    expect(inlineTokens("Thêm **Tìm & thay** (`Ctrl+Shift+F`) nhé")).toEqual([
      { kind: "text", text: "Thêm " },
      { kind: "bold", text: "Tìm & thay" },
      { kind: "text", text: " (" },
      { kind: "code", text: "Ctrl+Shift+F" },
      { kind: "text", text: ") nhé" },
    ]);
  });

  it("dấu ** lẻ hoặc chữ không đánh dấu giữ nguyên là chữ", () => {
    expect(inlineTokens("a ** b <b>x</b>")).toEqual([{ kind: "text", text: "a ** b <b>x</b>" }]);
    expect(inlineTokens("")).toEqual([]);
  });
});
