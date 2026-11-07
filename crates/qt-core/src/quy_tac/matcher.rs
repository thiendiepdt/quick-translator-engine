//! Chạy các nhóm quy tắc trên token: khớp mẫu, điền khuôn, gộp khúc thành một
//! token mang nhãn kết quả. Port từ `match`, `fill`, `run_section` của bản thử,
//! giữ cả chỗ kỳ quặc: token gộp chỉ giữ chữ Hán của phần bên phải; quy tắc
//! khớp mà không tiêu token nào thì token hiện tại bị bỏ.

use std::sync::LazyLock;

use regex::Regex;

use super::numbers::num_text;
use super::prepare::Token;
use super::program::{El, ElKind, LeftKind, Part, Program, RefKey, Rule, Section};

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
static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

fn has(list: &[&str], word: &str) -> bool {
    list.contains(&word)
}

fn is_person(t: &Token) -> bool {
    has(PRONOUNS, &t.zh) || t.tag == "người"
}

fn is_name(t: &Token) -> bool {
    t.tag.starts_with("nr") || matches!(t.tag, "ns" | "nt" | "tên")
}

fn is_noun(t: &Token) -> bool {
    has(NOUN_TAGS, t.tag) && !has(PRONOUNS, &t.zh)
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

fn slot_ok(program: &Program, name: &str, t: &Token) -> bool {
    if let Some(class) = program.classes.get(name) {
        return class.contains_key(&t.zh);
    }
    match name {
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
        _ => false,
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
    let first: String = chars.next().map(|c| c.to_lowercase().collect()).unwrap_or_default();
    format!("{first}{}", chars.as_str()).trim().to_string()
}

/// Số token liền nhau ghép đúng thành một chữ cố định; 0 nếu không khớp.
fn lit_span(toks: &[Token], j: usize, lits: &[String]) -> usize {
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
        (ElKind::Slot(name) | ElKind::Many(name), Some(t)) => slot_ok(program, name, t),
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
) -> Option<(usize, usize, Vec<Capture<'p>>)> {
    let n = toks.len();
    let mut caps: Vec<Capture<'p>> = Vec::new();
    let mut j = i;
    let mut take = 0;
    let mut idx = 0;
    if let ElKind::Left(kind) = &rule.pat[0].kind {
        let kind = *kind;
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
            ElKind::Slot(name) => {
                if j < n && slot_ok(program, name, &toks[j]) {
                    caps.push(Capture { name: name.as_str(), cap: Cap::One(j) });
                    j += 1;
                } else if !el.opt {
                    return None;
                } else {
                    caps.push(Capture { name: name.as_str(), cap: Cap::Empty });
                }
            }
            ElKind::Many(name) => {
                let mut k = j;
                while k < n && k - j < 3 && slot_ok(program, name, &toks[k]) {
                    k += 1;
                }
                if k == j {
                    return None;
                }
                caps.push(Capture { name: name.as_str(), cap: Cap::Toks(j, k) });
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
    Some((j - i, take, caps))
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
    SPACES.replace_all(&text, " ").trim().to_string()
}

fn run_section<'q>(
    program: &'q Program,
    section: &'q Section,
    mut toks: Vec<Token<'q>>,
) -> (Vec<Token<'q>>, usize) {
    let text: String = toks.iter().map(|t| t.zh.as_str()).collect();
    let mut fired = 0;
    for _ in 0..section.rounds {
        let mut out: Vec<Token<'q>> = Vec::with_capacity(toks.len());
        let mut i = 0;
        let mut changed = false;
        while i < toks.len() {
            let head = toks[i].zh.chars().next();
            let groups: [&[usize]; 3] = [
                head.and_then(|c| section.left.get(&c)).map_or(&[][..], Vec::as_slice),
                head.and_then(|c| section.first.get(&c)).map_or(&[][..], Vec::as_slice),
                section.generic.as_slice(),
            ];
            let mut done = false;
            'groups: for group in groups {
                for &index in group {
                    let rule = &section.rules[index];
                    if !rule
                        .needs
                        .iter()
                        .all(|lits| lits.iter().any(|lit| text.contains(lit.as_str())))
                    {
                        continue;
                    }
                    let Some((used, take, caps)) = match_rule(program, rule, &out, &toks, i) else {
                        continue;
                    };
                    let vi = fill(program, rule, &caps, &out, &toks);
                    let zh: String = toks[i..i + used].iter().map(|t| t.zh.as_str()).collect();
                    out.truncate(out.len() - take);
                    out.push(Token::new(zh, vi, rule.result.as_str()));
                    fired += 1;
                    i += used.max(1);
                    done = true;
                    changed = true;
                    break 'groups;
                }
            }
            if !done {
                out.push(toks[i].clone());
                i += 1;
            }
        }
        toks = out;
        if !changed {
            break;
        }
    }
    (toks, fired)
}

/// Chạy lần lượt mọi nhóm quy tắc; trả về token mới và số lần có quy tắc chạy.
pub(crate) fn apply_program<'q>(
    program: &'q Program,
    mut toks: Vec<Token<'q>>,
) -> (Vec<Token<'q>>, usize) {
    let mut fired = 0;
    for section in &program.sections {
        let (next, count) = run_section(program, section, toks);
        toks = next;
        fired += count;
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
    fn names_keep_capitals_and_de_reorders() {
        let rules = "[của 3]\n{tên} 的 {danh-từ}+ = {2} của {1}\n";
        let (text, _) = run(
            rules,
            vec![tok("叶妩", "Diệp Vũ", "nr"), tok("的", "", "uj"), tok("手", "Tay", "n")],
        );
        assert_eq!(text, "Tay của Diệp Vũ");
    }
}
