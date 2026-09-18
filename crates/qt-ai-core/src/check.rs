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
    (r#"(^|[“"']|,\s+)(?:mình|tôi)(?:\s|[,.!?…])"#, "i", "Dùng mình/tôi làm đại từ → thay bằng ta trong style mặc định", Some("ancient")),
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
    (r"(?<!\p{L})(?:Hừm|Ừm)(?!\p{L})", "iu", "Hừm/Ừm → Ân", Some("ancient")),
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
    (r"(?<!\p{L})(?:ngươi|nàng|bọn ta|các ngươi)(?!\p{L})", "iu", "Xưng hô cổ trang trong truyện hiện đại → hắn/cô trong lời kể, tôi/anh/em trong thoại", Some("modern")),
    (r"(?<!\p{L})(?:thê tử|phu quân|lang quân|phụ thân|mẫu thân)(?!\p{L})", "iu", "Từ gia đình cổ trang → vợ/chồng/bố/mẹ", Some("modern")),
    (r"(?<!\p{L})tổng tài(?!\p{L})", "iu", "tổng tài → tổng giám đốc", Some("modern")),
    (r"nói đạo", "i", "说道 → nói / đáp, không \"nói đạo\"", Some("modern")),
    (r#"^[^"“”]*(?<!\p{L})tôi(?!\p{L})[^"“”]*$"#, "iu", "Lời kể ngôi một dùng tôi → ta (tôi chỉ trong thoại theo quan hệ)", Some("modern")),
];

type RuleSpec = (&'static str, &'static str, &'static str, Option<&'static str>);

/// Rule trung lập + rule gắn đúng bối cảnh, giữ thứ tự khai báo — như `rulesForSetting` của web.
fn rules_for(setting: GenreSetting) -> impl Iterator<Item = &'static RuleSpec> {
    DEFAULT_RULES.iter().filter(move |(_, _, _, tag)| tag.is_none_or(|t| t == setting.as_str()))
}

/// Rule cứng chạy trong mọi trường hợp — sót Hán tự là lỗi tuyệt đối.
const MANDATORY_RULES: &[(&str, &str, &str)] = &[(r"\p{Script=Han}", "u", "CJK còn sót (chưa dịch hết!)")];

pub fn default_rules_as_check_rules(setting: GenreSetting) -> Vec<CheckRule> {
    rules_for(setting)
        .map(|(pattern, flags, message, _)| CheckRule {
            pattern: pattern.to_string(),
            flags: (!flags.is_empty()).then(|| flags.to_string()),
            message: message.to_string(),
        })
        .collect()
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
    let mut violations = Vec::new();
    for (index, line) in split_lines(text).enumerate() {
        for rule in &rules {
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
