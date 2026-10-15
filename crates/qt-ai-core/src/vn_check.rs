//! Quét nội dung nhạy cảm về Việt Nam trong một truyện — spec docs/superpowers/specs/2026-10-15-vn-sensitive-check-design.md.
//! Luật khớp ở `scan_chapter`; lưu + quét tăng dần ở `refresh` (Task 3).

use crate::paragraphs::paragraphs_of;
use crate::vn_lexicon::Lexicon;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Từ đi kèm cho từ yếu (ngoài một mục strong bất kỳ cùng đoạn).
const COOCCUR: &[&str] = &[
    "主权", "领土", "领海", "岛礁", "群岛", "越南", "安南", "交趾", "chủ quyền", "lãnh thổ", "lãnh hải", "quần đảo",
    "Việt Nam", "An Nam", "Giao Chỉ",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Raw,
    Out,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub paragraph: usize,
    pub entry: usize,
    pub side: Side,
    pub matched: String,
    pub count: usize,
}

/// Đoạn của bản dịch = các dòng không rỗng — cùng cách `replace.rs` đánh số đoạn.
pub fn out_paragraphs(out: &str) -> Vec<String> {
    out.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_string).collect()
}

/// `find_zh`/`find_vi` đã bỏ chỗ bị loại trừ nên không cần kiểm tra lại ở đây.
fn cooccur(lex: &Lexicon, texts: &[&str], entry: usize) -> bool {
    texts.iter().any(|text| COOCCUR.iter().any(|word| text.contains(word)))
        || texts.iter().any(|text| {
            lex.find_zh(text).into_iter().chain(lex.find_vi(text)).any(|hit| hit.entry != entry && !lex.entries[hit.entry].weak)
        })
}

/// Các chỗ khớp của một chương, gộp theo (đoạn, mục). `out` = None → chương chưa dịch, quét raw.
pub fn scan_chapter(lex: &Lexicon, raw: &str, out: Option<&str>) -> Vec<Finding> {
    let raw_paragraphs = paragraphs_of(raw);
    let mut found: BTreeMap<(usize, usize), Finding> = BTreeMap::new();
    let mut push = |paragraph: usize, entry: usize, side: Side, matched: &str| {
        found
            .entry((paragraph, entry))
            .and_modify(|finding| finding.count += 1)
            .or_insert(Finding { paragraph, entry, side, matched: matched.to_string(), count: 1 });
    };
    match out {
        Some(out) => {
            let out_paragraphs = out_paragraphs(out);
            let aligned = out_paragraphs.len() == raw_paragraphs.len();
            for (index, paragraph) in out_paragraphs.iter().enumerate() {
                let raw = if aligned { raw_paragraphs.get(index).map(String::as_str) } else { None };
                for hit in lex.find_vi(paragraph) {
                    if raw.is_some_and(|raw| !lex.mentions_zh(hit.entry, raw)) {
                        continue;
                    }
                    if lex.entries[hit.entry].weak && !cooccur(lex, &[paragraph, raw.unwrap_or("")], hit.entry) {
                        continue;
                    }
                    push(index, hit.entry, Side::Out, &paragraph[hit.start..hit.end]);
                }
            }
        }
        None => {
            for (index, paragraph) in raw_paragraphs.iter().enumerate() {
                for hit in lex.find_zh(paragraph) {
                    if lex.entries[hit.entry].weak && !cooccur(lex, &[paragraph], hit.entry) {
                        continue;
                    }
                    push(index, hit.entry, Side::Raw, &paragraph[hit.start..hit.end]);
                }
            }
        }
    }
    found.into_values().collect()
}
