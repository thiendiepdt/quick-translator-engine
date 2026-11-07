//! Ráp token thành câu cho đoạn có quy tắc chạy: nối bằng dấu cách, bỏ cách
//! trước dấu câu / sau ngoặc mở, viết hoa đầu câu. Giống `render` của bản thử.
//!
//! Ba bước của bản thử (ba regex, giữ trong test để so) gộp thành một lượt
//! quét: `\s` của regex là Unicode White_Space, đúng như `char::is_whitespace`;
//! `\w` vẫn hỏi regex, trừ chữ ASCII.

use std::sync::LazyLock;

use regex::Regex;

use super::prepare::Token;

/// `\s+([,.!?:;…”’)\]】])` → `$1`: dấu câu đóng.
fn is_closer(c: char) -> bool {
    matches!(c, ',' | '.' | '!' | '?' | ':' | ';' | '…' | '”' | '’' | ')' | ']' | '】')
}

/// `([“‘(\[【])\s+` → `$1`: ngoặc mở.
fn is_opener(c: char) -> bool {
    matches!(c, '“' | '‘' | '(' | '[' | '【')
}

/// Đầu câu trong `(^|[.!?…]\s+|[“‘]\s*)(\w)`.
fn is_stop(c: char) -> bool {
    matches!(c, '.' | '!' | '?' | '…')
}

fn is_quote(c: char) -> bool {
    matches!(c, '“' | '‘')
}

static WORD_START: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\A\w").unwrap());

/// Ký tự đầu của `rest` là `\w`. ASCII: `\w` là `[0-9A-Za-z_]`.
fn word_start(rest: &str) -> bool {
    match rest.chars().next() {
        Some(c) if c.is_ascii() => c.is_ascii_alphanumeric() || c == '_',
        Some(_) => WORD_START.is_match(rest),
        None => false,
    }
}

/// Đang chờ chữ `\w` để viết hoa: sau dấu dừng (cần thêm ít nhất một khoảng
/// trắng) hay sau ngoặc kép mở (khoảng trắng tuỳ ý).
#[derive(Clone, Copy, PartialEq)]
enum Arm {
    None,
    Stop { spaced: bool },
    Quote,
}

/// Một lượt cho cả ba bước của bản thử, y như
/// `capitalize(drop_space_after(drop_space_before(text)))` (test so):
/// - một quãng trắng bị bỏ khi ký tự kế là dấu đóng, hoặc ký tự khác trắng
///   ngay trước nó là ngoặc mở; quãng trắng không bao giờ dính nhau sau khi bỏ
///   vì giữa hai quãng luôn có ký tự khác trắng;
/// - trên chuỗi đã bỏ, viết hoa chữ `\w` đứng đầu chuỗi, sau dấu dừng + ít
///   nhất một khoảng trắng, hoặc sau ngoặc kép mở + khoảng trắng tuỳ ý; tìm
///   tiếp ngay sau chữ vừa viết hoa. Chữ `\w` không là dấu dừng hay ngoặc.
fn tidy(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 4);
    let mut run: Option<usize> = None;
    let mut after_opener = false;
    let mut arm = Arm::None;
    for (at, c) in text.char_indices() {
        if c.is_whitespace() {
            run.get_or_insert(at);
            continue;
        }
        if let Some(start) = run.take() {
            if !is_closer(c) && !after_opener {
                out.push_str(&text[start..at]);
                if let Arm::Stop { .. } = arm {
                    arm = Arm::Stop { spaced: true };
                }
            }
        }
        let capital = match arm {
            _ if out.is_empty() => word_start(&text[at..]),
            Arm::Stop { spaced: true } | Arm::Quote => word_start(&text[at..]),
            _ => false,
        };
        if capital {
            out.extend(c.to_uppercase());
            arm = Arm::None;
        } else {
            out.push(c);
            arm = if is_stop(c) {
                Arm::Stop { spaced: false }
            } else if is_quote(c) {
                Arm::Quote
            } else {
                Arm::None
            };
        }
        after_opener = is_opener(c);
    }
    if let Some(start) = run {
        if !after_opener {
            out.push_str(&text[start..]);
        }
    }
    out
}

pub(crate) fn render(toks: &[Token]) -> String {
    let mut text = String::with_capacity(toks.iter().map(|t| t.vi.len() + 1).sum());
    for piece in toks.iter().map(|t| t.vi.trim()).filter(|piece| !piece.is_empty()) {
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(piece);
    }
    tidy(&text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quy_tac::prepare::Token;
    use regex::Captures;

    /// Như `SPACE_BEFORE.replace_all(text, "$1")`: bỏ cả quãng trắng đứng ngay trước dấu đóng.
    fn drop_space_before(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut run: Option<usize> = None;
        for (at, c) in text.char_indices() {
            if c.is_whitespace() {
                run.get_or_insert(at);
                continue;
            }
            if let Some(start) = run.take() {
                if !is_closer(c) {
                    out.push_str(&text[start..at]);
                }
            }
            out.push(c);
        }
        if let Some(start) = run {
            out.push_str(&text[start..]);
        }
        out
    }

    /// Như `SPACE_AFTER.replace_all(text, "$1")`: bỏ quãng trắng ngay sau ngoặc mở.
    fn drop_space_after(text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut skip = false;
        for c in text.chars() {
            if skip && c.is_whitespace() {
                continue;
            }
            skip = is_opener(c);
            out.push(c);
        }
        out
    }

    /// Byte ngay sau quãng trắng bắt đầu ở `at`.
    fn skip_space(text: &str, at: usize) -> usize {
        at + text[at..].len() - text[at..].trim_start().len()
    }

    /// Như `CAPITAL.replace_all` với khuôn `{1}{2 viết hoa}`: tìm từ trái sang, mỗi
    /// chỗ khớp là `^`, `[.!?…]` + quãng trắng (ít nhất một), hoặc `[“‘]` + quãng
    /// trắng (có thể rỗng), theo sau là một chữ `\w`; viết hoa chữ đó rồi tìm tiếp
    /// sau nó. Tại một vị trí chỉ một nhánh có thể khớp (chữ `\w` không là dấu câu).
    fn capitalize(text: &str) -> String {
        let word_at = |at: usize| at < text.len() && WORD_START.is_match(&text[at..]);
        let mut out = String::with_capacity(text.len());
        let (mut copied, mut at) = (0, 0);
        while let Some(c) = text[at..].chars().next() {
            let after = at + c.len_utf8();
            let word = if at == 0 && word_at(0) {
                Some(0)
            } else if is_stop(c) {
                let end = skip_space(text, after);
                (end > after && word_at(end)).then_some(end)
            } else if is_quote(c) {
                let end = skip_space(text, after);
                word_at(end).then_some(end)
            } else {
                None
            };
            match word {
                Some(start) => {
                    let w = text[start..].chars().next().expect("word_at có chữ");
                    out.push_str(&text[copied..start]);
                    out.extend(w.to_uppercase());
                    copied = start + w.len_utf8();
                    at = copied;
                }
                None => at = after,
            }
        }
        out.push_str(&text[copied..]);
        out
    }

    #[test]
    fn joins_fixes_spacing_and_capitalizes() {
        let toks: Vec<Token> = ["hắn", "đi", ",", "“", "ta", "nói", "”", "."]
            .iter()
            .map(|vi| Token::new("x", *vi, "n"))
            .collect();
        assert_eq!(render(&toks), "Hắn đi, “Ta nói”.");
    }

    #[test]
    fn scans_match_the_prototype_regexes() {
        // Ba bước của bản thử, viết bằng regex.
        let space_before = Regex::new(r"\s+([,.!?:;…”’)\]】])").unwrap();
        let space_after = Regex::new(r"([“‘(\[【])\s+").unwrap();
        let capital = Regex::new(r"(^|[.!?…]\s+|[“‘]\s*)(\w)").unwrap();
        let by_regex = |text: &str| {
            let text = space_before.replace_all(text, "$1");
            let text = space_after.replace_all(&text, "$1");
            capital
                .replace_all(&text, |caps: &Captures| {
                    format!("{}{}", &caps[1], caps[2].to_uppercase())
                })
                .into_owned()
        };
        let alphabet: Vec<char> = " \u{3000}\u{a0}\ta1ăđß.!?…,;:”’)]】“‘([【-_x\u{301}"
            .chars()
            .collect();
        // Chuỗi giả ngẫu nhiên cố định (LCG) trên bảng chữ có đủ loại ký tự.
        let mut seed: u64 = 7;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) as usize
        };
        for _ in 0..20_000 {
            let length = next() % 10;
            let text: String = (0..length).map(|_| alphabet[next() % alphabet.len()]).collect();
            let ours = capitalize(&drop_space_after(&drop_space_before(&text)));
            assert_eq!(ours, by_regex(&text), "{text:?}");
        }
    }

    /// Lượt gộp `tidy` cho kết quả y như ba bước tách rời (cũng là ba regex).
    #[test]
    fn one_pass_matches_the_three_steps() {
        let alphabet: Vec<char> = " \u{3000}\u{a0}\ta1Zăđß.!?…,;:”’)]】“‘([【-_x\u{301}\u{2028}"
            .chars()
            .collect();
        let mut seed: u64 = 13;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) as usize
        };
        for _ in 0..100_000 {
            let length = next() % 14;
            let text: String = (0..length).map(|_| alphabet[next() % alphabet.len()]).collect();
            let steps = capitalize(&drop_space_after(&drop_space_before(&text)));
            assert_eq!(tidy(&text), steps, "{text:?}");
        }
        for c in (0..0x80u8).map(char::from) {
            let rest = c.to_string();
            assert_eq!(word_start(&rest), WORD_START.is_match(&rest), "{c:?}");
        }
    }
}
