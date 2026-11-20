// Sinh crates/qt-ai-core/data/common-words.txt từ QT2025/Resources/idf.txt (IDF của jieba): từ Hán 2–4 chữ có
// IDF < 9,5 là từ thường (一个, 经济, 值班…), không đưa vào gợi ý thuật ngữ lặp (term_hints.rs). Thuật ngữ riêng
// (规培, 玉简) không có trong bảng hoặc IDF cao. Chạy: node apps/qt-ai-cli/scripts/gen-common-words.mjs
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const words = [];
for (const line of fs.readFileSync(path.join(repo, "QT2025/Resources/idf.txt"), "utf8").split(/\r?\n/)) {
  const [word, idf] = line.trim().split(/\s+/);
  if (word && /^\p{Script=Han}{2,4}$/u.test(word) && Number(idf) < 9.5) words.push(word);
}
words.sort();
fs.writeFileSync(path.join(repo, "crates/qt-ai-core/data/common-words.txt"), words.join("\n") + "\n");
console.log(`Đã ghi ${words.length} từ`);
