import collections, json, re, sys, unicodedata
sys.stdout.reconfigure(encoding="utf-8")
NUM = "[零〇一二两三四五六七八九十百千万亿\d]+"
CHECKS = {
    "第一次": ("第一次", {"lần đầu": r"lần đầu", "lần thứ nhất": r"lần thứ nhất"}),
    "第一个/位/名": ("第一(个|位|名|道|颗|只)", {"đầu tiên": r"đầu tiên", "thứ nhất": r"thứ nhất"}),
    "第二天": ("第二天", {"hôm sau": r"hôm sau|hôm thứ hai", "ngày thứ hai": r"ngày thứ hai"}),
    "品灵石": ("[上中下极]品灵石", {"linh thạch X phẩm": r"linh thạch (thượng|trung|hạ|cực) phẩm", "X phẩm linh thạch": r"(thượng|trung|hạ|cực) phẩm linh thạch"}),
    "公斤": (rf"{NUM}公斤", {"kg": r"\bkg\b", "cân": r"\bcân\b", "ký": r"\bký\b|\bkí\b", "ki-lô-gam": r"ki-?lô-?gam|kilôgam|kilogram"}),
    "吨": (rf"{NUM}吨", {"tấn": r"\btấn\b"}),
    "斤": (rf"(?<!公){NUM}斤", {"cân": r"\bcân\b", "ký": r"\bký\b|\bkí\b"}),
    "厘米": (rf"{NUM}厘米", {"cm": r"\bcm\b", "phân": r"\bphân\b", "xen-ti-mét": r"xen-?ti-?mét|centimet"}),
    "平方米": (rf"{NUM}平方米|{NUM}平米", {"m²": r"m²|m2", "mét vuông": r"mét vuông"}),
    "亩": (rf"{NUM}亩", {"mẫu": r"\bmẫu\b"}),
    "块钱": (rf"{NUM}块钱", {"đồng": r"\bđồng\b", "tệ": r"\btệ\b"}),
    "万元": (rf"{NUM}万元", {"vạn tệ": r"vạn tệ", "vạn": r"\bvạn\b", "triệu": r"triệu"}),
    "两银子": (rf"{NUM}两(银子|白银|纹银)", {"lượng bạc": r"lượng bạc", "lạng bạc": r"lạng bạc"}),
    "倍": (rf"{NUM}倍", {"gấp N lần": r"gấp \w+( \w+)? lần", "N lần": r"(?<!gấp )\b\w+ lần"}),
    "点一刻": (rf"{NUM}点(一|三|两)刻", {"khắc": r"khắc", "mười lăm": r"mười lăm|bốn mươi lăm|rưỡi"}),
    "小数点": (rf"{NUM}点[零〇一二三四五六七八九]{{1,2}}(?![刻分钟点])", {"dấu chấm/phẩy": r"\d+[.,]\d+", "chấm/phẩy chữ": r"\b(chấm|phẩy)\b"}),
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
        print(f"{key:<12} n={n:<5}", ", ".join(f"{l} {c*100/n:.0f}%" for l, c in counts[key].most_common()))
