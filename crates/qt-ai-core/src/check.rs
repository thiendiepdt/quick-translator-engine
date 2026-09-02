//! Port `checkAiTranslationViolations` của qt-web. Rule là regex JS lưu trong story.json;
//! dịch sang cú pháp fancy-regex trước khi compile.

use crate::paragraphs::split_lines;
use crate::story::CheckRule;
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

/// Copy nguyên văn `CHECK_RULES` của qt-web (pattern.source, flags, message) — golden test so với
/// `DEFAULT_AI_CHECK_RULES` nên gõ sai một ký tự là đỏ.
pub const DEFAULT_RULES: &[(&str, &str, &str)] = &[
    (r"[，。、；：？！]", "", "Dấu câu tiếng Trung còn sót → dùng dấu câu thường"),
    (r#"(^|[.!?]\s+|[【(])["“']?(?:but|and|so|the|in|on|at|from|with)\b"#, "i", "Từ nối tiếng Anh lọt vào bản dịch → dịch sang tiếng Việt hoặc chỉ giữ khi có căn cứ"),
    (r"(?<!\p{L})(?:vợ|chồng)(?!\p{L})", "iu", "Dùng vợ/chồng → thay bằng thê tử/phu quân"),
    (r"\banh ấy\b|\banh ta\b|\bcô ấy\b|\bchị ấy\b", "i", "Đại từ sai → dùng hắn/nàng"),
    (r#"(^|[“"']|,\s+)(?:mình|tôi)(?:\s|[,.!?…])"#, "i", "Dùng mình/tôi làm đại từ → thay bằng ta trong style mặc định"),
    (r"Miêu Ảnh Vô Tông", "", "Sai âm tên riêng → dùng Miêu Ảnh Vô Tung"),
    (r"một tấc vuông", "", "方寸 là không gian hệ thống → dùng Phương Thốn"),
    (r"tinh thần đại chấn", "", "精神大振 → dùng tinh thần phấn chấn hẳn lên"),
    (r"mơ hồ nghiệm ra|mùi vị không bình thường", "", "品出意味 → dùng nhận ra/nhận thấy điều bất thường"),
    (r"bình loạn bắt sống", "", "平叛生擒 → dùng dẹp loạn, bắt sống"),
    (r"không có rèm che chuyên biệt", "", "四面无帷 → dùng không rèm che bốn phía"),
    (r"nóng bóng", "", "Lỗi chính tả → nóng bỏng"),
    (r"ta vất vả một chút", "", "我辛苦点 → dùng ta chịu khó một chút"),
    (r"đẳng tước vị quân công", "", "二十等军功爵 → dùng hai mươi bậc tước quân công"),
    (r"vệt đỏ đắc ý", "", "一抹得意的红晕 → dùng vệt ửng đỏ vì đắc ý"),
    (r"toàn bộ người nghênh đón có mặt đều", "", "Tránh chồng chủ thể/lượng từ"),
    (r"đã thưởng Minh chủ", "", "感谢 X 打赏的盟主 → dùng cảm ơn minh chủ X đã thưởng/ủng hộ"),
    (r"não hải", "", "não hải → đầu óc / tâm trí"),
    (r"(?<!\p{L})(?:Hừm|Ừm)(?!\p{L})", "iu", "Hừm/Ừm → Ân"),
    (r"Ơ\s*[?!,.…]", "", "Thán từ Ơ → dùng A trong bối cảnh cổ đại/huyền huyễn"),
    (r"\bthập phần\b", "", "thập phần → vô cùng / hết sức"),
    (r"\bsong doanh\b", "", "song doanh → đôi bên cùng có lợi"),
    (r"còn đừng nói", "i", "还别说 → Mà phải nói / Không ngờ thật"),
    (r"phụ thân (ở|vào|lên|trong)", "", "附身 → nương thân/bám vào"),
    (r"nhận dạng", "", "nhận dạng → kiểm trắc"),
    (r"kho tàng|kho báu", "", "kho tàng/báu → bảo khố"),
    (r"xao động", "", "xao động → rung động"),
    (r"phát xạ", "", "phát xạ → phóng ra"),
    (r"thích dụng", "", "thích dụng → áp dụng"),
    (r"thúc động", "", "thúc động → thôi động"),
    (r"tiền xa", "", "tiền xa → vết xe đổ"),
    (r"lãnh tình", "", "lãnh tình → cảm kích"),
    (r"đợi lát nữa", "", "đợi lát nữa → chờ một hồi"),
    (r"đại động can qua", "", "đại động can qua → làm to chuyện"),
    (r"nước thu\b", "", "nước thu → thu thủy"),
    (r"là tính là", "", "là tính là → xem như"),
    (r"\bthê tử danh nghĩa\b", "", "Sai vị trí → trên danh nghĩa thê tử"),
    (r"(?<!\p{L})đặc ý(?!\p{L})", "iu", "đặc ý → cố ý"),
    (r"\bvô ý trung\b", "", "vô ý trung → trong lúc vô tình"),
    (r"(?<!\p{L})bi thê(?!\p{L})", "iu", "bi thê → bi thương"),
    (r"(?<!\p{L})u thê(?!\p{L})", "iu", "u thê → u sầu"),
    (r"\bnhức óc\b", "", "nhức óc → đau đầu"),
    (r"(?<!\p{L})thôi thì(?!\p{L})", "iu", "thôi thì → vậy thì / đã vậy"),
    (r"(?<!\p{L})vô ngữ(?!\p{L})", "iu", "vô ngữ → bó tay"),
    (r"(?<!\p{L})địch phương(?!\p{L})", "iu", "địch phương → quân địch"),
    (r"\bhữu phương\b|\bhữu quân\b", "", "hữu phương/quân → phe bạn"),
    (r"quả thực đúng là", "", "quả thực đúng là → chọn quả thực hoặc đúng là"),
    (r"cư nhiên", "", "cư nhiên → lại / dám / không ngờ"),
    (r"\bkhấp huyết\b", "i", "泣血 → nhuộm máu / đẫm máu"),
    (r"\bma diệt\b", "i", "抹杀 → xóa sổ / mạt sát"),
    (r"lãnh di[êễ]m", "i", "冷艳 → lạnh lùng sắc sảo / lạnh lùng kiêu sa"),
    (r"\bthị phạm\b", "i", "示范 → làm mẫu"),
    (r"\bchồng cộng\b", "i", "叠加 → chồng lên nhau / kết hợp"),
    (r"nửa xẻ", "i", "衣衫半解 → y phục bán khai / xiêm y cởi dở"),
    (r"vỏ dao|rút dao|thanh dao\b", "i", "刀 là đao → vỏ đao / rút đao / thanh đao"),
    (r"bom khói", "i", "烟雾弹 trong bối cảnh cổ → màn khói / hỏa mù"),
    (r"đông cứng thành", "i", "凝成 → ngưng tụ thành"),
    (r"phụ lòng tạo hóa", "i", "暴殄天物 → phí phạm của quý"),
    (r"nhìn theo bụi", "i", "望尘莫及 → không sao theo kịp / tự thẹn không bằng"),
    (r"đẹp đến nghẹt thở", "i", "惊心动魄 → đẹp đến kinh tâm động phách"),
    (r"ngón.{0,20}mảnh khảnh", "i", "mảnh khảnh chỉ tả người → ngón tay dùng thon / thon dài"),
    (r"trời sinh [A-Z]", "", "Danh xưng lai nửa Việt nửa Hán → dùng Hán-Việt cả cụm (Thiên Sinh ...)"),
    (r"…", "", "Còn ký tự … → chuẩn hóa thành dấu chấm ASCII, giữ số lượng (… → ..., …… → ......)"),
];

/// Rule cứng chạy trong mọi trường hợp — sót Hán tự là lỗi tuyệt đối.
const MANDATORY_RULES: &[(&str, &str, &str)] = &[(r"\p{Script=Han}", "u", "CJK còn sót (chưa dịch hết!)")];

pub fn default_rules_as_check_rules() -> Vec<CheckRule> {
    DEFAULT_RULES
        .iter()
        .map(|(pattern, flags, message)| CheckRule {
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

pub fn check_violations(text: &str, configured: &[CheckRule]) -> Vec<Violation> {
    let mut rules: Vec<CompiledRule> = if configured.is_empty() {
        DEFAULT_RULES.iter().filter_map(|(p, f, m)| compile(p, f, m)).collect()
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
