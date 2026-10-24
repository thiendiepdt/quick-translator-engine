//! Âm tiết không thể có trong tiếng Việt ("mồhôi", "pháthuy", "khếước", "nghẩn ngơ") — model thỉnh thoảng sinh
//! hỏng token hoặc sai chính tả. Bắt bằng luật cấu tạo âm tiết (phụ âm đầu + vần + thanh), không tốn token; đoạn
//! dính thì đưa vào vòng soát (model sửa, hoặc giữ nếu là chữ lóng cố ý).
//!
//! Luật chép từ noveltools (`vnSyllableProblem`), bổ sung vần `yêm` (yểm, yếm — họ thiếu) và các nhóm bỏ qua đo
//! được trên 40.185 chương thư viện: thán từ kéo dài (Kéttt), nói lắp / câu ngắt (Vậ, vậy), chửi che dấu sao
//! (Đ* m*), từ mượn (kilômét, micrô), pinyin trong ngoặc. Chống báo oan ưu tiên hơn bắt đủ: chỉ xét từ có dấu
//! tiếng Việt, bỏ từ viết hoa giữa câu (tên riêng).

/// Thông điệp vi phạm (kèm danh sách từ) — vào issues, review.md, warnings.
pub const VN_SYLLABLE_ISSUE: &str = "Âm tiết không có trong tiếng Việt";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tone {
    Ngang,
    Huyen,
    Sac,
    Hoi,
    Nga,
    Nang,
}

/// Nguyên âm có dấu → (chữ gốc giữ mũ/móc, thanh). Thứ tự trong mỗi chuỗi: ngang, huyền, sắc, hỏi, ngã, nặng.
const VOWELS: [&str; 12] = [
    "aàáảãạ", "ăằắẳẵặ", "âầấẩẫậ", "eèéẻẽẹ", "êềếểễệ", "iìíỉĩị", "oòóỏõọ", "ôồốổỗộ", "ơờớởỡợ", "uùúủũụ", "ưừứửữự",
    "yỳýỷỹỵ",
];
const TONES: [Tone; 6] = [Tone::Ngang, Tone::Huyen, Tone::Sac, Tone::Hoi, Tone::Nga, Tone::Nang];

fn split_vowel(c: char) -> Option<(char, Tone)> {
    VOWELS.iter().find_map(|row| {
        let base = row.chars().next()?;
        row.chars().position(|x| x == c).map(|i| (base, TONES[i]))
    })
}

/// Bỏ thanh: chữ thường, giữ ă â ê ô ơ ư đ. `None` khi từ có ký tự ngoài chữ Việt (Latin f/j/w/z, chữ Hán,
/// dấu tổ hợp rời) — không phán.
fn split_tones(word: &str) -> Option<(String, Vec<Tone>)> {
    let mut base = String::new();
    let mut tones = Vec::new();
    for c in word.chars().flat_map(char::to_lowercase) {
        if let Some((vowel, tone)) = split_vowel(c) {
            base.push(vowel);
            if tone != Tone::Ngang {
                tones.push(tone);
            }
        } else if matches!(c, 'b'..='d' | 'g'..='h' | 'k'..='n' | 'p'..='t' | 'v' | 'x' | 'đ') {
            base.push(c);
        } else {
            return None;
        }
    }
    Some((base, tones))
}

const ONSETS: [&str; 27] = [
    "ngh", "ch", "gh", "gi", "kh", "ng", "nh", "ph", "qu", "th", "tr", "b", "c", "d", "đ", "g", "h", "k", "l", "m", "n",
    "p", "r", "s", "t", "v", "x",
];
const CODAS: [&str; 9] = ["ch", "ng", "nh", "c", "m", "n", "p", "t", ""];
const NUCLEI: [&str; 54] = [
    "a", "ă", "â", "e", "ê", "i", "o", "ô", "ơ", "u", "ư", "y", "ai", "ao", "au", "ay", "âu", "ây", "eo", "êu", "ia",
    "iê", "iu", "oa", "oă", "oe", "oi", "ôi", "ơi", "ua", "uâ", "uê", "ui", "uô", "uơ", "ưa", "ưi", "ươ", "ưu", "uy",
    "yê", "oo", "iêu", "yêu", "oai", "oao", "oay", "oeo", "uây", "uôi", "ươi", "ươu", "uyê", "uya",
];

/// Lý do từ (chữ thường, đã bỏ thanh) không thể là một âm tiết tiếng Việt; `None` nếu có ít nhất một cách tách hợp lệ.
fn syllable_problem(base: &str, tones: &[Tone]) -> Option<String> {
    if tones.len() > 1 {
        return Some(format!("{} dấu thanh", tones.len()));
    }
    let tone = tones.first().copied().unwrap_or(Tone::Ngang);
    let mut first: Option<String> = None;
    for onset in ONSETS.iter().copied().chain(std::iter::once("")) {
        let Some(rest) = base.strip_prefix(onset) else { continue };
        for coda in CODAS {
            let Some(nucleus) = rest.strip_suffix(coda) else { continue };
            if nucleus.is_empty() {
                continue;
            }
            let mut fail = |why: String| {
                first.get_or_insert(why);
            };
            if !NUCLEI.contains(&nucleus) && nucleus != "uyu" {
                fail(format!("nguyên âm \"{nucleus}\""));
                continue;
            }
            let front = ["i", "e", "ê", "y", "iê", "yê"].iter().any(|p| nucleus.starts_with(p));
            if onset == "k" && !front {
                fail("\"k\" chỉ đứng trước i/e/ê/y".into());
                continue;
            }
            if onset == "c" && front {
                fail("\"c\" không đứng trước i/e/ê/y".into());
                continue;
            }
            if (onset == "gh" || onset == "ngh") && !front {
                fail(format!("\"{onset}\" chỉ đứng trước i/e/ê"));
                continue;
            }
            if (coda == "ch" || coda == "nh") && !["a", "ê", "i", "e", "y", "oa", "uy", "uê"].contains(&nucleus) {
                fail(format!("vần \"{nucleus}{coda}\""));
                continue;
            }
            if coda.is_empty() && ["iê", "yê", "uyê", "uô", "ươ", "ă", "â"].contains(&nucleus) {
                fail(format!("vần mở \"{nucleus}\""));
                continue;
            }
            let allowed = match nucleus {
                "iê" | "ươ" => Some(&["c", "m", "n", "ng", "p", "t"][..]),
                // Thêm `m` so với bản gốc: yểm, yếm, yêm.
                "yê" => Some(&["m", "n", "t", "ng"][..]),
                "uyê" => Some(&["n", "t"][..]),
                "uô" => Some(&["c", "m", "n", "ng", "t"][..]),
                _ => None,
            };
            if allowed.is_some_and(|list| !coda.is_empty() && !list.contains(&coda)) {
                fail(format!("vần \"{nucleus}{coda}\""));
                continue;
            }
            if ["c", "ch", "p", "t"].contains(&coda) && !matches!(tone, Tone::Sac | Tone::Nang) {
                fail(format!("vần tắc \"-{coda}\" chỉ mang thanh sắc / nặng"));
                continue;
            }
            return None;
        }
    }
    Some(first.unwrap_or_else(|| "không tách được phụ âm đầu + vần".into()))
}

/// Từ mượn, đơn vị đo phiên âm hay gặp trong bản dịch — đúng chính tả nhưng không theo cấu tạo âm tiết thuần Việt.
const LOANWORDS: [&str; 20] = [
    "kilômét", "milimét", "xentimét", "centimét", "kilôgam", "mililít", "micrô", "gilê", "capô", "nitơ", "garô",
    "amiđan", "amiăng", "virút", "xiphông", "nilông", "môtô", "alô", "clohiđric", "đm",
];

fn has_vietnamese_marks(word: &str) -> bool {
    word.chars().flat_map(char::to_lowercase).any(|c| {
        matches!(c, 'ă' | 'â' | 'ê' | 'ô' | 'ơ' | 'ư' | 'đ') || split_vowel(c).is_some_and(|(_, tone)| tone != Tone::Ngang)
    })
}

/// Từ đứng đầu câu (đầu đoạn, sau dấu kết câu, hoặc sau mở ngoặc kép đi sau `:`/`,`) — viết hoa ở đó không phải tên riêng.
fn at_sentence_start(before: &str) -> bool {
    let mut saw_quote = false;
    for c in before.chars().rev() {
        if c.is_whitespace() || "\"“”‘'「『«(([—–-".contains(c) {
            saw_quote |= "\"“”‘'「『«".contains(c);
            continue;
        }
        return ".!?…。！？\n".contains(c) || (saw_quote && ":,，：".contains(c));
    }
    true
}

/// Vế sau của tên ghép gạch nối có vế đầu viết hoa: "Belle-mère", "Jean-Pièrre" (không tính nói lắp "Đ-đây").
fn hyphen_name_tail(before: &str) -> bool {
    let Some(head) = before.strip_suffix('-') else { return false };
    let head: Vec<char> = head.chars().rev().take_while(|c| c.is_alphabetic()).collect();
    head.len() >= 2 && head.last().is_some_and(|c| c.is_uppercase())
}

/// Chữ bị ngắt giữa chừng trước "...", "—" ("chuyệ... Hả?", "lần đầu tiê —"): thêm một âm cuối là thành âm tiết đúng.
fn cut_off(base: &str, tones: &[Tone], after: &str) -> bool {
    let rest = after.trim_start();
    ["...", "…", "—", "–", "-"].iter().any(|p| rest.starts_with(p))
        && CODAS.iter().any(|coda| !coda.is_empty() && syllable_problem(&format!("{base}{coda}"), tones).is_none())
}

/// Cùng một chữ cái lặp từ 3 lần liền: thán từ kéo dài ("Kéttt", "khônggggg", "Ưmmm").
fn elongated(word: &str) -> bool {
    let letters: Vec<char> = word.chars().flat_map(char::to_lowercase).map(|c| split_vowel(c).map_or(c, |(v, _)| v)).collect();
    letters.windows(3).any(|w| w[0] == w[1] && w[1] == w[2])
}

/// Các từ trong `text` có dấu tiếng Việt mà không thể là âm tiết tiếng Việt, theo thứ tự, không lặp.
pub fn suspect_syllables(text: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut start: Option<usize> = None;
    let mut check = |from: usize, to: usize| {
        let word = &text[from..to];
        let before = &text[..from];
        let after = &text[to..];
        let next = after.chars().next();
        if !has_vietnamese_marks(word) || elongated(word) || LOANWORDS.contains(&word.to_lowercase().as_str()) {
            return;
        }
        // Tên riêng: viết hoa giữa câu, hoặc vế sau tên ghép gạch nối.
        if word.chars().next().is_some_and(char::is_uppercase) && !at_sentence_start(before) || hyphen_name_tail(before) {
            return;
        }
        // Chửi che dấu sao (Đ* m*), nói lắp / câu bị ngắt (Vậ, vậy — Điệ, Điện hạ — lớp trưở...).
        if next == Some('*') || (word.chars().count() <= 4 && next.is_some_and(|c| "-,.…—–~".contains(c))) {
            return;
        }
        // Pinyin chú trong ngoặc: "(qiàn)".
        if before.ends_with('(') && next == Some(')') {
            return;
        }
        let Some((base, tones)) = split_tones(word) else { return };
        if syllable_problem(&base, &tones).is_some() && !cut_off(&base, &tones, after) && !found.iter().any(|w| w == word) {
            found.push(word.to_string());
        }
    };
    for (i, c) in text.char_indices() {
        if c.is_alphabetic() && !('\u{2E80}'..='\u{9FFF}').contains(&c) {
            start.get_or_insert(i);
        } else if let Some(from) = start.take() {
            check(from, i);
        }
    }
    if let Some(from) = start {
        check(from, text.len());
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bat_loi_that_trong_thu_vien() {
        for (text, word) in [
            ("mặt nhễ nhại mồhôi, nụ cười", "mồhôi"),
            ("lòng bàn tay ướt đẫm mồôi lạnh.", "mồôi"),
            ("lá chắn cũng pháthuy tác dụng", "pháthuy"),
            ("ký kết khếước!", "khếước"),
            ("lấp liửng nói", "liửng"),
            ("năm tháng dài đằng đẵngường ấy", "đẵngường"),
            ("không ảnhảnh gì đâu", "ảnhảnh"),
            ("lời hẹn ước tươnglai xa xôi", "tươnglai"),
            ("Thấy hắn thờơ không động tĩnh", "thờơ"),
            ("mang theo một chút kiêu hãp nho nhỏ", "hãp"),
            ("nhìn đến nghẩn ngơ chết lặng", "nghẩn"),
            ("viết nghuệch ngoạc một năm", "nghuệch"),
            ("Đạp bằng ghập ghềnh thành đại lộ", "ghập"),
            ("lồi lõm ghồ ghề", "ghồ"),
            ("chờ quái nghoẻo củ tỏi", "nghoẻo"),
        ] {
            assert_eq!(suspect_syllables(text), vec![word.to_string()], "{text}");
        }
    }

    #[test]
    fn am_tiet_hop_le_khong_bat() {
        let text = "Binh bất yếm trá, yểm hộ, yêm nô. Nghiêng người, khuya khoắt, quyết tâm, giường, gì, thuở ấy, \
                    khuỷu tay, ngoằn ngoèo, nguệch ngoạc, gập ghềnh, gồ ghề, chết ngoẻo, quạu quọ, oẳn tù tì, \
                    hoắc, uống rượu, khuyên, tuyết, huých, ạch, ếch, ịch, oách, ngoạm, thoăn thoắt, đường, nước.";
        assert_eq!(suspect_syllables(text), Vec::<String>::new());
    }

    #[test]
    fn bo_qua_than_tu_noi_lap_che_chui_tu_muon_ten_rieng() {
        let text = "“Kéttt!” “Khônggggg!” “Ưmmm~” “Vậ, vậy thì…” “Điệ, Điện hạ” “lớp trưở... khụ” “Đ* m* tra!” \
                    hai trăm kilômét, bật micrô, áo gilê, nilông, xin lỗi (qiàn), bất đồng (yuàn). \
                    Hắn gặp Dégel ở Château Latour. hello world much love";
        assert_eq!(suspect_syllables(text), Vec::<String>::new());
    }

    #[test]
    fn bo_qua_ten_ghep_gach_noi_va_chu_bi_ngat() {
        let text = "dưỡng mẫu Belle-mère của các nàng. “Không nhìn ra cũng là chuyệ... Hả?” “Lần đầu tiê —”";
        assert_eq!(suspect_syllables(text), Vec::<String>::new());
        // Thêm âm cuối vẫn không thành chữ đúng thì vẫn là lỗi, dù đứng trước "...".
        assert_eq!(suspect_syllables("ướt đẫm mồhôi..."), vec!["mồhôi".to_string()]);
        // Nói lắp gạch nối một chữ cái không phải tên ghép.
        assert_eq!(suspect_syllables("“Đ-đẵngường”"), vec!["đẵngường".to_string()]);
    }

    #[test]
    fn viet_hoa_dau_cau_van_xet() {
        assert_eq!(suspect_syllables("“Nguời ta ngẩu dồi!”"), vec!["Nguời".to_string()]);
        assert_eq!(suspect_syllables("Xong. Thưở ấy"), vec!["Thưở".to_string()]);
    }
}
