// Sinh crates/qt-ai-core/data/novel-common.txt: cụm Hán 2–4 chữ "lặp nhiều chương" (cùng ngưỡng với term_hints.rs:
// có ở ≥5% chương, tối thiểu 10) ở nhiều truyện khác nhau của một thư viện mẫu. Cụm truyện mạng nào cũng lặp (看向,
// 回头看, 没来得及) không phải thuật ngữ riêng của truyện; term_hints.rs bỏ chúng khỏi gợi ý.
// Chạy: node apps/qt-ai-cli/scripts/gen-novel-common.mjs <thư mục chứa các truyện> [số truyện tối thiểu, mặc định 8]
// Mỗi truyện lấy tối đa 400 chương rải đều trong raw/. Đo trên 136 truyện (2026-11-20): thuật ngữ hẹp (规培, 主刀,
// 督察局, 熟练度, 武魂) lặp ở 1–6 truyện; từ văn tả (蜷缩, 面庞, 后脑, 做完, 能感觉到) và cụm thường (看向 127, 回头看
// 44) lặp ở ≥8 truyện. Ngưỡng 8 bỏ luôn thuật ngữ thể loại phổ biến (灵石, 玉简, 宿主) — các từ này vốn dịch thống nhất.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repo = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const [library, minArg] = process.argv.slice(2);
if (!library) {
  console.error("Thiếu thư mục thư viện: node gen-novel-common.mjs <thư mục> [số truyện tối thiểu]");
  process.exit(1);
}
const minStories = Number(minArg ?? 8);
const han = /^\p{Script=Han}+$/u;

function grams(text, n) {
  const chars = Array.from(text);
  const out = new Set();
  for (let i = 0; i + n <= chars.length; i++) {
    const gram = chars.slice(i, i + n).join("");
    if (han.test(gram)) out.add(gram);
  }
  return out;
}

const stories = new Map();
let total = 0;
for (const slug of fs.readdirSync(library).sort()) {
  const raw = path.join(library, slug, "raw");
  if (!fs.existsSync(raw) || !fs.statSync(raw).isDirectory()) continue;
  const files = fs.readdirSync(raw).filter((f) => f.endsWith(".txt")).sort();
  if (files.length < 30) continue;
  const step = Math.max(1, Math.floor(files.length / 400));
  const texts = files
    .filter((_, i) => i % step === 0)
    .slice(0, 400)
    .map((f) => fs.readFileSync(path.join(raw, f), "utf8"));
  const threshold = Math.max(10, Math.ceil((texts.length * 5) / 100));
  let previous = new Set();
  const frequent = new Set();
  for (const n of [2, 3, 4]) {
    const df = new Map();
    for (const text of texts) {
      for (const gram of grams(text, n)) {
        if (n > 2) {
          const chars = Array.from(gram);
          if (!previous.has(chars.slice(0, -1).join("")) || !previous.has(chars.slice(1).join(""))) continue;
        }
        df.set(gram, (df.get(gram) ?? 0) + 1);
      }
    }
    previous = new Set([...df].filter(([, count]) => count >= threshold).map(([gram]) => gram));
    for (const gram of previous) frequent.add(gram);
  }
  for (const gram of frequent) stories.set(gram, (stories.get(gram) ?? 0) + 1);
  total++;
}
const words = [...stories].filter(([, count]) => count >= minStories).map(([gram]) => gram).sort();
fs.writeFileSync(path.join(repo, "crates/qt-ai-core/data/novel-common.txt"), words.join("\n") + "\n");
console.log(`Đã ghi ${words.length} cụm (lặp ở ≥${minStories}/${total} truyện)`);
