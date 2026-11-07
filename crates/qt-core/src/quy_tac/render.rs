//! Ráp kết quả Quy Tắc vào output của engine (spec mục 5).
//!
//! Khác `render` của bản thử, cố ý: bản thử nối mọi token của đoạn bằng dấu
//! cách rồi sửa khoảng cách theo ngoặc cong `“ ” ‘ ’`. Bước chuẩn hoá của QT
//! đổi chúng thành ngoặc thẳng (`“` → `"`, `”` → `" `) trước khi dịch, nên
//! ráp kiểu đó làm mất cách đặt dấu cách quanh `"` của engine trên cả đoạn
//! (`" Ngươi … ngơ ngác. "`), kể cả khúc không quy tắc nào đụng tới. Ở đây:
//! - chỉ ráp lại các *cửa sổ*: khúc token mà quy tắc đã đổi, nới tới ranh giới
//!   chung của token engine và token Quy Tắc; ngoài cửa sổ output giữ nguyên
//!   từng byte;
//! - trong cửa sổ, nối như engine (`text::append_translated_word`): viết hoa
//!   khi output trước đó kết thúc câu theo engine (`text::ends_sentence`: `. `,
//!   `: `, `"`…), không cách trước dấu đóng và sau ngoặc mở; ngoặc thẳng là
//!   ngoặc đóng khi chữ nguồn sau nó là khoảng trắng, dấu đóng hay hết đoạn, hoặc
//!   khi chữ đã ráp kết thúc câu mà ngoặc không mở câu thoại kế (`closes`);
//! - chữ đầu của chữ ngay sau cửa sổ đổi hoa thường khi cửa sổ đổi việc nó có
//!   đứng đầu câu hay không (`Chương 251:` của nhóm `[số]` rồi `tương lai…`).

use std::ops::Range;

use super::prepare::Token;
use crate::text::{ends_sentence, opens_after_sentence_end, SENTENCE_ENDERS};

/// Dấu đóng: không có dấu cách trước nó. Hai tập dấu này phải theo lối nối của
/// engine (`text.rs`); chỗ lệch đã biết: `【 x 】`, ở đây ráp sát thành `【x】`.
fn is_closer(c: char) -> bool {
    matches!(
        c,
        ',' | '.' | '!' | '?' | ':' | ';' | '…' | '”' | '’' | ')' | ']' | '】' | '》' | '」' | '』'
    )
}

/// Ngoặc mở: không có dấu cách sau nó.
fn is_opener(c: char) -> bool {
    matches!(c, '“' | '‘' | '(' | '[' | '【' | '《' | '「' | '『')
}

/// Ngoặc thẳng `"` hoặc `'` đứng một mình (bước chuẩn hoá đổi ngoặc cong ra nó).
fn is_straight_quote(piece: &str) -> bool {
    piece == "\"" || piece == "'"
}

/// Ngoặc thẳng `quote` ở byte `src` của chữ Hán cả đoạn là ngoặc đóng: sau nó
/// là khoảng trắng (`”` → `" `), dấu đóng, hoặc hết đoạn; hoặc chữ đã ráp tới đây
/// kết thúc bằng `.` `?` `!` mà ngoặc không mở câu thoại kế (như nhánh ngoặc lẻ
/// của `translate.rs`, cùng `text::opens_after_sentence_end`).
fn closes(text: &str, src: &Range<usize>, quote: char, so_far: &str) -> bool {
    let next = text.get(src.end..).and_then(|rest| rest.chars().next());
    next.is_none_or(|c| c.is_whitespace() || is_closer(c))
        || (so_far.trim_end_matches(' ').ends_with(['.', '?', '!'])
            && !opens_after_sentence_end(
                text.get(..src.start).unwrap_or("").chars().rev(),
                quote,
                next,
            ))
}

/// Như `text::to_upper_case`: viết hoa chữ đầu, hoặc chữ sau `[`.
fn push_capitalized(out: &mut String, piece: &str) {
    let mut chars = piece.chars();
    match chars.next() {
        Some('[') if piece.len() > 1 => {
            out.push('[');
            push_capitalized(out, chars.as_str());
        }
        Some(first) => {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
        None => {}
    }
}

/// `before + gap` kết thúc câu theo engine, không cần nối hai chuỗi.
fn ends_sentence_after(before: &str, gap: &str) -> bool {
    SENTENCE_ENDERS.iter().any(|e| match e.len().checked_sub(gap.len()) {
        None | Some(0) => gap.ends_with(e),
        Some(head) => e.ends_with(gap) && before.ends_with(&e[..head]),
    })
}

/// Trả chữ đầu engine đã viết hoa về chữ gốc `c`; không khớp thì để nguyên.
pub(crate) fn push_uncapitalized(out: &mut String, word: &str, c: char) {
    let upper: String = c.to_uppercase().collect();
    match word.strip_prefix(upper.as_str()) {
        Some(rest) => {
            out.push(c);
            out.push_str(rest);
        }
        None => out.push_str(word),
    }
}

/// Ráp các token của một cửa sổ nối vào `out` (output đã ráp tới đầu cửa sổ).
/// `text` là chữ Hán cả đoạn (biết ngoặc thẳng mở hay đóng). `keep_gap`: dấu
/// cách trước cửa sổ là của output cũ, token đầu nối thẳng (cửa sổ thay một
/// khúc chữ); `false` khi cửa sổ chèn vào chỗ output cũ không có chữ.
pub(crate) fn render_window(out: &mut String, toks: &[Token], text: &str, keep_gap: bool) {
    let mut first = true;
    let mut after_opener = false;
    for t in toks {
        let piece = t.vi.trim();
        let Some(head) = piece.chars().next() else { continue };
        let quote = is_straight_quote(piece);
        let closing = quote && closes(text, &t.src, head, out);
        let attach = closing || (!quote && is_closer(head));
        if first && keep_gap {
            if attach {
                out.truncate(out.trim_end_matches(' ').len());
            }
        } else if !(attach || after_opener || out.is_empty() || out.ends_with(char::is_whitespace))
        {
            out.push(' ');
        }
        first = false;
        if ends_sentence(out) {
            push_capitalized(out, piece);
        } else {
            out.push_str(piece);
        }
        after_opener = (quote && !closing) || piece.ends_with(is_opener);
    }
}

/// Một token engine của đoạn, như bước ráp cần: `src` là byte trong chữ Hán
/// cả đoạn, `out` là chữ của nó trong output (đã cắt khoảng trắng; rỗng khi
/// không có chữ, khi đó bắt đầu ở chỗ token nằm), `capital` như `SourceToken`.
#[derive(Debug, Clone)]
pub(crate) struct Piece {
    pub(crate) src: Range<usize>,
    pub(crate) out: Range<usize>,
    pub(crate) capital: Option<char>,
}

/// Một cửa sổ: token engine `pieces[p]`, token Quy Tắc `toks[t]`, cùng phủ một
/// khúc chữ Hán.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Window {
    pub(crate) pieces: Range<usize>,
    pub(crate) toks: Range<usize>,
}

/// Các cửa sổ của đoạn, theo thứ tự. Mỗi token do quy tắc tạo (`fired`) nới ra
/// tới ranh giới chung gần nhất của hai cách cắt (token engine và token Quy
/// Tắc đều nối liền nhau phủ trọn đoạn, nên luôn có ranh giới chung ở đầu và
/// cuối đoạn). Cửa sổ chồng, chạm nhau, hay chỉ cách nhau bởi token engine
/// không có chữ thì gộp.
pub(crate) fn windows(pieces: &[Piece], toks: &[Token]) -> Vec<Window> {
    // Ranh giới chung: (byte, chỉ số token engine bắt đầu ở đó, chỉ số token Quy Tắc).
    let mut common = Vec::new();
    let (mut p, mut t) = (0, 0);
    loop {
        let pb = pieces.get(p).map_or(usize::MAX, |piece| piece.src.start);
        let tb = toks.get(t).map_or(usize::MAX, |tok| tok.src.start);
        if pb == usize::MAX && tb == usize::MAX {
            break;
        }
        if pb == tb {
            common.push((pb, p, t));
            p += 1;
            t += 1;
        } else if pb < tb {
            p += 1;
        } else {
            t += 1;
        }
    }
    common.push((usize::MAX, pieces.len(), toks.len()));
    let mut out: Vec<Window> = Vec::new();
    for (index, tok) in toks.iter().enumerate() {
        if !tok.fired {
            continue;
        }
        let end = toks.get(index + 1).map_or(usize::MAX, |next| next.src.start);
        let a = common.partition_point(|&(byte, _, _)| byte <= tok.src.start) - 1;
        let b = common.partition_point(|&(byte, _, _)| byte < end);
        let window = Window {
            pieces: common[a].1..common[b].1,
            toks: common[a].2..common[b].2,
        };
        if let Some(last) = out.last_mut() {
            let between = &pieces[last.pieces.end.min(window.pieces.start)..window.pieces.start];
            if between.iter().all(|piece| piece.out.is_empty()) {
                last.pieces.end = last.pieces.end.max(window.pieces.end);
                last.toks.end = last.toks.end.max(window.toks.end);
                continue;
            }
        }
        out.push(window);
    }
    out
}

/// Ráp đoạn: chép output từ `*cursor`, thay từng cửa sổ bằng bản ráp lại, dời
/// `*cursor` tới sau khúc đã thay. `text` là chữ Hán cả đoạn.
pub(crate) fn splice(
    output: &str,
    text: &str,
    pieces: &[Piece],
    toks: &[Token],
    result: &mut String,
    cursor: &mut usize,
) {
    for window in windows(pieces, toks) {
        let span = &pieces[window.pieces.clone()];
        if span.is_empty() {
            debug_assert!(false, "cửa sổ không có token engine");
            continue;
        }
        let first = span.iter().find(|piece| !piece.out.is_empty());
        let last = span.iter().rev().find(|piece| !piece.out.is_empty());
        let (from, to) = match (first, last) {
            (Some(first), Some(last)) => (first.out.start, last.out.end),
            _ => (span[0].out.start, span[0].out.start),
        };
        if from < *cursor || to < from {
            debug_assert!(false, "khoảng output của cửa sổ lùi hoặc đảo");
            continue;
        }
        let replaced = from < to;
        result.push_str(&output[*cursor..from]);
        let before = result.len();
        render_window(result, &toks[window.toks.clone()], text, replaced);
        let rendered = result.len() > before;
        let rest = &output[to..];
        let next = rest.chars().next();
        if rendered && !replaced {
            // Chèn vào chỗ không có chữ: cách chữ kế như engine.
            let open = result.ends_with(|c: char| c.is_whitespace() || is_opener(c));
            if next.is_some_and(|c| !c.is_whitespace() && !is_closer(c)) && !open {
                result.push(' ');
            }
        } else if !rendered && replaced && next.is_none_or(|c| c == ' ' || is_closer(c)) {
            // Cửa sổ ra rỗng: khỏi để lại hai dấu cách hay cách trước dấu đóng.
            result.truncate(result.trim_end_matches(' ').len());
        }
        *cursor = to;
        // Chữ ngay sau cửa sổ: cửa sổ đổi việc nó đứng đầu câu thì đổi chữ đầu.
        let Some(after) = pieces[window.pieces.end..].iter().find(|p| !p.out.is_empty()) else {
            continue;
        };
        let gap = &output[to..after.out.start];
        let was = ends_sentence(&output[..after.out.start]);
        let now = ends_sentence_after(result, gap);
        if was == now {
            continue;
        }
        let word = &output[after.out.clone()];
        result.push_str(gap);
        match after.capital {
            _ if now => push_capitalized(result, word),
            Some(c) => push_uncapitalized(result, word, c),
            None => result.push_str(word),
        }
        *cursor = after.out.end;
    }
}

/// Ráp mọi token như một cửa sổ đứng đầu câu (cho test của các bước khác).
#[cfg(test)]
pub(crate) fn render_sentence(toks: &[Token]) -> String {
    let mut out = String::from("\n");
    render_window(&mut out, toks, "", false);
    out.split_off(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quy_tac::prepare::Token;

    fn tok(vi: &str, src: Range<usize>, fired: bool) -> Token<'static> {
        let mut t = Token::new("x", vi.to_string(), "n").at(src);
        t.fired = fired;
        t
    }

    #[test]
    fn joins_fixes_spacing_and_capitalizes() {
        let toks: Vec<Token> = ["hắn", "đi", ",", "“", "ta", "nói", "”", "."]
            .iter()
            .map(|vi| Token::new("x", *vi, "n"))
            .collect();
        assert_eq!(render_sentence(&toks), "Hắn đi, “ta nói”.");
    }

    /// Ngoặc thẳng mở hay đóng theo chữ nguồn sau nó; sau ngoặc mở và sau `: `
    /// viết hoa như engine.
    #[test]
    fn straight_quotes_open_or_close_by_the_source() {
        // Chữ Hán đoạn: 他说: "你好" 。  (byte của từng token ghi trong `src`)
        let text = "他说: \"你好\" 。";
        let at = |piece: &str| {
            let start = text.find(piece).unwrap();
            start..start + piece.len()
        };
        let quote_open = text.find('"').unwrap();
        let quote_close = text.rfind('"').unwrap();
        let toks = vec![
            tok("hắn nói", at("他说"), true),
            tok(":", at(":"), false),
            tok("\"", quote_open..quote_open + 1, false),
            tok("ngươi tốt", at("你好"), true),
            tok("\"", quote_close..quote_close + 1, false),
            tok(".", at("。"), false),
        ];
        let mut out = String::new();
        render_window(&mut out, &toks, text, true);
        assert_eq!(out, "hắn nói: \"Ngươi tốt\".");
        // Đầu đoạn không viết hoa, như engine (chữ đầu của request).
        assert!(!ends_sentence(""));
    }

    #[test]
    fn sentence_end_check_spans_the_join() {
        for (before, gap, want) in [
            ("vậy?", "\" ", true),
            ("vậy", "\" ", false),
            ("Chương 251:", " ", true),
            ("x", "", false),
            ("x.", " ", true),
            ("", "\n", true),
            ("\"", "", true),
        ] {
            let joined = format!("{before}{gap}");
            assert_eq!(ends_sentence_after(before, gap), ends_sentence(&joined), "{joined:?}");
            assert_eq!(ends_sentence(&joined), want, "{joined:?}");
        }
    }

    /// Dựng đoạn từ các cụm `(chữ Hán, chữ engine ra kể cả dấu cách trước nó)`:
    /// trả chữ Hán cả đoạn, output và token engine.
    fn layout(parts: &[(&str, &str)]) -> (String, String, Vec<Piece>) {
        let (mut text, mut output, mut pieces) = (String::new(), String::new(), Vec::new());
        for (zh, chunk) in parts {
            let src = text.len()..text.len() + zh.len();
            text.push_str(zh);
            let lead = chunk.len() - chunk.trim_start().len();
            let start = output.len() + lead.min(chunk.len());
            output.push_str(chunk);
            pieces.push(Piece { src, out: start..start + chunk.trim().len(), capital: None });
        }
        (text, output, pieces)
    }

    /// Token Quy Tắc y như token engine (chưa quy tắc nào chạy).
    fn same(output: &str, pieces: &[Piece]) -> Vec<Token<'static>> {
        pieces.iter().map(|p| tok(&output[p.out.clone()], p.src.clone(), false)).collect()
    }

    fn spliced(output: &str, text: &str, pieces: &[Piece], toks: &[Token]) -> String {
        let (mut result, mut cursor) = (String::new(), 0);
        splice(output, text, pieces, toks, &mut result, &mut cursor);
        result.push_str(&output[cursor..]);
        result
    }

    /// Ngoài cửa sổ output giữ nguyên từng byte; cửa sổ chỉ phủ khúc quy tắc đổi.
    #[test]
    fn text_outside_the_windows_is_byte_identical() {
        // Engine: hai dấu cách lạ, ngoặc thẳng sát chữ, `" ` đóng.
        let (text, output, pieces) = layout(&[
            ("\"", " \""),
            ("我", "Ta"),
            ("走", "  đi"),
            ("啊", " a"),
            (",", ","),
            ("\"", "\""),
            (" ", " "),
            ("他", "hắn"),
            ("说", " nói"),
            (".", "."),
        ]);
        assert_eq!(output, " \"Ta  đi a,\" hắn nói.");
        // Quy tắc gộp 他说 thành "nói hắn"; mọi token khác y nguyên.
        let mut toks = same(&output, &pieces);
        let src = pieces[7].src.start..pieces[8].src.end;
        toks.splice(7..9, [tok("nói hắn", src, true)]);
        assert_eq!(windows(&pieces, &toks), vec![Window { pieces: 7..9, toks: 7..8 }]);
        assert_eq!(spliced(&output, &text, &pieces, &toks), " \"Ta  đi a,\" nói hắn.");
        // Không quy tắc nào chạy thì không có cửa sổ.
        assert!(windows(&pieces, &same(&output, &pieces)).is_empty());
    }

    /// Cửa sổ nới tới ranh giới chung: token Quy Tắc tách một token engine
    /// (đuôi dấu câu, `的`) thì cả token engine đó vào cửa sổ.
    #[test]
    fn windows_grow_to_common_boundaries_and_merge() {
        let piece = |src: Range<usize>, out: Range<usize>| Piece { src, out, capital: None };
        // Token engine: [0,6) [6,9) [9,12) [12,15) [15,18); token thứ ba không có chữ.
        let pieces =
            [piece(0..6, 0..4), piece(6..9, 5..7), piece(9..12, 7..7), piece(12..15, 8..9)];
        // Quy Tắc tách [0,6) thành [0,3) [3,6); quy tắc gộp [3,6)+[6,9) và đổi [12,15).
        let toks = vec![
            tok("a", 0..3, false),
            tok("b", 3..9, true),
            tok("c", 9..12, false),
            tok("d", 12..15, true),
        ];
        assert_eq!(windows(&pieces, &toks), vec![Window { pieces: 0..4, toks: 0..4 }]);
        let toks = vec![tok("a", 0..3, false), tok("b", 3..6, true), tok("c", 6..15, false)];
        let pieces = [piece(0..6, 0..4), piece(6..15, 5..7)];
        assert_eq!(windows(&pieces, &toks), vec![Window { pieces: 0..1, toks: 0..2 }]);
    }

    /// `Chương 251:` của nhóm `[số]` đứng trước chữ không đổi: chữ đó viết hoa
    /// như sau `: ` của engine; cửa sổ bỏ dấu kết câu thì chữ hoa do engine
    /// viết được trả về chữ gốc.
    #[test]
    fn the_word_after_a_window_follows_the_new_sentence_start() {
        let (text, output, pieces) =
            layout(&[("第251章", " thứ 251 chương"), (" ", " "), ("未来", "tương lai")]);
        let mut toks = same(&output, &pieces);
        toks[0] = tok("Chương 251:", pieces[0].src.clone(), true);
        assert_eq!(spliced(&output, &text, &pieces, &toks), " Chương 251: Tương lai");
        let (text, output, mut pieces) =
            layout(&[("完了", " hết."), (" ", " "), ("未来", "Tương lai")]);
        pieces[2].capital = Some('t');
        let mut toks = same(&output, &pieces);
        toks[0] = tok("xong", pieces[0].src.clone(), true);
        assert_eq!(spliced(&output, &text, &pieces, &toks), " xong tương lai");
        // Chữ hoa của chính nghĩa (không do engine viết) thì giữ.
        pieces[2].capital = None;
        assert_eq!(spliced(&output, &text, &pieces, &toks), " xong Tương lai");
    }

    /// Ngoặc thẳng sau chữ đã ráp kết thúc bằng `.` là ngoặc đóng dù chữ nguồn
    /// sau nó là chữ (`"你好。"他说`), như nhánh ngoặc lẻ của engine.
    #[test]
    fn straight_quote_after_sentence_end_closes() {
        let text = "\"你好。\"他说";
        let toks = vec![
            tok("\"", 0..1, false),
            tok("ngươi tốt", 1..7, true),
            tok(".", 7..10, false),
            tok("\"", 10..11, false),
            tok("hắn nói", 11..17, false),
        ];
        let mut out = String::new();
        render_window(&mut out, &toks, text, false);
        assert_eq!(out, "\"Ngươi tốt.\" Hắn nói");
    }

    /// Ngoặc thẳng sau `.` mà dính chữ và đứng sau một số chẵn ngoặc trong dòng
    /// (`“` sau bước chuẩn hoá) là ngoặc mở câu thoại kế: dấu cách trước nó.
    #[test]
    fn straight_quote_after_sentence_end_opens_the_next_line() {
        let text = "说完。\"你好。\" ";
        let toks = vec![
            tok("nói xong", 0..6, true),
            tok(".", 6..9, false),
            tok("\"", 9..10, false),
            tok("ngươi tốt", 10..16, false),
            tok(".", 16..19, false),
            tok("\"", 19..20, false),
        ];
        let mut out = String::new();
        render_window(&mut out, &toks, text, false);
        assert_eq!(out, "nói xong. \"Ngươi tốt.\"");
    }

    /// Hai cửa sổ trong một đoạn: cửa sổ đầu đổi chữ đầu của chữ ngay sau và dời
    /// con trỏ qua nó; cửa sổ sau vẫn ráp đúng, không chép lại chữ đã đổi.
    #[test]
    fn two_windows_after_a_handover() {
        let (text, output, mut pieces) = layout(&[
            ("完了", " hết."),
            (" ", " "),
            ("未来", "Tương lai"),
            (" ", " "),
            ("他", "hắn"),
        ]);
        pieces[2].capital = Some('t');
        let mut toks = same(&output, &pieces);
        toks[0] = tok("xong", pieces[0].src.clone(), true);
        toks[4] = tok("nó", pieces[4].src.clone(), true);
        assert_eq!(windows(&pieces, &toks).len(), 2);
        assert_eq!(spliced(&output, &text, &pieces, &toks), " xong tương lai nó");
    }

    /// Cửa sổ chèn vào chỗ output cũ không có chữ (token engine rỗng): thêm dấu
    /// cách trước chữ kế.
    #[test]
    fn window_inserting_where_the_output_had_no_text() {
        let (text, output, pieces) = layout(&[("你", "ngươi"), ("的", ""), ("好", " tốt")]);
        let mut toks = same(&output, &pieces);
        toks[1] = tok("của", pieces[1].src.clone(), true);
        assert_eq!(spliced(&output, &text, &pieces, &toks), "ngươi của tốt");
        // Chữ kế là dấu đóng thì không thêm dấu cách.
        let (text, output, pieces) = layout(&[("你", "ngươi"), ("的", ""), (",", ",")]);
        let mut toks = same(&output, &pieces);
        toks[1] = tok("của", pieces[1].src.clone(), true);
        assert_eq!(spliced(&output, &text, &pieces, &toks), "ngươi của,");
    }

    /// Cửa sổ thay chữ bằng không gì: không để hai dấu cách.
    #[test]
    fn window_rendering_nothing_leaves_one_space() {
        let (text, output, pieces) = layout(&[("你", "ngươi"), ("的", " của"), ("好", " tốt")]);
        let mut toks = same(&output, &pieces);
        toks[1] = tok("", pieces[1].src.clone(), true);
        assert_eq!(spliced(&output, &text, &pieces, &toks), "ngươi tốt");
    }
}
