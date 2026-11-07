"""Chuyển bộ luật số thứ hai trong corpus/rules/ (cú pháp <n>/<y>/<L>/<ne|pn>/<w>) sang QuyTac để đo.

Luật số -> nhóm [số] (quét ký tự). Luật có tên/đại từ -> nhóm token [luật-r2].
Dùng: convert_rules2.py <file luật> <QuyTac nền>
Ra: QT_r2A.txt (nguyên đầu ra, số Ả Rập) và QT_r2B.txt (số giữ kiểu nguồn).
"""
import pathlib
import re
import sys

sys.stdout.reconfigure(encoding="utf-8")
src, base_path = sys.argv[1], sys.argv[2]
TOKEN = re.compile(r"<([a-zA-Z|]+)(?::(\d+)(?:-(\d+))?)?>")
SPECIAL = set(".+*?^$[]\\×")


def esc(text):
    return "".join("\\" + c if c in SPECIAL else c for c in text)


def char_pattern(lhs, kinds):
    """Mẫu ký tự: nhóm (a|b) giữ cấu trúc, chữ thường escape, token thành khoá."""
    out, i = [], 0
    while i < len(lhs):
        m = TOKEN.match(lhs, i)
        if m:
            kind, lo, hi = m.group(1), m.group(2), m.group(3) or m.group(2)
            kinds.append(kind)
            if kind == "L":
                out.append("{nhãn-chương}")
            else:
                name = "số" if kind == "n" else "năm"
                out.append("{" + name + (f":{lo}-{hi}" if lo else "") + "}")
            i = m.end()
            continue
        if lhs[i] == "(":
            end = lhs.index(")", i)
            alts = lhs[i + 1:end].split("|")
            out.append("(?:" + "|".join(esc(a) for a in alts) + ")")
            i = end + 1
            if i < len(lhs) and lhs[i] == "?":
                out.append("?")
                i += 1
            continue
        out.append(esc(lhs[i]))
        i += 1
    return "".join(out)


SLOT = {"ne|pn": "{người}", "ne": "{tên}", "pn": "{đại-từ}", "w": "{cụm}+", "vp": "{cụm}", "ne|pn|vp": "{cụm}+", "hv": None}


def token_pattern(lhs):
    """Mẫu token: chữ giữa các khoá thành một phần tử literal; bỏ luật có nhóm hoặc dấu đặc biệt."""
    parts, i = [], 0
    for m in TOKEN.finditer(lhs):
        lit = lhs[i:m.start()]
        if lit:
            parts.append(lit)
        slot = SLOT.get(m.group(1))
        if slot is None:
            return None
        parts.append(slot)
        i = m.end()
    if lhs[i:]:
        parts.append(lhs[i:])
    for p in parts:
        if not p.startswith("{") and (set(p) & set("()|?!^$ ") or not p):
            return None
    return " ".join(parts)


num_a, num_b, tok = [], [], []
skipped = 0
for raw in open(src, encoding="utf-8-sig"):
    line = raw.strip()
    if not line or line.startswith("#") or " = " not in line:
        continue
    lhs, _, rhs = line.partition(" = ")
    lhs, rhs = lhs.strip(), rhs.strip()
    kinds_found = [m.group(1) for m in TOKEN.finditer(lhs)]
    if any(k not in ("n", "y", "L") for k in kinds_found):
        pat = token_pattern(lhs)
        if pat is None:
            skipped += 1
            continue
        tmpl = re.sub(r"\{(\d+)\}", lambda m: "{" + str(int(m.group(1)) + 1) + "}", rhs)
        tok.append(f"{pat} = {tmpl}")
        continue
    kinds = []
    pat = char_pattern(lhs, kinds)
    for bucket, arabic in ((num_a, True), (num_b, False)):
        def ref(m):
            idx = int(m.group(1))
            kind = kinds[idx] if idx < len(kinds) else "n"
            return "{" + str(idx + 1) + ("|số" if arabic and kind == "n" else "") + "}"
        bucket.append(f"{pat} = {re.sub(r'\{(\d+)\}', ref, rhs)} -> m")

print(f"luật số {len(num_a)}, luật tên/đại từ {len(tok)}, bỏ qua {skipped}")
base = pathlib.Path(base_path).read_text(encoding="utf-8")
if "nhãn-chương:" not in base:
    base = base.replace("[lớp]\n", "[lớp]\nnhãn-chương: 章=Chương 卷=Quyển 集=Tập 节=Tiết 幕=Màn 回=Hồi 折=Chiết\n", 1)
for label, nums in (("A", num_a), ("B", num_b)):
    text = base.replace("\n[lượng", "\n# --- bộ luật số thứ hai ---\n" + "\n".join(nums) + "\n\n[lượng", 1)
    text = text.replace("[của 3]\n", "[luật-r2 1]\n" + "\n".join(tok) + "\n\n[của 3]\n", 1)
    pathlib.Path(f"QT_r2{label}.txt").write_text(text, encoding="utf-8")
