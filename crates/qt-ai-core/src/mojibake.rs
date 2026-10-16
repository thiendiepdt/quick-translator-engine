//! Chữ lỗi mã hoá (mojibake): text UTF-8 bị đọc nhầm thành Windows-1252 rồi ghi lại dạng UTF-8 —
//! "Chương" → "ChÆ°Æ¡ng", "ễ" → "á»…", "“" → "â€œ". Hub OpenAI-compatible thỉnh thoảng trả về nội dung đã
//! hỏng như vậy (phía mình đọc HTTP đúng UTF-8): không còn chữ Hán, độ dài vẫn đủ nên check cũ không bắt.

/// Byte 0x80–0x9F của Windows-1252. Byte chưa định nghĩa (0x81, 0x8D, 0x8F, 0x90, 0x9D) đọc như Latin-1
/// (U+0081…) — đúng như bộ giải mã WHATWG mà hub dùng (thấy thật trong out/ của sự cố).
const CP1252_HIGH: [char; 32] = [
    '\u{20AC}', '\u{81}', '\u{201A}', '\u{192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}', '\u{2C6}', '\u{2030}',
    '\u{160}', '\u{2039}', '\u{152}', '\u{8D}', '\u{17D}', '\u{8F}', '\u{90}', '\u{2018}', '\u{2019}', '\u{201C}',
    '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}', '\u{2DC}', '\u{2122}', '\u{161}', '\u{203A}', '\u{153}', '\u{9D}',
    '\u{17E}', '\u{178}',
];

/// Byte Windows-1252 của `c`; `None` nếu ký tự không thuộc bảng mã đó (ư, ơ, đ, ạ, chữ Hán… — tức chắc chắn
/// không phải sản phẩm của việc đọc nhầm).
fn cp1252_byte(c: char) -> Option<u8> {
    let code = c as u32;
    if code < 0x80 || (0xA0..=0xFF).contains(&code) {
        return Some(code as u8);
    }
    CP1252_HIGH.iter().position(|&high| high == c).map(|index| 0x80 + index as u8)
}

/// Ký tự đứng thay cho một byte nối UTF-8 (0x80–0xBF).
fn is_continuation(c: char) -> bool {
    matches!(cp1252_byte(c), Some(0x80..=0xBF))
}

/// Byte nối mà trong văn bản đúng vẫn có thể đứng ngay sau một chữ cái: dấu câu đóng, gạch, ngoặc «», độ,
/// chấm giữa tên, khoảng trắng không ngắt. Không tính là dấu hiệu (vd "NGÃ…", «Hoa Lá», "Lý·Phàm").
const SOFT_FOLLOWERS: [char; 10] = ['\u{A0}', '…', '”', '’', '–', '—', '«', '»', '°', '·'];

fn is_marker_tail(c: char) -> bool {
    is_continuation(c) && !SOFT_FOLLOWERS.contains(&c)
}

/// Số dấu hiệu mojibake trong `text`. Mỗi dấu hiệu là một cụm không bao giờ có trong tiếng Việt/Trung/Anh viết
/// đúng nhưng là dạng đọc nhầm của chữ tiếng Việt:
/// - `Â Ã Ä Å Æ Ì` + ký tự byte nối không phải dấu câu mềm: `Ã©` (é), `Ã´` (ô), `Ä‘` (đ), `Ä\u{90}` (Đ), `Æ°` (ư),
///   `Æ¡` (ơ), `Å©` (ũ), `Ì€` (dấu huyền tổ hợp). `NGÃ BA`, `Ägypten`, `Ålesund` không dính: theo sau là chữ/khoảng.
/// - `áº`/`á»` + ký tự byte nối không phải dấu câu mềm: `á»©` (ứ), `áº¿` (ế). `á»` đứng riêng (vd «Hoa Lá» …) không dính.
/// - `â€` (dấu câu “ ” … – —) và `ã€` (《》【】、。), `ï¼`/`ï½` (dấu câu toàn khổ ，！？).
///
/// Đếm không chồng nhau: mỗi chữ hỏng tính một lần.
pub fn mojibake_score(text: &str) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let tail = |at: usize| chars.get(at).is_some_and(|&c| is_marker_tail(c));
    let mut score = 0;
    let mut at = 0;
    while at + 1 < chars.len() {
        let width = match (chars[at], chars[at + 1]) {
            ('Â' | 'Ã' | 'Ä' | 'Å' | 'Æ' | 'Ì', _) if tail(at + 1) => 2,
            ('á', 'º' | '»') if tail(at + 2) => 3,
            ('â' | 'ã', '€') | ('ï', '¼' | '½') => 2,
            _ => 0,
        };
        if width > 0 {
            score += 1;
            at += width;
        } else {
            at += 1;
        }
    }
    score
}

/// Văn bản có vẻ đã bị mojibake: ≥ 3 dấu hiệu, hoặc ít hơn mà dày (≥ 1 dấu hiệu / 40 ký tự không trắng —
/// cho text ngắn như tiêu đề "ChÆ°Æ¡ng 5" hay JSON glossary vài tên). Văn bản thật bị hỏng có ~1 dấu hiệu / 4 ký tự;
/// dấu hiệu được chọn để văn bản đúng không bao giờ có, ngưỡng 3 chỉ là biên an toàn.
pub fn looks_mojibake(text: &str) -> bool {
    let score = mojibake_score(text);
    score >= 3 || (score > 0 && score * 40 >= text.chars().filter(|c| !c.is_whitespace()).count())
}

/// Sửa ngược mojibake. Dòng có dấu hiệu (hoặc mọi dòng chỉ gồm ký tự Windows-1252 khi cả văn bản `looks_mojibake`)
/// được giải mã lại: ký tự ứng với byte đầu UTF-8 (0xC2–0xF4) ăn đủ số ký tự byte nối theo sau; ghép ra UTF-8 hợp lệ
/// thì thay bằng ký tự đó, không thì giữ nguyên. Dòng còn lại (văn bản đúng xen giữa) không đụng tới.
/// `...` đứng đúng chỗ một byte nối được đọc là `…` (0x85) — bản đã qua chuẩn hoá `…` → `...` vẫn sửa được.
///
/// `Some` chỉ khi có thay đổi và kết quả hết sạch dấu hiệu; không thì `None` (không sửa được, phải gọi lại).
pub fn repair_mojibake(text: &str) -> Option<String> {
    let whole = looks_mojibake(text);
    let mut repaired = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        if mojibake_score(line) > 0 {
            repaired.push_str(&decode_line(line, false).unwrap_or_else(|| line.to_string()));
        } else if whole && line.chars().all(|c| cp1252_byte(c).is_some()) {
            // Dòng hỏng chỉ gồm cụm "mềm" (vd `Ã”i` = Ôi) nằm trong văn bản hỏng: sửa nếu mọi chữ ghép ra đều là chữ
            // tiếng Việt/dấu câu — dòng đúng như “Con bé…” ghép ra chữ Hán/Yi lạ thì giữ nguyên.
            repaired.push_str(&decode_line(line, true).unwrap_or_else(|| line.to_string()));
        } else {
            repaired.push_str(line);
        }
    }
    (repaired != text && mojibake_score(&repaired) == 0).then_some(repaired)
}

/// Chữ hợp lý trong bản dịch tiếng Việt: Latin (kể cả dấu tổ hợp, Latin mở rộng thêm), dấu câu chung,
/// dấu câu CJK và toàn khổ.
fn plausible(c: char) -> bool {
    matches!(c as u32, 0xA0..=0x24F | 0x300..=0x36F | 0x1E00..=0x1EFF | 0x2000..=0x206F | 0x3000..=0x303F | 0xFF00..=0xFFEF)
}

/// Giải mã lại một dòng. `strict`: chữ ghép ra phải `plausible`, không thì bỏ cả dòng (`None`).
fn decode_line(line: &str, strict: bool) -> Option<String> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut at = 0;
    while at < chars.len() {
        if let Some((decoded, used)) = decode_at(&chars, at) {
            if strict && !plausible(decoded) {
                return None;
            }
            out.push(decoded);
            at += used;
        } else {
            out.push(chars[at]);
            at += 1;
        }
    }
    Some(out)
}

/// Ghép một chữ UTF-8 bắt đầu tại `at`: (chữ, số ký tự đã ăn). Byte nối 0x85 có thể đã bị đổi thành `...`.
fn decode_at(chars: &[char], at: usize) -> Option<(char, usize)> {
    let lead = cp1252_byte(chars[at]).filter(|&b| chars[at] as u32 >= 0x80 && (0xC2..=0xF4).contains(&b))?;
    let need = match lead {
        0xC2..=0xDF => 1,
        0xE0..=0xEF => 2,
        _ => 3,
    };
    let mut bytes = vec![lead];
    let mut next = at + 1;
    for _ in 0..need {
        match chars.get(next) {
            Some(&c) if is_continuation(c) => {
                bytes.push(cp1252_byte(c)?);
                next += 1;
            }
            Some('.') if chars.get(next + 1) == Some(&'.') && chars.get(next + 2) == Some(&'.') => {
                bytes.push(0x85);
                next += 3;
            }
            _ => return None,
        }
    }
    let decoded = std::str::from_utf8(&bytes).ok()?.chars().next()?;
    // Chữ điều khiển C1 (vd `Ã` + `...` → U+0085) không bao giờ là nội dung thật.
    (!(0x80..=0x9F).contains(&(decoded as u32))).then_some((decoded, next - at))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mã hoá ngược để dựng mojibake: mỗi byte UTF-8 → ký tự Windows-1252 tương ứng.
    fn to_mojibake(text: &str) -> String {
        text.bytes()
            .map(|byte| match byte {
                0x80..=0x9F => CP1252_HIGH[(byte - 0x80) as usize],
                _ => byte as char,
            })
            .collect()
    }

    // Trích nguyên văn out/ của sự cố (ta-mo-phong-truong-sinh-lo, chương 1325/1326/1330).
    const FIX_1325_TITLE: &str = "ChÆ°Æ¡ng 1314: KÃ½ á»©c cá»§a CÆ¡ DÆ° Trinh";
    const FIX_1325_PARA: &str = "Tu hÃ\u{a0}nh ã€ŠVÃ´ LÆ°á»£ng ThiÃªn Xu Ly Tháº§n CÃ´ngã€‹ sáº½ cÃ³ sá»± phÃ¢n biá»‡t giá»¯a muÃ´n ngÃ\u{a0}n phÃ¢n há»“n vÃ\u{a0} nguyÃªn thá»§y chi há»“n. TrÆ°á»›c Ä‘Ã³ LÃ½ PhÃ\u{a0}m tÃ¬m tháº¥y táº¡i HoÃ\u{a0}n HÃ¬nh SÆ¡n thuá»™c Ä\u{90}Ã´ng VÄ©nh ChÃ¢u chá»‰ lÃ\u{a0} kÃ½ á»©c trong muÃ´n ngÃ\u{a0}n phÃ¢n há»“n kia.";
    const FIX_1330_QUOTE: &str = "Tuy nhiÃªn bá»\u{8d}n há»\u{8d} cÅ©ng Ä‘Æ°a ra yÃªu cáº§u cá»§a mÃ¬nh: â€œChÃ¢n TiÃªn triá»‡n tá»± lÃ\u{a0} tuyá»‡t máº\u{ad}t cá»§a TiÃªn Chu. DÃ¹ nÆ¡i Ä‘Ã¢y náº±m sÃ¢u giá»¯a tinh háº£i khÃ´ng má»™t bÃ³ng ngÆ°á»\u{9d}i, nhÆ°ng cÅ©ng pháº£i Ä‘á»\u{81} phÃ²ng tai vÃ¡ch máº¡ch rá»«ng, cÃ³ káº» dÃ²m ngÃ³.â€\u{9d}";
    /// Đã qua bước chuẩn hoá `…` → `...` sau khi hỏng: "vĩnh viễn" thành "vÄ©nh viá»...n".
    const FIX_1326_DOTS: &str = "ngÅ© quan bá»‹ há»§y, vÄ©nh viá»...n Ä‘eo má»™t chiáº¿c máº·t náº¡ rÄƒng nanh";

    // Đoạn đúng của cùng truyện (chương 1324) và các mẫu dễ báo nhầm.
    const HEALTHY: &[&str] = &[
        "Chương 1313: Phù Độ đại trận tề",
        "Vô số đường nét đen trắng trong nháy mắt sống dậy, thay đổi quỹ tích ban đầu, lao vút về phía Lý Phàm. Tựa như từng chiếc xúc tu, muốn quấn quanh khóa chặt Lý Phàm.",
        "Không gian mang tên 【Thiên Địa Chi Căn】 này chính là vị trí thai nghén Ngũ Hành đại động thiên.",
        "Tỷ như, tuy đã rất mờ nhạt, gần như hòa vào trong toàn bộ không gian Thiên Địa Chi Căn, nhưng vẫn từ tầng cơ sở ảnh hưởng tới sự vận hành tổng thể......",
        "Miệng thì nói như vậy, nhưng trong lòng thực chất không hề cảm thấy có bao nhiêu tiếc nuối. Lý Phàm híp mắt lại, hồi tưởng lại những “dấu vết nhân vi” vừa mới nhìn thấy kia.",
        "“Đáng tiếc.”",
        "赵静文抬头看向远方的高塔。《剑谱》",
        "The quick brown fox — “jumps” over the lazy dog… © 2024, 25°C, Ägypten, Ålesund, Æsir, naïve, café.",
        "NGÃ BA ĐƯỜNG, ÂM THANH, CHƯƠNG MỘT, HÌNH ẢNH — “NGÃ” rồi “NGÃ…”, à… và…”",
        "«Hoa Lá» … «Cây Lá»… Lý·Phàm, “Con bé…” “Quê…” “Ê!” “Ô…”",
        "Thời Đại Của Ã, chữ Ì đứng riêng, ÁO DÀI.",
    ];

    #[test]
    fn van_ban_dung_khong_co_dau_hieu_nao() {
        for text in HEALTHY {
            assert_eq!(mojibake_score(text), 0, "{text}");
            assert!(!looks_mojibake(text), "{text}");
            assert_eq!(repair_mojibake(text), None, "{text}");
        }
        let all = HEALTHY.join("\n\n");
        assert_eq!(mojibake_score(&all), 0);
        assert_eq!(repair_mojibake(&all), None);
    }

    #[test]
    fn bat_van_ban_that_bi_hong() {
        assert!(mojibake_score(FIX_1325_TITLE) >= 5, "{}", mojibake_score(FIX_1325_TITLE));
        assert!(looks_mojibake(FIX_1325_TITLE));
        for text in [FIX_1325_PARA, FIX_1330_QUOTE, FIX_1326_DOTS] {
            assert!(mojibake_score(text) * 10 >= text.chars().count(), "dày: {} / {}", mojibake_score(text), text.chars().count());
            assert!(looks_mojibake(text), "{text}");
        }
    }

    #[test]
    fn nguong_text_ngan_va_dau_hieu_le_trong_van_dai() {
        assert!(looks_mojibake("ChÆ°Æ¡ng 5"), "tiêu đề ngắn 2 dấu hiệu");
        assert!(looks_mojibake("LÃ½ PhÃ\u{a0}m"), "một tên trong JSON ngắn");
        // Văn dài đúng lẫn một cụm lạ: không coi cả bài là hỏng.
        let long = format!("{} Ã© {}", HEALTHY[1], HEALTHY[4]);
        assert_eq!(mojibake_score(&long), 1);
        assert!(!looks_mojibake(&long));
    }

    #[test]
    fn sua_tron_ve_nguyen_ban() {
        for original in [
            "Chương 1314: Ký ức của Cơ Dư Trinh",
            "Ông lão nói: “Đừng đi… Ừ, ổn rồi — 《Vô Lượng》【Căn】，thật sao？”",
            "Thật là ồn ào, Ơ, Ư, Ũ, ũ, ỹ, Ỵ, ạ, ặ, ẫ, đến đây",
            HEALTHY[1],
            HEALTHY[4],
        ] {
            let broken = to_mojibake(original);
            assert!(looks_mojibake(&broken), "{broken}");
            assert_eq!(repair_mojibake(&broken).as_deref(), Some(original), "{broken}");
        }
    }

    #[test]
    fn sua_doan_trich_tu_out_hong() {
        assert_eq!(repair_mojibake(FIX_1325_TITLE).as_deref(), Some("Chương 1314: Ký ức của Cơ Dư Trinh"));
        let para = repair_mojibake(FIX_1325_PARA).unwrap();
        assert!(para.starts_with("Tu hành 《Vô Lượng Thiên Xu Ly Thần Công》 sẽ có sự phân biệt"), "{para}");
        assert!(para.contains("Trước đó Lý Phàm tìm thấy tại Hoàn Hình Sơn thuộc Đông Vĩnh Châu"), "{para}");
        let quote = repair_mojibake(FIX_1330_QUOTE).unwrap();
        assert!(quote.starts_with("Tuy nhiên bọn họ cũng đưa ra yêu cầu của mình: “Chân Tiên triện tự là tuyệt mật"), "{quote}");
        assert!(quote.ends_with("cũng phải đề phòng tai vách mạch rừng, có kẻ dòm ngó.”"), "{quote}");
        assert_eq!(
            repair_mojibake(FIX_1326_DOTS).as_deref(),
            Some("ngũ quan bị hủy, vĩnh viễn đeo một chiếc mặt nạ răng nanh")
        );
    }

    #[test]
    fn van_tron_lan_chi_sua_phan_hong() {
        let text = format!("{}\n\n{}\n\nLý Phàm nói: â€œXin chÃ\u{a0}o, Ä‘áº¡o há»¯u.â€\u{9d}", HEALTHY[5], FIX_1325_TITLE);
        assert_eq!(
            repair_mojibake(&text).as_deref(),
            Some("“Đáng tiếc.”\n\nChương 1314: Ký ức của Cơ Dư Trinh\n\nLý Phàm nói: “Xin chào, đạo hữu.”")
        );
        // Dòng đúng toàn ký tự Windows-1252 nằm cạnh dòng hỏng có dấu hiệu: không bị giải mã sai thành chữ lạ.
        let text = format!("“Con bé…”\n{}", to_mojibake("Chương một: Đạo hữu"));
        assert_eq!(repair_mojibake(&text).as_deref(), Some("“Con bé…”\nChương một: Đạo hữu"));
    }

    #[test]
    fn khong_sua_duoc_thi_none() {
        // Có dấu hiệu nhưng byte nối đã mất — không ghép lại được UTF-8.
        assert_eq!(repair_mojibake("â€ x â€ y â€ z"), None);
        assert_eq!(repair_mojibake("ChÆ°Æ¡ng 1 Ä"), Some("Chương 1 Ä".to_string()), "Ä lẻ không phải dấu hiệu");
    }
}
