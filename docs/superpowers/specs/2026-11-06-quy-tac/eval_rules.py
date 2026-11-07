"""Chấm bộ quy tắc: điểm tổng và thắng/thua từng quy tắc (bỏ riêng nó ra rồi so)."""
import collections
import json
import random
import sys

from qtlib import Pos, grams, names_of, nfc, score
from quytac import Program, apply_chars, apply_program, compile_char_rules, prepare, render

sys.stdout.reconfigure(encoding="utf-8")
import qtlib, quytac
QT = 'D:/Work/projects/quick-translator-engine/QT2025'
qtlib.load_base_names(QT + '/Names.txt')
quytac.load_lexicon(QT + '/VietPhrase/VietPhrase.txt')
tokens_path, pos_path, books, rules_path = sys.argv[1:5]
limit = int(sys.argv[5]) if len(sys.argv) > 5 else 0
sample_rule = sys.argv[6] if len(sys.argv) > 6 else ""

pos = Pos(pos_path)
prog = Program(rules_path)
prog.sections = [s for s in prog.sections if s.name != 'số']
char_rules = compile_char_rules(prog, rules_path)
by_id = {r.id: r for s in prog.sections for r in s.rules}
by_id.update({r.id: r for r in char_rules})

rows = []
with open(tokens_path, encoding="utf-8") as fh:
    for line in fh:
        rows.append(json.loads(line))
random.Random(5).shuffle(rows)
if limit:
    rows = rows[:limit]

stat = collections.defaultdict(lambda: [0, 0, 0, 0])  # lần, thắng, thua, hoà
examples = collections.defaultdict(list)
b2 = n2 = b3 = n3 = 0
changed = 0
for row in rows:
    names = names_of(books, row["b"])
    quytac.STORY = qtlib.story_names(books, row["b"])
    plain = prepare(row["tok"], pos, names, prog.classes)
    chars, char_trace = apply_chars(prog, char_rules, row["tok"])
    base = prepare(chars, pos, names, prog.classes)
    ref = nfc(row["ref"])
    r2, r3 = grams(ref, 2), grams(ref, 3)
    base_text = render(plain)
    new, trace = apply_program(prog, list(base))
    trace = trace + char_trace
    new_text = render(new)
    g2 = sum((grams(base_text, 2) & r2).values())
    g3 = sum((grams(base_text, 3) & r3).values())
    h2 = sum((grams(new_text, 2) & r2).values())
    h3 = sum((grams(new_text, 3) & r3).values())
    b2 += g2
    b3 += g3
    n2 += h2
    n3 += h3
    if not trace:
        continue
    changed += new_text != base_text
    full = h2 + h3
    for rid in set(trace):
        if rid >= 100000:
            kept, _ = apply_chars(prog, char_rules, row["tok"], frozenset((rid,)))
            without, _ = apply_program(prog, prepare(kept, pos, names, prog.classes))
        else:
            without, _ = apply_program(prog, list(base), frozenset((rid,)))
        delta = full - score(render(without), r2, r3)
        cell = stat[rid]
        cell[0] += trace.count(rid)
        cell[1 if delta > 0 else 2 if delta < 0 else 3] += 1
        ex = examples[rid]
        if len(ex) < 400 and len(base_text) < 120:
            ex.append((delta, base_text, new_text, row["ref"]))

print(f"{len(rows)} đoạn, đổi {changed} ({changed * 100 / len(rows):.1f}%)")
print(f"cụm 2 từ: {b2} -> {n2} ({(n2 - b2) * 100 / max(b2, 1):+.2f}%)   cụm 3 từ: {b3} -> {n3} ({(n3 - b3) * 100 / max(b3, 1):+.2f}%)")
print("\nlần  thắng thua  hoà  tỉ lệ  quy tắc")
for rid, (count, win, lose, tie) in sorted(stat.items(), key=lambda kv: -kv[1][0]):
    decided = max(win + lose, 1)
    print(f"{count:>6} {win:>5} {lose:>5} {tie:>5} {win * 100 / decided:5.0f}%  {by_id[rid].text}")
unused = [r.text for r in by_id.values() if r.id not in stat]
print(f"\nkhông khớp lần nào: {len(unused)}")
for text in unused[:40]:
    print("   ", text)

if sample_rule:
    rng = random.Random(3)
    for rid, ex in examples.items():
        if sample_rule in by_id[rid].text:
            rng.shuffle(ex)
            print("\n###", by_id[rid].text)
            for d, b, nw, r in [e for e in ex if e[0] < 0][:5] + [e for e in ex if e[0] > 0][:3] + [e for e in ex if e[0] == 0][:2]:
                print(f"  [{d:+d}]\n    cũ : {b}\n    mới: {nw}\n    ref: {r}")
