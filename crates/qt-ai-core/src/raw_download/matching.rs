//! Khớp mục lục với raw đã có (spec 10.2). Chương tải qua app có trong sổ (`download.json` → `chapters`) nên khớp
//! theo mã. Truyện cũ chưa có sổ thì file thứ N (thứ tự tự nhiên) ứng với mục N; số trong tiêu đề không dùng được
//! (file 1283 của một truyện trong thư viện có tiêu đề 第1284章), nên kiểm lại bằng tiêu đề ở vài vị trí.

use crate::zh_convert::to_simplified;
use indexmap::IndexMap;
use serde::Serialize;

pub struct TocRef<'a> {
    pub id: &'a str,
    pub title: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Existing {
    /// Id file raw.
    pub id: String,
    /// Dòng đầu file.
    pub title: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MatchedBy {
    Ledger,
    Order,
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mismatch {
    /// Vị trí trong mục lục, từ 1.
    pub index: usize,
    pub toc_title: String,
    pub file_id: String,
    pub file_title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matching {
    /// Theo vị trí mục lục: id file raw đã có.
    pub files: Vec<Option<String>>,
    pub by: MatchedBy,
    /// Vị trí (từ 1) của chương cuối đã có.
    pub last_index: Option<usize>,
    pub mismatches: Vec<Mismatch>,
    /// File raw không ứng được với mục nào của mục lục.
    pub unmatched_files: usize,
}

const SAMPLE_POINTS: usize = 5;

/// Bỏ khoảng trắng, dấu câu, ký hiệu; phồn → giản; chữ thường.
fn key(title: &str) -> String {
    to_simplified(title).chars().filter(|c| c.is_alphanumeric()).flat_map(char::to_lowercase).collect()
}

/// Bằng nhau, hoặc bên này chứa bên kia (raw có khi thiếu hoặc thừa tiền tố "第N章").
pub fn title_matches(a: &str, b: &str) -> bool {
    let (a, b) = (key(a), key(b));
    if a.is_empty() || b.is_empty() {
        return a == b;
    }
    a == b || a.contains(&b) || b.contains(&a)
}

fn sample_positions(count: usize) -> Vec<usize> {
    if count == 0 {
        return vec![];
    }
    let mut points: Vec<usize> = (0..SAMPLE_POINTS).map(|k| k * (count - 1) / SAMPLE_POINTS.max(1)).collect();
    points.push(count - 1);
    points.sort_unstable();
    points.dedup();
    points
}

/// `anchor`: người dùng chỉ "chương cuối đã có là mục số K" (từ 1); `None` = số file raw đang có.
pub fn match_existing(toc: &[TocRef], ledger: &IndexMap<String, String>, existing: &[Existing], anchor: Option<usize>) -> Matching {
    let mut files: Vec<Option<String>> = vec![None; toc.len()];
    if existing.is_empty() {
        return Matching { files, by: MatchedBy::Empty, last_index: None, mismatches: vec![], unmatched_files: 0 };
    }
    let present: std::collections::HashSet<&str> = existing.iter().map(|e| e.id.as_str()).collect();

    let mut from_ledger = 0;
    for (i, item) in toc.iter().enumerate() {
        if let Some(file) = ledger.get(item.id).filter(|f| present.contains(f.as_str())) {
            files[i] = Some(file.clone());
            from_ledger += 1;
        }
    }
    if from_ledger > 0 && anchor.is_none() {
        let last_index = files.iter().rposition(Option::is_some).map(|i| i + 1);
        let unmatched_files = existing.len().saturating_sub(from_ledger);
        return Matching { files, by: MatchedBy::Ledger, last_index, mismatches: vec![], unmatched_files };
    }

    files = vec![None; toc.len()];
    let anchor = anchor.unwrap_or(existing.len());
    let offset = anchor as isize - existing.len() as isize;
    let mut unmatched_files = 0;
    let mut placed: Vec<(usize, usize)> = Vec::new();
    for (j, file) in existing.iter().enumerate() {
        let position = j as isize + offset;
        if position < 0 || position as usize >= toc.len() {
            unmatched_files += 1;
            continue;
        }
        files[position as usize] = Some(file.id.clone());
        placed.push((position as usize, j));
    }
    let mut mismatches = Vec::new();
    for k in sample_positions(placed.len()) {
        let (position, j) = placed[k];
        let file = &existing[j];
        if !title_matches(toc[position].title, &file.title) {
            mismatches.push(Mismatch { index: position + 1, toc_title: toc[position].title.to_string(), file_id: file.id.clone(), file_title: file.title.clone() });
        }
    }
    let last_index = placed.last().map(|(position, _)| position + 1);
    Matching { files, by: MatchedBy::Order, last_index, mismatches, unmatched_files }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toc(titles: &[&'static str]) -> Vec<(String, &'static str)> {
        titles.iter().enumerate().map(|(i, t)| (format!("c{}", i + 1), *t)).collect()
    }

    fn refs<'a>(list: &'a [(String, &'static str)]) -> Vec<TocRef<'a>> {
        list.iter().map(|(id, title)| TocRef { id, title }).collect()
    }

    fn existing(titles: &[&str]) -> Vec<Existing> {
        titles.iter().enumerate().map(|(i, t)| Existing { id: format!("chuong-{:04}", i + 1), title: t.to_string() }).collect()
    }

    #[test]
    fn tieu_de_khop_bo_dau_phon_gian_va_tien_to() {
        assert!(title_matches("第1章 蓝田公子", "第1章  蓝田公子"));
        assert!(title_matches("第2章 狼心如鐵", "第2章 狼心如铁"));
        assert!(title_matches("本源", "第1284章 本源"));
        assert!(title_matches("第3章：开始！", "第3章 开始"));
        assert!(!title_matches("第3章 开始", "第4章 结束"));
    }

    #[test]
    fn theo_thu_tu_mac_dinh_va_bao_lech() {
        let list = toc(&["第1章 起", "第2章 承", "第3章 转", "第4章 合"]);
        let m = match_existing(&refs(&list), &IndexMap::new(), &existing(&["第1章 起", "第2章 承"]), None);
        assert_eq!(m.by, MatchedBy::Order);
        assert_eq!(m.files, vec![Some("chuong-0001".into()), Some("chuong-0002".into()), None, None]);
        assert_eq!(m.last_index, Some(2));
        assert!(m.mismatches.is_empty());

        // Mục lục có thêm "lời mở đầu" ở đầu: theo thứ tự thì lệch, mốc đúng thì hết lệch.
        let shifted = toc(&["序章", "第1章 起", "第2章 承", "第3章 转"]);
        let off = match_existing(&refs(&shifted), &IndexMap::new(), &existing(&["第1章 起", "第2章 承"]), None);
        assert!(!off.mismatches.is_empty());
        assert_eq!(off.mismatches[0].index, 1);
        let fixed = match_existing(&refs(&shifted), &IndexMap::new(), &existing(&["第1章 起", "第2章 承"]), Some(3));
        assert!(fixed.mismatches.is_empty());
        assert_eq!(fixed.files[1].as_deref(), Some("chuong-0001"));
        assert_eq!(fixed.last_index, Some(3));
    }

    #[test]
    fn theo_so_ke_ca_khi_muc_luc_chen_chuong() {
        let list = toc(&["第1章", "番外", "第2章"]);
        let mut ledger = IndexMap::new();
        ledger.insert("c1".to_string(), "0001_a".to_string());
        ledger.insert("c3".to_string(), "0002_b".to_string());
        ledger.insert("cx".to_string(), "đã xoá".to_string());
        let files = vec![Existing { id: "0001_a".into(), title: "x".into() }, Existing { id: "0002_b".into(), title: "y".into() }];
        let m = match_existing(&refs(&list), &ledger, &files, None);
        assert_eq!(m.by, MatchedBy::Ledger);
        assert_eq!(m.files, vec![Some("0001_a".into()), None, Some("0002_b".into())]);
        assert_eq!(m.last_index, Some(3));
        assert!(m.mismatches.is_empty(), "theo sổ không kiểm tiêu đề");
    }

    #[test]
    fn truyen_moi_va_file_thua() {
        let list = toc(&["a", "b"]);
        let m = match_existing(&refs(&list), &IndexMap::new(), &[], None);
        assert_eq!((m.by, m.last_index), (MatchedBy::Empty, None));
        let many = match_existing(&refs(&list), &IndexMap::new(), &existing(&["a", "b", "c"]), None);
        assert_eq!(many.unmatched_files, 1);
    }
}
