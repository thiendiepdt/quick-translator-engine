"""Bản biên tập chọn cách viết nào: đếm ứng viên trong đoạn tham chiếu có đúng một lần mẫu nguồn."""
import collections
import json
import re
import sys
import unicodedata

sys.stdout.reconfigure(encoding="utf-8")
paths = sys.argv[1].split(",")
NUM = "[零〇一二两三四五六七八九十百千万亿\\d]+"
CHECKS = {
    "N块灵石": (rf"{NUM}块灵石", {"N khối linh thạch": r"khối linh thạch", "N viên linh thạch": r"viên linh thạch", "N linh thạch (bỏ lượng từ)": r"(?<!khối )(?<!viên )linh thạch"}),
    "N个小时": (rf"{NUM}个?小时", {"giờ": r"\bgiờ\b(?! đồng)", "tiếng": r"\btiếng\b", "canh giờ": r"canh giờ", "giờ đồng hồ": r"giờ đồng hồ"}),
    "第N层": (rf"第{NUM}层", {"tầng thứ N": r"tầng thứ", "tầng N": r"tầng (?!thứ)\w+"}),
    "N阶": (rf"(?<!第){NUM}阶", {"giai N": r"giai (một|hai|ba|bốn|năm|sáu|bảy|tám|chín|mười|\d)", "N giai": r"(một|hai|ba|bốn|năm|sáu|bảy|tám|chín|mười|\d) giai", "cấp N": r"cấp (một|hai|ba|bốn|năm|sáu|bảy|tám|chín|mười|\d)", "N cấp/bậc": r"(một|hai|ba|bốn|năm|sáu|bảy|tám|chín|mười|\d) (cấp|bậc)", "tam/tứ giai": r"\b(nhất|nhị|tam|tứ|ngũ|lục|thất|bát|cửu) giai"}),
    "N元": (rf"{NUM}(?:万|亿)?元(?!素|婴|神|气|宝|帅|老|旦|首)", {"tệ": r"\btệ\b", "đồng": r"\bđồng\b", "nguyên": r"\bnguyên\b"}),
    "N岁": (rf"{NUM}岁", {"chữ": r"(một|hai|ba|bốn|năm|sáu|bảy|tám|chín|mười|mươi|trăm) tuổi", "số": r"\d+ tuổi"}),
    "N点N分": (rf"{NUM}点{NUM}分", {"giờ phút": r"giờ .*phút", "chữ số": r"\d+ giờ", "chữ": r"(một|hai|ba|bốn|năm|sáu|bảy|tám|chín|mười|mươi) giờ"}),
    "N点半": (rf"{NUM}点半", {"giờ rưỡi": r"giờ rưỡi", "giờ ba mươi": r"giờ ba mươi"}),
    "N倍": (rf"{NUM}倍", {"lần": r"\blần\b", "gấp": r"\bgấp\b"}),
    "N米": (rf"{NUM}米(?!饭|粒|色)", {"mét": r"\bmét\b", "m": r"\d+ ?m\b", "thước": r"\bthước\b"}),
    "N公里": (rf"{NUM}公里", {"km": r"\bkm\b", "cây số": r"cây số", "ki-lô-mét": r"ki-?lô-?mét"}),
    "N天": (rf"(?<![第一有某])({NUM})天", {"chữ ngày": r"(hai|ba|bốn|năm|sáu|bảy|tám|chín|mười|mươi|trăm) ngày", "số ngày": r"\d+ ngày"}),
}

counts = {k: collections.Counter() for k in CHECKS}
total = collections.Counter()
for path in paths:
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            row = json.loads(line)
            zh = "".join(z for z, _ in row["tok"])
            ref = unicodedata.normalize("NFC", row["ref"]).lower()
            for key, (pat, cands) in CHECKS.items():
                if len(re.findall(pat, zh)) != 1:
                    continue
                total[key] += 1
                for label, rx in cands.items():
                    if re.search(rx, ref):
                        counts[key][label] += 1
for key in CHECKS:
    n = total[key]
    if not n:
        continue
    parts = ", ".join(f"{label} {c * 100 / n:.0f}%" for label, c in counts[key].most_common())
    print(f"{key:<10} n={n:<5} {parts}")
