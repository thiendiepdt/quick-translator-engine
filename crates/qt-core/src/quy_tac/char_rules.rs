//! Nhóm `[số]` (spec mục 8): quét ký tự trên cả đoạn, trước bước tách hư từ.
//!
//! Một khúc được nhận khi nó bắt đầu ở ranh giới token và từ điển không có cụm
//! dài hơn bắt đầu cùng chỗ. Đuôi khúc được cắt giữa một token; phần dư tách lại
//! bằng từ điển cùng token kế tiếp (`分出|发` → `出发`). Khác bản thử một chỗ:
//! phần tách lại không lấn sang khúc số kế tiếp.

use std::sync::LazyLock;

use regex::Regex;

use super::numbers::{num_text, zh_value, DIGIT_WORDS, NUM_CHARS};
use super::prepare::{seg, Lookups, Raw};
use super::program::{has_lit, CharRule, Part, Program, RefKey};

static HAS_NUMBER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&format!(r"[{NUM_CHARS}\d]")).unwrap());
static MIXED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d+(?:\.\d+)?)([十百千万亿]+)$").unwrap());
static SPACES: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s+").unwrap());

struct Span<'q> {
    a: usize,
    b: usize,
    vi: String,
    result: &'q str,
}

fn digit_char(c: char) -> Option<char> {
    if c.is_ascii_digit() {
        return Some(c);
    }
    let index = if c == '〇' {
        0
    } else {
        "零一二三四五六七八九".chars().position(|d| d == c)?
    };
    char::from_digit(index as u32, 10)
}

/// Giá trị của một khoá trong khuôn `[số]`, theo bộ lọc (`số`, `thứ`, `năm`, `từng-chữ`…).
fn char_value(program: &Program, name: &str, text: &str, how: Option<&str>) -> String {
    // Dấu cách QT chèn quanh 〇 nằm trong chuỗi bắt được: bỏ đi trước khi đọc số.
    let squeezed: String;
    let text = if matches!(name, "số" | "năm") {
        squeezed = text.chars().filter(|c| !c.is_whitespace()).collect();
        squeezed.as_str()
    } else {
        text
    };
    if name == "năm" {
        let digits: String = text.chars().filter_map(digit_char).collect();
        if how == Some("từng-chữ") {
            let words: Vec<&str> = digits
                .chars()
                .filter_map(|d| d.to_digit(10))
                .map(|d| DIGIT_WORDS[d as usize])
                .collect();
            return words.join(" ");
        }
        return digits;
    }
    if name != "số" {
        return program
            .classes
            .get(name)
            .and_then(|class| class.get(text))
            .filter(|value| !value.is_empty())
            .cloned()
            .unwrap_or_else(|| text.to_string());
    }
    if let Some(caps) = MIXED.captures(text) {
        let units: Vec<&str> = caps[2]
            .chars()
            .map(|c| match c {
                '十' => "mươi",
                '百' => "trăm",
                '千' => "ngàn",
                '万' => "vạn",
                _ => "ức",
            })
            .collect();
        return format!("{} {}", &caps[1], units.join(" "));
    }
    if text.contains('.') {
        return text.to_string();
    }
    let digit = text.chars().next().is_some_and(|c| c.is_ascii_digit());
    let value = if digit {
        text.parse::<i64>().ok()
    } else {
        zh_value(text)
    };
    let Some(value) = value else {
        return text.to_string();
    };
    if how == Some("năm") && !digit && text.chars().all(|c| "零〇一二三四五六七八九".contains(c)) {
        return text.chars().filter_map(digit_char).collect();
    }
    num_text(value, digit, how.unwrap_or(""))
}

fn fill_char(program: &Program, rule: &CharRule, values: &[Option<&str>]) -> String {
    let mut out = String::new();
    for part in &rule.tmpl {
        match part {
            Part::Text(text) => out.push_str(text),
            Part::Ref { key, filter } => {
                let index = match key {
                    RefKey::Index(index) => Some(index - 1),
                    RefKey::Name(name) => rule.slots.iter().position(|slot| slot == name),
                };
                let Some(index) = index else { continue };
                if let (Some(name), Some(Some(text))) = (rule.slots.get(index), values.get(index)) {
                    out.push_str(&char_value(program, name, text, filter.as_deref()));
                }
            }
        }
    }
    SPACES.replace_all(&out, " ").trim().to_string()
}

pub(crate) fn apply_chars<'q>(
    program: &'q Program,
    mut raw: Vec<(String, String)>,
    lk: &Lookups,
) -> (Vec<Raw<'q>>, usize) {
    let plain = |raw: Vec<(String, String)>| {
        raw.into_iter()
            .map(|(zh, vi)| Raw::Plain(zh, vi))
            .collect::<Vec<_>>()
    };
    let mut text = String::with_capacity(raw.iter().map(|(zh, _)| zh.len()).sum());
    raw.iter().for_each(|(zh, _)| text.push_str(zh));
    if !HAS_NUMBER.is_match(&text) {
        return (plain(raw), 0);
    }
    // Chỉ chạy regex của luật có đủ chữ cố định bắt buộc trong đoạn; luật thiếu
    // chữ thì regex không khớp được, bỏ qua không đổi kết quả.
    let mut present = Vec::new();
    program.char_lits.find(&text, &mut present);
    let rules: Vec<&CharRule> = program
        .char_rules
        .iter()
        .filter(|rule| {
            let has_one = |ids: &Box<[u32]>| ids.iter().any(|&id| has_lit(&present, id as usize));
            rule.needs.as_ref().is_none_or(|needs| needs.iter().all(has_one))
        })
        .collect();
    if rules.is_empty() {
        return (plain(raw), 0);
    }
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut char_of_byte = vec![usize::MAX; text.len() + 1];
    for (char_index, (byte_index, _)) in text.char_indices().enumerate() {
        char_of_byte[byte_index] = char_index;
    }
    char_of_byte[text.len()] = n;
    // bounds[vị trí ký tự] = chỉ số token bắt đầu ở đó.
    let mut bounds: Vec<Option<usize>> = vec![None; n + 1];
    let mut starts = Vec::with_capacity(raw.len() + 1);
    let mut position = 0;
    for (index, (zh, _)) in raw.iter().enumerate() {
        bounds[position] = Some(index);
        starts.push(position);
        position += zh.chars().count();
    }
    bounds[n] = Some(raw.len());
    starts.push(n);

    let mut taken = vec![false; n];
    let mut spans: Vec<Span<'q>> = Vec::new();
    let mut buffer = String::new();
    for rule in rules {
        for caps in rule.regex.captures_iter(&text) {
            let whole = caps.get(0).expect("nhóm 0 luôn có");
            let (a, b) = (char_of_byte[whole.start()], char_of_byte[whole.end()]);
            if a == b || bounds[a].is_none() || taken[a..b].iter().any(|&t| t) {
                continue;
            }
            // Từ điển có cụm dài hơn bắt đầu cùng chỗ thì nhường (两点一线, 一个人…).
            let vetoed = ((b - a + 1)..=15.min(n - a)).any(|length| {
                buffer.clear();
                buffer.extend(&chars[a..a + length]);
                lk.has_word(&buffer)
            });
            if vetoed {
                continue;
            }
            let values: Vec<Option<&str>> = (0..rule.slots.len())
                .map(|i| caps.get(i + 1).map(|m| m.as_str()))
                .collect();
            spans.push(Span {
                a,
                b,
                vi: fill_char(program, rule, &values),
                result: rule.result.as_str(),
            });
            taken[a..b].iter_mut().for_each(|t| *t = true);
        }
    }
    if spans.is_empty() {
        return (plain(raw), 0);
    }
    spans.sort_by_key(|span| span.a);
    let fired = spans.len();
    let mut out = Vec::with_capacity(raw.len() + fired);
    let mut index = 0;
    for (k, span) in spans.iter().enumerate() {
        let first = bounds[span.a].expect("khúc luôn bắt đầu ở ranh giới token");
        // Mỗi token gốc được đọc nhiều nhất một lần: chuyển hẳn sang thay vì chép.
        while index < first {
            let (zh, vi) = std::mem::take(&mut raw[index]);
            out.push(Raw::Plain(zh, vi));
            index += 1;
        }
        out.push(Raw::Tagged(
            chars[span.a..span.b].iter().collect(),
            span.vi.clone(),
            span.result,
        ));
        if let Some(next) = bounds[span.b] {
            index = next;
            continue;
        }
        // Đuôi khúc cắt giữa một token: tách lại phần dư cùng token kế tiếp.
        let mut end = *starts.iter().find(|&&p| p > span.b).expect("luôn có vị trí n");
        let next_span = spans.get(k + 1).map(|s| s.a);
        if let Some(&later) = starts.iter().find(|&&p| p > end) {
            if later - span.b <= 6 && next_span.is_none_or(|a| later <= a) {
                end = later;
            }
        }
        let rest: String = chars[span.b..end].iter().collect();
        out.extend(seg(&rest, lk).into_iter().map(|(z, v)| Raw::Plain(z, v)));
        index = bounds[end].expect("end là ranh giới token");
    }
    while index < raw.len() {
        let (zh, vi) = std::mem::take(&mut raw[index]);
        out.push(Raw::Plain(zh, vi));
        index += 1;
    }
    (out, fired)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hash::FxHashMap;

    const RULES: &str = "[lớp]
buổi: 晚上=tối

[số]
{buổi}{số:1-2}点半 = {2} giờ rưỡi {1} -> thời-gian
{số}点半 = {1} giờ rưỡi -> thời-gian
{số:1-2}点{số:1-2}分 = {1} giờ {2} phút -> thời-gian
{năm:3-4}年 = năm {1} -> thời-gian
{số:1-2}点{năm:1-2} = {1} phẩy {2|từng-chữ} -> m
";

    fn map(pairs: &[(&str, &str)]) -> FxHashMap<String, String> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    fn raw(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs.iter().map(|(z, v)| (z.to_string(), v.to_string())).collect()
    }

    #[test]
    fn replaces_number_spans_with_templates() {
        let program = Program::parse(RULES).unwrap();
        let (names, lexicon) = (map(&[]), map(&[]));
        let lk = Lookups { names: &names, lexicon: &lexicon };
        let tokens = raw(&[("晚上", "buổi tối"), ("九点", "chín điểm"), ("半", "bán")]);
        let (out, fired) = apply_chars(&program, tokens, &lk);
        assert_eq!(fired, 1);
        assert_eq!(out, vec![Raw::Tagged("晚上九点半".into(), "chín giờ rưỡi tối".into(), "thời-gian")]);
        let (out, _) = apply_chars(&program, raw(&[("二〇一九年", "x")]), &lk);
        assert_eq!(out, vec![Raw::Tagged("二〇一九年".into(), "năm 2019".into(), "thời-gian")]);
        // QT chèn dấu cách quanh 〇: token nguồn là "二 〇 一九".
        let tokens = raw(&[("二 〇 一九", "9"), ("年", "năm")]);
        let (out, _) = apply_chars(&program, tokens, &lk);
        assert_eq!(out, vec![Raw::Tagged("二 〇 一九年".into(), "năm 2019".into(), "thời-gian")]);
        let (out, _) = apply_chars(&program, raw(&[("三点一四", "x")]), &lk);
        assert_eq!(out, vec![Raw::Tagged("三点一四".into(), "ba phẩy một bốn".into(), "m")]);
    }

    /// Token đúng dạng engine ra sau chuẩn hoá của QT (dấu cách quanh 〇).
    #[test]
    fn tolerates_only_the_spaces_qt_inserts_around_zero() {
        let program = Program::parse(
            "[số]
{số:1-3}个 = {1} cái -> m
{năm:3-4}年 = năm {1} -> thời-gian
",
        )
        .unwrap();
        let (names, lexicon) = (map(&[]), map(&[]));
        let lk = Lookups { names: &names, lexicon: &lexicon };
        let tagged = |zh: &str, vi: &str, tag| vec![Raw::Tagged(zh.into(), vi.into(), tag)];
        // 一〇八个: ba chữ số, tính cả 〇 ở giữa là một.
        let (out, _) = apply_chars(&program, raw(&[("一 〇 八", "x"), ("个", "cái")]), &lk);
        assert_eq!(out, tagged("一 〇 八个", "một trăm lẻ tám cái", "m"));
        // 二〇〇〇年: QT ra "二 〇〇〇 年" (dấu cách sau 〇 đi theo token kế).
        let (out, _) = apply_chars(&program, raw(&[("二 〇〇〇", "x"), ("年", "năm")]), &lk);
        assert_eq!(out, tagged("二 〇〇〇年", "năm 2000", "thời-gian"));
        // Dấu cách thật giữa hai chữ số không phải 〇 thì không bắc cầu.
        let spaced = raw(&[("一 二", "x"), ("个", "cái")]);
        let (out, fired) = apply_chars(&program, spaced, &lk);
        assert_eq!(fired, 0);
        let plain = |zh: &str, vi: &str| Raw::Plain(zh.into(), vi.into());
        assert_eq!(out, vec![plain("一 二", "x"), plain("个", "cái")]);
    }

    #[test]
    fn longer_dictionary_phrase_vetoes_the_rule() {
        let program = Program::parse(RULES).unwrap();
        let names = map(&[]);
        let lexicon = map(&[("三点半钟", "ba giờ rưỡi đúng")]);
        let lk = Lookups { names: &names, lexicon: &lexicon };
        let (out, fired) = apply_chars(&program, raw(&[("三点半钟", "ba giờ rưỡi đúng")]), &lk);
        assert_eq!(fired, 0);
        assert_eq!(out, vec![Raw::Plain("三点半钟".into(), "ba giờ rưỡi đúng".into())]);
    }

    #[test]
    fn span_cut_inside_a_token_resegments_the_rest() {
        let program = Program::parse(RULES).unwrap();
        let names = map(&[]);
        let lexicon = map(&[("出发", "xuất phát")]);
        let lk = Lookups { names: &names, lexicon: &lexicon };
        let (out, fired) = apply_chars(
            &program,
            raw(&[("三点", "ba điểm"), ("十五", "mười lăm"), ("分出", "phân ra"), ("发", "phát")]),
            &lk,
        );
        assert_eq!(fired, 1);
        assert_eq!(
            out,
            vec![
                Raw::Tagged("三点十五分".into(), "ba giờ mười lăm phút".into(), "thời-gian"),
                Raw::Plain("出发".into(), "xuất phát".into()),
            ]
        );
    }

    #[test]
    fn duration_is_not_a_calendar_year() {
        let program = Program::parse(RULES).unwrap();
        let (names, lexicon) = (map(&[]), map(&[]));
        let lk = Lookups { names: &names, lexicon: &lexicon };
        let (_, fired) = apply_chars(&program, raw(&[("500年", "500 năm")]), &lk);
        assert_eq!(fired, 0);
    }
}
