"""Chuyển rules.txt (cú pháp <n>/<y>/<L>, {0}) sang nhóm [số] của QuyTac, rồi dựng file đo.

Dùng: convert_rules.py rules.txt QuyTac.txt
Ra: theirs_A.txt (số Ả Rập như bản gốc), theirs_B.txt (giữ kiểu chữ của nguồn),
    QT_A.txt / QT_B.txt (bộ của mình + bộ của họ chèn vào [số]),
    QT_only_theirs.txt (bỏ [số] của mình, thay bằng bộ của họ, bản A).
"""
import pathlib
import re
import sys

sys.stdout.reconfigure(encoding="utf-8")
src, ours_path = sys.argv[1], sys.argv[2]

TOKEN = re.compile(r"<(n|y|L)(?::(\d+)-(\d+))?>")


def convert(line, arabic):
    lhs, _, rhs = line.partition(" = ")
    kinds = []

    def tok(m):
        kind, lo, hi = m.group(1), m.group(2), m.group(3)
        kinds.append(kind)
        if kind == "L":
            return "{nhãn-chương}"
        name = "số" if kind == "n" else "năm"
        return "{" + name + (f":{lo}-{hi}" if lo else "") + "}"

    lhs = TOKEN.sub(tok, lhs.strip())

    def ref(m):
        idx = int(m.group(1))
        kind = kinds[idx] if idx < len(kinds) else "n"
        filt = "|số" if arabic and kind == "n" else ""
        return "{" + str(idx + 1) + filt + "}"

    rhs = re.sub(r"\{(\d+)\}", ref, rhs.strip())
    return f"{lhs} = {rhs} -> m"


rules = []
for raw in open(src, encoding="utf-8-sig"):
    line = raw.strip()
    if not line or line.startswith("#") or " = " not in line:
        continue
    rules.append(line)
print(f"{len(rules)} luật trong rules.txt")

ours = pathlib.Path(ours_path).read_text(encoding="utf-8")
cls = "nhãn-chương: 章=Chương 卷=Quyển 集=Tập 节=Tiết 幕=Màn 回=Hồi 折=Chiết\n"
ours = ours.replace("[lớp]\n", "[lớp]\n" + cls, 1)
head, sep, tail = ours.partition("\n[số]\n")
num_end = tail.index("\n[lượng")
our_num = tail[:num_end]
rest = tail[num_end:]

for label, arabic in (("A", True), ("B", False)):
    converted = [convert(r, arabic) for r in rules]
    pathlib.Path(f"theirs_{label}.txt").write_text("\n".join(converted) + "\n", encoding="utf-8")
    body = our_num.rstrip("\n") + "\n\n# --- rules.txt ---\n" + "\n".join(converted) + "\n"
    pathlib.Path(f"QT_{label}.txt").write_text(head + sep + body + rest, encoding="utf-8")
    if arabic:
        only = "\n".join(converted) + "\n"
        pathlib.Path("QT_only_theirs.txt").write_text(head + sep + only + rest, encoding="utf-8")
pathlib.Path("QT_ours.txt").write_text(ours, encoding="utf-8")
print("\n".join(convert(r, True) for r in rules[:6]))
