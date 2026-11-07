//! Ráp token thành câu cho đoạn có quy tắc chạy: nối bằng dấu cách, bỏ cách
//! trước dấu câu / sau ngoặc mở, viết hoa đầu câu. Giống `render` của bản thử.

use std::sync::LazyLock;

use regex::{Captures, Regex};

use super::prepare::Token;

static SPACE_BEFORE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\s+([,.!?:;…”’)\]】])").unwrap());
static SPACE_AFTER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([“‘(\[【])\s+").unwrap());
static CAPITAL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(^|[.!?…]\s+|[“‘]\s*)(\w)").unwrap());

pub(crate) fn render(toks: &[Token]) -> String {
    let pieces: Vec<&str> = toks
        .iter()
        .map(|t| t.vi.trim())
        .filter(|piece| !piece.is_empty())
        .collect();
    let text = pieces.join(" ");
    let text = SPACE_BEFORE.replace_all(&text, "$1");
    let text = SPACE_AFTER.replace_all(&text, "$1");
    CAPITAL
        .replace_all(&text, |caps: &Captures| {
            format!("{}{}", &caps[1], caps[2].to_uppercase())
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quy_tac::prepare::Token;

    #[test]
    fn joins_fixes_spacing_and_capitalizes() {
        let toks: Vec<Token> = ["hắn", "đi", ",", "“", "ta", "nói", "”", "."]
            .iter()
            .map(|vi| Token::new("x", *vi, "n"))
            .collect();
        assert_eq!(render(&toks), "Hắn đi, “Ta nói”.");
    }
}
