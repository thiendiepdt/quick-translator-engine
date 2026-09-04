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

/// Port `formatAiTranslation`: gom dòng thành khối, khối cách nhau đúng 1 dòng trống, kết bằng `\n`.
pub fn format_translation(text: &str) -> String {
    let mut blocks: Vec<Vec<&str>> = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in split_lines(text) {
        let trimmed = line.trim_end();
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
