//! Cấu trúc convert chắc chắn sai mà prompt cấm vẫn lọt: accept sửa thẳng, không tốn lượt soát.
//!
//! `đối với X mà nói` (对…来说) → `đối với X`. `với X mà nói` chỉ đổi khi đứng đầu vế và theo sau là dấu phẩy, vì
//! `với` còn là giới từ cách thức: `với giọng lạnh lùng mà nói:` thì `mà nói` là động từ thật.

use regex::{Captures, Regex};
use std::sync::LazyLock;

static DOI_VOI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(\bđối với\s+[^,.;:!?“”\n]{1,40}?)\s+mà nói(\s*[,.!?…”])").unwrap());
static VOI: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)(^|[“.!?…:]\s*|,\s+)(với\s+[^,.;:!?“”\n]{1,40}?)\s+mà nói(\s*,)").unwrap());

/// Chữ đầu của X cho thấy `với` chỉ cách thức, `mà nói` là động từ.
const MANNER_HEADS: &[&str] = &["giọng", "vẻ", "ánh", "nụ", "thái", "dáng", "khuôn", "bộ", "tư", "nét", "ngữ", "cặp", "đôi"];

fn is_manner(phrase: &str) -> bool {
    let lower = phrase.to_lowercase();
    lower.split_whitespace().nth(1).is_some_and(|head| MANNER_HEADS.contains(&head))
}

/// Bỏ `mà nói` trong `đối với X mà nói`. Trả văn bản mới và số chỗ đã đổi.
pub fn drop_ma_noi(text: &str) -> (String, usize) {
    let mut fixed = 0;
    let first = DOI_VOI.replace_all(text, |caps: &Captures| {
        fixed += 1;
        format!("{}{}", &caps[1], &caps[2])
    });
    let second = VOI.replace_all(&first, |caps: &Captures| {
        if is_manner(&caps[2]) {
            return caps[0].to_string();
        }
        fixed += 1;
        format!("{}{}{}", &caps[1], &caps[2], &caps[3])
    });
    (second.into_owned(), fixed)
}

#[cfg(test)]
mod tests {
    use super::drop_ma_noi;

    #[test]
    fn bo_ma_noi_sau_doi_voi() {
        assert_eq!(drop_ma_noi("Thế nên đối với ta mà nói, đồ vật càng tốt.").0, "Thế nên đối với ta, đồ vật càng tốt.");
        assert_eq!(drop_ma_noi("“Đối với hắn mà nói.”").0, "“Đối với hắn.”");
        assert_eq!(drop_ma_noi("Với ta mà nói, chuyện nhỏ.").0, "Với ta, chuyện nhỏ.");
        assert_eq!(drop_ma_noi("Hắn cười, với ta mà nói, chuyện nhỏ.").1, 1);
    }

    #[test]
    fn khong_dung_ma_noi_la_dong_tu() {
        for text in [
            "Hắn nhìn nàng với giọng lạnh lùng mà nói: “Đi.”",
            "Với giọng nhỏ nhẹ mà nói, nàng cúi đầu.",
            "Hắn ngồi với nàng mà nói chuyện.",
            "Xét trên khía cạnh qua lại giữa người với người mà nói, biết cho đi là tốt.",
        ] {
            assert_eq!(drop_ma_noi(text), (text.to_string(), 0), "{text}");
        }
    }
}
