"""Danh sách tên / thành ngữ ngoài có thêm được gì cho bước gắn nhãn, đo trên token dev."""
import collections
import glob
import json
import os
import re
import sys

import qtlib
from qtlib import Pos

sys.stdout.reconfigure(encoding="utf-8")
corpus_dir, tokens_path, qt_dir, books = sys.argv[1:5]
CJK = re.compile(r"^[一-鿿]{2,6}$")


def load(pattern):
    words = set()
    for path in glob.glob(os.path.join(corpus_dir, pattern)):
        with open(path, encoding="utf-8-sig", errors="replace") as fh:
            for line in fh:
                w = line.strip().split("|")[0].split(",")[0]
                if CJK.match(w):
                    words.add(w)
    return words


idioms = load("*ChengYu*")
names = {
    "tên Trung": load("Chinese_Names_Corpus__Chinese_Names_Corpus120W.txt"),
    "tên cổ": load("*Ancient*"),
    "tên phiên âm": load("English_Names_Corpus__English_Cn_Name_Corpus48W.txt"),
    "tên Nhật": load("*Japanese*"),
}
print("thành ngữ", len(idioms), {k: len(v) for k, v in names.items()})
pos = Pos(qt_dir + "/Resources/dict.txt")
qtlib.load_base_names(qt_dir + "/Names.txt")
first = {}
with open(qt_dir + "/VietPhrase/VietPhrase.txt", encoding="utf-8-sig") as fh:
    for line in fh:
        k, sep, v = line.rstrip("\r\n").partition("=")
        if sep and k not in first:
            first[k] = v.split("/")[0].strip()

idiom_hit = collections.Counter()
idiom_tag = collections.Counter()
name_hit = {k: collections.Counter() for k in names}
name_case = {k: collections.Counter() for k in names}
samples = collections.defaultdict(list)
total = 0
with open(tokens_path, encoding="utf-8") as fh:
    for line in fh:
        row = json.loads(line)
        story = qtlib.names_of(books, row["b"])
        for zh, vi in row["tok"]:
            if not CJK.match(zh):
                continue
            total += 1
            tag = "nr" if zh in story else pos.phrase(zh)
            if len(zh) == 4 and zh in idioms:
                idiom_hit["có"] += 1
                idiom_tag["đã là i/l" if tag in ("i", "l") else "chưa"] += 1
                if tag not in ("i", "l") and len(samples["idiom"]) < 30:
                    samples["idiom"].append(f"{zh}({tag})")
            if tag.startswith("nr") or tag in ("ns", "nt", "tên"):
                continue
            for label, words in names.items():
                if zh in words:
                    name_hit[label]["khớp"] += 1
                    meaning = first.get(zh, vi)
                    cap = meaning[:1].isupper()
                    name_case[label]["nghĩa viết hoa" if cap else "nghĩa viết thường"] += 1
                    key = f"{label}-{'hoa' if cap else 'thường'}"
                    if len(samples[key]) < 18:
                        samples[key].append(f"{zh}={meaning}")
print(f"token Hán 2–6 chữ: {total}")
print("thành ngữ 4 chữ khớp:", idiom_hit["có"], dict(idiom_tag))
print("  chưa gắn i/l:", " ".join(samples["idiom"]))
for label in names:
    print(f"{label}: token chưa là tên mà có trong danh sách {name_hit[label]['khớp']}, {dict(name_case[label])}")
    print("   hoa   :", " ".join(samples[f"{label}-hoa"]))
    print("   thường:", " ".join(samples[f"{label}-thường"]))
