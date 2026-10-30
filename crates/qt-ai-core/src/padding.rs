//! Bù chữ cho chương ngắn: nền tảng đăng truyện không nhận chương dưới N chữ (không tính dòng tên chương). Chương
//! thiếu được chèn thêm văn mẫu (hoặc lặp lại nội dung chương) bọc giữa hai dòng ngăn `PADDING_MARKER` (gộp chương
//! về sau thì tách phần chèn ra được), từng đoạn nguyên vẹn tới khi đủ, nên chỉ vượt ngưỡng một chút.
//!
//! Phần chèn nằm luôn trong `out/` (đăng thẳng file được). Mọi chỗ ghép đoạn bản dịch với đoạn raw (Tìm & thay, rà
//! glossary chương cũ, kiểm tra VN) chỉ xét phần trước dòng ngăn (`translation_part`, `content_lines`) — phần chèn
//! không lệch số đoạn và không bị sửa theo.
//!
//! "Chữ" là tiếng: cụm chữ / số liền nhau ("Hôm nay đi chơi" = 4 chữ).

use serde::{Deserialize, Serialize};

/// Dòng ngăn mở và đóng phần chèn — ký tự gần như không có trong truyện để tool khác so nguyên dòng mà tách. File
/// chèn từ bản cũ chỉ có dòng mở: phần chèn kéo tới hết chương.
pub const PADDING_MARKER: &str = "❖❖❖❖❖❖❖❖❖❖";
pub const DEFAULT_THRESHOLD: usize = 500;
/// Văn mẫu mặc định (người dùng viết). Đoạn cuối là câu kết, luôn được chèn sau cùng.
pub const DEFAULT_FILLER: &str = "Đến đây là hết chữ của tác giả rồi bà con ơi. Chương này bên tác viết ngắn quá trớn, mà hệ thống lại khó tính như mẹ chồng: dưới năm trăm chữ là đè ra khóa nút đăng, nộp đi nộp lại bao nhiêu lần cũng bị văng ngược về. Thêm thắt tình tiết của người ta thì mang tội xàm xí, nên thôi, đành ngồi đây kéo ghế buôn chuyện thiên hạ cho máy nó đếm đủ chỉ tiêu cân nặng.

Bác nào đang cắm tai nghe audio thì quẹt tay sang chương sau luôn đi cho nóng ruột. Dưới này tuyệt đối không có plot twist, nhân vật chính chẳng lén nhặt được bí kíp, phản diện cũng chẳng rảnh núp lùm sau mấy con chữ này đâu, lấy cả tháng lương ra bảo đảm.

Còn vị huynh đài nào đang lướt mắt đọc từng câu tới tận đây thì xứng đáng nhận danh hiệu độc giả vàng trong làng nhẫn nại. Tại hạ xin kính cẩn nghiêng mình cúi đầu ba cái. Mà thôi, ba cái nghe hơi xui xẻo lại còn mỏi cổ, cúi tạm một cái lấy thảo, hai cái kia gửi ngân hàng để dành chương sau trả góp.

Kể thêm chút chuyện đời thường cho cái chương suy dinh dưỡng này mau béo tốt: hôm nay trời vẫn trong lành, cà phê vẫn đắng nghét, bàn phím vẫn gõ lách cách, còn con mèo hàng xóm vẫn réo đúng giờ như cài báo thức. Đời vẫn trôi êm ả, chỉ có chương truyện này hơi thiếu cân nên phải cho nạp đạm cấp tốc để kịp giờ lên sàn.

Cái máy quét chữ của nền tảng soi kỹ từng ly từng tí, thiếu đúng một chữ cũng bị bắt bẻ hệt như bà bán rau cân thiếu cọng hành. Nếu cái thuật toán đang trực tiếp đọc dòng này thì xin kính cẩn thưa với anh bot: em xin anh, đếm hộ em xem đủ cân chưa rồi duyệt bài cho em đi ngủ, đừng bắt em ngồi viết văn miêu tả tâm hồn nữa.

Sẵn tiện chúc toàn thể các đồng đạo tu luyện hanh thông, gặp nút thắt thì cắn thuốc là qua, đụng quái thì một chưởng là bay màu, gặp mỹ nhân thì có duyên trăm năm, còn lỡ va phải mấy chương ngắn ngủn cỡ này thì cứ mạnh dạn ấn nút nhảy cóc, chẳng ai nỡ phán xét đâu.

Sẽ có người thắc mắc sao không gộp đại chương này với chương sau cho rảnh nợ. Gộp thì sướng một phút bốc đồng, nhưng số chương lệch với bản gốc một phát là sau này sửa lỗi, soát chính tả nó rối như một nồi canh hẹ. Thà để dịch giả ngồi gõ mấy câu trời ơi đất hỡi này còn hơn để cả bộ truyện dẫm chân lên nhau.

Dòng này vẫn là nước lã pha loãng cho đủ dung tích, đọc lướt qua hay bỏ qua luôn cũng chẳng mất miếng kinh nghiệm nào. Ai lỡ đọc kỹ đến đây thì coi như tranh thủ nghỉ mắt, uống ngụm nước rồi chuẩn bị tinh thần qua chương kế.

Sắp chạm mốc rồi, ráng vài nhịp nữa thôi. Xin hứa chương nào tác giả viết đàng hoàng, dài ngoằng thì sẽ không có màn độc thoại tấu hài câu giờ này đâu, chỉ có nguyên chất truyện xịn xò không pha nửa giọt nước.

Máy vừa báo đủ cân rồi. Cảm ơn sự kiên nhẫn phi thường của quý vị, lật sang chương sau lẹ lẹ nào!";

/// Cách bù chương ngắn khi chốt chương. Mặc định tắt; nút thủ công dùng văn mẫu khi đang tắt.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaddingMode {
    #[default]
    Off,
    /// Chèn văn mẫu.
    Filler,
    /// Lặp lại các đoạn của chính chương (bỏ dòng tên chương).
    Repeat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PaddingSettings {
    pub mode: PaddingMode,
    pub threshold: usize,
    pub filler: String,
}

impl Default for PaddingSettings {
    fn default() -> Self {
        PaddingSettings { mode: PaddingMode::Off, threshold: DEFAULT_THRESHOLD, filler: DEFAULT_FILLER.to_string() }
    }
}

fn is_marker(line: &str) -> bool {
    line.trim() == PADDING_MARKER
}

/// Phần bản dịch trước dòng ngăn (cả văn bản khi chưa chèn).
pub fn translation_part(text: &str) -> &str {
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        if is_marker(line) {
            return &text[..offset];
        }
        offset += line.len();
    }
    text
}

/// Vị trí các dòng không rỗng (= đoạn) trước dòng ngăn — dùng khi ghép đoạn bản dịch với đoạn raw.
pub fn content_lines(lines: &[String]) -> Vec<usize> {
    lines
        .iter()
        .take_while(|line| !is_marker(line))
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, _)| index)
        .collect()
}

/// Số chữ (cụm chữ / số liền nhau) trong một đoạn văn.
pub fn count_words(text: &str) -> usize {
    let mut words = 0;
    let mut inside = false;
    for c in text.chars() {
        let letter = c.is_alphanumeric();
        if letter && !inside {
            words += 1;
        }
        inside = letter;
    }
    words
}

/// Số chữ của chương: không tính dòng tên chương (dòng không rỗng đầu tiên) và phần chèn.
pub fn chapter_words(text: &str) -> usize {
    translation_part(text).lines().filter(|line| !line.trim().is_empty()).skip(1).map(count_words).sum()
}

/// Số chữ nền tảng đếm khi đăng: cả phần chèn, chỉ không tính dòng tên chương (dòng ngăn không có chữ).
pub fn published_words(text: &str) -> usize {
    text.lines().filter(|line| !line.trim().is_empty()).skip(1).map(count_words).sum()
}

fn paragraphs(text: &str) -> Vec<String> {
    text.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_string).collect()
}

/// Lấy lần lượt (vòng lại nếu hết) các đoạn của `pool` tới khi `have + đã lấy + reserve` đủ `threshold`.
fn take_until(pool: &[String], have: usize, reserve: usize, threshold: usize) -> Vec<String> {
    let mut taken = Vec::new();
    let mut total = have + reserve;
    if pool.iter().all(|p| count_words(p) == 0) {
        return taken;
    }
    for paragraph in pool.iter().cycle() {
        if total >= threshold {
            break;
        }
        total += count_words(paragraph);
        taken.push(paragraph.clone());
    }
    taken
}

/// Chương sau khi bù cho đủ `settings.threshold` chữ bằng `mode` (phần chèn cũ bỏ trước khi tính). None khi không
/// có gì đổi: chương đã đủ mà chưa chèn, chế độ tắt, hoặc không có gì để chèn.
pub fn pad_chapter(text: &str, mode: PaddingMode, settings: &PaddingSettings) -> Option<String> {
    let base = translation_part(text).trim_end();
    let had_padding = base.len() != text.trim_end().len();
    let words = chapter_words(base);
    let unpadded = || had_padding.then(|| format!("{base}\n"));
    if words >= settings.threshold || mode == PaddingMode::Off {
        return unpadded();
    }
    let added = match mode {
        PaddingMode::Off => unreachable!(),
        PaddingMode::Filler => {
            let mut pool = paragraphs(&settings.filler);
            // Đoạn cuối văn mẫu là câu kết: luôn chèn sau cùng, không lặp.
            let closing = (pool.len() > 1).then(|| pool.pop()).flatten();
            let reserve = closing.as_deref().map_or(0, count_words);
            // Thiếu ít chữ (vd 497/500): riêng câu kết đã đủ, không lấy đoạn nào — vẫn phải chèn câu kết.
            let mut taken = take_until(&pool, words, reserve, settings.threshold);
            taken.extend(closing);
            if taken.is_empty() {
                return unpadded();
            }
            taken
        }
        PaddingMode::Repeat => {
            let pool: Vec<String> = paragraphs(base).into_iter().skip(1).collect();
            let taken = take_until(&pool, words, 0, settings.threshold);
            if taken.is_empty() {
                return unpadded();
            }
            taken
        }
    };
    Some(format!("{base}\n\n{PADDING_MARKER}\n\n{}\n\n{PADDING_MARKER}\n", added.join("\n\n")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(threshold: usize, filler: &str) -> PaddingSettings {
        PaddingSettings { mode: PaddingMode::Filler, threshold, filler: filler.into() }
    }

    #[test]
    fn dem_chu_bo_ten_chuong_va_phan_chen() {
        let text = format!("Chương 1: Mở đầu\n\nHôm nay đi chơi.\n\nTrời đẹp, 3 người.\n\n{PADDING_MARKER}\n\nVăn mẫu dài dòng.\n");
        assert_eq!(chapter_words(&text), 8);
        assert_eq!(count_words("Lục Đình Vân—“Đi!” 2026"), 5);
        assert_eq!(translation_part(&text), "Chương 1: Mở đầu\n\nHôm nay đi chơi.\n\nTrời đẹp, 3 người.\n\n");
    }

    #[test]
    fn chen_tung_doan_van_mau_toi_khi_du_roi_them_cau_ket() {
        let filler = "một hai ba bốn năm\n\nsáu bảy tám chín mười\n\nmười một mười hai\n\nHết.";
        let text = "Chương 1\n\nTa đi.\n";
        let padded = pad_chapter(text, PaddingMode::Filler, &settings(10, filler)).unwrap();
        assert_eq!(padded, format!("Chương 1\n\nTa đi.\n\n{PADDING_MARKER}\n\nmột hai ba bốn năm\n\nsáu bảy tám chín mười\n\nHết.\n\n{PADDING_MARKER}\n"));
        assert!(chapter_words(&padded) == 2, "đếm chữ chương không tính phần chèn");
        assert_eq!(published_words(&padded), 13, "nền tảng đếm cả phần chèn");
        // Chèn lại (đổi văn mẫu) thì thay phần chèn cũ, không chèn chồng.
        let again = pad_chapter(&padded, PaddingMode::Filler, &settings(10, "a b c d e f g h\n\nKết.")).unwrap();
        assert_eq!(again, format!("Chương 1\n\nTa đi.\n\n{PADDING_MARKER}\n\na b c d e f g h\n\nKết.\n\n{PADDING_MARKER}\n"));
        // Văn mẫu ngắn hơn ngưỡng thì vòng lại từ đầu.
        let looped = pad_chapter(text, PaddingMode::Filler, &settings(12, "a b c\n\nKết.")).unwrap();
        assert_eq!(looped.matches("a b c").count(), 3);
    }

    #[test]
    fn lap_noi_dung_chuong_va_chuong_du_chu_thi_go_phan_chen() {
        let text = "Chương 1\n\nmột hai ba\n\nbốn năm\n";
        let padded = pad_chapter(text, PaddingMode::Repeat, &settings(9, "")).unwrap();
        assert_eq!(padded, format!("Chương 1\n\nmột hai ba\n\nbốn năm\n\n{PADDING_MARKER}\n\nmột hai ba\n\nbốn năm\n\n{PADDING_MARKER}\n"));
        assert_eq!(pad_chapter(text, PaddingMode::Repeat, &settings(5, "")), None, "đủ chữ thì không đổi");
        assert_eq!(pad_chapter(text, PaddingMode::Off, &settings(9, "")), None);
        // Sửa tay cho đủ chữ rồi bấm lại: gỡ phần chèn.
        let longer = format!("Chương 1\n\nmột hai ba bốn năm sáu bảy tám chín\n\n{PADDING_MARKER}\n\nx\n");
        assert_eq!(pad_chapter(&longer, PaddingMode::Filler, &settings(9, "y")), Some("Chương 1\n\nmột hai ba bốn năm sáu bảy tám chín\n".into()));
    }

    #[test]
    fn thieu_vai_chu_thi_chen_rieng_cau_ket() {
        // 497/500 chữ: câu kết 5 chữ đã đủ — trước đây không lấy đoạn nào rồi bỏ luôn, báo "đã đủ chữ".
        let body = vec!["chữ"; 497].join(" ");
        let text = format!("Chương 1

{body}
");
        let padded = pad_chapter(&text, PaddingMode::Filler, &settings(500, "đoạn một hai

một hai ba bốn năm")).unwrap();
        assert_eq!(padded, format!("Chương 1

{body}

{PADDING_MARKER}

một hai ba bốn năm

{PADDING_MARKER}
"));
        assert_eq!(published_words(&padded), 502);
    }

    #[test]
    fn dong_noi_dung_dung_o_dong_ngan() {
        let lines: Vec<String> = format!("Tên\n\nĐoạn một.\n{PADDING_MARKER}\nchèn\n").split_inclusive('\n').map(str::to_string).collect();
        assert_eq!(content_lines(&lines), vec![0, 2]);
    }

    #[test]
    fn van_mau_mac_dinh_du_500_chu() {
        let pool = paragraphs(DEFAULT_FILLER);
        assert_eq!(pool.len(), 10);
        let padded = pad_chapter("Chương 74\n\nA, vậy mà không bị khóa.\n", PaddingMode::Filler, &PaddingSettings::default()).unwrap();
        let added: usize = paragraphs(&padded[padded.find(PADDING_MARKER).unwrap()..]).iter().map(|p| count_words(p)).sum();
        assert!((500..600).contains(&(added + 6)), "vượt ngưỡng một chút: {}", added + 6);
    }
}
