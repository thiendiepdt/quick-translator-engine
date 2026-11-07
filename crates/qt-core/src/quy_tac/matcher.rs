//! Chạy các nhóm quy tắc trên token: khớp mẫu, điền khuôn, gộp khúc thành một
//! token mang nhãn kết quả. Port từ `match`, `fill`, `run_section` của bản thử,
//! giữ cả chỗ kỳ quặc: token gộp chỉ giữ chữ Hán của phần bên phải; quy tắc
//! khớp mà không tiêu token nào thì token hiện tại bị bỏ.

use std::cell::Cell;
use std::sync::LazyLock;

use regex::Regex;
use rustc_hash::FxHashSet;

use super::numbers::num_text;
use super::prepare::Token;
use super::program::{
    Builtin, CharBloom, El, ElKind, LeftKind, Part, Program, RefKey, Rule, Section, Slot,
    SlotKind,
};

const PRONOUNS: &[&str] = &[
    "我", "你", "他", "她", "它", "您", "咱", "我们", "你们", "他们", "她们", "它们", "咱们", "大家",
    "大伙", "自己", "人家", "别人", "对方", "本座", "本王", "在下", "老夫", "朕", "贫道", "小女子",
    "谁", "彼此", "众人",
];
const DEMONSTRATIVE: &[&str] = &[
    "这", "那", "此", "该", "这个", "那个", "这些", "那些", "这种", "那种", "这样", "那样", "这位",
    "那位", "这名", "那名",
];
const CLAUSE_OPEN: &[&str] = &[
    "但", "但是", "可是", "可", "而", "却", "然后", "于是", "所以", "因为", "如果", "虽然", "不过",
    "只是", "就", "便", "才", "并", "并且", "而且", "或者", "还是", "甚至", "只要", "要是",
];
const AUX: &[&str] = &[
    "不", "没", "没有", "未", "能", "会", "要", "想", "可以", "应该", "已经", "曾", "曾经", "正",
    "正在", "最", "很", "更", "所", "都", "也", "还", "再", "又", "才", "就", "刚", "刚刚", "总",
    "总是", "一直", "不会", "不能", "不想", "不敢", "能够", "可能", "将要", "快要", "敢", "肯",
    "愿意", "需要", "必须", "该", "得",
];
const STOP_VERB: &[&str] = &["是", "说", "道", "问", "笑道", "说道", "喊道", "叫道"];
const NOUN_TAGS: &[&str] = &["n", "ng", "nz", "vn", "an", "j", "cụm", "nrfg"];
const ADJ_TAGS: &[&str] = &["a", "ad", "ag", "z", "b"];

static BO_TAI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\s*(tại|ở|trên|tới|vào|đến)\s*$").unwrap());

fn has(list: &[&str], word: &str) -> bool {
    list.contains(&word)
}

/// Như `has(list, word)`, tra bảng băm; lọc Bloom trên chữ đầu khỏi băm phần lớn từ.
struct WordSet {
    words: FxHashSet<&'static str>,
    heads: CharBloom,
}

impl WordSet {
    fn new(list: &[&'static str]) -> WordSet {
        let mut heads = CharBloom::default();
        list.iter().filter_map(|word| word.chars().next()).for_each(|c| heads.insert(c));
        WordSet {
            words: list.iter().copied().collect(),
            heads,
        }
    }

    fn contains(&self, word: &str) -> bool {
        word.chars().next().is_some_and(|c| self.heads.may_contain(c)) && self.words.contains(word)
    }
}

static PRONOUN_SET: LazyLock<WordSet> = LazyLock::new(|| WordSet::new(PRONOUNS));
static DEMONSTRATIVE_SET: LazyLock<WordSet> = LazyLock::new(|| WordSet::new(DEMONSTRATIVE));

fn is_pronoun(word: &str) -> bool {
    PRONOUN_SET.contains(word)
}

fn is_person(t: &Token) -> bool {
    is_pronoun(&t.zh) || t.tag == "người"
}

fn is_name(t: &Token) -> bool {
    t.tag.starts_with("nr") || matches!(t.tag, "ns" | "nt" | "tên")
}

fn is_noun(t: &Token) -> bool {
    has(NOUN_TAGS, t.tag) && !is_pronoun(&t.zh)
}

fn nominal(t: &Token) -> bool {
    is_noun(t)
        || is_person(t)
        || is_name(t)
        || matches!(t.tag, "s" | "t" | "nơi" | "cụm" | "số-lượng" | "thời-gian")
}

fn verbal(t: &Token) -> bool {
    t.tag.starts_with('v') && !has(STOP_VERB, &t.zh) && !t.zh.starts_with('是')
}

/// Kết quả khớp khoá của một token, tính một lần cho mọi khoá: bit theo
/// `Builtin` và theo lớp từ. Khoá chỉ đọc `zh`, `vi`, `tag` của token, mà token
/// không bị sửa sau khi tạo, nên nhớ lại là đúng. Bit lớp từ còn tuỳ `Program`
/// (`class_bits`): mỗi token chỉ được khớp với một `Program`, như `apply_program`.
/// Không tính vào phép so sánh token.
#[derive(Debug, Clone, Default)]
pub(crate) struct SlotMemo {
    builtins: Cell<Option<u32>>,
    classes: Cell<Option<u64>>,
}

impl PartialEq for SlotMemo {
    fn eq(&self, _: &SlotMemo) -> bool {
        true
    }
}

impl Eq for SlotMemo {}

/// Mọi nhãn có sẵn của `t` trong một lượt, bit thứ `Builtin as u32`.
fn builtin_bits(t: &Token) -> u32 {
    let tag = t.tag;
    let pronoun = is_pronoun(&t.zh);
    let person = pronoun || tag == "người";
    let name = is_name(t);
    let noun = has(NOUN_TAGS, tag) && !pronoun;
    let verb = tag.starts_with('v');
    let count = t.zh.chars().count();
    let tests = [
        (Builtin::DaiTu, pronoun),
        (Builtin::Ten, name),
        (Builtin::Nguoi, person || name),
        (Builtin::DanhTu, noun),
        (Builtin::Cum, noun || person || name || tag == "s"),
        (Builtin::DongTu, verb),
        (Builtin::DanhDong, tag == "vn"),
        (Builtin::HeTu, tag == "hệ-từ"),
        (Builtin::CoDich, count >= 2 && t.zh.ends_with('的') && !t.vi.trim().is_empty()),
        (Builtin::TinhTu, has(ADJ_TAGS, tag) || tag == "tính-cụm"),
        (Builtin::ThanhNgu, matches!(tag, "i" | "l")),
        (Builtin::So, tag == "số"),
        (Builtin::SoLuong, tag == "số-lượng"),
        (Builtin::ThoiGian, matches!(tag, "t" | "tg" | "thời-gian")),
        (Builtin::Noi, matches!(tag, "s" | "nơi")),
        (Builtin::PhoTu, tag.starts_with('d') && tag != "dấu"),
        (Builtin::Dau, tag == "w"),
        (Builtin::ChiDinh, DEMONSTRATIVE_SET.contains(&t.zh)),
        (Builtin::NoiCum, tag == "nơi-cụm"),
        (Builtin::NoiTu, matches!(tag, "nơi-cụm" | "s" | "f") && count > 1),
        (Builtin::DongTuTai, count >= 2 && t.zh.ends_with('在') && verb),
    ];
    tests
        .iter()
        .filter(|(_, hit)| *hit)
        .fold(0, |bits, (builtin, _)| bits | (1 << *builtin as u32))
}

fn builtins_of(t: &Token) -> u32 {
    t.memo.builtins.get().unwrap_or_else(|| {
        let bits = builtin_bits(t);
        t.memo.builtins.set(Some(bits));
        bits
    })
}

fn classes_of(program: &Program, t: &Token) -> u64 {
    t.memo.classes.get().unwrap_or_else(|| {
        let bits = program.class_bits.get(t.zh.as_str()).copied().unwrap_or(0);
        t.memo.classes.set(Some(bits));
        bits
    })
}

fn slot_ok(program: &Program, slot: &Slot, t: &Token) -> bool {
    match slot.kind {
        SlotKind::Builtin(builtin) => builtins_of(t) & (1 << builtin as u32) != 0,
        SlotKind::Class(k) if k < 64 => classes_of(program, t) & (1 << k) != 0,
        SlotKind::Class(_) => program
            .classes
            .get(&*slot.name)
            .is_some_and(|class| class.contains_key(&t.zh)),
        SlotKind::Unknown => false,
    }
}

#[derive(Debug, Clone, Copy)]
enum Cap {
    Empty,
    One(usize),
    Toks(usize, usize),
    Out(usize, usize),
}

struct Capture<'p> {
    name: &'p str,
    cap: Cap,
}

/// Nghĩa của token khi đứng giữa câu: tên giữ hoa, còn lại hạ chữ đầu.
fn low(t: &Token) -> String {
    if is_name(t) || t.vi.is_empty() {
        return t.vi.trim().to_string();
    }
    let mut chars = t.vi.chars();
    let mut out = String::with_capacity(t.vi.len() + 4);
    if let Some(first) = chars.next() {
        out.extend(first.to_lowercase());
    }
    out.push_str(chars.as_str());
    // Như `.trim().to_string()`, cắt ngay trên chuỗi vừa dựng.
    out.truncate(out.trim_end().len());
    let start = out.len() - out.trim_start().len();
    out.drain(..start);
    out
}

/// Số token liền nhau ghép đúng thành một chữ cố định; 0 nếu không khớp.
/// Ghép ít token nhất thắng; chỉ ghép thêm khi phần đã ghép còn ngắn hơn chữ
/// cố định dài nhất. So từng khúc với phần tương ứng của mỗi chữ, khỏi dựng chuỗi.
fn lit_span(toks: &[Token], j: usize, lits: &[String]) -> usize {
    if j >= toks.len() || lits.len() > 64 {
        return lit_span_slow(toks, j, lits);
    }
    // Bit thứ `n`: phần đã ghép còn là đầu của `lits[n]`.
    let mut alive: u64 = if lits.len() == 64 { u64::MAX } else { (1 << lits.len()) - 1 };
    let (mut bytes, mut chars) = (0, 0);
    let mut longest = None;
    for (k, tok) in toks.iter().enumerate().skip(j) {
        if k > j {
            let longest = *longest
                .get_or_insert_with(|| lits.iter().map(|lit| lit.chars().count()).max());
            if chars >= longest.unwrap_or(0) {
                break;
            }
        }
        let piece = tok.zh.as_bytes();
        let end = bytes + piece.len();
        let mut done = false;
        for (n, lit) in lits.iter().enumerate() {
            if alive & (1 << n) == 0 {
                continue;
            }
            if lit.as_bytes().get(bytes..end) != Some(piece) {
                alive &= !(1 << n);
            } else if lit.len() == end {
                done = true;
            }
        }
        if done {
            return k - j + 1;
        }
        if alive == 0 {
            return 0;
        }
        bytes = end;
        chars += tok.zh.chars().count();
    }
    0
}

/// Bản gốc của `lit_span` (dựng chuỗi ghép), cho trường hợp hơn 64 chữ cố định.
fn lit_span_slow(toks: &[Token], j: usize, lits: &[String]) -> usize {
    let Some(first) = toks.get(j) else { return 0 };
    let mut acc = first.zh.clone();
    if lits.iter().any(|lit| *lit == acc) {
        return 1;
    }
    let longest = lits.iter().map(|lit| lit.chars().count()).max().unwrap_or(0);
    let mut k = j + 1;
    while k < toks.len() && acc.chars().count() < longest {
        acc.push_str(&toks[k].zh);
        k += 1;
        if lits.iter().any(|lit| *lit == acc) {
            return k - j;
        }
    }
    0
}

/// Mệnh đề định ngữ kết thúc ở cuối `out`: `[chủ ngữ] [phó từ] động từ [tân ngữ]`.
fn clause_span(program: &Program, out: &[Token]) -> Option<usize> {
    let main = program.classes.get("động-từ-chính");
    let in_main = |word: &str| main.is_some_and(|m| m.contains_key(word));
    let starts_main = |word: &str| {
        main.is_some_and(|m| {
            m.keys().any(|key| {
                word.starts_with(key.as_str()) && word.chars().count() > key.chars().count()
            })
        })
    };
    let mut k = out.len();
    let start_object = k;
    while k > 0 && start_object - k < 2 && nominal(&out[k - 1]) {
        k -= 1;
    }
    let has_object = k != start_object;
    if k == 0 || !verbal(&out[k - 1]) {
        return None;
    }
    let verb = &out[k - 1];
    if (in_main(&verb.zh) && has_object) || starts_main(&verb.zh) {
        return None;
    }
    k -= 1;
    while k > 0 && (has(AUX, &out[k - 1].zh) || matches!(out[k - 1].tag, "d" | "ad")) {
        k -= 1;
    }
    if k > 0 && (is_person(&out[k - 1]) || is_name(&out[k - 1]) || is_noun(&out[k - 1])) {
        k -= 1;
    }
    Some(k)
}

/// Khúc bên trái lùi tới ranh giới vế (tối đa 16 token).
fn left_span(program: &Program, out: &[Token], kind: LeftKind) -> usize {
    if kind == LeftKind::MenhDe {
        return clause_span(program, out).unwrap_or(out.len());
    }
    let mut i = out.len();
    while i > 0 && out.len() - i < 16 {
        let t = &out[i - 1];
        if t.tag == "w" || has(CLAUSE_OPEN, &t.zh) {
            break;
        }
        if t.tag == "hệ-từ" || matches!(t.zh.as_str(), "到" | "等" | "等到") {
            break;
        }
        i -= 1;
    }
    i
}

fn ctx_ok(program: &Program, el: &El, tok: Option<&Token>, at_edge: bool) -> bool {
    let caret = matches!(&el.kind, ElKind::Lit(lits) if lits.iter().any(|lit| lit == "^"));
    let mut hit = match (&el.kind, tok) {
        (ElKind::Start | ElKind::End, _) => at_edge,
        (_, None) => caret,
        (ElKind::Lit(lits), Some(t)) => lits.iter().any(|lit| *lit == t.zh),
        (ElKind::Slot(slot) | ElKind::Many(slot), Some(t)) => slot_ok(program, slot, t),
        (_, Some(_)) => false,
    };
    if caret && at_edge {
        hit = true;
    }
    hit != el.neg
}

/// Khớp `rule` tại `toks[i]`. Trả về (số token tiêu bên phải, số token lấy từ `out`, khoá).
fn match_rule<'p>(
    program: &Program,
    rule: &'p Rule,
    out: &[Token],
    toks: &[Token],
    i: usize,
    caps: &mut Vec<Capture<'p>>,
) -> Option<(usize, usize)> {
    let n = toks.len();
    caps.clear();
    let mut j = i;
    let mut take = 0;
    let mut idx = 0;
    if let ElKind::Left(kind) = &rule.pat[0].kind {
        let kind = *kind;
        // Chữ cố định ngay sau khúc bên trái không khớp thì khỏi lùi tìm khúc.
        let next = &rule.pat[1];
        if let (ElKind::Lit(lits), false, false) = (&next.kind, next.opt, next.neg) {
            if lit_span(toks, i, lits) == 0 {
                return None;
            }
        }
        let start = left_span(program, out, kind);
        let span = &out[start..];
        if span.is_empty() {
            return None;
        }
        if kind == LeftKind::VeDong && !span.iter().any(|t| t.tag.starts_with('v') && t.tag != "vn")
        {
            return None;
        }
        caps.push(Capture {
            name: kind.name(),
            cap: Cap::Out(start, out.len()),
        });
        take = span.len();
        idx = 1;
    }
    while idx < rule.pat.len() {
        let el = &rule.pat[idx];
        match &el.kind {
            ElKind::Lit(lits) => {
                let span = lit_span(toks, j, lits);
                if (span > 0) != el.neg {
                    if !el.neg {
                        j += span;
                    }
                } else if !el.opt {
                    return None;
                }
            }
            ElKind::Slot(slot) => {
                let name = &*slot.name;
                if j < n && slot_ok(program, slot, &toks[j]) {
                    caps.push(Capture { name, cap: Cap::One(j) });
                    j += 1;
                } else if !el.opt {
                    return None;
                } else {
                    caps.push(Capture { name, cap: Cap::Empty });
                }
            }
            ElKind::Many(slot) => {
                let mut k = j;
                while k < n && k - j < 3 && slot_ok(program, slot, &toks[k]) {
                    k += 1;
                }
                if k == j {
                    return None;
                }
                caps.push(Capture { name: &slot.name, cap: Cap::Toks(j, k) });
                j = k;
            }
            ElKind::Gap => {
                let ElKind::Lit(next) = &rule.pat[idx + 1].kind else {
                    return None;
                };
                let mut k = j;
                let mut found = false;
                while k < n && k - j <= 12 && toks[k].tag != "w" {
                    if k > j && lit_span(toks, k, next) > 0 {
                        found = true;
                        break;
                    }
                    k += 1;
                }
                if !found {
                    return None;
                }
                caps.push(Capture { name: "đoạn", cap: Cap::Toks(j, k) });
                j = k;
            }
            ElKind::Start | ElKind::End | ElKind::Left(_) => {}
        }
        idx += 1;
    }
    for el in &rule.post {
        let tok = toks.get(j);
        if !ctx_ok(program, el, tok, tok.is_none_or(|t| t.tag == "w")) {
            return None;
        }
    }
    if !rule.pre.is_empty() {
        let tok = out.len().checked_sub(take + 1).map(|back| &out[back]);
        for el in &rule.pre {
            if !ctx_ok(program, el, tok, tok.is_none_or(|t| t.tag == "w")) {
                return None;
            }
        }
    }
    Some((j - i, take))
}

fn value(program: &Program, hit: &Capture, out: &[Token], toks: &[Token]) -> String {
    let join = |slice: &[Token]| {
        slice
            .iter()
            .map(low)
            .filter(|piece| !piece.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    };
    match hit.cap {
        Cap::Empty => String::new(),
        Cap::Toks(a, b) => join(&toks[a..b]),
        Cap::Out(a, b) => join(&out[a..b]),
        Cap::One(index) => {
            let t = &toks[index];
            if let Some(v) = program
                .classes
                .get(hit.name)
                .and_then(|class| class.get(&t.zh))
                .filter(|v| !v.is_empty())
            {
                return v.clone();
            }
            if t.tag == "số" {
                if let Some(v) = t.val {
                    return num_text(v, t.digit, "");
                }
            }
            low(t)
        }
    }
}

fn fill(program: &Program, rule: &Rule, caps: &[Capture], out: &[Token], toks: &[Token]) -> String {
    let mut text = String::new();
    for part in &rule.tmpl {
        match part {
            Part::Text(piece) => text.push_str(piece),
            Part::Ref { key, filter } => {
                let hit = match key {
                    RefKey::Index(index) => caps.get(index - 1),
                    RefKey::Name(name) => caps.iter().find(|c| c.name == name.as_str()),
                };
                let Some(hit) = hit else { continue };
                if let (Some(how), Cap::One(index)) = (filter.as_deref(), hit.cap) {
                    let t = &toks[index];
                    if t.tag == "số" {
                        text.push_str(&match t.val {
                            Some(v) => num_text(v, t.digit, how),
                            None => low(t),
                        });
                        continue;
                    }
                }
                let piece = value(program, hit, out, toks);
                if filter.as_deref() == Some("bỏ-tại") {
                    text.push_str(&BO_TAI.replace(&piece, ""));
                } else {
                    text.push_str(&piece);
                }
            }
        }
    }
    squeeze_spaces(&text)
}

/// Như `Regex::new(r"\s+").replace_all(text, " ").trim()`: `\s` của regex và
/// `char::is_whitespace` cùng là thuộc tính Unicode White_Space.
fn squeeze_spaces(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut space = false;
    for c in text.chars() {
        if c.is_whitespace() {
            space = true;
            continue;
        }
        if space && !out.is_empty() {
            out.push(' ');
        }
        space = false;
        out.push(c);
    }
    out
}

/// Bộ đệm dùng lại giữa các lượt và các nhóm của một đoạn.
struct Scratch<'q> {
    /// Lọc chữ cố định của từng quy tắc trong nhóm: 0 chưa tính, 1 không qua, 2 qua.
    active: Vec<u8>,
    /// Mảng token của lượt trước, giữ lại cho lượt sau khỏi cấp phát.
    spare: Vec<Token<'q>>,
    caps: Vec<Capture<'q>>,
    /// Bit `n`: quy tắc chung `generic[n]` qua lọc chữ cố định.
    generic: Vec<u64>,
}

/// Như bản thử: chữ cố định bắt buộc của quy tắc có trong đoạn lúc vào nhóm
/// (`present`). Tính lần đầu cần tới rồi nhớ trong `state`.
fn is_active(rule: &Rule, state: &mut u8, present: &[u64]) -> bool {
    if *state == 0 {
        let pass = rule
            .needs
            .chunks(present.len())
            .all(|mask| mask.iter().zip(present).any(|(m, p)| m & p != 0));
        *state = 1 + pass as u8;
    }
    *state == 2
}

fn run_section<'q>(
    program: &'q Program,
    section: &'q Section,
    mut toks: Vec<Token<'q>>,
    present: &[u64],
    scratch: &mut Scratch<'q>,
) -> (Vec<Token<'q>>, usize) {
    scratch.active.clear();
    scratch.active.resize(section.rules.len(), 0);
    let heads = &section.heads;
    scratch.generic.clear();
    scratch.generic.resize(heads.words, 0);
    // Hợp khoá đầu của các quy tắc chung qua lọc: chỉ tính bit khoá của token khi cần.
    let (mut any_builtins, mut any_classes) = (0u32, 0u64);
    for (n, &index) in section.generic.iter().enumerate() {
        if is_active(&section.rules[index], &mut scratch.active[index], present) {
            scratch.generic[n / 64] |= 1 << (n % 64);
            any_builtins |= heads.builtin_of[n];
            any_classes |= heads.class_of[n];
        }
    }
    let any_generic = scratch.generic.iter().any(|&word| word != 0);
    let mut fired = 0;
    for _ in 0..section.rounds {
        let mut out = std::mem::take(&mut scratch.spare);
        out.clear();
        out.reserve(toks.len());
        let mut i = 0;
        let mut changed = false;
        while i < toks.len() {
            let head = toks[i].zh.chars().next().filter(|&c| section.key_chars.may_contain(c));
            let left = head.and_then(|c| section.left.get(&c)).map_or(&[][..], Vec::as_slice);
            let first = head.and_then(|c| section.first.get(&c)).map_or(&[][..], Vec::as_slice);
            let caps = &mut scratch.caps;
            // Thứ tự thử như cũ: mở đầu bằng `{vế}`…, bằng chữ cố định, rồi quy tắc
            // chung. Hai nhóm đầu không mở đầu bằng khoá nên khỏi lọc khoá đầu.
            let hit = 'find: {
                for &index in left.iter().chain(first) {
                    let rule = &section.rules[index];
                    if !is_active(rule, &mut scratch.active[index], present) {
                        continue;
                    }
                    if let Some((used, take)) = match_rule(program, rule, &out, &toks, i, caps) {
                        break 'find Some((rule, used, take));
                    }
                }
                if !any_generic {
                    break 'find None;
                }
                // Ứng viên: quy tắc mà khoá đầu đòi một bit token có, cộng quy tắc
                // không lọc được, trong số qua lọc chữ cố định. Duyệt bit tăng dần
                // là đúng thứ tự `generic`.
                let t = &toks[i];
                let tb = if any_builtins != 0 { builtins_of(t) & any_builtins } else { 0 };
                let tc = if any_classes != 0 { classes_of(program, t) & any_classes } else { 0 };
                let words = heads.words;
                for word in 0..words {
                    let mut cand = heads.open[word];
                    let mut bits = tb;
                    while bits != 0 {
                        cand |= heads.by_builtin[bits.trailing_zeros() as usize * words + word];
                        bits &= bits - 1;
                    }
                    let mut bits = tc;
                    while bits != 0 {
                        cand |= heads.by_class[bits.trailing_zeros() as usize * words + word];
                        bits &= bits - 1;
                    }
                    let mut bits = cand & scratch.generic[word];
                    while bits != 0 {
                        let n = word * 64 + bits.trailing_zeros() as usize;
                        bits &= bits - 1;
                        let rule = &section.rules[section.generic[n]];
                        if let Some((used, take)) =
                            match_rule(program, rule, &out, &toks, i, caps)
                        {
                            break 'find Some((rule, used, take));
                        }
                    }
                }
                None
            };
            match hit {
                Some((rule, used, take)) => {
                    let vi = fill(program, rule, caps, &out, &toks);
                    let zh: String = toks[i..i + used].iter().map(|t| t.zh.as_str()).collect();
                    out.truncate(out.len() - take);
                    out.push(Token::new(zh, vi, rule.result.as_str()));
                    fired += 1;
                    i += used.max(1);
                    changed = true;
                }
                None => {
                    // Không gì đọc lại `toks[..i]` (mẫu và khuôn chỉ đọc từ `i` trở
                    // đi và `out`), nên chuyển hẳn token sang thay vì chép.
                    let placeholder = Token::new(String::new(), String::new(), "");
                    out.push(std::mem::replace(&mut toks[i], placeholder));
                    i += 1;
                }
            }
        }
        scratch.spare = std::mem::replace(&mut toks, out);
        if !changed {
            break;
        }
    }
    (toks, fired)
}

/// Chạy lần lượt mọi nhóm quy tắc; trả về token mới và số lần có quy tắc chạy.
///
/// Như bản thử, mỗi nhóm lọc quy tắc theo chữ cố định có trong đoạn lúc vào
/// nhóm. Nhóm không có quy tắc nào chạy thì token y nguyên, nên chỉ quét lại
/// chữ sau nhóm có quy tắc chạy. Nhóm mà mọi quy tắc đều cần chữ cố định đoạn
/// không có thì bỏ qua (không quy tắc nào qua lọc, token y nguyên).
pub(crate) fn apply_program<'q>(
    program: &'q Program,
    mut toks: Vec<Token<'q>>,
) -> (Vec<Token<'q>>, usize) {
    let mut fired = 0;
    let mut present = Vec::new();
    let mut scratch = Scratch {
        active: Vec::new(),
        spare: Vec::new(),
        caps: Vec::new(),
        generic: Vec::new(),
    };
    let mut stale = true;
    for section in &program.sections {
        if stale {
            let mut text = String::with_capacity(toks.iter().map(|t| t.zh.len()).sum());
            toks.iter().for_each(|t| text.push_str(&t.zh));
            program.need_lits.find(&text, &mut present);
            stale = false;
        }
        if !section.always && !section.lits.iter().zip(&present).any(|(m, p)| m & p != 0) {
            continue;
        }
        let (next, count) = run_section(program, section, toks, &present, &mut scratch);
        toks = next;
        fired += count;
        stale = count > 0;
    }
    (toks, fired)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quy_tac::program::Program;

    fn tok(zh: &str, vi: &str, tag: &'static str) -> Token<'static> {
        Token::new(zh, vi, tag)
    }

    fn run(rules: &str, toks: Vec<Token<'static>>) -> (String, usize) {
        let program = Box::leak(Box::new(Program::parse(rules).unwrap()));
        let (toks, fired) = apply_program(program, toks);
        (crate::quy_tac::render::render(&toks), fired)
    }

    #[test]
    fn reorders_locative_with_class_value() {
        let rules = "[lớp]\nvị-trí: 上=trên\n\n[của 1]\n{danh-từ} {vị-trí} = {vị-trí} {danh-từ}\n";
        let (text, fired) = run(rules, vec![tok("桌子", "cái bàn", "n"), tok("上", "bên trên", "f")]);
        assert_eq!((text.as_str(), fired), ("Trên cái bàn", 1));
    }

    #[test]
    fn left_clause_before_de_shi_hou() {
        let rules = "[khung 1]\n{vế-động} 的时候 = khi {vế-động}\n";
        let (text, _) = run(
            rules,
            vec![tok("他", "Hắn", "r"), tok("走进来", "đi tới", "v"), tok("的时候", "thời điểm", "n")],
        );
        assert_eq!(text, "Khi hắn đi tới");
    }

    #[test]
    fn context_start_and_number_filter() {
        let rules = "[lớp]\nlượng-từ: 层=tầng\n\n[lượng 1]\n第 {số} {lượng-từ} = {lượng-từ} thứ {số|thứ}\n\n[nghĩa 1]\n^ | 可是 | = Nhưng -> lit\n";
        let mut four = tok("四", "bốn", "số");
        four.val = Some(4);
        let (text, _) = run(rules, vec![tok("第", "thứ", "lit"), four, tok("层", "tầng", "q")]);
        assert_eq!(text, "Tầng thứ tư");
        let (text, _) = run(rules, vec![tok("可是", "có thể là", "c"), tok("他", "hắn", "r")]);
        assert_eq!(text, "Nhưng hắn");
        let (_, fired) = run(rules, vec![tok("他", "hắn", "r"), tok("可是", "có thể là", "c")]);
        assert_eq!(fired, 0);
    }

    #[test]
    fn squeeze_spaces_matches_the_regex() {
        // `\s` của regex và `char::is_whitespace` phải là cùng một tập ký tự.
        let all: String = (0..=0x10FFFF).filter_map(char::from_u32).collect();
        let spaces = Regex::new(r"\s").unwrap();
        let by_regex: String = spaces.find_iter(&all).map(|m| m.as_str()).collect();
        let by_std: String = all.chars().filter(|c| c.is_whitespace()).collect();
        assert_eq!(by_regex, by_std);
        let spaces = Regex::new(r"\s+").unwrap();
        for text in ["", "  ", " a \t b\u{3000}\u{a0}c  ", "x", "\u{2028}y z\n"] {
            assert_eq!(squeeze_spaces(text), spaces.replace_all(text, " ").trim());
        }
    }

    /// `builtin_bits` viết lại `slot_ok` cũ (so chuỗi tên, `is_noun`, `is_person`…)
    /// trong một lượt: hai bên phải khớp nhau trên mọi nhãn và chữ đáng ngờ.
    #[test]
    fn builtin_bits_match_the_named_predicates() {
        let named = |name: &str, t: &Token| match name {
            "đại-từ" => has(PRONOUNS, &t.zh),
            "tên" => is_name(t),
            "người" => is_person(t) || is_name(t),
            "danh-từ" => is_noun(t),
            "cụm" => is_noun(t) || is_person(t) || is_name(t) || t.tag == "s",
            "động-từ" => t.tag.starts_with('v'),
            "danh-động" => t.tag == "vn",
            "hệ-từ" => t.tag == "hệ-từ",
            "có-đích" => {
                t.zh.chars().count() >= 2 && t.zh.ends_with('的') && !t.vi.trim().is_empty()
            }
            "tính-từ" => has(ADJ_TAGS, t.tag) || t.tag == "tính-cụm",
            "thành-ngữ" => matches!(t.tag, "i" | "l"),
            "số" => t.tag == "số",
            "số-lượng" => t.tag == "số-lượng",
            "thời-gian" => matches!(t.tag, "t" | "tg" | "thời-gian"),
            "nơi" => matches!(t.tag, "s" | "nơi"),
            "phó-từ" => t.tag.starts_with('d') && t.tag != "dấu",
            "dấu" => t.tag == "w",
            "chỉ-định" => has(DEMONSTRATIVE, &t.zh),
            "nơi-cụm" => t.tag == "nơi-cụm",
            "nơi-từ" => matches!(t.tag, "nơi-cụm" | "s" | "f") && t.zh.chars().count() > 1,
            "động-từ-tại" => {
                t.zh.chars().count() >= 2 && t.zh.ends_with('在') && t.tag.starts_with('v')
            }
            _ => unreachable!("{name}"),
        };
        let mut tags: Vec<&str> = NOUN_TAGS.iter().chain(ADJ_TAGS).copied().collect();
        tags.extend([
            "nr", "nrfg", "ns", "nt", "tên", "người", "s", "t", "tg", "thời-gian", "nơi", "v",
            "vn", "vd", "hệ-từ", "tính-cụm", "i", "l", "số", "số-lượng", "d", "dấu", "w",
            "nơi-cụm", "f", "m", "",
        ]);
        let mut words: Vec<&str> = PRONOUNS.iter().chain(DEMONSTRATIVE).copied().collect();
        words.extend(["", "的", "好的", "坐在", "在", "上面", "桌子", "我的", "这个人"]);
        for &tag in &tags {
            for &zh in &words {
                for vi in ["", " ", "x"] {
                    let t = tok(zh, vi, tag);
                    let bits = builtin_bits(&t);
                    for name in crate::quy_tac::program::BUILTIN_SLOTS {
                        let builtin = Builtin::from_name(name).unwrap();
                        let hit = bits & (1 << builtin as u32) != 0;
                        assert_eq!(hit, named(name, &t), "{name} {zh:?} {vi:?} {tag:?}");
                    }
                }
            }
        }
    }

    #[test]
    fn lit_span_matches_the_string_version() {
        let pieces = ["", "的", "时", "候", "的时", "时候", "a", "ab", "b"];
        let mut seed: u64 = 11;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) as usize
        };
        for _ in 0..20_000 {
            let toks: Vec<Token> = (0..next() % 6)
                .map(|_| tok(pieces[next() % pieces.len()], "", "x"))
                .collect();
            let lits: Vec<String> = (0..1 + next() % 3)
                .map(|_| (0..1 + next() % 3).map(|_| pieces[1 + next() % 8]).collect())
                .collect();
            let j = next() % (toks.len() + 1);
            assert_eq!(lit_span(&toks, j, &lits), lit_span_slow(&toks, j, &lits), "{lits:?}");
        }
    }

    #[test]
    fn names_keep_capitals_and_de_reorders() {
        let rules = "[của 3]\n{tên} 的 {danh-từ}+ = {2} của {1}\n";
        let (text, _) = run(
            rules,
            vec![tok("叶妩", "Diệp Vũ", "nr"), tok("的", "", "uj"), tok("手", "Tay", "n")],
        );
        assert_eq!(text, "Tay của Diệp Vũ");
    }
}
