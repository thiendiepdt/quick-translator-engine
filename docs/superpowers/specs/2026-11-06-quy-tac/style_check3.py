import collections, json, re, sys, unicodedata
sys.stdout.reconfigure(encoding="utf-8")
N = r"[零〇一二两三四五六七八九十百千万亿0-9]+"
CHECKS = {
    "百分之": (rf"百分之{N}", {"%": r"\d+ ?%", "phần trăm chữ": r"\w+ phần trăm", "phần trăm số": r"\d+ phần trăm"}),
    "小数点": (rf"{N}点[零〇一二三四五六七八九]{{1,2}}(?![刻分钟点])", {"phẩy": r"\bphẩy\b", "chấm": r"\bchấm\b", "rưỡi": r"rưỡi"}),
    "年 (năm lịch)": (r"[零〇一二三四五六七八九0-9]{4}年", {"năm + số": r"năm \d{4}", "năm + chữ": r"năm (hai|một) (ngàn|nghìn)"}),
    "N点 (giờ)": (rf"{N}点(?:钟|半|整|{N}分)", {"chữ giờ": r"(một|hai|ba|bốn|năm|sáu|bảy|tám|chín|mười|mươi) giờ", "số giờ": r"\d+ giờ"}),
}
counts = {k: collections.Counter() for k in CHECKS}
total = collections.Counter()
for path in sys.argv[1].split(","):
    for line in open(path, encoding="utf-8"):
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
    if n:
        print(f"{key:<14} n={n:<5}", ", ".join(f"{l} {c*100/n:.0f}%" for l, c in counts[key].most_common()))
