"""Số cuối trên bộ test + ca tốt nhất / tệ nhất so với convert hiện tại."""
import collections, json, random, sys
from qtlib import Pos, grams, names_of, nfc
from quytac import Program, apply_chars, apply_program, compile_char_rules, prepare, render
sys.stdout.reconfigure(encoding="utf-8")
import qtlib, quytac
QT = 'D:/Work/projects/quick-translator-engine/QT2025'
qtlib.load_base_names(QT + '/Names.txt')
quytac.load_lexicon(QT + '/VietPhrase/VietPhrase.txt')
off_path, on_path, pos_path, books, rules_path, out_path = sys.argv[1:7]
pos = Pos(pos_path)
prog = Program(rules_path)
prog.sections = [s for s in prog.sections if s.name != "số"]
char_rules = compile_char_rules(prog, rules_path)
by_id = {r.id: r for s in prog.sections for r in s.rules}
by_id.update({r.id: r for r in char_rules})
current = {}
with open(on_path, encoding="utf-8") as b:
    for lb in b:
        rb = json.loads(lb)
        current.setdefault((rb["b"], rb["ref"]), []).append(rb)
tot = {"off": [0, 0], "cur": [0, 0], "new": [0, 0]}
rows = changed = better = worse = 0
samples = []
group_fire = collections.Counter()
with open(off_path, encoding="utf-8") as a:
    for la in a:
        ra = json.loads(la)
        stack = current.get((ra["b"], ra["ref"]))
        if not stack:
            continue
        rb = stack.pop()
        names = names_of(books, ra["b"])
        quytac.STORY = qtlib.story_names(books, ra["b"])
        ref = nfc(ra["ref"])
        r2, r3 = grams(ref, 2), grams(ref, 3)
        off = render([quytac.Tok(z, v, "x") for z, v in ra["tok"]])
        cur = render([quytac.Tok(z, v, "x") for z, v in rb["tok"]])
        chars, ctrace = apply_chars(prog, char_rules, ra["tok"])
        toks, trace = apply_program(prog, prepare(chars, pos, names, prog.classes))
        new = render(toks)
        for rid in trace + ctrace:
            group_fire[by_id[rid].text.split("]")[0][1:]] += 1
        rows += 1
        sc = {}
        for key, text in (("off", off), ("cur", cur), ("new", new)):
            g2 = sum((grams(text, 2) & r2).values())
            g3 = sum((grams(text, 3) & r3).values())
            tot[key][0] += g2
            tot[key][1] += g3
            sc[key] = g2 + g3
        if new != cur:
            changed += 1
            d = sc["new"] - sc["cur"]
            better += d > 0
            worse += d < 0
            if 15 < len(cur) < 130:
                samples.append((d, ra["b"], cur, new, ra["ref"]))
c, n, o = tot["cur"], tot["new"], tot["off"]
print(f"{rows} đoạn; đổi {changed} ({changed*100/rows:.1f}%); tốt hơn {better}, kém hơn {worse}, hoà {changed-better-worse}; thắng {better*100/max(better+worse,1):.1f}%")
print(f"cụm 2 từ: tắt LN {o[0]}, hiện tại {c[0]}, mới {n[0]} ({(n[0]-c[0])*100/c[0]:+.2f}%)")
print(f"cụm 3 từ: tắt LN {o[1]}, hiện tại {c[1]}, mới {n[1]} ({(n[1]-c[1])*100/c[1]:+.2f}%)")
print("số lần chạy theo nhóm:", group_fire.most_common())
print("tổng quy tắc:", len(by_id))
samples.sort(key=lambda x: x[0])
rng = random.Random(9)
with open(out_path, "w", encoding="utf-8") as fh:
    fh.write("=== TỆ NHẤT\n")
    for d, b, cur, new, ref in samples[:40]:
        fh.write(f"[{d:+d}] {b}\ncũ : {cur}\nmới: {new}\nref: {ref}\n\n")
    fh.write("=== NGẪU NHIÊN TRONG NHÓM KÉM\n")
    bad = [s for s in samples if s[0] < 0]
    for d, b, cur, new, ref in rng.sample(bad, min(25, len(bad))):
        fh.write(f"[{d:+d}] {b}\ncũ : {cur}\nmới: {new}\nref: {ref}\n\n")
    fh.write("=== TỐT NHẤT\n")
    for d, b, cur, new, ref in samples[-30:]:
        fh.write(f"[{d:+d}] {b}\ncũ : {cur}\nmới: {new}\nref: {ref}\n\n")
