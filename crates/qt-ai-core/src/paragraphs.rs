//! Port qt-web/src/lib/ai-paragraphs.ts + formatAiTranslation: đoạn đánh nhãn [[n]].

use regex::Regex;
use std::sync::LazyLock;

// JS `\d` chỉ là [0-9]; Rust `\d` là Unicode nên phải viết tường minh.
static MARKER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\[([0-9]{1,4})\]\]").unwrap());
static INNER_WHITESPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s*\n+\s*").unwrap());
static STRIP: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\[\[[0-9]{1,4}\]\] ?").unwrap());

/// Tương đương `text.split(/\r?\n/)`.
pub fn split_lines(text: &str) -> impl Iterator<Item = &str> {
    text.split('\n').map(|line| line.strip_suffix('\r').unwrap_or(line))
}

/// Mỗi dòng không rỗng là một đoạn.
pub fn paragraphs_of(text: &str) -> Vec<String> {
    split_lines(text).map(str::trim).filter(|line| !line.is_empty()).map(String::from).collect()
}

fn aligned_format_header(count: usize) -> String {
    [
        format!("Nguyên văn gồm {count} đoạn, mỗi đoạn mở đầu bằng nhãn dạng [[n]]."),
        "Bản dịch BẮT BUỘC giữ đúng định dạng đó: mỗi đoạn dịch mở đầu bằng nhãn [[n]] của đoạn nguyên văn tương ứng,".to_string(),
        format!("đủ và đúng thứ tự từ [[1]] đến [[{count}]]; mỗi nhãn xuất hiện đúng một lần; không gộp, không tách, không thêm hay bỏ đoạn."),
        "Ngoài nhãn, không thêm chú thích hay lời dẫn nào khác.".to_string(),
    ]
    .join("\n")
}

/// User message cho lượt dịch chính.
pub fn labeled_source_payload(paragraphs: &[String]) -> String {
    let body = paragraphs
        .iter()
        .enumerate()
        .map(|(index, paragraph)| format!("[[{}]] {}", index + 1, paragraph))
        .collect::<Vec<_>>()
        .join("\n\n");
    format!("{}\n\n{}", aligned_format_header(paragraphs.len()), body)
}

/// User message cho lượt dịch bổ sung đoạn thiếu (`missing` là chỉ số 0-based).
pub fn labeled_repair_payload(paragraphs: &[String], missing: &[usize]) -> String {
    let body = missing
        .iter()
        .map(|index| {
            format!("[[{}]] {}", index + 1, paragraphs.get(*index).map(String::as_str).unwrap_or("undefined"))
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    [
        "Bản dịch trước bị thiếu các đoạn dưới đây (trích từ cùng chương, giữ nguyên nhãn [[n]] gốc).",
        "Dịch bổ sung đúng các đoạn này theo toàn bộ quy tắc đã cho: mỗi đoạn dịch mở đầu bằng nhãn [[n]] tương ứng,",
        "không gộp, không bỏ, không thêm chú thích.",
        "",
        &body,
    ]
    .join("\n")
}

/// Tách bản dịch có nhãn thành mảng đoạn; None khi không còn nhãn nào; đoạn thiếu = None.
pub fn parse_labeled_translation(output: &str, count: usize) -> Option<Vec<Option<String>>> {
    let matches: Vec<(usize, usize, usize)> = MARKER
        .captures_iter(output)
        .map(|caps| {
            let whole = caps.get(0).unwrap();
            (whole.start(), whole.end(), caps[1].parse::<usize>().unwrap())
        })
        .collect();
    if matches.is_empty() {
        return None;
    }
    let mut paragraphs: Vec<Option<String>> = vec![None; count];
    for (index, (_, end, label)) in matches.iter().enumerate() {
        if *label == 0 || *label > count {
            continue;
        }
        let text_end = matches.get(index + 1).map(|next| next.0).unwrap_or(output.len());
        let text = INNER_WHITESPACE.replace_all(&output[*end..text_end], " ").trim().to_string();
        if !text.is_empty() && paragraphs[label - 1].is_none() {
            paragraphs[label - 1] = Some(text);
        }
    }
    Some(paragraphs)
}

pub fn strip_markers(text: &str) -> String {
    STRIP.replace_all(text, "").into_owned()
}

// Port normalizeAiPunctuation: mỗi phiên model trình bày dấu câu một kiểu, nên chuẩn hoá bằng code.
const QUOTE_SPACE: &str = " \t\u{a0}\u{3000}";
const OPEN_BEFORE_QUOTE: &str = "([{“‘『「【:";
const CLOSE_AFTER_QUOTE: &str = "!?,.;:…”’』)]】}";
static DASH_BEFORE_CLOSER: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"— +([!?,.;:…”’』)\]】])").unwrap());
static OPENER_BEFORE_DASH: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"([“‘『(]) +—").unwrap());

/// Ngoặc kép thẳng `"` → `“`/`”` theo chữ đứng cạnh (không ghép cặp, nên thoại kéo nhiều đoạn vẫn đúng); gạch dài
/// sát dấu đóng và ngoặc mở (`“Ngươi — !”` → `“Ngươi —!”`, `“ — Giết!”` → `“— Giết!”`). Nháy đơn và `『』` giữ nguyên.
pub fn normalize_punctuation(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let quoted: String = chars
        .iter()
        .enumerate()
        .map(|(index, &ch)| {
            if ch != '"' {
                return ch;
            }
            let Some(&next) = chars.get(index + 1).filter(|c| !QUOTE_SPACE.contains(**c)) else {
                return '”';
            };
            let prev = index.checked_sub(1).map(|i| chars[i]);
            // Xét chữ đứng trước trước dấu câu đứng sau: thoại mở bằng `"...` / `"?` vẫn là ngoặc mở.
            if prev.is_none_or(|c| QUOTE_SPACE.contains(c) || OPEN_BEFORE_QUOTE.contains(c)) {
                return '“';
            }
            if CLOSE_AFTER_QUOTE.contains(next) {
                return '”';
            }
            if prev == Some('—') { '“' } else { '”' }
        })
        .collect();
    let quoted = DASH_BEFORE_CLOSER.replace_all(&quoted, "—$1");
    OPENER_BEFORE_DASH.replace_all(&quoted, "$1—").into_owned()
}

/// Chuẩn hoá tất định theo raw cùng nhãn (thẩm định 11/2026, D-V3), trả số đoạn đã sửa:
/// - raw có n cụm `……` và không có `…` lẻ hay `...` ASCII, bản dịch có đúng n cụm `...` và không có `......` → 6 chấm
///   (21% đoạn có dấu lửng bị rút còn 3 chấm dù prompt dặn giữ số chấm);
/// - đoạn trước kết bằng `.` / `!` / `?` (không phải dấu lửng) mà đoạn này mở bằng chữ thường → viết hoa chữ đầu.
pub fn normalize_against_raw(raw: &[String], parsed: &mut [Option<String>]) -> usize {
    static THREE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\.{3,}").unwrap());
    let mut fixed = 0;
    let mut previous_end: Option<char> = None;
    let mut previous_ellipsis = false;
    for (index, slot) in parsed.iter_mut().enumerate() {
        let Some(text) = slot.as_mut() else {
            previous_end = None;
            continue;
        };
        let mut changed = false;
        if let Some(source) = raw.get(index) {
            let pairs = source.matches("……").count();
            let singles = source.matches('…').count() - pairs * 2;
            let runs: Vec<&str> = THREE.find_iter(text).map(|m| m.as_str()).collect();
            if pairs > 0 && singles == 0 && !source.contains("...") && runs.len() == pairs && runs.iter().all(|run| run.len() == 3) {
                *text = THREE.replace_all(text, "......").into_owned();
                changed = true;
            }
        }
        let sentence_closed = matches!(previous_end, Some('.' | '!' | '?')) && !previous_ellipsis;
        if sentence_closed {
            let skip = text.chars().take_while(|c| "“‘(\"'".contains(*c)).count();
            // Tên kiểu `iPhone`, `eSports` (có chữ hoa giữa từ) giữ nguyên.
            let first_word: String = text.chars().skip(skip).take_while(|c| c.is_alphanumeric()).collect();
            let camel = first_word.chars().skip(1).any(char::is_uppercase);
            if let Some(first) = text.chars().nth(skip).filter(|c| c.is_lowercase() && !camel) {
                let at: usize = text.chars().take(skip).map(char::len_utf8).sum();
                let upper: String = first.to_uppercase().collect();
                text.replace_range(at..at + first.len_utf8(), &upper);
                changed = true;
            }
        }
        let trimmed = text.trim_end().trim_end_matches(['”', '’', ')', '\"', '\'']);
        previous_end = trimmed.chars().last();
        previous_ellipsis = trimmed.ends_with("..");
        fixed += usize::from(changed);
    }
    fixed
}

/// Port `formatAiTranslation`: gom dòng thành khối, khối cách nhau đúng 1 dòng trống, kết bằng `\n`.
pub fn format_translation(text: &str) -> String {
    let mut blocks: Vec<Vec<String>> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    for line in split_lines(text) {
        let trimmed = normalize_punctuation(line.trim_end());
        if trimmed.is_empty() {
            if !current.is_empty() {
                blocks.push(std::mem::take(&mut current));
            }
        } else {
            current.push(trimmed);
        }
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    let output = blocks.iter().map(|block| block.join("\n")).collect::<Vec<_>>().join("\n\n");
    if output.is_empty() {
        String::new()
    } else {
        format!("{output}\n")
    }
}

#[cfg(test)]
mod normalize_tests {
    use super::normalize_against_raw;

    fn run(raw: &[&str], out: &[&str]) -> Vec<String> {
        let raw: Vec<String> = raw.iter().map(|s| s.to_string()).collect();
        let mut parsed: Vec<Option<String>> = out.iter().map(|s| Some(s.to_string())).collect();
        normalize_against_raw(&raw, &mut parsed);
        parsed.into_iter().map(Option::unwrap).collect()
    }

    #[test]
    fn dau_lung_sau_cham_theo_raw() {
        assert_eq!(run(&["但是……也能接受了"], &["nhưng mà... cũng chấp nhận được"]), vec!["nhưng mà...... cũng chấp nhận được"]);
        // Raw có dấu lửng lẻ hoặc số cụm lệch thì để nguyên.
        assert_eq!(run(&["但是…也能"], &["nhưng mà... cũng"]), vec!["nhưng mà... cũng"]);
        assert_eq!(run(&["但是……也能……"], &["nhưng mà... cũng"]), vec!["nhưng mà... cũng"]);
    }

    #[test]
    fn viet_hoa_dau_doan_sau_cau_da_ket() {
        let got = run(
            &["她察觉到了变化。", "连忙起身。", "他说，", "然后走了。", "iPhone响了。"],
            &["Nàng nhận ra biến hóa.", "vội vàng đứng dậy.", "hắn nói,", "rồi bỏ đi.", "iPhone reo lên."],
        );
        assert_eq!(got, vec!["Nàng nhận ra biến hóa.", "Vội vàng đứng dậy.", "Hắn nói,", "rồi bỏ đi.", "iPhone reo lên."]);
        // Đoạn trước kết bằng dấu lửng: câu chưa hết, giữ chữ thường.
        assert_eq!(run(&["你也……", "从来没有忘记我。"], &["Chàng cũng......", "chưa từng quên muội."])[1], "chưa từng quên muội.");
    }
}
