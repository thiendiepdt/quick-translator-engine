"""Chuyển LuatNhan.txt sang cú pháp quy tắc mới. {n}/{0} -> {người}; {s} -> {số} (nhóm [số]).

Dùng: migrate_ln.py <LuatNhan.txt> -> in hai nhóm ra stdout.
"""
import itertools
import re
import sys

sys.stdout.reconfigure(encoding="utf-8")


def expand(segment):
    """Khai triển regex-lite của Luật Nhân thành danh sách chuỗi."""
    parts = []
    i = 0
    while i < len(segment):
        ch = segment[i]
        if ch == "[":
            end = segment.index("]", i)
            choices = list(segment[i + 1:end])
            i = end + 1
        elif ch == "(":
            end = segment.index(")", i)
            choices = segment[i + 1:end].split("|")
            i = end + 1
        else:
            choices = [ch]
            i += 1
        if i < len(segment) and segment[i] == "?":
            choices = choices + [""]
            i += 1
        parts.append(choices)
    return ["".join(combo) for combo in itertools.product(*parts)]


def literal(segment):
    alts = sorted(set(expand(segment)), key=len, reverse=True)
    optional = "" in alts
    alts = [a for a in alts if a]
    if not alts:
        return ""
    text = alts[0] if len(alts) == 1 else "(" + "|".join(alts) + ")"
    return text + ("?" if optional else "")


def convert(key, value):
    key = key.replace("{0}", "{n}")
    value = value.replace("{0}", "{n}")
    left, _, right = key.partition("{n}")
    parts = [p for p in (literal(left), "{người}", literal(right)) if p]
    return " ".join(parts) + " = " + value.replace("{n}", "{người}")


n_rules = []
s_rules = []
with open(sys.argv[1], encoding="utf-8-sig") as fh:
    for line in fh:
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        if "{n}" in key or "{0}" in key:
            if "{s}" not in key:
                n_rules.append(convert(key, value))
        elif "{s}" in key and key != "{s}":
            s_rules.append(key.replace("{s}", "{số}") + " = " + value.replace("{s}", "{1}") + " -> m")

print("[luật-cũ 1]")
print("\n".join(n_rules))
print("\n# luật số:", len(s_rules))
