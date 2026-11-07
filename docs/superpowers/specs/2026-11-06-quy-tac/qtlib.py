"""Phần dùng chung của bản thử: từ loại, số, đo trùng cụm với bản biên tập."""
import collections
import json
import os
import re
import unicodedata

CJK = re.compile(r"[一-鿿]")
WORD = re.compile(r"\w+")
NUM_CHARS = "零〇一二三四五六七八九十百千万亿两"
NUM_ONLY = re.compile(rf"^[{NUM_CHARS}]+$")
DIGITS = re.compile(r"^\d+(?:\.\d+)?$")
PRONOUNS = {"我", "你", "他", "她", "它", "您", "咱", "我们", "你们", "他们", "她们", "它们", "咱们", "大家", "大伙", "自己", "人家", "别人", "对方", "本座", "本王", "在下", "老夫", "朕", "贫道", "小女子", "谁", "彼此", "众人"}
DEMONSTRATIVE = {"这", "那", "此", "该", "这个", "那个", "这些", "那些", "这种", "那种", "这样", "那样", "这位", "那位", "这名", "那名"}
TRAILERS = "了着过吗呢啊吧呀嘛哦哈"


def nfc(text):
    return unicodedata.normalize("NFC", text)


class Pos:
    def __init__(self, path):
        self.table = {}
        self.maxlen = 1
        with open(path, encoding="utf-8") as fh:
            for line in fh:
                parts = line.split()
                if len(parts) == 3:
                    self.table[parts[0]] = parts[2]
                    self.maxlen = max(self.maxlen, len(parts[0]))
        self.cache = {}
        fixes = [os.path.join(os.path.dirname(os.path.abspath(__file__)), "tag_fix.tsv")]
        if os.environ.get("QT_EXTRA_TAGS"):
            fixes.append(os.environ["QT_EXTRA_TAGS"])
        for fix in fixes:
            if not os.path.isfile(fix):
                continue
            with open(fix, encoding="utf-8") as fh:
                for line in fh:
                    word, tag = line.rstrip("\n").split("\t")
                    self.table[word] = tag

    def segment(self, zh):
        out, i = [], 0
        while i < len(zh):
            for length in range(min(self.maxlen, len(zh) - i), 0, -1):
                word = zh[i:i + length]
                if word in self.table:
                    out.append((word, self.table[word]))
                    i += length
                    break
            else:
                out.append((zh[i], "?"))
                i += 1
        return out

    def phrase(self, zh):
        """Từ loại của một token: tra thẳng, không có thì suy từ các từ con."""
        hit = self.cache.get(zh)
        if hit is not None:
            return hit
        tag = self.table.get(zh)
        if tag is None:
            core = zh.rstrip(TRAILERS) or zh
            tag = self.table.get(core)
        if tag is None:
            parts = [p for p in self.segment(core) if p[1] != "?"] or [(zh, "?")]
            tags = [p[1] for p in parts]
            if tags[0][0] == "p" and len(tags) > 1:
                tag = "pp"
            elif any(t[0] == "v" and t != "vn" for t in tags):
                tag = "v"
            elif tags[-1][0] == "u" and len(tags) > 1:
                tag = tags[-2]
            else:
                tag = tags[-1]
        self.cache[zh] = tag
        return tag


_names = {}
BASE_NAMES = set()


def load_base_names(path):
    with open(path, encoding="utf-8-sig") as fh:
        for line in fh:
            key, sep, _ = line.partition("=")
            if sep:
                BASE_NAMES.add(key)


def names_of(books, book):
    if book not in _names:
        try:
            with open(os.path.join(books, book, "story.json"), encoding="utf-8") as fh:
                _names[book] = BASE_NAMES | set(((json.load(fh).get("glossary") or {}).get("names") or {}).keys())
        except OSError:
            _names[book] = set(BASE_NAMES)
    return _names[book]


def grams(text, n):
    words = WORD.findall(nfc(text).lower())
    return collections.Counter(tuple(words[i:i + n]) for i in range(len(words) - n + 1))


def score(text, ref2, ref3):
    return sum((grams(text, 2) & ref2).values()) + sum((grams(text, 3) & ref3).values())


# ---- số ----
DIG = {"零": 0, "〇": 0, "一": 1, "二": 2, "两": 2, "三": 3, "四": 4, "五": 5, "六": 6, "七": 7, "八": 8, "九": 9}
UNIT = {"十": 10, "百": 100, "千": 1000, "万": 10000, "亿": 100000000}
WORDS = ["không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín", "mười"]


def zh_value(s):
    if s.isdigit():
        return int(s)
    if all(ch in DIG for ch in s) and len(s) > 1:
        return int("".join(str(DIG[ch]) for ch in s))
    total = section = num = 0
    last_unit = 1
    for ch in s:
        if ch in DIG:
            num = DIG[ch]
        elif ch in ("万", "亿"):
            section = (section + num) * UNIT[ch]
            total += section
            section = num = 0
            last_unit = UNIT[ch]
        elif ch in UNIT:
            section += (num or 1) * UNIT[ch]
            num = 0
            last_unit = UNIT[ch]
        else:
            return None
    # 三千五 = 3500: chữ số cuối theo sau đơn vị mang đơn vị kế dưới.
    if num and last_unit >= 100 and len(s) >= 2 and s[-2] in UNIT:
        num *= last_unit // 10
    return total + section + num


def below_thousand(n):
    hundreds, rest = divmod(n, 100)
    tens, ones = divmod(rest, 10)
    parts = []
    if hundreds:
        parts += [WORDS[hundreds], "trăm"]
        if rest and tens == 0:
            parts.append("lẻ")
    if tens == 1:
        parts.append("mười")
    elif tens:
        parts += [WORDS[tens], "mươi"]
    if ones:
        if ones == 1 and tens >= 2:
            parts.append("mốt")
        elif ones == 5 and tens >= 1:
            parts.append("lăm")
        elif ones == 4 and tens >= 2:
            parts.append("tư")
        else:
            parts.append(WORDS[ones])
    return " ".join(parts)


def number_words(n):
    if n == 0:
        return "không"
    parts = []
    for unit, name in ((100000000, "ức"), (10000, "vạn"), (1000, "ngàn")):
        if n >= unit:
            high, n = divmod(n, unit)
            parts += [number_words(high), name]
    if n:
        parts.append(below_thousand(n))
    return " ".join(parts)


def number_ordinal(n):
    return {1: "nhất", 2: "hai", 4: "tư"}.get(n) or number_words(n)


_story = {}


def story_names(books, book):
    if book not in _story:
        try:
            with open(os.path.join(books, book, "story.json"), encoding="utf-8") as fh:
                table = (json.load(fh).get("glossary") or {}).get("names") or {}
            _story[book] = {k: str(v).split("/")[0].strip() for k, v in table.items()}
        except OSError:
            _story[book] = {}
    return _story[book]
