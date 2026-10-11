//! Port `checkAiTranslationViolations` của qt-web. Rule là regex JS lưu trong story.json;
//! dịch sang cú pháp fancy-regex trước khi compile.

use crate::paragraphs::split_lines;
use crate::story::{CheckRule, GenreSetting};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Violation {
    pub line: usize,
    pub message: String,
    pub text: String,
}

/// JS `\b` với ký tự từ chỉ là [A-Za-z0-9_] — viết tường minh bằng lookaround.
const ASCII_WORD_BOUNDARY: &str =
    "(?:(?<![A-Za-z0-9_])(?=[A-Za-z0-9_])|(?<=[A-Za-z0-9_])(?![A-Za-z0-9_]))";

/// Dịch regex JS (source + flags) sang fancy-regex. Chỉ xử các construct web đang dùng.
pub fn js_regex_to_rust(pattern: &str, flags: &str) -> String {
    let mut out = String::new();
    if flags.contains('i') {
        out.push_str("(?i)");
    }
    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('b') => out.push_str(ASCII_WORD_BOUNDARY),
            Some('d') => out.push_str("[0-9]"),
            Some('w') => out.push_str("[A-Za-z0-9_]"),
            Some('p') => {
                let mut name = String::new();
                if chars.peek() == Some(&'{') {
                    chars.next();
                    for inner in chars.by_ref() {
                        if inner == '}' {
                            break;
                        }
                        name.push(inner);
                    }
                }
                let mapped = match name.as_str() {
                    "Script=Han" | "sc=Han" => "Han",
                    other => other,
                };
                out.push_str(&format!("\\p{{{mapped}}}"));
            }
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Copy nguyên văn `CHECK_RULES` của qt-web (pattern.source, flags, message, bối cảnh) — golden test
/// so với `defaultAiCheckRules(setting)` nên gõ sai một ký tự là đỏ. `None` = trung lập, chạy mọi truyện.
pub const DEFAULT_RULES: &[(&str, &str, &str, Option<&str>)] = &[
    (r"[，。、；：？！]", "", "Dấu câu tiếng Trung còn sót → dùng dấu câu thường", None),
    (r#"(^|[.!?]\s+|[【(])["“']?(?:but|and|the|in|on|at|from|with)\b"#, "i", "Từ nối tiếng Anh lọt vào bản dịch → dịch sang tiếng Việt hoặc chỉ giữ khi có căn cứ", None),
    (r"(?<!\p{L})(?:vợ|chồng)(?!\p{L})", "iu", "Dùng vợ/chồng → thay bằng thê tử/phu quân", Some("ancient")),
    (r"(?<!\p{L})(?:đàn ông|đàn bà|phụ nữ)(?!\p{L})", "iu", "Từ chỉ người đời thường trong bối cảnh cổ → nam nhân/nữ nhân (nam tử/nữ tử)", Some("ancient")),
    (r"\banh ấy\b|\banh ta\b|\bcô ấy\b|\bchị ấy\b", "i", "Đại từ sai → dùng hắn/nàng", Some("ancient")),
    (r"(?<!\p{L})(?:ông ta|ông ấy|bà ta|bà ấy)(?!\p{L})", "iu", "ông ta/bà ta cho nhân vật lớn tuổi trong bối cảnh cổ → hắn/lão, nàng/bà", Some("ancient")),
    (r#"(^|[“"']|,\s+)(?:mình|tôi)(?:\s|[,.!?…])"#, "i", "Dùng mình/tôi làm đại từ → ta, hoặc tự xưng theo thân phận (tiểu nhân/nô tỳ/thuộc hạ/đệ tử/con)", Some("ancient")),
    (r"tinh thần đại chấn", "", "精神大振 → dùng tinh thần phấn chấn hẳn lên", None),
    (r"mơ hồ nghiệm ra|mùi vị không bình thường", "", "品出意味 → dùng nhận ra/nhận thấy điều bất thường", None),
    (r"bình loạn bắt sống", "", "平叛生擒 → dùng dẹp loạn, bắt sống", None),
    (r"không có rèm che chuyên biệt", "", "四面无帷 → dùng không rèm che bốn phía", None),
    (r"nóng bóng", "", "Lỗi chính tả → nóng bỏng", None),
    (r"ta vất vả một chút", "", "我辛苦点 → dùng ta chịu khó một chút", None),
    (r"đẳng tước vị quân công", "", "二十等军功爵 → dùng hai mươi bậc tước quân công", None),
    (r"vệt đỏ đắc ý", "", "一抹得意的红晕 → dùng vệt ửng đỏ vì đắc ý", None),
    (r"toàn bộ người nghênh đón có mặt đều", "", "Tránh chồng chủ thể/lượng từ", None),
    (r"đã thưởng Minh chủ", "", "感谢 X 打赏的盟主 → dùng cảm ơn minh chủ X đã thưởng/ủng hộ", None),
    (r"não hải", "", "não hải → đầu óc / tâm trí", None),
    (r"(?<!\p{L})kiểm trắc(?!\p{L})", "iu", "检测 → phát hiện / quét thấy / kiểm tra, không dùng kiểm trắc", None),
    (r"(?<!\p{L})[Cc]hiêu lãm(?!\p{L})", "u", "招揽 → chiêu mộ / lôi kéo / kéo khách theo cảnh, không dùng chiêu lãm", None),
    (r"không tiền khoáng hậu", "iu", "空前绝后 → vô tiền khoáng hậu, không dùng không tiền khoáng hậu", None),
    (r"(?<!\p{L})liếm chó(?!\p{L})", "iu", "舔狗 → liếm cẩu, không dùng liếm chó", None),
    (r"Ơ\s*[?!,.…]", "", "Thán từ Ơ → dùng A trong bối cảnh cổ đại/huyền huyễn", Some("ancient")),
    (r"\bthập phần\b", "", "thập phần → vô cùng / hết sức", None),
    (r"(?<!\p{L})đắc được(?!\p{L})", "iu", "得到 / 深得 → được / nhận được / học được, không \"đắc được\"", None),
    (r"(?<!\p{L})đắc thủ(?!\p{L})", "iu", "得手 → thành công / ra tay trót lọt, không \"đắc thủ\"", None),
    (r"(?<!\p{L})thu hoạch được(?!\p{L})", "iu", "有收获 → thu được / tìm được gì, không \"thu hoạch được\"", None),
    (r"(?<!\p{L})có biệt(?!\p{L})(?!\s*(?:hiệu|xưng|danh))", "iu", "有别 → hữu biệt / khác biệt, không \"có biệt\"", None),
    (r"\bsong doanh\b", "", "song doanh → đôi bên cùng có lợi", None),
    (r"còn đừng nói", "i", "还别说 → Mà phải nói / Không ngờ thật", None),
    (r"phụ thân (ở|vào|lên|trong)", "", "附身 → nương thân/bám vào", None),
    (r"kho tàng|kho báu", "", "kho tàng/báu → bảo khố", Some("ancient")),
    (r"phát xạ", "", "phát xạ → phóng ra", None),
    (r"thích dụng", "", "thích dụng → áp dụng", None),
    (r"thúc động", "", "thúc động → thôi động", None),
    (r"tiền xa", "", "tiền xa → vết xe đổ", None),
    (r"lãnh tình", "", "lãnh tình → cảm kích", None),
    (r"đại động can qua", "", "đại động can qua → làm to chuyện", None),
    (r"nước thu\b", "", "nước thu → thu thủy", None),
    (r"là tính là", "", "là tính là → xem như", None),
    (r"\bthê tử danh nghĩa\b", "", "Sai vị trí → trên danh nghĩa thê tử", Some("ancient")),
    (r"(?<!\p{L})đặc ý(?!\p{L})", "iu", "đặc ý → cố ý", None),
    (r"\bvô ý trung\b", "", "vô ý trung → trong lúc vô tình", None),
    (r"(?<!\p{L})bi thê(?!\p{L})", "iu", "bi thê → bi thương", None),
    (r"(?<!\p{L})u thê(?!\p{L})", "iu", "u thê → u sầu", None),
    (r"(?<!\p{L})vô ngữ(?!\p{L})", "iu", "vô ngữ → bó tay", None),
    (r"(?<!\p{L})địch phương(?!\p{L})", "iu", "địch phương → quân địch", None),
    (r"\bhữu phương\b|\bhữu quân\b", "", "hữu phương/quân → phe bạn", None),
    (r"quả thực đúng là", "", "quả thực đúng là → chọn quả thực hoặc đúng là", None),
    (r"cư nhiên", "", "cư nhiên → lại / dám / không ngờ", None),
    (r"\bkhấp huyết\b", "i", "泣血 → nhuộm máu / đẫm máu", None),
    (r"\bma diệt\b", "i", "抹杀 → xóa sổ / mạt sát", None),
    (r"lãnh di[êễ]m", "i", "冷艳 → lạnh lùng sắc sảo / lạnh lùng kiêu sa", None),
    (r"\bthị phạm\b", "i", "示范 → làm mẫu", None),
    (r"\bchồng cộng\b", "i", "叠加 → chồng lên nhau / kết hợp", None),
    (r"nửa xẻ", "i", "衣衫半解 → y phục bán khai / xiêm y cởi dở", None),
    (r"vỏ dao|rút dao|thanh dao\b", "i", "刀 là đao → vỏ đao / rút đao / thanh đao", Some("ancient")),
    (r"bom khói", "i", "烟雾弹 trong bối cảnh cổ → màn khói / hỏa mù", Some("ancient")),
    (r"đông cứng thành", "i", "凝成 → ngưng tụ thành", None),
    (r"phụ lòng tạo hóa", "i", "暴殄天物 → phí phạm của quý", None),
    (r"nhìn theo bụi", "i", "望尘莫及 → không sao theo kịp / tự thẹn không bằng", None),
    (r"đẹp đến nghẹt thở", "i", "惊心动魄 → đẹp đến kinh tâm động phách", None),
    (r"ngón.{0,20}mảnh khảnh", "i", "mảnh khảnh chỉ tả người → ngón tay dùng thon / thon dài", None),
    (r"trời sinh [A-Z]", "", "Danh xưng lai nửa Việt nửa Hán → dùng Hán-Việt cả cụm (Thiên Sinh ...)", None),
    (r"…", "", "Còn ký tự … → chuẩn hóa thành dấu chấm ASCII, giữ số lượng (… → ..., …… → ......)", None),
    (r"(?<!\p{L})(?<!con )(?<!cô )(?:ngươi|nàng|bọn ta|các ngươi)(?!\p{L})(?!\s(?:thơ|dâu|tiên)(?!\p{L}))", "iu", "Xưng hô cổ trang trong truyện hiện đại → hắn/cô trong lời kể, tôi/anh/em trong thoại", Some("modern")),
    (r"(?<!\p{L})(?:thê tử|phu quân|lang quân|phụ thân|mẫu thân)(?!\p{L})", "iu", "Từ gia đình cổ trang → vợ/chồng/bố/mẹ", Some("modern")),
    (r"(?<!\p{L})tổng tài(?!\p{L})", "iu", "tổng tài → tổng giám đốc", Some("modern")),
    (r"nói đạo", "i", "说道 → nói / đáp, không \"nói đạo\"", Some("modern")),
];

/// Copy `ADVISORY_CHECK_RULES` của qt-web: rule chỉ cảnh báo — không đẩy chương vào vòng soát,
/// accept ghi vào warnings. Dành cho mẫu hay sai nhưng cũng có lúc đúng (tên riêng trùng chữ…).
pub const ADVISORY_RULES: &[(&str, &str, &str, Option<&str>)] = &[
    (r#"(?:^|[“"'(—]|[.!?…]\s+)Ân(?=\s*[,.!?~…])"#, "u", "嗯 → Ừ/Ừm/Ưm/Hả theo bảng Thán từ, không phiên âm Ân", None),
    (r#"(?<!\p{L})["“]?[Ââ]n["”]? một tiếng"#, "u", "ân một tiếng → ừm một tiếng", None),
    (r"(?<!\p{L})bản thân (?:hắn|nàng|cô|y|lão|gã|nó)(?!\p{L})", "iu", "自己 → hắn/nàng/cô hoặc lược, không ghép bản thân + đại từ", None),
    (r#"^(?!Chương \d)(?![【『])(?!\[)(?![^:："“”]{1,40}[:：])[^"“”『』]*(?<!\p{L})tôi(?!\p{L})[^"“”『』]*$"#, "iu", "Người kể ngôi một xưng tôi → ta (truyện kể ngôi ba thì bỏ qua)", Some("modern")),
    (r#"[“"](?:Thiếu gia|Chủ nhân|Lão gia|Đại nhân|Sư phụ|Sư tôn|Điện hạ|Bệ hạ|Tiền bối|Phụ thân|Mẫu thân|Phụ hoàng|Mẫu hậu)[,!.…][^”"]{0,160}(?<!\p{L})(?:các )?ngươi(?!\p{L})"#, "iu", "Gọi bề trên bằng danh xưng mà xưng ngươi → ngài/người hoặc danh xưng (mục Vai vế trong thoại)", Some("ancient")),
    (r#"[“"](?:Thiếu gia|Chủ nhân|Lão gia|Đại nhân|Sư phụ|Sư tôn|Điện hạ|Bệ hạ|Tiền bối|Phụ thân|Mẫu thân|Phụ hoàng|Mẫu hậu)[,!.…][^”"]{0,160}(?<!chúng )(?<!bọn )(?<!người )(?<!\p{L})ta(?!\p{L})"#, "iu", "Gọi bề trên bằng danh xưng mà tự xưng ta → tiểu nhân/nô tỳ/thuộc hạ/đệ tử/vãn bối/con (mục Vai vế trong thoại)", Some("ancient")),
    (r"(?<!\p{L})(?:\d[\d.,]*|một|hai|ba|bốn|năm|sáu|bảy|tám|chín|mười|mươi|trăm|nghìn|ngàn|vạn|mấy|vài)\s*ức(?!\p{L})(?!\s+(?:vạn|hiếp))", "iu", "亿 → trăm triệu / tỷ (百亿 = 10 tỷ, 千亿 = 100 tỷ), không viết ức", None),
    (r"(?<!\p{L})(?:\d[\d.,]*|một|hai|ba|bốn|năm|sáu|bảy|tám|chín|mười|mấy|vài)\s*vạn(?!\p{L})(?!\s+(?:tuế|vật|nhất|sự|năng|phần))", "iu", "万 trong truyện hiện đại → nghìn / triệu (20万 = 200 nghìn), không viết vạn", Some("modern")),
    (r#"(?:^|[.!?…]\s+|[“"])Bản thân(?!\s+(?:là|hắn|nàng|cô|y|lão|gã|nó)(?!\p{L}))(?!\p{L})"#, "u", "自己 làm chủ ngữ → hắn/nàng/cô hoặc lược, không mở câu bằng Bản thân", None),
    (r#"chiếm (?:hết )?tiện nghi"#, "iu", "占便宜 → lợi dụng / sàm sỡ / bắt nạt theo cảnh, không chiếm tiện nghi", None),
    (r#"tổng không thể"#, "iu", "总不能 → đâu thể / chẳng lẽ, không tổng không thể", None),
];

type RuleSpec = (&'static str, &'static str, &'static str, Option<&'static str>);

/// Rule trung lập + rule gắn đúng bối cảnh, giữ thứ tự khai báo — như `rulesForSetting` của web.
fn rules_for(setting: GenreSetting) -> impl Iterator<Item = &'static RuleSpec> {
    rules_in(DEFAULT_RULES, setting)
}

fn rules_in(list: &'static [RuleSpec], setting: GenreSetting) -> impl Iterator<Item = &'static RuleSpec> {
    list.iter().filter(move |(_, _, _, tag)| tag.is_none_or(|t| t == setting.as_str()))
}

/// Còn chữ Hán (Script=Han — không tính 《》【】「」 vốn là Common). Check coi là lỗi cứng như thiếu đoạn:
/// không bao giờ chốt kèm cảnh báo.
pub fn contains_han(text: &str) -> bool {
    han().is_match(text)
}

fn han() -> &'static regex::Regex {
    static HAN: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    HAN.get_or_init(|| regex::Regex::new(r"\p{Han}").unwrap())
}

/// Bỏ dấu tiếng Việt, giữ hoa/thường và đúng một ký tự ra cho một ký tự vào (đ → d) — vị trí trên chuỗi
/// đã bỏ dấu trùng vị trí trên chuỗi gốc.
fn fold_vietnamese(c: char) -> char {
    const GROUPS: &[(&str, char)] = &[
        ("àáảãạăằắẳẵặâầấẩẫậ", 'a'),
        ("ÀÁẢÃẠĂẰẮẲẴẶÂẦẤẨẪẬ", 'A'),
        ("èéẻẽẹêềếểễệ", 'e'),
        ("ÈÉẺẼẸÊỀẾỂỄỆ", 'E'),
        ("ìíỉĩị", 'i'),
        ("ÌÍỈĨỊ", 'I'),
        ("òóỏõọôồốổỗộơờớởỡợ", 'o'),
        ("ÒÓỎÕỌÔỒỐỔỖỘƠỜỚỞỠỢ", 'O'),
        ("ùúủũụưừứửữự", 'u'),
        ("ÙÚỦŨỤƯỪỨỬỮỰ", 'U'),
        ("ỳýỷỹỵ", 'y'),
        ("ỲÝỶỸỴ", 'Y'),
        ("đ", 'd'),
        ("Đ", 'D'),
    ];
    GROUPS.iter().find(|(group, _)| group.contains(c)).map_or(c, |(_, base)| *base)
}

/// Tên trong bản dịch lệch dấu so với glossary: key có trong raw của đoạn, target không có trong bản dịch
/// của đoạn, nhưng có cụm chỉ khác dấu thanh (段锋 → `Đoàn Phong`, bản dịch viết `Đoạn Phong`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlossaryDrift {
    /// Chỉ số đoạn 0-based.
    pub paragraph: usize,
    pub source: String,
    pub target: String,
    pub found: String,
}

/// Chỉ xét entry ≥2 chữ Hán và target ≥2 tiếng — một tiếng bỏ dấu khớp quá nhiều từ thường. Cụm trùng
/// target của entry khác có mặt trong đoạn raw thì bỏ qua (李贤 Lý Hiền / 李显 Lý Hiển là hai người).
pub fn glossary_drifts(raw: &[String], translated: &[Option<String>], glossary: &crate::story::Glossary) -> Vec<GlossaryDrift> {
    let entries: Vec<(&str, &str)> = glossary
        .iter()
        .filter(|(group, _)| group.as_str() != "addressing" && group.as_str() != "signature_phrases")
        .flat_map(|(_, entries)| entries.iter().map(|(source, target)| (source.trim(), target.trim())))
        .filter(|(source, target)| {
            han().find_iter(source).count() >= 2 && target.split_whitespace().count() >= 2
        })
        .collect();
    let mut drifts = Vec::new();
    for (index, (raw_paragraph, text)) in raw.iter().zip(translated).enumerate() {
        let Some(text) = text else { continue };
        let present: HashSet<&str> =
            entries.iter().filter(|(source, _)| raw_paragraph.contains(source)).map(|(_, target)| *target).collect();
        if present.is_empty() {
            continue;
        }
        let chars: Vec<char> = text.chars().collect();
        let folded: Vec<char> = chars.iter().copied().map(fold_vietnamese).collect();
        for (source, target) in entries.iter().filter(|(source, _)| raw_paragraph.contains(source)) {
            if text.contains(target) {
                continue;
            }
            let needle: Vec<char> = target.chars().map(fold_vietnamese).collect();
            let found = (0..=folded.len().saturating_sub(needle.len()))
                .filter(|&at| folded.len() >= needle.len() && folded[at..at + needle.len()] == needle[..])
                .filter(|&at| at == 0 || !chars[at - 1].is_alphanumeric())
                .filter(|&at| chars.get(at + needle.len()).is_none_or(|c| !c.is_alphanumeric()))
                .map(|at| chars[at..at + needle.len()].iter().collect::<String>())
                .find(|span| !present.contains(span.as_str()));
            if let Some(found) = found {
                drifts.push(GlossaryDrift { paragraph: index, source: source.to_string(), target: target.to_string(), found });
            }
        }
    }
    drifts
}

/// Tiền tố thông báo của lỗi tên theo glossary — soát tách thành mục bắt buộc sửa, không cho giữ nguyên.
pub const GLOSSARY_NAME_ISSUE: &str = "Tên theo glossary:";

/// Gấp để so tên gần đúng: thường hoá, bỏ dấu, `đ` → `d`, `y` → `i` (Hy/Hi, Ty/Ti là hai cách viết cùng âm).
fn fold_loose(word: &str) -> String {
    word.chars().map(fold_vietnamese).flat_map(char::to_lowercase).map(|c| if c == 'y' { 'i' } else { c }).collect()
}

fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            cur.push((prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + usize::from(ca != *cb)));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Tên riêng viết gần đúng glossary nhưng sai hơn dấu thanh (`Mao Thiên Dực` thay `Mao Thiên Dịch`, `Đại Phạm Thiên`
/// thay `Đại Phạn Thiên`). Chỉ xét target ≥3 tiếng viết hoa mọi tiếng; khoá có trong raw của đoạn mà đoạn dịch không có
/// target; có cụm cùng số tiếng, cùng kiểu viết hoa, lệch đúng một tiếng (≤2 chữ cái sau khi gấp) và không trùng target
/// khác. Đo trên 2.405 chương thật: bắt 1,4% chương, gần như toàn lỗi thật — tên ngắn, danh từ chung thì báo nhầm nhiều.
pub fn glossary_near_misses(
    raw: &[String],
    translated: &[Option<String>],
    glossary: &crate::story::Glossary,
) -> Vec<GlossaryDrift> {
    let all: Vec<(&str, &str)> = glossary
        .iter()
        .filter(|(group, _)| group.as_str() != "addressing" && group.as_str() != "signature_phrases")
        .flat_map(|(_, entries)| entries.iter().map(|(source, target)| (source.trim(), target.trim())))
        .collect();
    let known: HashSet<String> =
        all.iter().map(|(_, target)| target.split_whitespace().map(fold_loose).collect::<Vec<_>>().join(" ")).collect();
    let entries: Vec<(&str, &str)> = all
        .iter()
        .copied()
        .filter(|(source, target)| {
            let words: Vec<&str> = target.split_whitespace().collect();
            han().find_iter(source).count() >= 2
                && words.len() >= 3
                && words.iter().all(|w| w.chars().next().is_some_and(char::is_uppercase))
        })
        .collect();
    let mut misses = Vec::new();
    for (index, (raw_paragraph, text)) in raw.iter().zip(translated).enumerate() {
        let Some(text) = text else { continue };
        // Từ và vị trí byte trong đoạn để cắt lại đúng cụm gốc.
        let mut words: Vec<(usize, usize)> = Vec::new();
        let mut start = None;
        for (at, c) in text.char_indices().chain(std::iter::once((text.len(), ' '))) {
            match (start, c.is_alphabetic()) {
                (None, true) => start = Some(at),
                (Some(from), false) => {
                    words.push((from, at));
                    start = None;
                }
                _ => {}
            }
        }
        let folded: Vec<String> = words.iter().map(|&(a, b)| fold_loose(&text[a..b])).collect();
        let folded_text = folded.join(" ");
        for (source, target) in entries.iter().filter(|(source, _)| raw_paragraph.contains(source)) {
            let want: Vec<&str> = target.split_whitespace().collect();
            let want_folded: Vec<String> = want.iter().map(|w| fold_loose(w)).collect();
            if folded_text.contains(&want_folded.join(" ")) {
                continue; // có tên (hoặc chỉ lệch dấu → fix_glossary_drifts lo)
            }
            let n = want.len();
            let found = (0..words.len().saturating_sub(n - 1)).find_map(|at| {
                let span = &words[at..at + n];
                let caps = span.iter().all(|&(a, _)| text[a..].chars().next().is_some_and(char::is_uppercase));
                let span_folded = &folded[at..at + n];
                let diffs: Vec<usize> = span_folded
                    .iter()
                    .zip(&want_folded)
                    .filter(|(got, want)| got != want)
                    .map(|(got, want)| edit_distance(got, want))
                    .collect();
                let near = caps && diffs.len() == 1 && diffs[0] <= 2 && !known.contains(&span_folded.join(" "));
                near.then(|| text[span[0].0..span[n - 1].1].to_string())
            });
            if let Some(found) = found {
                misses.push(GlossaryDrift { paragraph: index, source: source.to_string(), target: target.to_string(), found });
            }
        }
    }
    misses
}

/// Thay cụm lệch dấu bằng đúng target glossary trong từng đoạn (glossary là chuẩn, không cần model soát).
/// Trả các cặp đã sửa `"sai → đúng"`, mỗi cặp một lần, theo thứ tự gặp.
pub fn fix_glossary_drifts(
    raw: &[String],
    translated: &mut [Option<String>],
    glossary: &crate::story::Glossary,
) -> Vec<String> {
    let mut notes: Vec<String> = Vec::new();
    // Một đoạn có thể lệch theo hai kiểu dấu khác nhau — lặp tới khi sạch (giới hạn để không vòng vô tận).
    for _ in 0..5 {
        let drifts = glossary_drifts(raw, translated, glossary);
        if drifts.is_empty() {
            break;
        }
        for drift in drifts {
            if let Some(text) = translated[drift.paragraph].as_mut() {
                *text = text.replace(&drift.found, &drift.target);
                let note = format!("{} → {}", drift.found, drift.target);
                if !notes.contains(&note) {
                    notes.push(note);
                }
            }
        }
    }
    notes
}

/// Rule cứng chạy trong mọi trường hợp — sót Hán tự là lỗi tuyệt đối.
const MANDATORY_RULES: &[(&str, &str, &str)] = &[(r"\p{Script=Han}", "u", "CJK còn sót (chưa dịch hết!)")];

pub fn default_rules_as_check_rules(setting: GenreSetting) -> Vec<CheckRule> {
    rules_for(setting).map(as_check_rule).collect()
}

pub fn advisory_rules_as_check_rules(setting: GenreSetting) -> Vec<CheckRule> {
    rules_in(ADVISORY_RULES, setting).map(as_check_rule).collect()
}

fn as_check_rule((pattern, flags, message, _): &RuleSpec) -> CheckRule {
    CheckRule {
        pattern: pattern.to_string(),
        flags: (!flags.is_empty()).then(|| flags.to_string()),
        message: message.to_string(),
    }
}

/// Vi phạm rule chỉ cảnh báo — luôn là bộ cứng theo bối cảnh, không bị rule riêng của truyện thay.
pub fn advisory_violations(text: &str, setting: GenreSetting) -> Vec<Violation> {
    let rules: Vec<CompiledRule> = rules_in(ADVISORY_RULES, setting).filter_map(|(p, f, m, _)| compile(p, f, m)).collect();
    matches(text, &rules)
}

/// Bẫy đồng âm — chép từ `RAW_PAIR_RULES` của web (golden so khớp): đoạn raw khớp `raw` (cờ `u`) mà không khớp
/// `unless`, và đoạn dịch cùng nhãn khớp `out` → sai nghĩa. `advisory`: chỉ cảnh báo, không đưa vào soát.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawPairRule {
    pub raw: &'static str,
    pub unless: Option<&'static str>,
    pub out: &'static str,
    pub out_flags: &'static str,
    pub message: &'static str,
    pub advisory: bool,
}

const fn pair(raw: &'static str, unless: Option<&'static str>, out: &'static str, message: &'static str) -> RawPairRule {
    RawPairRule { raw, unless, out, out_flags: "iu", message, advisory: false }
}

pub const RAW_PAIR_RULES: &[RawPairRule] = &[
    pair("头颅|脑袋|人头", Some("骷髅|头骨"), "đầu lâu", "头颅/脑袋/人头 là cái đầu, thủ cấp → không dịch đầu lâu (sọ người)"),
    pair("放肆", None, "phóng túng", "放肆 → láo xược / càn rỡ, không phóng túng"),
    pair("翻.{0,2}白眼", None, "trợn tròn", "翻白眼 → trợn trắng mắt / lườm, không trợn tròn"),
    pair("总不能", None, "tổng thể", "总不能 → đâu thể / chẳng lẽ, không tổng thể"),
    pair("交[待代]", None, "lời ăn tiếng nói", "交待/交代 → lời giải thích / câu trả lời, không lời ăn tiếng nói"),
    pair("一个头两个大", None, "hai thứ tóc", "一个头两个大 → đau cả đầu / rối như tơ vò, không một đầu hai thứ tóc"),
    pair("拳力", Some("权力"), "quyền lực", "拳力 → lực quyền / sức đấm, không quyền lực"),
    pair("藏着掖着", None, "giấu đầu lòi đuôi", "藏着掖着 → giấu giấu giếm giếm, không giấu đầu lòi đuôi"),
    pair("(?<![巨鱼矮兽半蛇狼羽魔亚])人族", None, r"(?<!\p{L})người tộc(?!\p{L})", "人族 → Nhân tộc, không dịch người tộc"),
    RawPairRule { advisory: true, ..pair("呵", Some("哼|呵斥"), r"(?<!\p{L})hừ(?!\p{L})", "呵 là cười khẩy / cười nhạt → Ha / Hà / Hề hề, không Hừ") },
];

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RawPairIssue {
    /// Chỉ số 0-based của nhãn [[n]].
    pub paragraph: usize,
    pub message: String,
    pub advisory: bool,
}

struct CompiledPair {
    raw: fancy_regex::Regex,
    unless: Option<fancy_regex::Regex>,
    out: fancy_regex::Regex,
    rule: RawPairRule,
}

fn compiled_pairs() -> &'static [CompiledPair] {
    static PAIRS: std::sync::OnceLock<Vec<CompiledPair>> = std::sync::OnceLock::new();
    PAIRS.get_or_init(|| {
        let build = |pattern: &str, flags: &str| fancy_regex::Regex::new(&js_regex_to_rust(pattern, flags)).ok();
        RAW_PAIR_RULES
            .iter()
            .filter_map(|rule| {
                Some(CompiledPair {
                    raw: build(rule.raw, "u")?,
                    unless: match rule.unless {
                        Some(pattern) => Some(build(pattern, "u")?),
                        None => None,
                    },
                    out: build(rule.out, rule.out_flags)?,
                    rule: *rule,
                })
            })
            .collect()
    })
}

/// Port `checkRawPairViolations`: bẫy đồng âm theo từng cặp đoạn raw / bản dịch.
pub fn raw_pair_issues(raw: &[String], translated: &[Option<String>]) -> Vec<RawPairIssue> {
    let found = |regex: &fancy_regex::Regex, text: &str| regex.is_match(text).unwrap_or(false);
    let mut issues = Vec::new();
    for (paragraph, (source, text)) in raw.iter().zip(translated).enumerate() {
        let Some(text) = text.as_deref().filter(|text| !text.is_empty()) else { continue };
        for pair in compiled_pairs() {
            if found(&pair.raw, source) && !pair.unless.as_ref().is_some_and(|unless| found(unless, source)) && found(&pair.out, text) {
                issues.push(RawPairIssue { paragraph, message: pair.rule.message.to_string(), advisory: pair.rule.advisory });
            }
        }
    }
    issues
}

struct CompiledRule {
    regex: fancy_regex::Regex,
    message: String,
}

fn compile(pattern: &str, flags: &str, message: &str) -> Option<CompiledRule> {
    fancy_regex::Regex::new(&js_regex_to_rust(pattern, flags))
        .ok()
        .map(|regex| CompiledRule { regex, message: message.to_string() })
}

pub fn check_violations(text: &str, configured: &[CheckRule], setting: GenreSetting) -> Vec<Violation> {
    // Rule riêng của truyện > bộ người dùng sửa ở app (file) > bộ cứng lọc theo bối cảnh.
    let mut rules: Vec<CompiledRule> = if configured.is_empty() {
        crate::base::BaseStore::from_env()
            .rules(setting)
            .iter()
            .filter_map(|rule| compile(&rule.pattern, rule.flags.as_deref().unwrap_or(""), &rule.message))
            .collect()
    } else {
        configured
            .iter()
            .filter_map(|rule| compile(&rule.pattern, rule.flags.as_deref().unwrap_or(""), &rule.message))
            .collect()
    };
    let configured_patterns: HashSet<&str> = configured.iter().map(|rule| rule.pattern.as_str()).collect();
    rules.extend(
        MANDATORY_RULES
            .iter()
            .filter(|(pattern, _, _)| !configured_patterns.contains(*pattern))
            .filter_map(|(p, f, m)| compile(p, f, m)),
    );
    matches(text, &rules)
}

fn matches(text: &str, rules: &[CompiledRule]) -> Vec<Violation> {
    let mut violations = Vec::new();
    for (index, line) in split_lines(text).enumerate() {
        for rule in rules {
            if rule.regex.is_match(line).unwrap_or(false) {
                violations.push(Violation {
                    line: index + 1,
                    message: rule.message.clone(),
                    text: line.trim().chars().take(120).collect(),
                });
            }
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bat_dong_tu_han_viet_dan_tro_tu_viet_tha_biet_hieu() {
        let text = "Hắn đắc được chân truyền.\nKhông để hắn đắc thủ.\nKhông thu hoạch được gì.\nNam nữ có biệt.\nCó biệt hiệu Thiết Chưởng, có biệt xưng.";
        let messages: Vec<String> = check_violations(text, &[], GenreSetting::Ancient)
            .into_iter()
            .map(|v| format!("{}:{}", v.line, v.message))
            .collect();
        assert_eq!(
            messages,
            vec![
                "1:得到 / 深得 → được / nhận được / học được, không \"đắc được\"",
                "2:得手 → thành công / ra tay trót lọt, không \"đắc thủ\"",
                "3:有收获 → thu được / tìm được gì, không \"thu hoạch được\"",
                "4:有别 → hữu biệt / khác biệt, không \"có biệt\"",
            ]
        );
    }

    #[test]
    fn bat_ong_ta_ba_ta_trong_boi_canh_co_ke_ca_dau_dong_viet_hoa() {
        let text = "Ông ta vuốt râu.\nBà ta cười lạnh.\nÔng nội hắn đã mất.\nLão bà bà lắc đầu.";
        let lines: Vec<usize> = check_violations(text, &[], GenreSetting::Ancient)
            .into_iter()
            .filter(|v| v.message.starts_with("ông ta/bà ta"))
            .map(|v| v.line)
            .collect();
        assert_eq!(lines, vec![1, 2]);
        assert!(check_violations("Ông ta cười.", &[], GenreSetting::Modern).is_empty());
    }
}
