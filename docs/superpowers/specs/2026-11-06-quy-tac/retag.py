"""Hạ nhãn tên riêng giả của jieba: nghĩa VietPhrase viết thường thì là từ thường.

Tính từ nhận bằng dấu hiệu 很X / 非常X... trên raw. Không dùng bản dịch tham chiếu.
"""
import collections
import json
import re
import sys

sys.stdout.reconfigure(encoding="utf-8")
tokens_paths, pos_path, vp_path, out_path = sys.argv[1].split(","), sys.argv[2], sys.argv[3], sys.argv[4]

table = {}
with open(pos_path, encoding="utf-8") as fh:
    for line in fh:
        parts = line.split()
        if len(parts) == 3:
            table[parts[0]] = parts[2]

first = {}
with open(vp_path, encoding="utf-8-sig") as fh:
    for line in fh:
        key, sep, value = line.rstrip("\r\n").partition("=")
        if sep and key not in first:
            first[key] = value.split("/")[0].strip()

PROPER = {"ns", "nr", "nrt", "nrfg", "nt", "nz"}
candidates = {w for w, t in table.items() if t in PROPER and first.get(w, "")[:1].islower() and len(w) >= 2}
print(f"nhãn riêng mà nghĩa viết thường: {len(candidates)}")

ADV = ("很", "非常", "十分", "太", "最", "更", "极", "好", "真", "多么", "如此", "那么", "这么", "特别", "相当", "格外", "无比", "有点", "有些", "更加", "比较")
pattern = re.compile("(" + "|".join(ADV) + ")([一-鿿]{2,3})")
adj_hits = collections.Counter()
seen = collections.Counter()
for path in tokens_paths:
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            text = "".join(z for z, _ in json.loads(line)["tok"])
            for m in pattern.finditer(text):
                tail = m.group(2)
                for length in (2, 3):
                    word = tail[:length]
                    if word in candidates:
                        adj_hits[word] += 1
            for length in (2, 3):
                for i in range(len(text) - length + 1):
                    word = text[i:i + length]
                    if word in candidates:
                        seen[word] += 1

fixes = {}
for word in candidates:
    if seen[word] == 0:
        continue
    fixes[word] = "a" if adj_hits[word] >= 2 and adj_hits[word] / seen[word] >= 0.04 else "n"
with open(out_path, "w", encoding="utf-8") as fh:
    for word, tag in sorted(fixes.items(), key=lambda kv: -seen[kv[0]]):
        fh.write(f"{word}\t{tag}\n")
top = sorted(fixes.items(), key=lambda kv: -seen[kv[0]])
print(f"sửa {len(fixes)} từ, trong đó tính từ {sum(1 for _, t in fixes.items() if t == 'a')}")
print("tính từ:", [w for w, t in top if t == "a"][:40])
print("danh từ:", [w for w, t in top if t == "n"][:40])
