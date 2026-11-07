//! Chuẩn bị token cho Quy Tắc (spec mục 6–7): tách dấu câu và `的` dính vào
//! cụm, tách cụm từ điển đóng băng trật tự convert (chỉ khi nghĩa của nó đúng
//! bằng nghĩa từng phần ghép theo thứ tự nguồn), gắn từ loại, tách số + lượng
//! từ. Port từ `untie`, `unfreeze`, `prepare` của bản thử.

use std::sync::LazyLock;

use regex::Regex;
use rustc_hash::FxHashMap;
use unicode_normalization::UnicodeNormalization;

use super::matcher::SlotMemo;
use super::numbers::{number_words, zh_value, NUM_CHARS};
use super::program::Class;
use super::tags::PosLexicon;
use crate::dict::DictionaryLookup;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Token<'q> {
    pub(crate) zh: String,
    pub(crate) vi: String,
    pub(crate) tag: &'q str,
    /// Giá trị khi `tag == "số"`.
    pub(crate) val: Option<i64>,
    /// Số viết bằng chữ số Ả Rập (giữ nguyên khi ra).
    pub(crate) digit: bool,
    /// Kết quả khớp khoá đã tính (bộ nhớ đệm của `matcher`).
    pub(crate) memo: SlotMemo,
}

impl<'q> Token<'q> {
    pub(crate) fn new(zh: impl Into<String>, vi: impl Into<String>, tag: &'q str) -> Self {
        Token {
            zh: zh.into(),
            vi: vi.into(),
            tag,
            val: None,
            digit: false,
            memo: SlotMemo::default(),
        }
    }

    fn number(zh: impl Into<String>, vi: impl Into<String>, val: i64, digit: bool) -> Self {
        Token {
            zh: zh.into(),
            vi: vi.into(),
            tag: "số",
            val: Some(val),
            digit,
            memo: SlotMemo::default(),
        }
    }
}

/// Token thô: `(chữ Hán, nghĩa)` từ engine, hoặc khúc nhóm `[số]` đã dịch kèm nhãn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Raw<'q> {
    Plain(String, String),
    Tagged(String, String, &'q str),
}

impl Raw<'_> {
    fn zh(&self) -> &str {
        match self {
            Raw::Plain(zh, _) | Raw::Tagged(zh, _, _) => zh,
        }
    }
}

/// Từ điển mà Quy Tắc đọc: tên (Names hoặc names của request) và VietPhrase.
pub(crate) struct Lookups<'a> {
    pub(crate) names: &'a dyn DictionaryLookup,
    pub(crate) lexicon: &'a dyn DictionaryLookup,
}

impl Lookups<'_> {
    pub(crate) fn name(&self, word: &str) -> Option<&str> {
        self.names.get(word).map(|value| crate::first_meaning(value).trim())
    }

    pub(crate) fn word(&self, word: &str) -> Option<&str> {
        self.lexicon.get(word).map(|value| crate::first_meaning(value).trim())
    }

    pub(crate) fn is_name(&self, word: &str) -> bool {
        self.names.contains_key(word)
    }

    pub(crate) fn has_word(&self, word: &str) -> bool {
        self.lexicon.contains_key(word)
    }
}

const KEEP_DE: [&str; 6] = ["的时候", "的话", "的确", "的是", "的吗", "的呢"];
const KEEP_TAIL_DE: [&str; 9] = ["有的", "别的", "目的", "是的", "真的", "好的", "似的", "假的", "对的"];
const LOCS: [&str; 15] = [
    "之上", "之中", "之下", "之内", "之外", "上面", "下面", "里面", "外面", "上", "里", "中", "下",
    "内", "外",
];
/// Chữ cuối của mọi chữ trong `LOCS` (test kiểm).
const LOC_TAILS: [char; 7] = ['上', '中', '下', '内', '外', '面', '里'];
/// Cụm số + lượng từ có nghĩa riêng, không tách.
const MEASURE_EXCLUDED: [&str; 26] = [
    "一点", "一下", "一些", "一样", "一直", "一定", "一起", "一切", "一般", "一边", "一旦", "一时",
    "一半", "一丝", "一身", "一面", "一口", "一眼", "一声", "一阵", "一番", "一片", "一场", "一股",
    "一道", "一天",
];

static PUNCT_TAIL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.*?[^，。！？、；：,.!?…])([，。！？、；：,.!?…]+)$").unwrap()
});
static WRONG_TAIL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?i)\s*(bên trên|phía trên|bên trong|phía trong|bên dưới|phía dưới|bên ngoài|phía ngoài)\s*$",
    )
    .unwrap()
});
static WORD: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\w+").unwrap());
static HAN_OR_DIGIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[\x{4e00}-\x{9fff}\d]").unwrap());
static NUM_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(r"^(?:[{NUM_CHARS}]+|\d+(?:\.\d+)?)$")).unwrap()
});
static DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\d+(?:\.\d+)?$").unwrap());
/// Dấu câu của `PUNCT_TAIL`.
fn is_punct(c: char) -> bool {
    matches!(c, '，' | '。' | '！' | '？' | '、' | '；' | '：' | ',' | '.' | '!' | '?' | '…')
}

/// `Some(là chữ số \d)` cho ký tự biết chắc, `None` thì phải hỏi regex. Ngoài
/// chữ ASCII, các khối dấu câu dưới đây không có chữ số `\d` nào (test kiểm
/// lại với regex); khối chữ rộng FF00–FFEF thì trừ các chữ số FF10–FF19.
fn known_digit(c: char) -> Option<bool> {
    match c {
        '\0'..='\u{7f}' => Some(c.is_ascii_digit()),
        '\u{80}'..='\u{ff}'
        | '\u{2000}'..='\u{206f}'
        | '\u{3000}'..='\u{303f}'
        | '\u{ff00}'..='\u{ff0f}'
        | '\u{ff1a}'..='\u{ffef}' => Some(false),
        _ => None,
    }
}

/// Như `HAN_OR_DIGIT.is_match(zh)`, hỏi regex chỉ khi gặp ký tự lạ.
fn han_or_digit(zh: &str) -> bool {
    let mut unknown = false;
    for c in zh.chars() {
        if is_han(c) {
            return true;
        }
        match known_digit(c) {
            Some(true) => return true,
            Some(false) => {}
            None => unknown = true,
        }
    }
    unknown && HAN_OR_DIGIT.is_match(zh)
}

/// Như `MEASURE_SPLIT.captures(zh)` (`^(第?)([số Hán]+)(.)$`): trả (第 hoặc rỗng,
/// phần số, chữ cuối). `第?` tham nên có 第 thì lấy; bỏ 第 thì phần số phải bắt
/// đầu bằng 第, không được. Chữ cuối là một ký tự bất kỳ trừ `\n`.
fn measure_split(zh: &str) -> Option<(&str, &str, &str)> {
    let (ordinal, rest) = match zh.strip_prefix('第') {
        Some(rest) => (&zh[..'第'.len_utf8()], rest),
        None => ("", zh),
    };
    let (last, unit) = rest.char_indices().last()?;
    let number = &rest[..last];
    let all_number = !number.is_empty() && number.chars().all(|c| NUM_CHARS.contains(c));
    (all_number && unit != '\n').then(|| (ordinal, number, &rest[last..]))
}

/// Chữ Hán khối cơ bản (U+4E00–U+9FFF): khớp `HAN_OR_DIGIT`, không bao giờ là `\d`.
/// Dùng để trả lời nhanh trước khi gọi regex; kết quả y như regex.
fn is_han(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c)
}

/// Chữ thường, NFC: để so nghĩa không vướng hoa thường.
fn lowered(text: &str) -> String {
    text.nfc().collect::<String>().to_lowercase()
}

/// Nghĩa ghép từ các phần có cùng dãy chữ (`\w+`) với `vi`, bỏ qua dấu câu, hoa thường.
fn same_order(parts: &[(String, String)], vi: &str) -> bool {
    let joined: Vec<&str> = parts.iter().map(|(_, v)| v.as_str()).collect();
    let (left, right) = (lowered(&joined.join(" ")), lowered(vi));
    let right_words = WORD.find_iter(&right).map(|m| m.as_str());
    WORD.find_iter(&left).map(|m| m.as_str()).eq(right_words)
}

/// Tách lại một khúc bằng từ điển, cụm dài trước (tối đa 15 ký tự); tên đi trước VietPhrase.
pub(crate) fn seg(text: &str, lk: &Lookups) -> Vec<(String, String)> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = Vec::new();
    let mut buffer = String::new();
    let mut i = 0;
    'outer: while i < chars.len() {
        for length in (1..=15.min(chars.len() - i)).rev() {
            buffer.clear();
            buffer.extend(&chars[i..i + length]);
            if let Some(value) = lk.name(&buffer).or_else(|| lk.word(&buffer)) {
                out.push((buffer.clone(), value.to_string()));
                i += length;
                continue 'outer;
            }
        }
        out.push((chars[i].to_string(), chars[i].to_string()));
        i += 1;
    }
    out
}

/// Cụm từ điển giữ trật tự convert (`美丽的少女`, `高台上`, `都不知道`): tách ra
/// để quy tắc đảo được. Cụm có nghĩa khác phép ghép từng phần (đã dịch tay)
/// thì giữ nguyên. Thành ngữ không tách.
fn unfreeze(zh: &str, vi: &str, pos: &PosLexicon, lk: &Lookups) -> Option<Vec<(String, String)>> {
    let n = zh.chars().count();
    let first = zh.chars().next();
    // 的 nằm giữa cụm (không là chữ đầu, chữ cuối).
    let inner_de = n >= 3 && zh.contains('的') && zh.chars().skip(1).take(n - 2).any(|c| c == '的');
    let head_du = n > 2 && matches!(first, Some('都' | '也'));
    let maybe_loc = zh.ends_with(LOC_TAILS) && LOCS.iter().any(|loc| zh.ends_with(loc));
    // Không nhánh nào chạy được thì kết quả vẫn là None: khỏi tra từ loại.
    if !inner_de && !head_du && !maybe_loc {
        return None;
    }
    let tag = pos.get(zh).unwrap_or("");
    if tag == "i" || tag == "l" {
        return None;
    }
    if inner_de {
        let parts: Vec<&str> = zh.split('的').collect();
        if parts.iter().all(|part| !part.is_empty()) {
            let mut out = Vec::new();
            for (index, part) in parts.iter().enumerate() {
                if index > 0 {
                    out.push(("的".to_string(), String::new()));
                }
                out.extend(seg(part, lk));
            }
            return same_order(&out, vi).then_some(out);
        }
    }
    for loc in LOCS {
        if n > loc.chars().count() && zh.ends_with(loc) && WRONG_TAIL.is_match(vi) {
            let mut parts = seg(&zh[..zh.len() - loc.len()], lk);
            parts.push((loc.to_string(), lk.word(loc).unwrap_or(loc).to_string()));
            return same_order(&parts, vi).then_some(parts);
        }
    }
    if let (true, Some(first)) = (head_du, first) {
        let rest = &zh[first.len_utf8()..];
        if let Some(meaning) = lk.word(rest) {
            let head = first.to_string();
            let head_vi = lk.word(&head).unwrap_or("").to_string();
            return Some(vec![(head, head_vi), (rest.to_string(), meaning.to_string())]);
        }
    }
    None
}

fn vi_punct(tail: &str) -> String {
    tail.chars()
        .map(|c| match c {
            '，' | '、' => ',',
            '。' => '.',
            '！' => '!',
            '？' => '?',
            '；' => ';',
            '：' => ':',
            other => other,
        })
        .collect()
}

/// Tách dấu câu cuối, `的` đầu/cuối, cụm đóng băng. Nghĩa giữ ở phần chữ.
pub(crate) fn untie<'q>(raw: Vec<Raw<'q>>, pos: &PosLexicon, lk: &Lookups) -> Vec<Raw<'q>> {
    let mut out = Vec::with_capacity(raw.len() + 4);
    let mut items = raw.into_iter().peekable();
    while let Some(item) = items.next() {
        let (zh, vi) = match item {
            Raw::Tagged(..) => {
                out.push(item);
                continue;
            }
            Raw::Plain(zh, vi) => (zh, vi),
        };
        let next_de = items.peek().is_some_and(|next| next.zh().starts_with('的'));
        // Cụm động từ + danh từ đứng ngay trước 的: tách theo từ để 的 bám đúng định ngữ.
        if next_de && zh.chars().count() >= 3 && pos.get(&zh).is_none() && !lk.is_name(&zh) {
            let words = pos.segment(&zh);
            if words.len() >= 2 && words[0].1.starts_with('v') {
                let pieces: Vec<(String, String)> = words
                    .iter()
                    .map(|(word, _)| (word.clone(), lk.word(word).unwrap_or(word).to_string()))
                    .collect();
                if same_order(&pieces, &vi) {
                    out.extend(pieces.into_iter().map(|(z, v)| Raw::Plain(z, v)));
                    continue;
                }
            }
        }
        // Tính xong trong một câu lệnh để `Captures` (mượn `zh`) hết hạn trước khi `zh` bị move.
        // `PUNCT_TAIL` chỉ khớp khi chữ cuối là dấu câu (`$` là cuối chuỗi): xét trước cho nhanh.
        let split = (zh.ends_with(is_punct) && zh.chars().count() > 1)
            .then(|| PUNCT_TAIL.captures(&zh))
            .flatten()
            .map(|caps| (caps[1].to_string(), caps[2].to_string()));
        let (mut zh, vi, tail) = match split {
            Some((body, tail)) => {
                let vi = vi
                    .trim_end_matches([' ', ',', '.', '!', '?', ';', ':', '…'])
                    .to_string();
                (body, vi, Some(tail))
            }
            None => (zh, vi, None),
        };
        if let Some(frozen) = unfreeze(&zh, &vi, pos, lk) {
            out.extend(frozen.into_iter().map(|(z, v)| Raw::Plain(z, v)));
            if let Some(tail) = tail {
                let tail_vi = vi_punct(&tail);
                out.push(Raw::Plain(tail, tail_vi));
            }
            continue;
        }
        if zh.starts_with('的')
            && zh.chars().count() >= 2
            && !KEEP_DE.contains(&zh.as_str())
            && pos.get(&zh['的'.len_utf8()..]).is_some()
        {
            out.push(Raw::Plain("的".to_string(), String::new()));
            zh = zh['的'.len_utf8()..].to_string();
        }
        let split_tail = zh.ends_with('的')
            && zh.chars().count() >= 2
            && !KEEP_TAIL_DE.contains(&zh.as_str())
            && zh
                .strip_suffix('的')
                .is_some_and(|head| pos.get(head).is_some() || lk.has_word(head));
        if split_tail {
            let head = zh[..zh.len() - '的'.len_utf8()].to_string();
            out.push(Raw::Plain(head, vi));
            out.push(Raw::Plain("的".to_string(), String::new()));
        } else {
            out.push(Raw::Plain(zh, vi));
        }
        if let Some(tail) = tail {
            let tail_vi = vi_punct(&tail);
            out.push(Raw::Plain(tail, tail_vi));
        }
    }
    out
}

/// Chuẩn hoá đầu vào của QT chèn dấu cách quanh 〇 ("一 〇 八"). Token toàn chữ số có 〇
/// được trả về như văn bản gốc, để còn nhận ra là số.
fn unspace_zero(zh: String) -> String {
    let only_number = zh.chars().all(|c| c.is_whitespace() || NUM_CHARS.contains(c));
    if only_number && zh.contains('〇') && zh.contains(char::is_whitespace) {
        zh.chars().filter(|c| !c.is_whitespace()).collect()
    } else {
        zh
    }
}

/// Tách hư từ, gắn nhãn, nhận số và tách số + lượng từ.
pub(crate) fn prepare<'q>(
    raw: Vec<Raw<'q>>,
    pos: &'q PosLexicon,
    lk: &Lookups,
    classes: &FxHashMap<String, Class>,
) -> Vec<Token<'q>> {
    let measure = classes.get("lượng-từ");
    let raw = untie(raw, pos, lk);
    let mut out = Vec::with_capacity(raw.len() + 4);
    for item in raw {
        let (zh, vi) = match item {
            Raw::Tagged(zh, vi, tag) => {
                out.push(Token::new(zh, vi, tag));
                continue;
            }
            Raw::Plain(zh, vi) => (unspace_zero(zh), vi),
        };
        if !han_or_digit(&zh) {
            out.push(Token::new(zh, vi, "w"));
            continue;
        }
        // Chữ đầu là chữ Hán ngoài bộ chữ số thì `NUM_TOKEN` không khớp được.
        let head = zh.chars().next();
        let maybe_number = !head.is_some_and(|c| is_han(c) && !NUM_CHARS.contains(c));
        if maybe_number
            && NUM_TOKEN.is_match(&zh)
            && !["一", "万", "千", "百", "十"].contains(&zh.as_str())
        {
            if DIGITS.is_match(&zh) {
                // Như bản thử: int(float(zh)), phần thập phân bị cắt.
                match zh.parse::<f64>() {
                    Ok(value) => out.push(Token::number(zh, vi, value as i64, true)),
                    Err(_) => out.push(Token::new(zh, vi, "m")),
                }
            } else if let Some(value) = zh_value(&zh) {
                out.push(Token::number(zh, vi, value, false));
            } else {
                out.push(Token::new(zh, vi, "m"));
            }
            continue;
        }
        if zh == "一" {
            out.push(Token::number(zh, vi, 1, false));
            continue;
        }
        if let (Some(measure), Some((ordinal, number, unit))) = (measure, measure_split(&zh)) {
            if measure.contains_key(unit)
                && !["万", "千", "百"].contains(&number)
                && !MEASURE_EXCLUDED.contains(&zh.as_str())
            {
                if let Some(value) = zh_value(number) {
                    let unit_vi = measure
                        .get(unit)
                        .filter(|v| !v.is_empty())
                        .cloned()
                        .unwrap_or_else(|| vi.clone());
                    if !ordinal.is_empty() {
                        out.push(Token::new("第", "thứ", "lit"));
                    }
                    out.push(Token::number(number, number_words(value), value, false));
                    out.push(Token::new(unit, unit_vi, "q"));
                    continue;
                }
            }
        }
        let tag = if lk.is_name(&zh) {
            "nr"
        } else if matches!(zh.as_str(), "时候" | "时" | "时间点") {
            "khung"
        } else if zh.ends_with('是') {
            "hệ-từ"
        } else {
            pos.phrase(&zh)
        };
        out.push(Token::new(zh, vi, tag));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> FxHashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn plain(pairs: &[(&str, &str)]) -> Vec<Raw<'static>> {
        pairs.iter().map(|(z, v)| Raw::Plain(z.to_string(), v.to_string())).collect()
    }

    #[test]
    fn untie_splits_punctuation_and_trailing_de() {
        let pos = PosLexicon::build("她 9 r\n", "");
        let names = map(&[]);
        let lexicon = map(&[]);
        let lk = Lookups { names: &names, lexicon: &lexicon };
        assert_eq!(
            untie(plain(&[("他走了。", "hắn đi nha.")]), &pos, &lk),
            plain(&[("他走了", "hắn đi nha"), ("。", ".")])
        );
        assert_eq!(
            untie(plain(&[("她的", "nàng"), ("手", "tay")]), &pos, &lk),
            plain(&[("她", "nàng"), ("的", ""), ("手", "tay")])
        );
    }

    #[test]
    fn unfreeze_only_when_meaning_keeps_source_order() {
        let pos = PosLexicon::build("", "");
        let names = map(&[]);
        let lexicon = map(&[
            ("美丽", "mỹ lệ"),
            ("少女", "thiếu nữ"),
            ("不好", "không tốt"),
            ("预感", "dự cảm"),
        ]);
        let lk = Lookups { names: &names, lexicon: &lexicon };
        assert_eq!(
            untie(plain(&[("美丽的少女", "mỹ lệ thiếu nữ")]), &pos, &lk),
            plain(&[("美丽", "mỹ lệ"), ("的", ""), ("少女", "thiếu nữ")])
        );
        // Đã dịch tay theo trật tự tiếng Việt: giữ nguyên.
        assert_eq!(
            untie(plain(&[("不好的预感", "dự cảm không tốt")]), &pos, &lk),
            plain(&[("不好的预感", "dự cảm không tốt")])
        );
    }

    #[test]
    fn prepare_tags_names_numbers_and_measure_words() {
        let pos = PosLexicon::build("桌子 9 n\n", "");
        let names = map(&[("叶妩", "Diệp Vũ")]);
        let lexicon = map(&[]);
        let lk = Lookups { names: &names, lexicon: &lexicon };
        let mut classes: FxHashMap<String, Class> = FxHashMap::default();
        classes.insert("lượng-từ".to_string(), map(&[("个", "cái")]));
        let toks = prepare(
            plain(&[("叶妩", "Diệp Vũ"), ("三个", "ba cái"), ("三百五十", "350"), ("12", "12"), ("桌子", "cái bàn"), ("，", ",")]),
            &pos,
            &lk,
            &classes,
        );
        let view: Vec<(&str, &str, &str, Option<i64>)> = toks
            .iter()
            .map(|t| (t.zh.as_str(), t.vi.as_str(), t.tag, t.val))
            .collect();
        assert_eq!(
            view,
            vec![
                ("叶妩", "Diệp Vũ", "nr", None),
                ("三", "ba", "số", Some(3)),
                ("个", "cái", "q", None),
                ("三百五十", "350", "số", Some(350)),
                ("12", "12", "số", Some(12)),
                ("桌子", "cái bàn", "n", None),
                ("，", ",", "w", None),
            ]
        );
        assert!(toks[4].digit && !toks[1].digit);
    }

    #[test]
    fn number_token_with_qt_spaces_around_zero_is_still_a_number() {
        let pos = PosLexicon::build("", "");
        let (names, lexicon) = (map(&[]), map(&[]));
        let lk = Lookups { names: &names, lexicon: &lexicon };
        let toks = prepare(plain(&[("一 〇 八", "108")]), &pos, &lk, &FxHashMap::default());
        let view: Vec<_> = toks.iter().map(|t| (t.zh.as_str(), t.tag, t.val)).collect();
        assert_eq!(view, vec![("一〇八", "số", Some(108))]);
    }

    #[test]
    fn hand_written_sets_match_their_sources() {
        assert!(LOCS.iter().all(|loc| loc.ends_with(LOC_TAILS)));
        // `PUNCT_TAIL` khớp "a" + c đúng khi c thuộc lớp dấu câu của nó (mọi dấu đều trong BMP).
        for c in (0..=0xffff).filter_map(char::from_u32) {
            assert_eq!(is_punct(c), PUNCT_TAIL.is_match(&format!("a{c}")), "{c:?}");
        }
    }

    #[test]
    fn fast_paths_match_the_regexes() {
        let digit = Regex::new(r"\d").unwrap();
        for c in (0..=0x10ffff).filter_map(char::from_u32) {
            if let Some(known) = known_digit(c) {
                assert_eq!(known, digit.is_match(c.encode_utf8(&mut [0; 4])), "{c:?}");
            }
            if is_han(c) {
                assert!(!digit.is_match(c.encode_utf8(&mut [0; 4])), "{c:?}");
            }
        }
        let measure = Regex::new(&format!(r"^(第?)([{NUM_CHARS}]+)(.)$")).unwrap();
        let samples = [
            "一个", "第一", "第一次", "第三层", "三百五十个", "两", "第", "一一", "第一\n",
            "个", "五万", "第五十章", "第个", "x一", "一", "",
        ];
        for zh in samples {
            let by_regex = measure.captures(zh).map(|c| {
                (c.get(1).unwrap().as_str(), c.get(2).unwrap().as_str(), c.get(3).unwrap().as_str())
            });
            assert_eq!(measure_split(zh), by_regex, "{zh:?}");
            assert_eq!(han_or_digit(zh), HAN_OR_DIGIT.is_match(zh), "{zh:?}");
        }
        for zh in ["，", "“", "……", "ａ", "１", "٣", "x", "(", "  "] {
            assert_eq!(han_or_digit(zh), HAN_OR_DIGIT.is_match(zh), "{zh:?}");
        }
    }

    #[test]
    fn seg_prefers_names_then_lexicon() {
        let names = map(&[("出发", "Xuất Phát")]);
        let lexicon = map(&[("出发", "xuất phát"), ("出", "ra")]);
        let lk = Lookups { names: &names, lexicon: &lexicon };
        assert_eq!(seg("出发", &lk), vec![("出发".to_string(), "Xuất Phát".to_string())]);
        let empty = map(&[]);
        let lk = Lookups { names: &empty, lexicon: &lexicon };
        assert_eq!(seg("出发了", &lk), vec![
            ("出发".to_string(), "xuất phát".to_string()),
            ("了".to_string(), "了".to_string()),
        ]);
    }
}
