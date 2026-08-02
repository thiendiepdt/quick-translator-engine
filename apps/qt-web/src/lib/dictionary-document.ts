import type { DictionaryKey } from "@/lib/types";

export type DictionaryRecordKind = "entry" | "raw";

export interface DictionaryRecord {
  id: string;
  kind: DictionaryRecordKind;
  key: string;
  value: string;
  lineNumber?: number;
}

export interface DictionaryDocument {
  hasBom: boolean;
  lineEnding: "\n" | "\r\n";
  trailingNewline: boolean;
  valueOnly: boolean;
  records: DictionaryRecord[];
}

export interface DictionaryDocumentChanges {
  edits: ReadonlyMap<string, DictionaryRecord>;
  deleted: ReadonlySet<string>;
  added: readonly DictionaryRecord[];
}

export interface DictionaryDocumentStats {
  recordCount: number;
  rawCount: number;
}

export function isValueOnlyDictionary(key: DictionaryKey): boolean {
  return key === "ignoredChinesePhrases";
}

/**
 * Nối nội dung nhập thêm (file/clipboard) vào cuối từ điển: dictionary là
 * danh sách cộng dồn nên nhập thêm không được phép đè phần đang có. Tôn trọng
 * kiểu xuống dòng của file gốc khi phải chèn dấu ngắt.
 */
export function appendDictionaryText(existing: string, incoming: string): string {
  if (!existing.trim()) return incoming;
  const lineEnding = existing.includes("\r\n") ? "\r\n" : "\n";
  const separator = existing.endsWith("\n") ? "" : lineEnding;
  return existing + separator + incoming;
}

export function getDictionaryDocumentStats(
  content: string,
  dictionaryKey: DictionaryKey,
): DictionaryDocumentStats {
  const valueOnly = isValueOnlyDictionary(dictionaryKey);
  const contentStart = content.startsWith("\uFEFF") ? 1 : 0;
  let lineStart = contentStart;
  let recordCount = 0;
  let rawCount = 0;

  while (lineStart < content.length) {
    const newlineIndex = content.indexOf("\n", lineStart);
    const lineEnd = newlineIndex === -1 ? content.length : newlineIndex;
    const contentEnd =
      lineEnd > lineStart && content.charCodeAt(lineEnd - 1) === 13
        ? lineEnd - 1
        : lineEnd;

    if (valueOnly) {
      recordCount += 1;
    } else {
      let equalsCount = 0;
      let firstEquals = -1;
      for (let index = lineStart; index < contentEnd; index += 1) {
        if (content.charCodeAt(index) !== 61) continue;
        if (firstEquals === -1) firstEquals = index;
        equalsCount += 1;
      }
      if (equalsCount === 1 && firstEquals > lineStart) {
        recordCount += 1;
      } else {
        rawCount += 1;
      }
    }

    if (newlineIndex === -1) break;
    lineStart = newlineIndex + 1;
  }

  return { recordCount, rawCount };
}

export function parseDictionaryDocument(
  content: string,
  dictionaryKey: DictionaryKey,
): DictionaryDocument {
  const hasBom = content.startsWith("\uFEFF");
  const body = hasBom ? content.slice(1) : content;
  const lineEnding = body.includes("\r\n") ? "\r\n" : "\n";
  const trailingNewline = body.endsWith("\n");
  const lines = body.length === 0 ? [] : body.split(/\r?\n/);
  if (trailingNewline) lines.pop();

  const valueOnly = isValueOnlyDictionary(dictionaryKey);
  const records = lines.map((line, index): DictionaryRecord => {
    if (valueOnly) {
      return {
        id: `base-${index}`,
        kind: "entry",
        key: "",
        value: line,
        lineNumber: index + 1,
      };
    }

    const firstEquals = line.indexOf("=");
    if (firstEquals > 0 && firstEquals === line.lastIndexOf("=")) {
      return {
        id: `base-${index}`,
        kind: "entry",
        key: line.slice(0, firstEquals),
        value: line.slice(firstEquals + 1),
        lineNumber: index + 1,
      };
    }

    return {
      id: `base-${index}`,
      kind: "raw",
      key: "",
      value: line,
      lineNumber: index + 1,
    };
  });

  return { hasBom, lineEnding, trailingNewline, valueOnly, records };
}

function formatRecord(record: DictionaryRecord, valueOnly: boolean): string {
  if (valueOnly || record.kind === "raw") return record.value;
  return `${record.key}=${record.value}`;
}

export function serializeDictionaryDocument(
  document: DictionaryDocument,
  changes: DictionaryDocumentChanges,
): string {
  const records = [...document.records, ...changes.added]
    .filter((record) => !changes.deleted.has(record.id))
    .map((record) => changes.edits.get(record.id) ?? record);
  const body = records
    .map((record) => formatRecord(record, document.valueOnly))
    .join(document.lineEnding);
  const trailingNewline = document.trailingNewline && records.length > 0
    ? document.lineEnding
    : "";
  return `${document.hasBom ? "\uFEFF" : ""}${body}${trailingNewline}`;
}
