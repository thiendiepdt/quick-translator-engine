#!/usr/bin/env python3
"""Faithful Python replica of QT2025 QuickTranslator.exe LocNameOff.LocNameQT.

Decompiled source: reference/decompiled/LocNameOff.decompiled.cs
Dictionaries are loaded from the QT2025 data directory the same way
TranslatorEngine loads them (Names + Names2 merged into vietPhrase, etc).
"""
import re
import sys
from pathlib import Path

import jieba

QT = Path(__file__).resolve().parent.parent / "QT2025"

SOURCE_STOPWORDS = "她/着/的/你/我/了/他/什么/也/什/们/在/您/那/这/这个/不过/尔/啊/吧/一边/没/哪个/就是/有些/很/非常/还是/再有/发现/数/十几".split("/")
RE_BOOK_TITLE = re.compile(r"《.*?》")
CN_NUMBERS = set("零一二三四五六七八九十百千万亿两〇")


def load_pairs(path):
    result = {}
    try:
        with open(path, encoding="utf-8-sig") as handle:
            for line in handle:
                parts = line.rstrip("\n").split("=")
                if len(parts) == 2 and parts[0] not in result:
                    result[parts[0]] = parts[1]
    except FileNotFoundError:
        pass
    return result


class Dicts:
    def __init__(self):
        self.only_name = load_pairs(QT / "Names.txt")
        self.only_name_phu = load_pairs(QT / "Names2" / "123.txt")
        for k, v in self.only_name_phu.items():
            self.only_name.setdefault(k, v)
        self.only_vietphrase = load_pairs(QT / "VietPhrase" / "VietPhrase.txt")
        self.vietphrase = dict(self.only_name)
        for k, v in self.only_vietphrase.items():
            self.vietphrase.setdefault(k, v)
        self.vietphrase_one = {
            k: re.split(r"[/|]", v)[0] for k, v in self.only_vietphrase.items()
        }
        self.danh_tu = load_pairs(QT / "Resources" / "DanhTu.txt")
        self.ho_nguoi = load_pairs(QT / "Resources" / "HoNguoi.txt")
        self.han_viet = load_pairs(QT / "Resources" / "ChinesePhienAmWords.txt")


D = Dicts()


def title_case(text):
    return " ".join(w[:1].upper() + w[1:] for w in text.split())


def chinese_to_hanviet(text):
    parts = []
    for ch in text:
        parts.append(D.han_viet.get(ch, ch))
    return " ".join(p for p in " ".join(parts).split())


def title_case_chinese(text):
    return title_case(chinese_to_hanviet(text).strip())


def match_any_start_end(text, keys, is_prefix):
    for key in keys:
        if text == key:
            continue
        if is_prefix:
            if text.startswith(key):
                return True
        elif text.endswith(key):
            return True
    return False


def is_title_case(value):
    if not value.strip():
        return False
    for word in value.split():
        letters = "".join(c for c in word if c.isalpha())
        if letters and not letters[0].isupper():
            return False
    return True


def title_case_with_dictionary(key):
    if key in D.danh_tu:
        return D.danh_tu[key].replace("{0}", "").strip()
    for dk in sorted(D.danh_tu, key=len, reverse=True):
        if dk and key.endswith(dk):
            value2 = D.danh_tu[dk]
            prefix = key[: len(key) - len(dk)].strip()
            if len(prefix) <= 2:
                text2 = title_case_chinese(prefix).strip()
            elif prefix in D.vietphrase_one:
                v3 = D.vietphrase_one[prefix]
                text2 = v3 if is_title_case(v3) else title_case(v3)
            else:
                text2 = title_case_chinese(prefix).strip()
            if "{0}" in value2:
                return value2.replace("{0}", text2)
            return text2 + " " + value2
    return title_case(key)


def build_formatted_hanviet(key, check_vietphrase=True):
    if not key.strip():
        return ""
    if len(key) > 2 and match_any_start_end(key, D.danh_tu.keys(), False):
        return title_case_with_dictionary(key)
    if check_vietphrase and key in D.vietphrase_one:
        value = D.vietphrase_one[key]
        return title_case_chinese(key) if not value.strip() else value.strip()
    return title_case_chinese(key)


def filter_unnecessary_phrases_optimized(phrases):
    groups = {}
    for phrase in phrases:
        if len(phrase) >= 2:
            groups.setdefault(phrase[:2], []).append(phrase)
    return {min(group, key=len) for group in groups.values()}


def filter_unnecessary_items(phrases, text_content):
    removed = set()
    for phrase in phrases:
        for key in D.only_name_phu:
            if key == phrase or not phrase.startswith(key):
                continue
            if phrase[:1] == key[-1:]:
                value = key + phrase[1:]
                if value in text_content:
                    removed.add(phrase)
                    break
    return phrases - removed


def filter_phrases_and_names(phrases, text_content):
    result = filter_unnecessary_items(phrases, text_content)
    if len(D.only_name_phu) > 0:
        result = filter_unnecessary_items(result, text_content)
    return result


def is_phrase_in_brackets_match(text_with_brackets, text_to_check):
    i = text_with_brackets.find("《")
    j = text_with_brackets.find("》")
    if i != -1 and j > i:
        return text_with_brackets[i + 1 : j].strip() in text_to_check
    return False


def is_part_of_any_phrase_in_dictionary(loc_name):
    if len(loc_name) <= 1:
        return False
    for key in D.only_name_phu:
        if len(key) > 1:
            if key.startswith(loc_name) or loc_name.startswith(key):
                return True
            if is_phrase_in_brackets_match(loc_name, key) or is_phrase_in_brackets_match(key, loc_name):
                return True
            if key in loc_name and not match_any_start_end(loc_name, D.ho_nguoi.keys(), True):
                return True
    return False


def process_two_character_term(term, segments, to_remove, to_add):
    merged = False
    has_surname = match_any_start_end(term, D.ho_nguoi.keys(), True)
    for i in range(len(segments) - 1):
        if segments[i] == term:
            nxt = segments[i + 1]
            if nxt in D.danh_tu:
                to_add.add(term + nxt)
                merged = True
    if not merged and not has_surname:
        to_remove.add(term)
    else:
        to_add.add(term)


def process_four_character_term(term, to_remove, to_add):
    left, right = term[:2], term[2:4]
    if right not in D.danh_tu or (left in D.vietphrase and right in D.vietphrase):
        to_remove.add(term)
    else:
        to_add.add(term)


def validate_and_merge_terms(valid_terms, segments):
    to_remove, to_add = set(), set()
    valid_terms.difference_update({t for t in valid_terms if not t.strip()})
    for term in valid_terms:
        if len(term) == 2 and term not in D.only_vietphrase:
            process_two_character_term(term, segments, to_remove, to_add)
        elif len(term) == 4:
            process_four_character_term(term, to_remove, to_add)
    valid_terms.difference_update(to_remove)
    valid_terms.update(to_add)


def is_chinese_number_sequence(text):
    return len(text) >= 3 and all(c in CN_NUMBERS for c in text)


def loc_name_qt(text_content, threshold):
    segments = [w.strip() for w in jieba.cut(text_content)]
    counts = {}
    for w in segments:
        if 2 <= len(w) <= 5:
            counts[w] = counts.get(w, 0) + 1
    phrases = {
        k
        for k, c in counts.items()
        if c >= threshold
        and k.strip()
        and not any(ch.isdigit() for ch in k)
        and not any(s in k for s in SOURCE_STOPWORDS)
    }
    phrases = filter_unnecessary_phrases_optimized(phrases)
    valid_terms = filter_phrases_and_names(phrases, text_content)
    validate_and_merge_terms(valid_terms, segments)

    entries = []
    for key in valid_terms:
        formatted = build_formatted_hanviet(key)
        if (
            key not in D.vietphrase
            and not is_part_of_any_phrase_in_dictionary(key)
            and formatted
            and " " in formatted
            and not is_chinese_number_sequence(key)
        ):
            entries.append((key, formatted))
    entries.sort(key=lambda kv: (len(kv[0]), kv[0]))
    lines = [f"{k}={v}" for k, v in entries]
    seen = {k for k, _ in entries}
    for match in RE_BOOK_TITLE.finditer(text_content):
        text = (match.group(0) or "").strip()
        if not text:
            continue
        if text not in seen:
            formatted = build_formatted_hanviet(text)
            if formatted:
                lines.append(f"{text}={formatted}")
                seen.add(text)
    def position(line):
        idx = text_content.find(line.split("=")[0])
        return idx if idx >= 0 else 1 << 62
    lines.sort(key=position)
    return lines


def get_threshold(num_words):
    if num_words < 50000:
        return 1
    if num_words < 100000:
        return 2
    if num_words < 200000:
        return 3
    return 4


if __name__ == "__main__":
    content = Path(sys.argv[1]).read_text(encoding="utf-8")
    num_chinese = sum(1 for c in content if "一" <= c <= "鿿")
    threshold = get_threshold(num_chinese) if len(sys.argv) < 3 else int(sys.argv[2])
    for line in loc_name_qt(content, threshold):
        print(line)
