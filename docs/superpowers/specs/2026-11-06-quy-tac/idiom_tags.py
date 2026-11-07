"""Thành ngữ 4 chữ (Chinese-Names-Corpus, Apache-2.0) đang bị jieba gắn danh từ / tên riêng -> gắn lại 'i'."""
import glob, os, re, sys
from qtlib import Pos
sys.stdout.reconfigure(encoding="utf-8")
corpus_dir, dict_path, out = sys.argv[1:4]
pos = Pos(dict_path)
idioms = set()
for path in glob.glob(os.path.join(corpus_dir, "*ChengYu*")):
    for line in open(path, encoding="utf-8-sig", errors="replace"):
        w = line.strip()
        if re.fullmatch(r"[\u4e00-\u9fff]{4}", w):
            idioms.add(w)
WRONG = {"nr", "nrt", "nrfg", "ns", "nt", "nz", "n", "?"}
fixed = [w for w in sorted(idioms) if pos.table.get(w, "?") in WRONG]
with open(out, "w", encoding="utf-8") as fh:
    for w in fixed:
        fh.write(f"{w}\ti\n")
print(len(idioms), "thành ngữ;", len(fixed), "gắn lại i; ví dụ:", fixed[:15])
