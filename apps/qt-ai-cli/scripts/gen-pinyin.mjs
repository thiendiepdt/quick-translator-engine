// Sinh crates/qt-ai-core/data/pinyin.tsv từ QT2025/Resources/LacViet.txt: mỗi dòng `chữ<TAB>pinyin1,pinyin2` (bỏ
// thanh, ü → u). Dùng cho cổng "nghi trùng" của glossary (glossary_pending.rs): hai tên chỉ khác một chữ đồng âm.
// Chạy: node apps/qt-ai-cli/scripts/gen-pinyin.mjs
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const text = fs.readFileSync(path.join(repo, "QT2025/Resources/LacViet.txt"), "utf8").replace(/^﻿/, "");
const toneless = (s) => s.normalize("NFD").replace(/[̀-ͯ]/g, "").replace(/ü|v/g, "u").toLowerCase();
const table = new Map();
for (const line of text.split(/\r?\n/)) {
  const eq = line.indexOf("=");
  if (eq < 0) continue;
  const key = line.slice(0, eq);
  if ([...key].length !== 1 || !/\p{Script=Han}/u.test(key)) continue;
  const readings = new Set();
  for (const match of line.matchAll(/✚\s*\[([^\]]+)\]/g)) {
    for (const part of match[1].split(/[,，\s]+/)) {
      const py = toneless(part.trim());
      if (/^[a-z]+$/.test(py)) readings.add(py);
    }
  }
  if (readings.size) table.set(key, [...new Set([...(table.get(key) ?? []), ...readings])]);
}
const lines = [...table].sort(([a], [b]) => (a < b ? -1 : 1)).map(([ch, py]) => `${ch}\t${py.join(",")}`);
fs.writeFileSync(path.join(repo, "crates/qt-ai-core/data/pinyin.tsv"), lines.join("\n") + "\n");
console.log(`Đã ghi ${lines.length} chữ`);
