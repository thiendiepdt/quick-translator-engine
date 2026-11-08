//! Số cho Quy Tắc: giá trị số Hán và cách đọc ra chữ.
//!
//! Port nguyên từ bản thử (`qtlib.py`) để output khớp từng chữ. Không dùng
//! chung `crate::number`: cách đọc ở đây là vạn / ức / ngàn, "lẻ", "mốt",
//! "lăm", "tư", đúng như bản biên tập viết.

/// Ký tự được coi là chữ số Hán trong mẫu `[số]`.
pub(crate) const NUM_CHARS: &str = "零〇一二三四五六七八九十百千万亿两";

/// Như `NUM_CHARS.contains(c)`, khỏi quét chuỗi (test kiểm trên mọi ký tự).
pub(crate) fn is_num_char(c: char) -> bool {
    matches!(
        c,
        '零' | '〇' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' | '十' | '百'
            | '千' | '万' | '亿' | '两'
    )
}

const WORDS: [&str; 11] = [
    "không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín", "mười",
];

/// Đọc từng chữ số (phần thập phân, năm đọc rời).
pub(crate) const DIGIT_WORDS: [&str; 10] =
    ["không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín"];

fn digit_value(c: char) -> Option<i64> {
    Some(match c {
        '零' | '〇' => 0,
        '一' => 1,
        '二' | '两' => 2,
        '三' => 3,
        '四' => 4,
        '五' => 5,
        '六' => 6,
        '七' => 7,
        '八' => 8,
        '九' => 9,
        _ => return None,
    })
}

fn unit_value(c: char) -> Option<i64> {
    Some(match c {
        '十' => 10,
        '百' => 100,
        '千' => 1_000,
        '万' => 10_000,
        '亿' => 100_000_000,
        _ => return None,
    })
}

/// Hai chữ số Hán liền nhau, không chữ nào là 零/〇: số ước lượng (`一两`, `两三`,
/// `十二三`, `二十七八`, `二三十`), không có một giá trị. `一百零五`, `五〇` thì là
/// số thật.
fn approximate(chars: &[char]) -> bool {
    let nonzero = |c: char| !matches!(c, '零' | '〇') && digit_value(c).is_some();
    chars.windows(2).any(|pair| nonzero(pair[0]) && nonzero(pair[1]))
}

/// Giá trị của số Hán hoặc số Ả Rập. Chuỗi toàn chữ số đơn từ ba chữ trở lên
/// (二〇一九, 一九九八) đọc rời từng chữ. `三千五` = 3500: chữ số cuối đứng sau đơn
/// vị mang đơn vị kế dưới. Số ước lượng (`approximate`) không có giá trị: token giữ
/// bản dịch của engine ("một hai", "mười hai mười ba").
pub(crate) fn zh_value(s: &str) -> Option<i64> {
    if !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) {
        return s.parse().ok();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() == 2 && approximate(&chars) {
        return None;
    }
    if chars.len() > 1 && chars.iter().all(|&c| digit_value(c).is_some()) {
        let mut value: i64 = 0;
        for &c in &chars {
            value = value.checked_mul(10)?.checked_add(digit_value(c)?)?;
        }
        return Some(value);
    }
    if approximate(&chars) {
        return None;
    }
    let (mut total, mut section, mut num, mut last_unit) = (0i64, 0i64, 0i64, 1i64);
    for &c in &chars {
        if let Some(digit) = digit_value(c) {
            num = digit;
        } else if c == '万' || c == '亿' {
            let unit = unit_value(c)?;
            section = section.checked_add(num)?.checked_mul(unit)?;
            total = total.checked_add(section)?;
            section = 0;
            num = 0;
            last_unit = unit;
        } else if let Some(unit) = unit_value(c) {
            section = section.checked_add(if num == 0 { 1 } else { num }.checked_mul(unit)?)?;
            num = 0;
            last_unit = unit;
        } else {
            return None;
        }
    }
    if num != 0
        && last_unit >= 100
        && chars.len() >= 2
        && unit_value(chars[chars.len() - 2]).is_some()
    {
        num = num.checked_mul(last_unit / 10)?;
    }
    total.checked_add(section)?.checked_add(num)
}

fn below_thousand(n: i64) -> String {
    let (hundreds, rest) = (n / 100, n % 100);
    let (tens, ones) = (rest / 10, rest % 10);
    let mut parts: Vec<&str> = Vec::new();
    if hundreds > 0 {
        parts.push(WORDS[hundreds as usize]);
        parts.push("trăm");
        if rest > 0 && tens == 0 {
            parts.push("lẻ");
        }
    }
    if tens == 1 {
        parts.push("mười");
    } else if tens > 1 {
        parts.push(WORDS[tens as usize]);
        parts.push("mươi");
    }
    if ones > 0 {
        parts.push(match ones {
            1 if tens >= 2 => "mốt",
            5 if tens >= 1 => "lăm",
            4 if tens >= 2 => "tư",
            _ => WORDS[ones as usize],
        });
    }
    parts.join(" ")
}

/// Đọc số ra chữ: ức / vạn / ngàn như bản biên tập.
pub(crate) fn number_words(n: i64) -> String {
    if n <= 0 {
        return "không".to_string();
    }
    let mut n = n;
    let mut parts: Vec<String> = Vec::new();
    for (unit, name) in [(100_000_000, "ức"), (10_000, "vạn"), (1_000, "ngàn")] {
        if n >= unit {
            parts.push(number_words(n / unit));
            parts.push(name.to_string());
            n %= unit;
        }
    }
    if n > 0 {
        parts.push(below_thousand(n));
    }
    parts.join(" ")
}

/// Số thứ tự: nhất, hai, tư; còn lại đọc như số đếm.
pub(crate) fn number_ordinal(n: i64) -> String {
    match n {
        1 => "nhất".to_string(),
        2 => "hai".to_string(),
        4 => "tư".to_string(),
        _ => number_words(n),
    }
}

/// Giá trị số ra chữ theo bộ lọc khuôn: `thứ`, `ngày`, `số`, hoặc rỗng (giữ kiểu nguồn).
pub(crate) fn num_text(value: i64, digit: bool, how: &str) -> String {
    match how {
        "thứ" if digit => value.to_string(),
        "thứ" => number_ordinal(value),
        "ngày" => {
            let text = if digit { value.to_string() } else { number_words(value) };
            if value <= 10 {
                format!("mùng {text}")
            } else {
                text
            }
        }
        "số" => value.to_string(),
        _ if digit => value.to_string(),
        _ => number_words(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn num_char_set_matches_the_string() {
        for c in (0..=0x10ffff).filter_map(char::from_u32) {
            assert_eq!(is_num_char(c), NUM_CHARS.contains(c), "{c:?}");
        }
    }

    #[test]
    fn reads_chinese_numbers_like_the_prototype() {
        assert_eq!(zh_value("三千五"), Some(3500));
        assert_eq!(zh_value("二〇一九"), Some(2019));
        assert_eq!(zh_value("一九九八"), Some(1998));
        assert_eq!(zh_value("十五"), Some(15));
        assert_eq!(zh_value("二十四"), Some(24));
        assert_eq!(zh_value("三百五十"), Some(350));
        assert_eq!(zh_value("两百"), Some(200));
        assert_eq!(zh_value("十万"), Some(100_000));
        assert_eq!(zh_value("一万三千"), Some(13_000));
        assert_eq!(zh_value("2015"), Some(2015));
        assert_eq!(zh_value("三点"), None);
    }

    /// Bản thử đọc `三五` thành 35, `一两` thành 12, `十二三` thành 13: số ước lượng
    /// không có một giá trị.
    #[test]
    fn approximate_numbers_have_no_value() {
        for s in ["三五", "一两", "两三", "七八", "十二三", "二十七八", "二三十", "三四百"] {
            assert_eq!(zh_value(s), None, "{s}");
        }
        assert_eq!(zh_value("一百零五"), Some(105));
        assert_eq!(zh_value("一千零五十"), Some(1050));
        assert_eq!(zh_value("〇五"), Some(5));
    }

    #[test]
    fn reads_numbers_in_vietnamese_words() {
        assert_eq!(number_words(0), "không");
        assert_eq!(number_words(15), "mười lăm");
        assert_eq!(number_words(21), "hai mươi mốt");
        assert_eq!(number_words(24), "hai mươi tư");
        assert_eq!(number_words(105), "một trăm lẻ năm");
        assert_eq!(number_words(350), "ba trăm năm mươi");
        assert_eq!(number_words(2019), "hai ngàn mười chín");
        assert_eq!(number_words(13_000), "một vạn ba ngàn");
        assert_eq!(number_ordinal(1), "nhất");
        assert_eq!(number_ordinal(4), "tư");
        assert_eq!(number_ordinal(3), "ba");
    }

    #[test]
    fn num_text_follows_filters() {
        assert_eq!(num_text(4, false, "thứ"), "tư");
        assert_eq!(num_text(4, true, "thứ"), "4");
        assert_eq!(num_text(3, false, "ngày"), "mùng ba");
        assert_eq!(num_text(15, false, "ngày"), "mười lăm");
        assert_eq!(num_text(50, false, "số"), "50");
        assert_eq!(num_text(50, false, ""), "năm mươi");
        assert_eq!(num_text(50, true, ""), "50");
    }
}
