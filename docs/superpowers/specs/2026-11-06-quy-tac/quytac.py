"""Bộ thông dịch thử cho ngôn ngữ quy tắc: tầng nối tiếp trên token.

Một dòng quy tắc:  [trước |] mẫu [| sau] = khuôn [-> nhãn]
Phần tử mẫu: chữ Hán, (a|b), {nhãn}, thêm ? là tuỳ chọn. {vế}: khúc câu bên trái
cho tới ranh giới vế. {đoạn}: khúc ngắn nhất trong vế cho tới phần tử kế tiếp.
Ngữ cảnh: ^ đầu vế, $ cuối vế, ! phủ định.
"""
import re

from qtlib import DEMONSTRATIVE, DIGITS, NUM_CHARS, NUM_ONLY, PRONOUNS, nfc, number_ordinal, number_words, zh_value

CLAUSE_OPEN = {"但", "但是", "可是", "可", "而", "却", "然后", "于是", "所以", "因为", "如果", "虽然", "不过", "只是", "就", "便", "才", "并", "并且", "而且", "或者", "还是", "甚至", "只要", "要是"}
NOUN_TAGS = {"n", "ng", "nz", "vn", "an", "j", "cụm", "nrfg"}
ADJ_TAGS = {"a", "ad", "ag", "z", "b"}


class Tok:
    __slots__ = ("zh", "vi", "tag", "val", "src")

    def __init__(self, zh, vi, tag, val=None, src=None):
        self.zh, self.vi, self.tag, self.val, self.src = zh, vi, tag, val, src


def is_person(t):
    return t.zh in PRONOUNS or t.tag == "người"


def is_name(t):
    return t.tag.startswith("nr") or t.tag in ("ns", "nt", "tên")


def is_noun(t):
    return t.tag in NOUN_TAGS and t.zh not in PRONOUNS


SLOTS = {
    "đại-từ": lambda t: t.zh in PRONOUNS,
    "tên": is_name,
    "người": lambda t: is_person(t) or is_name(t),
    "danh-từ": is_noun,
    "cụm": lambda t: is_noun(t) or is_person(t) or is_name(t) or t.tag == "s",
    "động-từ": lambda t: t.tag[0] == "v" or t.tag == "v-cụm",
    "danh-động": lambda t: t.tag == "vn",
    "hệ-từ": lambda t: t.tag == "hệ-từ",
    "có-đích": lambda t: len(t.zh) >= 2 and t.zh.endswith("的") and bool(t.vi.strip()),
    "tính-từ": lambda t: t.tag in ADJ_TAGS or t.tag == "tính-cụm",
    "thành-ngữ": lambda t: t.tag in ("i", "l"),
    "số": lambda t: t.tag == "số",
    "số-lượng": lambda t: t.tag == "số-lượng",
    "thời-gian": lambda t: t.tag in ("t", "tg", "thời-gian"),
    "nơi": lambda t: t.tag in ("s", "nơi"),
    "phó-từ": lambda t: t.tag[0] == "d" and t.tag != "dấu",
    "dấu": lambda t: t.tag == "w",
    "chỉ-định": lambda t: t.zh in DEMONSTRATIVE,
    "nơi-cụm": lambda t: t.tag == "nơi-cụm",
    "nơi-từ": lambda t: t.tag in ("nơi-cụm", "s", "f") and len(t.zh) > 1,
    "động-từ-tại": lambda t: len(t.zh) >= 2 and t.zh.endswith("在") and t.tag[0] == "v",
}


class El:
    __slots__ = ("kind", "lits", "name", "opt", "neg")

    def __init__(self, kind, lits=None, name=None, opt=False, neg=False):
        self.kind, self.lits, self.name, self.opt, self.neg = kind, lits, name, opt, neg


def parse_elements(text):
    out = []
    for part in text.split():
        neg = part.startswith("!")
        if neg:
            part = part[1:]
        opt = part.endswith("?")
        if opt:
            part = part[:-1]
        if part == "^":
            out.append(El("start", neg=neg))
        elif part == "$":
            out.append(El("end", neg=neg))
        elif part in ("{vế}", "{vế-động}", "{mệnh-đề}"):
            out.append(El("left", name=part[1:-1]))
        elif part == "{đoạn}":
            out.append(El("gap", name="đoạn"))
        elif part.endswith("}+"):
            out.append(El("many", name=part[1:-2]))
        elif part.startswith("{"):
            out.append(El("slot", name=part[1:-1], opt=opt, neg=neg))
        else:
            lits = set(part.strip("()").split("|"))
            out.append(El("lit", lits=lits, opt=opt, neg=neg))
    return out


class Rule:
    __slots__ = ("id", "text", "pre", "pat", "post", "tmpl", "result", "needs")

    def __init__(self, rid, text, pre, pat, post, tmpl, result):
        self.id, self.text, self.pre, self.pat, self.post, self.tmpl, self.result = rid, text, pre, pat, post, tmpl, result
        self.needs = [e.lits for e in pat if e.kind == "lit" and not e.opt and not e.neg]


class Section:
    def __init__(self, name, rounds, result):
        self.name, self.rounds, self.result = name, rounds, result
        self.first = {}
        self.generic = []
        self.left = {}
        self.rules = []

    def add(self, rule):
        self.rules.append(rule)
        head = rule.pat[0]
        if head.kind == "left":
            for lit in rule.pat[1].lits:
                self.left.setdefault(lit[0], []).append(rule)
        elif head.kind == "lit" and not head.opt:
            for first in {lit[0] for lit in head.lits}:
                self.first.setdefault(first, []).append(rule)
        else:
            self.generic.append(rule)

    def finish(self):
        def key(r):
            return (-len(r.pat), -sum(1 for e in r.pat if e.kind == "lit"))

        for table in (self.first, self.left):
            for rules in table.values():
                rules.sort(key=key)
        self.generic.sort(key=key)


class Program:
    def __init__(self, path):
        self.classes = {}
        self.sections = []
        current = None
        rid = 0
        with open(path, encoding="utf-8") as fh:
            for raw in fh:
                line = raw.split("#", 1)[0].strip()
                if not line:
                    continue
                if line.startswith("["):
                    header = line.strip("[]").split()
                    name = header[0]
                    rounds = int(header[1]) if len(header) > 1 else 1
                    result = header[2] if len(header) > 2 else "cụm"
                    current = None if name == "lớp" else Section(name, rounds, result)
                    if current:
                        self.sections.append(current)
                    continue
                if current is None:
                    cname, members = line.split(":", 1)
                    table = self.classes.setdefault(cname.strip(), {})
                    for member in members.split():
                        zh, _, vi = member.partition("=")
                        table[zh] = vi.replace("_", " ")
                    continue
                lhs, _, rhs = line.partition(" = ")
                result = current.result
                if " -> " in rhs:
                    rhs, _, result = rhs.partition(" -> ")
                    result = result.strip()
                parts = lhs.split("|") if " | " in lhs or lhs.startswith("| ") or lhs.endswith(" |") else [lhs]
                pieces = re.split(r"(?:^|\s)\|(?:\s|$)", lhs)
                if len(pieces) == 1:
                    pre, pat, post = "", pieces[0], ""
                elif len(pieces) == 2:
                    pre, pat, post = "", pieces[0], pieces[1]
                else:
                    pre, pat, post = pieces[0], pieces[1], pieces[2]
                rid += 1
                current.add(Rule(rid, f"[{current.name}] {line}", parse_elements(pre), parse_elements(pat), parse_elements(post), rhs.strip(), result))
        for section in self.sections:
            section.finish()

    def slot_ok(self, name, t):
        table = self.classes.get(name)
        if table is not None:
            return t.zh in table
        return SLOTS[name](t)


def low(t):
    if is_name(t) or not t.vi:
        return t.vi.strip()
    return (t.vi[:1].lower() + t.vi[1:]).strip()


def num_text(t, how):
    if t.val is None:
        return low(t)
    if how == "thứ":
        return str(t.val) if t.src == "digit" else number_ordinal(t.val)
    if how == "ngày":
        text = str(t.val) if t.src == "digit" else number_words(t.val)
        return f"mùng {text}" if t.val <= 10 else text
    if how == "số":
        return str(t.val)
    return str(t.val) if t.src == "digit" else number_words(t.val)


def value(prog, cap):
    name, toks = cap
    if isinstance(toks, list):
        return " ".join(x for x in (low(t) for t in toks) if x)
    table = prog.classes.get(name)
    if table is not None and table.get(toks.zh):
        return table[toks.zh]
    if toks.tag == "số":
        return num_text(toks, "")
    return low(toks)


REF = re.compile(r"\{([^{}|]+)(?:\|([^{}]+))?\}")


def fill(prog, rule, caps):
    def sub(m):
        key, how = m.group(1), m.group(2)
        hit = None
        if key.isdigit():
            idx = int(key) - 1
            hit = caps[idx] if idx < len(caps) else None
        else:
            for cap in caps:
                if cap[0] == key:
                    hit = cap
                    break
        if hit is None:
            return ""
        if how and not isinstance(hit[1], list) and hit[1].tag == "số":
            return num_text(hit[1], how)
        if how == "bỏ-tại":
            return re.sub(r"\s*(tại|ở|trên|tới|vào|đến)\s*$", "", value(prog, hit), flags=re.I)
        return value(prog, hit)

    return re.sub(r"\s+", " ", REF.sub(sub, rule.tmpl)).strip()


def clause_edge(t):
    return t.tag == "w"


AUX = {"不", "没", "没有", "未", "能", "会", "要", "想", "可以", "应该", "已经", "曾", "曾经", "正", "正在", "最", "很", "更", "所", "都", "也", "还", "再", "又", "才", "就", "刚", "刚刚", "总", "总是", "一直", "不会", "不能", "不想", "不敢", "能够", "可能", "将要", "快要", "敢", "肯", "愿意", "需要", "必须", "该", "得"}
STOP_VERB = {"是", "说", "道", "问", "笑道", "说道", "喊道", "叫道"}


def nominal(t):
    return is_noun(t) or is_person(t) or is_name(t) or t.tag in ("s", "t", "nơi", "cụm", "số-lượng", "thời-gian")


def verbal(t):
    return (t.tag[0] == "v" or t.tag == "v-cụm") and t.zh not in STOP_VERB and not t.zh.startswith("是")


def clause_span(prog, out):
    """Mệnh đề định ngữ kết thúc ở cuối out; None nếu không thấy động từ."""
    main = prog.classes.get("động-từ-chính", {})
    k = len(out)
    start_obj = k
    while k > 0 and start_obj - k < 2 and nominal(out[k - 1]):
        k -= 1
    has_object = k != start_obj
    if k == 0 or not verbal(out[k - 1]):
        return None
    verb = out[k - 1]
    if verb.zh in main and has_object or any(verb.zh.startswith(m) and len(verb.zh) > len(m) for m in main):
        return None
    k -= 1
    while k > 0 and (out[k - 1].zh in AUX or out[k - 1].tag in ("d", "ad")):
        k -= 1
    if k > 0 and (is_person(out[k - 1]) or is_name(out[k - 1]) or is_noun(out[k - 1])):
        k -= 1
    return k


def left_span(out, kind, prog=None):
    """Khúc bên trái: lùi tới ranh giới vế. Trả về chỉ số bắt đầu trong out."""
    if kind == "mệnh-đề":
        hit = clause_span(prog, out)
        return len(out) if hit is None else hit
    i = len(out)
    limit = 16
    while i > 0 and len(out) - i < limit:
        t = out[i - 1]
        if clause_edge(t) or t.zh in CLAUSE_OPEN:
            break
        if t.tag == "hệ-từ" or t.zh in ("到", "等", "等到"):
            break
        i -= 1
    return i


def ctx_ok(prog, el, tok, at_edge):
    if el.kind == "start" or el.kind == "end":
        hit = at_edge
    elif tok is None:
        hit = False
        if el.kind == "lit" and "^" in el.lits:
            hit = True
    elif el.kind == "lit":
        hit = tok.zh in el.lits
    else:
        hit = prog.slot_ok(el.name, tok)
    if el.kind == "lit" and "^" in el.lits and at_edge:
        hit = True
    return hit != el.neg


def match(prog, rule, out, toks, i):
    """Khớp tại i. Trả về (số token tiêu thụ bên phải, số token lấy từ out, captures)."""
    caps = []
    n = len(toks)
    j = i
    take = 0
    pat = rule.pat
    idx = 0
    if pat[0].kind == "left":
        start = left_span(out, pat[0].name, prog)
        span = out[start:]
        if not span:
            return None
        if pat[0].name == "vế-động" and not any(t.tag[0] == "v" and t.tag != "vn" for t in span):
            return None
        caps.append((pat[0].name, span))
        take = len(span)
        idx = 1
    while idx < len(pat):
        el = pat[idx]
        if el.kind == "lit":
            span = lit_span(toks, j, el.lits)
            if (span > 0) != el.neg:
                if not el.neg:
                    j += span
            elif not el.opt:
                return None
        elif el.kind == "slot":
            if j < n and prog.slot_ok(el.name, toks[j]):
                caps.append((el.name, toks[j]))
                j += 1
            elif not el.opt:
                return None
            else:
                caps.append((el.name, []))
        elif el.kind == "many":
            k = j
            while k < n and k - j < 3 and prog.slot_ok(el.name, toks[k]):
                k += 1
            if k == j:
                return None
            caps.append((el.name, toks[j:k]))
            j = k
        elif el.kind == "gap":
            nxt = pat[idx + 1]
            k = j
            found = False
            while k < n and k - j <= 12 and toks[k].tag != "w":
                if k > j and lit_span(toks, k, nxt.lits):
                    found = True
                    break
                k += 1
            if not found:
                return None
            caps.append(("đoạn", toks[j:k]))
            j = k
        idx += 1
    for el in rule.post:
        tok = toks[j] if j < n else None
        if not ctx_ok(prog, el, tok, tok is None or tok.tag == "w"):
            return None
    if rule.pre:
        back = len(out) - take - 1
        tok = out[back] if back >= 0 else None
        for el in rule.pre:
            if not ctx_ok(prog, el, tok, tok is None or tok.tag == "w"):
                return None
    return j - i, take, caps


def lit_span(toks, j, lits):
    """Số token liền nhau ghép thành đúng một literal; 0 nếu không khớp."""
    if j >= len(toks):
        return 0
    acc = toks[j].zh
    if acc in lits:
        return 1
    longest = max(map(len, lits))
    k = j + 1
    while k < len(toks) and len(acc) < longest:
        acc += toks[k].zh
        k += 1
        if acc in lits:
            return k - j
    return 0


def run_section(prog, section, toks, trace, disabled):
    text = "".join(t.zh for t in toks)
    for _ in range(section.rounds):
        out = []
        i = 0
        n = len(toks)
        changed = False
        while i < n:
            t = toks[i]
            cands = section.left.get(t.zh, ()) + () if False else None
            done = False
            head = t.zh[:1]
            for group in (section.left.get(head), section.first.get(head), section.generic):
                if not group:
                    continue
                for rule in group:
                    if rule.id in disabled:
                        continue
                    if rule.needs and not all(any(lit in text for lit in lits) for lits in rule.needs):
                        continue
                    m = match(prog, rule, out, toks, i)
                    if m is None:
                        continue
                    used, take, caps = m
                    if take:
                        del out[len(out) - take:]
                    consumed = toks[i:i + used]
                    zh = "".join(x.zh for x in consumed)
                    out.append(Tok(zh, fill(prog, rule, caps), rule.result))
                    trace.append(rule.id)
                    i += max(used, 1) if used else 1
                    done = changed = True
                    break
                if done:
                    break
            if not done:
                out.append(t)
                i += 1
        toks = out
        if not changed:
            break
    return toks


def apply_program(prog, toks, disabled=frozenset()):
    trace = []
    for section in prog.sections:
        toks = run_section(prog, section, toks, trace, disabled)
    return toks, trace


def render(toks):
    pieces = []
    for t in toks:
        s = t.vi.strip()
        if s:
            pieces.append(s)
    text = " ".join(pieces)
    text = re.sub(r"\s+([,.!?:;…”’)\]】])", r"\1", text)
    text = re.sub(r"([“‘(\[【])\s+", r"\1", text)
    return re.sub(r"(^|[.!?…]\s+|[“‘]\s*)(\w)", lambda m: m.group(1) + m.group(2).upper(), text)


NUM_TOKEN = re.compile(rf"^(?:[{NUM_CHARS}]+|\d+(?:\.\d+)?)$")


VI_PUNCT = str.maketrans({"，": ",", "。": ".", "！": "!", "？": "?", "；": ";", "：": ":", "、": ","})
PUNCT_TAIL = re.compile(r"^(.*?[^，。！？、；：,.!?…]{1})([，。！？、；：,.!?…]+)$")
KEEP_DE = {"的时候", "的话", "的确", "的是", "的吗", "的呢"}
KEEP_TAIL_DE = {"有的", "别的", "目的", "是的", "真的", "好的", "似的", "假的", "对的"}


LOCS = ("之上", "之中", "之下", "之内", "之外", "上面", "下面", "里面", "外面", "上", "里", "中", "下", "内", "外")
WRONG_TAIL = re.compile(r"\s*(bên trên|phía trên|bên trong|phía trong|bên dưới|phía dưới|bên ngoài|phía ngoài)\s*$", re.I)
LEX = {}
NAMES_NOW = set()
POS_SEG = None


def load_lexicon(path):
    with open(path, encoding="utf-8-sig") as fh:
        for line in fh:
            key, sep, value = line.rstrip("\r\n").partition("=")
            if sep and len(key) <= 15 and key not in LEX:
                LEX[key] = value.split("/")[0].strip()


STORY = {}


def seg(text):
    """Tách lại một khúc bằng từ điển, cụm dài trước. Tên của truyện đi trước."""
    out, i = [], 0
    while i < len(text):
        for length in range(min(15, len(text) - i), 0, -1):
            word = text[i:i + length]
            if word in STORY:
                out.append((word, STORY[word]))
                i += length
                break
            if word in LEX:
                out.append((word, LEX[word]))
                i += length
                break
        else:
            out.append((text[i], text[i]))
            i += 1
    return out


def plain_words(text):
    return re.findall(r"\w+", nfc(text).lower())


def same_order(parts, vi):
    """Nghĩa cụm có đúng bằng nghĩa từng phần ghép theo thứ tự nguồn không."""
    return plain_words(" ".join(v for _, v in parts)) == plain_words(vi)


def unfreeze(zh, vi, table):
    """Cụm từ điển giữ trật tự convert: tách để quy tắc đảo được. None nếu giữ nguyên.

    Cụm nào nghĩa khác phép ghép từng phần tức là đã dịch tay, giữ nguyên.
    """
    tag = table.get(zh, "")
    if tag in ("i", "l") or not LEX:
        return None
    if "的" in zh[1:-1]:
        parts = zh.split("的")
        if all(parts):
            out = []
            for idx, part in enumerate(parts):
                if idx:
                    out.append(("的", ""))
                out.extend(seg(part))
            return out if same_order(out, vi) else None
    for loc in LOCS:
        if len(zh) > len(loc) and zh.endswith(loc) and WRONG_TAIL.search(vi):
            parts = seg(zh[: -len(loc)]) + [(loc, LEX.get(loc, loc))]
            return parts if same_order(parts, vi) else None
    if zh[0] in "都也" and len(zh) > 2 and zh[1:] in LEX:
        return [(zh[0], LEX.get(zh[0], "")), (zh[1:], LEX[zh[1:]])]
    return None


def untie(raw_tokens, table):
    """Tách dấu câu cuối token, 的 đầu/cuối token, cụm đóng băng. Nghĩa giữ ở phần chữ."""
    out = []
    for index, item in enumerate(raw_tokens):
        if len(item) == 3:
            out.append(item)
            continue
        zh, vi = item
        following = raw_tokens[index + 1][0] if index + 1 < len(raw_tokens) else ""
        nxt_de = following.startswith("的")
        # Token gộp động từ + danh từ đứng ngay trước 的: tách theo từ để 的 bám đúng định ngữ.
        if nxt_de and len(zh) >= 3 and LEX and table.get(zh) is None and zh not in NAMES_NOW:
            words = POS_SEG(zh) if POS_SEG else []
            if len(words) >= 2 and words[0][1][0] == "v":
                pieces = [(w, LEX.get(w, w)) for w, _ in words]
                if same_order(pieces, vi):
                    out.extend(pieces)
                    continue
        tail = None
        m = PUNCT_TAIL.match(zh)
        if m and len(zh) > 1:
            zh, tail = m.group(1), m.group(2)
            vi = vi.rstrip(" ,.!?;:…")
        frozen = unfreeze(zh, vi, table)
        if frozen:
            out.extend(frozen)
            if tail:
                out.append((tail, tail.translate(VI_PUNCT)))
            continue
        if len(zh) >= 2 and zh[0] == "的" and zh not in KEEP_DE and zh[1:] in table:
            out.append(("的", ""))
            zh = zh[1:]
        if nxt_de and len(zh) >= 3 and zh not in table and zh not in LEX and False:
            pass
        if len(zh) >= 2 and zh[-1] == "的" and zh not in KEEP_TAIL_DE and (zh[:-1] in table or zh[:-1] in LEX):
            out.append((zh[:-1], vi))
            out.append(("的", ""))
        else:
            out.append((zh, vi))
        if tail:
            out.append((tail, tail.translate(VI_PUNCT)))
    return out


def prepare(raw_tokens, pos, names, classes):
    """Gắn nhãn; ghép số; tách số + lượng từ."""
    measure = classes.get("lượng-từ", {})
    out = []
    global POS_SEG, NAMES_NOW
    POS_SEG = pos.segment
    NAMES_NOW = names
    raw_tokens = untie(raw_tokens, pos.table)
    for item in raw_tokens:
        if len(item) == 3:
            out.append(Tok(item[0], item[1], item[2]))
            continue
        zh, vi = item
        if not re.search(r"[一-鿿\d]", zh):
            out.append(Tok(zh, vi, "w"))
            continue
        if NUM_TOKEN.match(zh) and zh not in ("一", "万", "千", "百", "十"):
            val = zh_value(zh) if not DIGITS.match(zh) else None
            if DIGITS.match(zh):
                out.append(Tok(zh, vi, "số", int(float(zh)), "digit"))
            elif val is not None:
                out.append(Tok(zh, vi, "số", val, "zh"))
            else:
                out.append(Tok(zh, vi, "m"))
            continue
        if zh == "一":
            out.append(Tok(zh, vi, "số", 1, "zh"))
            continue
        m = re.match(rf"^(第?)([{NUM_CHARS}]+)(.)$", zh)
        if m and m.group(3) in measure and m.group(2) not in ("万", "千", "百") and zh not in ("一点", "一下", "一些", "一样", "一直", "一定", "一起", "一切", "一般", "一边", "一旦", "一时", "一半", "一丝", "一身", "一面", "一口", "一眼", "一声", "一阵", "一番", "一片", "一场", "一股", "一道", "一天"):
            val = zh_value(m.group(2))
            if val is not None:
                if m.group(1):
                    out.append(Tok("第", "thứ", "lit"))
                out.append(Tok(m.group(2), number_words(val), "số", val, "zh"))
                out.append(Tok(m.group(3), measure[m.group(3)] or vi, "q"))
                continue
        if zh in names:
            tag = "nr"
        elif zh in ("时候", "时", "时间点"):
            tag = "khung"
        elif zh.endswith("是"):
            tag = "hệ-từ"
        else:
            tag = pos.phrase(zh)
        out.append(Tok(zh, vi, tag))
    return out


NUM_RE = rf"[{NUM_CHARS}]+|\d+(?:\.\d+)?"


class CharRule:
    def __init__(self, rid, text, regex, slots, tmpl, result):
        self.id, self.text, self.regex, self.slots, self.tmpl, self.result = rid, text, regex, slots, tmpl, result


def compile_char_rules(prog, path):
    """Nhóm [số]: mẫu viết liền, quét trên ký tự. Hai khoá {số} và lớp từ."""
    rules = []
    active = False
    rid = 100000
    with open(path, encoding="utf-8") as fh:
        for raw in fh:
            line = raw.split("#", 1)[0].strip()
            if not line:
                continue
            if line.startswith("["):
                active = line.strip("[]").split()[0] == "số"
                continue
            if not active:
                continue
            lhs, _, rhs = line.partition(" = ")
            result = "số-cụm"
            if " -> " in rhs:
                rhs, _, result = rhs.partition(" -> ")
                result = result.strip()
            slots = []

            def sub(m):
                name, _, bounds = m.group(1).partition(":")
                slots.append(name)
                lo, _, hi = bounds.partition("-")
                if name == "số" and bounds:
                    return f"\\s*((?:[{NUM_CHARS}]|\\d){{{lo},{hi}}})\\s*"
                if name == "số":
                    return f"\\s*({NUM_RE})\\s*"
                if name == "năm" and lo == "3":
                    # Năm lịch: chữ số Hán đọc từng chữ, hoặc số Ả Rập trông như năm. 500年 là "500 năm".
                    return "\s*([零〇一二三四五六七八九]{3,4}|1[5-9]\d\d|20\d\d)\s*"
                if name == "năm":
                    rep = f"{{{lo},{hi}}}" if bounds else "+"
                    return f"\\s*([零〇一二三四五六七八九\\d]{rep})\\s*"
                keys = sorted(prog.classes[name], key=len, reverse=True)
                return "(" + "|".join(re.escape(k) for k in keys) + ")"

            lhs = re.sub(r"\((?!\?)", "(?:", lhs)
            pattern = re.sub(r"\{([^{}]+)\}", sub, lhs)
            rid += 1
            rules.append(CharRule(rid, f"[số] {line}", re.compile(pattern), slots, rhs.strip(), result))
    rules.sort(key=lambda r: -len(re.sub(r"\{[^{}]*\}|\[số\]| = .*|.\?", "", r.text)))
    return rules


WORDS_DIGIT = ["không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín"]


def char_value(prog, name, text, how):
    if name == "năm":
        digits = "".join(c if c.isdigit() else str("零一二三四五六七八九".index(c.replace("〇", "零"))) for c in text)
        if how == "từng-chữ":
            return " ".join(WORDS_DIGIT[int(d)] for d in digits)
        return digits
    if name != "số":
        table = prog.classes[name]
        return table.get(text) or text
    digit = text[0].isdigit()
    mixed = re.fullmatch(r"(\d+(?:\.\d+)?)([十百千万亿]+)", text)
    if mixed:
        names = {"十": "mươi", "百": "trăm", "千": "ngàn", "万": "vạn", "亿": "ức"}
        return mixed.group(1) + " " + " ".join(names[c] for c in mixed.group(2))
    val = float(text) if "." in text else int(text) if digit else zh_value(text)
    if val is None:
        return text
    t = Tok(text, text, "số", val, "digit" if digit else "zh")
    if how == "năm" and not digit and all(c in "零〇一二三四五六七八九" for c in text):
        return "".join(str("零一二三四五六七八九".index(c.replace("〇", "零"))) for c in text)
    if isinstance(val, float):
        return text
    return num_text(t, how or "")


def apply_chars(prog, rules, raw, disabled=frozenset()):
    """Trả về danh sách token thô đã thay các khúc số/ngày; khúc phải khớp ranh giới token."""
    text = "".join(z for z, _ in raw)
    if not re.search(rf"[{NUM_CHARS}\d]", text):
        return raw, []
    bounds = {}
    pos = 0
    for idx, (z, _) in enumerate(raw):
        bounds[pos] = idx
        pos += len(z)
    bounds[pos] = len(raw)
    spans = []
    taken = [False] * (len(text) + 1)
    for rule in rules:
        if rule.id in disabled:
            continue
        for m in rule.regex.finditer(text):
            a, b = m.span()
            if a == b or a not in bounds or any(taken[a:b]):
                continue
            # Từ điển có cụm dài hơn bắt đầu cùng chỗ thì nhường (两点一线, 一个人…).
            if any(text[a:a + length] in LEX for length in range(b - a + 1, min(16, len(text) - a + 1))):
                continue
            caps = [(rule.slots[i], m.group(i + 1)) for i in range(len(rule.slots))]

            def sub(ref):
                key, how = ref.group(1), ref.group(2)
                idx = int(key) - 1 if key.isdigit() else next((i for i, c in enumerate(caps) if c[0] == key), -1)
                if idx < 0 or idx >= len(caps) or caps[idx][1] is None:
                    return ""
                return char_value(prog, caps[idx][0], caps[idx][1], how)

            vi = re.sub(r"\s+", " ", REF.sub(sub, rule.tmpl)).strip()
            spans.append((a, b, vi, rule))
            for i in range(a, b):
                taken[i] = True
    if not spans:
        return raw, []
    spans.sort(key=lambda x: x[0])
    starts = sorted(bounds)
    out, trace, idx = [], [], 0
    for a, b, vi, rule in spans:
        while idx < bounds[a]:
            out.append(raw[idx])
            idx += 1
        out.append((text[a:b], vi, rule.result))
        trace.append(rule.id)
        if b in bounds:
            idx = bounds[b]
            continue
        # b cắt giữa một token: phần còn lại của token đó tách lại bằng từ điển.
        end = next(p for p in starts if p > b)
        # Tách lại cả token kế tiếp, để phần dư ghép được với nó (分出|发 -> 出发).
        later = [p for p in starts if p > end]
        end = later[0] if later and later[0] - b <= 6 else end
        out.extend(seg(text[b:end]))
        idx = bounds[end]
    out.extend(raw[idx:])
    return out, trace
