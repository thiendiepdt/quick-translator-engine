"""Chọn nghĩa theo ngữ cảnh còn bao nhiêu chỗ: VietPhrase có nhiều nghĩa, nghĩa nào hay có trong bản biên tập.

Chỉ xét nghĩa có sẵn trong VietPhrase (giữ giọng convert). Tách theo từ loại token kế tiếp.
Ngoài ra đếm token có nghĩa không ở dạng NFC.
"""
import collections
import json
import re
import sys
import unicodedata

from qtlib import Pos

sys.stdout.reconfigure(encoding="utf-8")
tokens_path, qt_dir, top = sys.argv[1], sys.argv[2], int(sys.argv[3])
pos = Pos(qt_dir + "/Resources/dict.txt")


def nfc(s):
    return unicodedata.normalize("NFC", s)


alts = {}
with open(qt_dir + "/VietPhrase/VietPhrase.txt", encoding="utf-8-sig") as fh:
    for line in fh:
        k, sep, v = line.rstrip("\r\n").partition("=")
        if sep and k not in alts and 1 <= len(k) <= 4:
            seen, items = set(), []
            for a in v.split("/"):
                a = " ".join(re.findall(r"\w+", nfc(a).lower()))
                if a and a not in seen:
                    seen.add(a)
                    items.append(a)
            if len(items) >= 2:
                alts[k] = items


def ctx(tok):
    if not tok or not re.search(r"[一-鿿]", tok):
        return "dấu/hết"
    t = pos.phrase(tok)
    return {"v": "trước động từ", "n": "trước danh từ", "r": "trước đại từ", "a": "trước tính từ"}.get(t[0], "khác")


hits = collections.defaultdict(lambda: collections.defaultdict(collections.Counter))
count = collections.Counter()
non_nfc = total = 0
with open(tokens_path, encoding="utf-8") as fh:
    for line in fh:
        row = json.loads(line)
        ref = " " + " ".join(re.findall(r"\w+", nfc(row["ref"]).lower())) + " "
        toks = row["tok"]
        for i, (zh, vi) in enumerate(toks):
            total += 1
            if vi != nfc(vi):
                non_nfc += 1
            items = alts.get(zh)
            if not items:
                continue
            c = ctx(toks[i + 1][0] if i + 1 < len(toks) else "")
            count[zh] += 1
            for a in items[:6]:
                if f" {a} " in ref:
                    hits[zh][c][a] += 1
                    hits[zh]["tất cả"][a] += 1

print(f"token có nghĩa không phải NFC: {non_nfc}/{total} ({non_nfc * 100 / total:.2f}%)")
gain = []
for zh, n in count.items():
    if n < 80:
        continue
    allc = hits[zh]["tất cả"]
    first = alts[zh][0]
    best, best_n = max(((a, allc[a]) for a in alts[zh][:6]), key=lambda x: x[1])
    if best != first:
        gain.append((best_n - allc[first], zh, n))
gain.sort(reverse=True)
print(f"token nhiều nghĩa gặp >= 80 lần mà nghĩa khác hay có trong bản biên tập hơn nghĩa đầu: {len(gain)}")
print(f"tổng chênh (lần có mặt, nghĩa tốt nhất trừ nghĩa đầu): {sum(g for g, _, _ in gain)}")
for g, zh, n in gain[:top]:
    first = alts[zh][0]
    parts = []
    for c in ("tất cả", "trước động từ", "trước danh từ", "trước đại từ", "dấu/hết"):
        cc = hits[zh][c]
        if not cc:
            continue
        ranked = sorted(cc.items(), key=lambda kv: -kv[1])[:2]
        parts.append(f"{c}: " + ", ".join(f"{a}={k}" for a, k in ranked))
    print(f"{zh} (n={n}, nghĩa đầu '{first}' có {hits[zh]['tất cả'][first]}) | " + " | ".join(parts))
