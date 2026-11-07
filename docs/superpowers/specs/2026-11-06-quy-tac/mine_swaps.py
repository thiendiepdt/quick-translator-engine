"""Máy tự đào luật đảo: thử đổi chỗ từng cặp token kề nhau, chấm bằng bản biên tập, gom theo loại cặp.

Loại token: hư từ hay gặp giữ nguyên chữ, còn lại dùng từ loại. Chỉ đổi đúng một cặp mỗi lần,
nên điểm chênh là của riêng phép đổi đó. Chạy trên output đã qua bộ quy tắc hiện tại, để chỉ tìm
cái bộ quy tắc chưa làm.
"""
import collections
import json
import random
import sys

import qtlib
import quytac
from qtlib import Pos, grams, names_of, nfc
from quytac import Program, apply_chars, apply_program, compile_char_rules, prepare, render

sys.stdout.reconfigure(encoding="utf-8")
tokens_path, qt_dir, books, rules_path, limit = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5])
qtlib.load_base_names(qt_dir + "/Names.txt")
quytac.load_lexicon(qt_dir + "/VietPhrase/VietPhrase.txt")
pos = Pos(qt_dir + "/Resources/dict.txt")
prog = Program(rules_path)
prog.sections = [s for s in prog.sections if s.name != "số"]
char_rules = compile_char_rules(prog, rules_path)

rows = [json.loads(l) for l in open(tokens_path, encoding="utf-8")]
random.Random(17).shuffle(rows)
rows = rows[:limit]

freq = collections.Counter()
for row in rows:
    for zh, _ in row["tok"]:
        freq[zh] += 1
LEX_KEEP = {w for w, _ in freq.most_common(400) if len(w) <= 2}


def cls(t):
    if t.tag == "w":
        return None
    if t.zh in LEX_KEEP and t.tag not in ("nr", "n", "v", "a"):
        return t.zh
    tag = t.tag
    if tag.startswith("nr") or tag in ("ns", "nt"):
        return "TÊN"
    return tag.upper()


def sc(text, r2, r3):
    return sum((grams(text, 2) & r2).values()) + sum((grams(text, 3) & r3).values())


stat = collections.defaultdict(lambda: [0, 0, 0])
example = collections.defaultdict(list)
for row in rows:
    names = names_of(books, row["b"])
    quytac.STORY = qtlib.story_names(books, row["b"])
    chars, _ = apply_chars(prog, char_rules, row["tok"])
    toks, _ = apply_program(prog, prepare(chars, pos, names, prog.classes))
    ref = nfc(row["ref"])
    r2, r3 = grams(ref, 2), grams(ref, 3)
    base_text = render(toks)
    base = sc(base_text, r2, r3)
    for i in range(len(toks) - 1):
        a, b = toks[i], toks[i + 1]
        ca, cb = cls(a), cls(b)
        if ca is None or cb is None or not a.vi.strip() or not b.vi.strip():
            continue
        swapped = toks[:i] + [b, a] + toks[i + 2:]
        new_text = render(swapped)
        d = sc(new_text, r2, r3) - base
        key = (ca, cb)
        cell = stat[key]
        cell[0 if d > 0 else 1 if d < 0 else 2] += 1
        if d > 0 and len(example[key]) < 3 and len(base_text) < 90:
            example[key].append(f"{a.zh}|{b.zh}: {base_text}  →  {new_text}")

print(f"{len(rows)} đoạn; loại cặp: {len(stat)}")
cands = []
for key, (w, l, t) in stat.items():
    decided = w + l
    if decided >= 40 and w / decided >= 0.7:
        cands.append((w - l, w, l, t, key))
cands.sort(reverse=True)
print(f"cặp đổi chỗ thắng >= 70% và >= 40 đoạn phân định: {len(cands)}")
for net, w, l, t, key in cands[:40]:
    print(f"{net:>5}  {w:>5}/{l:<5} hoà {t:<6} {key[0]} + {key[1]}")
    for e in example[key][:1]:
        print("        ", e[:170])
