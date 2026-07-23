import { describe, expect, it } from "vitest";

import {
  getDictionaryDocumentStats,
  parseDictionaryDocument,
  serializeDictionaryDocument,
  type DictionaryRecord,
} from "@/lib/dictionary-document";

describe("dictionary document editor", () => {
  it("parses entries while preserving BOM, CRLF, comments, and trailing newline", () => {
    const content = "\uFEFF萧炎=Tiêu Viêm\r\n# comment\r\ninvalid=a=b\r\n";
    const document = parseDictionaryDocument(content, "names");

    expect(document).toMatchObject({
      hasBom: true,
      lineEnding: "\r\n",
      trailingNewline: true,
      valueOnly: false,
    });
    expect(document.records).toEqual([
      {
        id: "base-0",
        kind: "entry",
        key: "萧炎",
        value: "Tiêu Viêm",
        lineNumber: 1,
      },
      {
        id: "base-1",
        kind: "raw",
        key: "",
        value: "# comment",
        lineNumber: 2,
      },
      {
        id: "base-2",
        kind: "raw",
        key: "",
        value: "invalid=a=b",
        lineNumber: 3,
      },
    ]);
  });

  it("serializes inline edits, deletions, and additions without changing file shape", () => {
    const document = parseDictionaryDocument(
      "\uFEFF萧炎=Tiêu Viêm\r\n药老=Dược Lão\r\n# note\r\n",
      "names",
    );
    const edited: DictionaryRecord = {
      ...document.records[0],
      value: "Tiêu Viêm mới",
    };
    const added: DictionaryRecord = {
      id: "added-1",
      kind: "entry",
      key: "云韵",
      value: "Vân Vận",
    };

    expect(
      serializeDictionaryDocument(document, {
        edits: new Map([[edited.id, edited]]),
        deleted: new Set(["base-1"]),
        added: [added],
      }),
    ).toBe("\uFEFF萧炎=Tiêu Viêm mới\r\n# note\r\n云韵=Vân Vận\r\n");
  });

  it("treats ignored Chinese phrases as value-only rows", () => {
    const document = parseDictionaryDocument("本章完\n（全文完）", "ignoredChinesePhrases");
    expect(document.valueOnly).toBe(true);
    expect(document.records.map(({ key, value }) => ({ key, value }))).toEqual([
      { key: "", value: "本章完" },
      { key: "", value: "（全文完）" },
    ]);
  });

  it("counts records and raw rows without parsing the whole document", () => {
    expect(
      getDictionaryDocumentStats(
        "\uFEFF萧炎=Tiêu Viêm\r\n# comment\r\ninvalid=a=b\r\n药老=Dược Lão\r\n",
        "names",
      ),
    ).toEqual({ recordCount: 2, rawCount: 2 });
    expect(getDictionaryDocumentStats("本章完\n（全文完）\n", "ignoredChinesePhrases"))
      .toEqual({ recordCount: 2, rawCount: 0 });
    expect(getDictionaryDocumentStats("", "names"))
      .toEqual({ recordCount: 0, rawCount: 0 });
  });
});
