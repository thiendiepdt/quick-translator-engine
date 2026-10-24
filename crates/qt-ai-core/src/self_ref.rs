//! Tự xưng 本 + danh xưng đọc một âm `bổn` (bổn cung, bổn tọa, bổn tiểu thư…). Model đọc lẫn bản/bổn ngay trong một
//! truyện (thư viện: 51/57 truyện lẫn, 5.396 chỗ `bổn` cạnh ~4.400 chỗ `bản`); accept đổi thẳng, không tốn lượt soát.
//!
//! Chỉ đổi khi `bản X` gần như chắc là tự xưng. Không đụng: `bản tôn` (本尊 hay là chân thân: "ma thân và bản tôn"),
//! `bản tướng` (本相 = hình dạng thật), `bản gia` (本家 = dòng chính), `bản phủ` ("thạch bản phủ kín"), `bản thiếu`
//! ("phiên bản thiếu nữ"); `bản` là vế sau từ ghép (văn bản, phiên bản, kịch bản, Nhật Bản…); `Bản` viết hoa giữa câu
//! (tên Nhật: Cung Bản 宫本, Sơn Bản 山本); cụm đi tiếp thành từ khác (bản quan trọng, bản miêu tả, bản tọa độ…).

use serde_json::Value;

/// Danh xưng sau 本 khi tự xưng, kèm chữ đi ngay sau làm nó thành từ khác (không phải tự xưng).
const TITLES: &[(&str, &[&str])] = &[
    ("tướng quân", &[]),
    ("tiểu thư", &[]),
    ("cô nương", &[]),
    ("công tử", &[]),
    ("công chúa", &[]),
    ("thiếu gia", &[]),
    ("đại gia", &[]),
    ("cung", &["cấp", "đình", "điện", "kính"]),
    ("vương", &["triều", "quốc"]),
    ("tọa", &["độ"]),
    ("quan", &["trọng", "điểm", "hệ", "sát", "niệm", "tâm", "ngại", "liêu"]),
    ("đế", &["chế", "quốc"]),
    ("hoàng", &["kim", "hôn"]),
    ("tiên", &["đoán", "tri", "quyết", "phong", "tiến"]),
    ("thần", &["thoại", "kinh", "bí", "kỳ", "chú", "tượng", "thông"]),
    ("thánh", &["kinh", "chỉ", "ca", "thư"]),
    ("soái", &[]),
    ("hầu", &[]),
    ("miêu", &["tả"]),
];

/// Chữ đứng trước làm `bản` thành vế sau của từ ghép.
const COMPOUND_HEADS: &[&str] = &[
    "văn", "phiên", "kịch", "nguyên", "cơ", "căn", "bút", "thiết", "thạch", "mộc", "ngọc", "kim", "đồng", "cổ", "tranh",
    "sách", "ấn", "xuất", "tái", "tư", "nhật", "hồ", "bảng", "tiêu", "kiếm", "lưỡi",
];

fn last_word(text: &str) -> Option<&str> {
    let text = text.trim_end_matches(' ');
    let start = text.char_indices().rev().take_while(|(_, c)| c.is_alphabetic()).last().map(|(i, _)| i)?;
    Some(&text[start..])
}

fn next_word(text: &str) -> Option<&str> {
    let text = text.strip_prefix(' ')?;
    let end = text.char_indices().find(|(_, c)| !c.is_alphabetic()).map_or(text.len(), |(i, _)| i);
    (end > 0).then(|| &text[..end])
}

/// Đầu đoạn hoặc sau dấu kết câu / mở ngoặc kép, xuống dòng — viết hoa ở đó là đầu câu, không phải tên riêng.
fn sentence_start(before: &str) -> bool {
    match before.trim_end().chars().last() {
        None => true,
        Some(c) => ".!?…:\"“‘'«([—–-\n【「『~".contains(c),
    }
}

/// Đổi `bản` → `bổn` trong tự xưng; `keep` là danh xưng giữ `bản` vì glossary của truyện cố ý ghi vậy (`bản miêu`).
/// Trả văn bản mới và số chỗ đã đổi.
pub fn bon_self_reference(text: &str, keep: &[&str]) -> (String, usize) {
    let mut out = String::with_capacity(text.len());
    let mut fixed = 0;
    let mut last = 0;
    let lower = text.to_lowercase();
    // `to_lowercase` giữ nguyên độ dài byte cho chữ Việt (bản/Bản/BẢN cùng 4 byte) — vẫn kiểm cho chắc.
    if lower.len() != text.len() {
        return (text.to_string(), 0);
    }
    let mut from = 0;
    while let Some(found) = lower[from..].find("bản ") {
        let at = from + found;
        from = at + "bản ".len();
        let before = &text[..at];
        if before.chars().last().is_some_and(char::is_alphabetic) {
            continue; // giữa một chữ khác
        }
        let rest = &lower[from..];
        let Some((title, blockers)) = TITLES.iter().find(|(title, _)| {
            rest.starts_with(title) && !rest[title.len()..].chars().next().is_some_and(char::is_alphabetic)
        }) else {
            continue;
        };
        if keep.contains(title) || next_word(&rest[title.len()..]).is_some_and(|word| blockers.contains(&word)) {
            continue;
        }
        if last_word(before).is_some_and(|word| COMPOUND_HEADS.contains(&word.to_lowercase().as_str())) {
            continue;
        }
        let original = &text[at..at + "bản".len()];
        let capital = original.starts_with('B');
        if capital && !sentence_start(before) {
            continue; // tên riêng: Cung Bản, Sơn Bản
        }
        out.push_str(&text[last..at]);
        out.push_str(match original {
            "BẢN" => "BỔN",
            _ if capital => "Bổn",
            _ => "bổn",
        });
        last = at + "bản".len();
        fixed += 1;
    }
    out.push_str(&text[last..]);
    (out, fixed)
}

/// Chuỗi bất kỳ (luật giọng, cặp xưng hô) đổi theo cùng luật, không có danh xưng nào được giữ `bản`.
pub fn bon_text(text: &str) -> String {
    bon_self_reference(text, &[]).0
}

/// Cặp xưng hô model khai kèm bản dịch (`bản cung–ngươi`) đổi theo cùng luật trước khi học — cặp đã học được nạp lại
/// vào prompt mỗi chương, để `bản` thì kéo model về `bản`.
pub fn bon_in_addressing(entries: &mut Value) {
    for item in entries.as_array_mut().into_iter().flatten() {
        let Some(record) = item.as_object_mut() else { continue };
        if record.get("category").and_then(Value::as_str) != Some("addressing") {
            continue;
        }
        if let Some(Value::String(target)) = record.get_mut("target") {
            let (fixed, count) = bon_self_reference(target, &[]);
            if count > 0 {
                *target = fixed;
            }
        }
    }
}

/// Danh xưng mà glossary của truyện ghi `bản X` (cố ý giữ `bản`): không đổi các chỗ đó. Truyền target các nhóm trừ
/// `addressing` — cặp xưng hô là model tự học, không phải lựa chọn chính tả.
pub fn glossary_keeps<'a>(targets: impl IntoIterator<Item = &'a str>) -> Vec<&'static str> {
    let targets: Vec<String> = targets.into_iter().map(str::to_lowercase).collect();
    TITLES
        .iter()
        .map(|(title, _)| *title)
        .filter(|title| targets.iter().any(|target| target.contains(&format!("bản {title}"))))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(text: &str) -> String {
        bon_self_reference(text, &[]).0
    }

    #[test]
    fn doi_tu_xung_ban_thanh_bon_giu_hoa_thuong() {
        assert_eq!(fix("“Bản cung nói được làm được!”"), "“Bổn cung nói được làm được!”");
        assert_eq!(fix("làm kinh động khiến bản tọa chẳng được yên ổn"), "làm kinh động khiến bổn tọa chẳng được yên ổn");
        assert_eq!(fix("Ăn một kiếm của bản cô nương này!"), "Ăn một kiếm của bổn cô nương này!");
        assert_eq!(fix("cho bản tướng quân xem nào. Bản đế tử đi đâu"), "cho bổn tướng quân xem nào. Bổn đế tử đi đâu");
        assert_eq!(fix("BẢN VƯƠNG"), "BỔN VƯƠNG");
        assert_eq!(
            bon_self_reference("bản vương, bản quan, bản thiếu gia, bản công chúa, bản hoàng tử, bản miêu", &[]).1,
            6
        );
    }

    #[test]
    fn khong_dung_nghia_khac_tu_ghep_ten_nhat() {
        for text in [
            "sự khác biệt giữa ma thân và bản tôn",
            "nhìn thấy bản tướng của vị thần này",
            "người trong bản gia họ Tiết",
            "thạch bản phủ kín ký tự",
            "thân thể phiên bản thiếu nữ của nàng",
            "một văn bản quan trọng",
            "đây là bản quan trọng nhất",
            "bản miêu tả chi tiết",
            "kịch bản thần thoại",
            "phiên bản cung đình",
            "Nhật Bản hoàng thất",
            "gặp Cung Bản tiểu thư ở Tokyo",
            "Sơn Bản công tử cười",
            "bản đồ, bản thân, bản lĩnh",
            "cuốn bản thánh kinh",
        ] {
            assert_eq!(fix(text), text, "{text}");
        }
    }

    #[test]
    fn cap_xung_ho_khai_kem_ban_dich_doi_bon_truoc_khi_hoc() {
        let mut entries = serde_json::json!([
            { "category": "addressing", "source": "武则天→薛绍", "target": "bản cung–ngươi" },
            { "category": "names", "source": "本因", "target": "Bản Nhân" },
        ]);
        bon_in_addressing(&mut entries);
        assert_eq!(entries[0]["target"], "bổn cung–ngươi");
        assert_eq!(entries[1]["target"], "Bản Nhân");
    }

    #[test]
    fn glossary_ghi_ban_thi_giu() {
        let keep = glossary_keeps(["bản miêu", "Bản Trần"]);
        assert_eq!(keep, vec!["miêu"]);
        assert_eq!(bon_self_reference("bản miêu và bản tọa", &keep).0, "bản miêu và bổn tọa");
    }
}
