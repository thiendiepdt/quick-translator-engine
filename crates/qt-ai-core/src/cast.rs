//! Bảng nhân vật của truyện (`cast.json`): giới tính từng nhân vật và cặp xưng hô theo mốc chương.
//! Lớp riêng của qt-ai-core, bọc quanh các hàm port từ qt-web (không sửa hàm port): `story.json` giữ
//! cặp xưng hô học lần đầu, file này giữ sự thật về nhân vật và các lần đổi về sau.

use crate::error::Result;
use crate::prompt::glossary_entry_matches_source;
use crate::story::{addressing_sides, natural_chapter_compare, GenreSetting, StringMap};
use crate::story_fs::{write_atomic, StoryPaths};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Ordering;
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    Male,
    Female,
}

impl Gender {
    pub fn label(self) -> &'static str {
        match self {
            Gender::Male => "nam",
            Gender::Female => "nữ",
        }
    }

    /// Nhận cách model/người dùng hay viết: nam/nữ, male/female.
    pub fn parse(text: &str) -> Option<Gender> {
        match text.trim().to_lowercase().as_str() {
            "nam" | "male" | "m" => Some(Gender::Male),
            "nữ" | "nu" | "female" | "f" => Some(Gender::Female),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntrySource {
    #[default]
    Auto,
    User,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Character {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gender: Option<Gender>,
    #[serde(default)]
    pub source: EntrySource,
    /// Chương khai giới (rỗng khi người dùng nhập tay hoặc quét bù).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub chapter: String,
    /// Các chương khai giới ngược lại — không tự đổi, để người dùng xét.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disputed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressChange {
    /// Chương khai thay đổi; cặp mới áp từ chính chương này trở đi.
    pub from: String,
    pub target: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
    #[serde(default)]
    pub source: EntrySource,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressTimeline {
    /// Người dùng đã sửa tay cặp này → harness không tự đổi nữa.
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub changes: Vec<AddressChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cast {
    pub version: u32,
    pub characters: IndexMap<String, Character>,
    pub addressing: IndexMap<String, AddressTimeline>,
}

impl Default for Cast {
    fn default() -> Self {
        Cast { version: 1, characters: IndexMap::new(), addressing: IndexMap::new() }
    }
}

impl Cast {
    pub fn gender_of(&self, name: &str) -> Option<Gender> {
        self.characters.get(name).and_then(|character| character.gender)
    }
}

impl Cast {
    /// Lenient: mục sai schema bị bỏ, không làm hỏng cả bảng (file tự sửa tay, JSON từ GUI).
    pub fn normalize(parsed: &Value) -> Cast {
        let mut cast = Cast::default();
        if let Some(characters) = parsed.get("characters").and_then(Value::as_object) {
            for (name, value) in characters {
                if let Ok(character) = serde_json::from_value::<Character>(value.clone()) {
                    cast.characters.insert(name.clone(), character);
                }
            }
        }
        if let Some(addressing) = parsed.get("addressing").and_then(Value::as_object) {
            for (key, value) in addressing {
                let Some(record) = value.as_object() else { continue };
                let changes = record
                    .get("changes")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items.iter().filter_map(|item| serde_json::from_value::<AddressChange>(item.clone()).ok()).collect()
                    })
                    .unwrap_or_default();
                let pinned = record.get("pinned").and_then(Value::as_bool).unwrap_or(false);
                cast.addressing.insert(key.clone(), AddressTimeline { pinned, changes });
            }
        }
        cast
    }
}

/// File thiếu/hỏng → bảng rỗng (truyện cũ chưa có cast.json chạy như trước).
pub fn load_cast(paths: &StoryPaths) -> Cast {
    fs::read_to_string(&paths.cast_json)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .map(|parsed| Cast::normalize(&parsed))
        .unwrap_or_default()
}

pub fn save_cast(paths: &StoryPaths, cast: &Cast) -> Result<()> {
    let json = serde_json::to_string_pretty(cast).expect("Cast luôn serialize được");
    write_atomic(&paths.cast_json, &format!("{json}\n"))
}

const MALE_TERMS: &[&str] = &[
    "anh", "ông", "chú", "thầy", "bố", "cha", "chồng", "huynh", "đệ", "chàng", "công tử", "thiếu gia", "lão gia",
    "tiên sinh", "phu quân", "tướng công", "sư huynh", "sư đệ", "đại ca", "ca ca", "thúc thúc", "ông chủ", "ông xã",
    "anh trai", "em trai", "học trưởng",
];
const FEMALE_TERMS: &[&str] = &[
    "chị", "bà", "cô", "dì", "thím", "mợ", "mẹ", "má", "vợ", "tỷ", "muội", "nàng", "thiếp", "cô nương", "tiểu thư",
    "phu nhân", "nương tử", "sư tỷ", "sư muội", "bà chủ", "bà xã", "chị gái", "em gái", "cô giáo", "chị đại",
    "học tỷ", "học muội", "tiên cô", "tiên tử", "tỷ tỷ", "muội muội",
];
/// Cụm có chữ đầu trùng từ mang giới nhưng không mang giới.
const NEUTRAL_TERMS: &[&str] = &["đệ tử", "anh em", "chị em", "cô chú", "ông bà", "bà con", "sư thúc"];

fn phrase_gender(phrase: &str) -> Option<Option<Gender>> {
    if NEUTRAL_TERMS.contains(&phrase) {
        Some(None)
    } else if MALE_TERMS.contains(&phrase) {
        Some(Some(Gender::Male))
    } else if FEMALE_TERMS.contains(&phrase) {
        Some(Some(Gender::Female))
    } else {
        None
    }
}

/// Giới mà một từ xưng hô tiếng Việt ấn định cho người được chỉ (`thầy Hạ` → nam, `Ninh sư tỷ` → nữ).
/// Chỉ xét từ viết thường ở đầu hoặc cuối cụm — chữ viết hoa là tên riêng (`Sương Anh`).
pub fn term_gender(term: &str) -> Option<Gender> {
    let words: Vec<&str> = term.split_whitespace().collect();
    let lower = |word: &str| word.chars().next().is_some_and(char::is_lowercase);
    let head: Vec<&str> = words.iter().copied().take_while(|word| lower(word)).collect();
    let mut tail: Vec<&str> = words.iter().rev().copied().take_while(|word| lower(word)).collect();
    tail.reverse();
    let candidates: [Option<String>; 4] = [
        (head.len() >= 2).then(|| head[..2].join(" ")),
        head.first().map(|word| word.to_string()),
        (tail.len() >= 2).then(|| tail[tail.len() - 2..].join(" ")),
        tail.last().map(|word| word.to_string()),
    ];
    for phrase in candidates.into_iter().flatten() {
        if let Some(found) = phrase_gender(&phrase) {
            return found;
        }
    }
    None
}

/// Đại từ chỉ có thể là ngôi một — đứng ở vế "gọi người kia" nghĩa là cặp bị viết ngược.
const FIRST_PERSON_ONLY: &[&str] = &["tao", "tôi", "tớ", "ta", "mình", "trẫm", "tại hạ"];

/// Tách `X–Y` thành (甲 tự xưng, 甲 gọi 乙); cặp viết ngược kiểu `mày–tao` được đảo lại.
pub fn normalize_pair(target: &str) -> Option<(String, String)> {
    let (own, other) = target.split_once('–')?;
    let (own, other) = (own.trim(), other.trim());
    if own.is_empty() || other.is_empty() {
        return None;
    }
    let first_only = |term: &str| FIRST_PERSON_ONLY.contains(&term.to_lowercase().as_str());
    if first_only(other) && !first_only(own) {
        Some((other.to_string(), own.to_string()))
    } else {
        Some((own.to_string(), other.to_string()))
    }
}

/// Cặp `甲→乙: X–Y` trái với giới đã chốt: X mang giới khác 甲, hoặc Y mang giới khác 乙.
pub fn pair_gender_conflict(key: &str, target: &str, cast: &Cast) -> bool {
    let sides = addressing_sides(key);
    let (Some((own, other)), [speaker, listener]) = (normalize_pair(target), sides.as_slice()) else { return false };
    let clash = |name: &str, term: &str| match (cast.gender_of(name), term_gender(term)) {
        (Some(known), Some(implied)) => known != implied,
        _ => false,
    };
    clash(speaker, &own) || clash(listener, &other)
}

/// Cặp xưng hô có hiệu lực khi dịch chương `chapter_id`: thay đổi cuối cùng có mốc từ chương này trở về
/// trước (chương khai đổi đã dùng cặp mới cho cả chương); không có thì lấy giá trị gốc ở `story.json`.
/// Cặp viết ngược được đảo lại.
pub fn effective_addressing(base: &StringMap, cast: &Cast, chapter_id: &str) -> StringMap {
    let keys = base.keys().chain(cast.addressing.keys().filter(|key| !base.contains_key(*key)));
    keys.filter_map(|key| {
        let changed = cast.addressing.get(key).and_then(|timeline| {
            timeline
                .changes
                .iter()
                .filter(|change| natural_chapter_compare(&change.from, chapter_id) != Ordering::Greater)
                .max_by(|a, b| natural_chapter_compare(&a.from, &b.from))
                .map(|change| change.target.as_str())
        });
        let target = changed.or_else(|| base.get(key).map(String::as_str))?;
        let target = match normalize_pair(target) {
            Some((own, other)) => format!("{own}–{other}"),
            None => target.to_string(),
        };
        Some((key.clone(), target))
    })
    .collect()
}

/// Chỉ giữ cặp mà CẢ HAI bên có mặt trong chương (tên đầy đủ hoặc dạng bỏ họ).
pub fn chapter_addressing(pairs: &StringMap, source: &str) -> StringMap {
    pairs
        .iter()
        .filter(|(key, _)| {
            let sides = addressing_sides(key);
            sides.len() == 2 && sides.iter().all(|side| glossary_entry_matches_source(side, source))
        })
        .map(|(key, target)| (key.clone(), target.clone()))
        .collect()
}

/// Chức danh tiếng Trung không mang giới, hay đi sau họ (`贺老师`, `周总`) — bản dịch phải tự chọn giới.
const NEUTRAL_TITLES: &[&str] = &[
    "老师", "老板", "总", "导", "医生", "教授", "主任", "队长", "经理", "教练", "前辈", "同学", "律师", "警官", "校长",
    "院长", "局长", "组长", "班长", "师傅", "大夫", "博士", "秘书", "助理", "主管", "总监", "主编", "掌柜", "大人", "长老",
];

/// Các cụm `họ + chức danh trung tính` của tên `name` có trong `source` (`贺静昭` → `贺老师`).
pub fn title_aliases(name: &str, source: &str) -> Vec<String> {
    let Some(surname) = name.chars().next() else { return vec![] };
    NEUTRAL_TITLES
        .iter()
        .map(|title| format!("{surname}{title}"))
        .filter(|alias| source.contains(alias.as_str()))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresentCharacter {
    pub source: String,
    pub name: String,
    pub gender: Gender,
}

/// Nhân vật đã rõ giới có mặt trong chương: tên đầy đủ, tên bỏ họ, hoặc họ + chức danh (`贺老师`).
pub fn present_characters(names: &StringMap, cast: &Cast, source: &str) -> Vec<PresentCharacter> {
    names
        .iter()
        .filter_map(|(han, viet)| {
            let gender = cast.gender_of(han)?;
            let present = glossary_entry_matches_source(han, source) || !title_aliases(han, source).is_empty();
            present.then(|| PresentCharacter { source: han.clone(), name: viet.clone(), gender })
        })
        .collect()
}

fn addressing_rules(setting: GenreSetting) -> String {
    let (ancient, modern) = match setting {
        GenreSetting::Ancient => (true, false),
        GenreSetting::Modern => (false, true),
        GenreSetting::Mixed => (true, true),
    };
    let mut rules = vec![
        "- **Theo giới người được nói tới.** Từ gọi hoặc chỉ người không mang giới trong tiếng Trung (老师, 老板, 老大, 前辈, 同学, 医生, 你, 您…) phải chọn từ Việt theo giới của đúng người đó. Tra danh sách nhân vật bên dưới trước; chưa có thì theo 她/他 và ngữ cảnh trong chương. Không mặc định là nam.".to_string(),
    ];
    let mut pronoun = String::from(
        "- **Danh xưng làm đại từ.** Khi nhân vật tự xưng hoặc gọi người đối diện bằng danh xưng, dùng chính danh xưng Việt tương ứng thay đại từ trong cả lượt thoại.",
    );
    if modern {
        pronoun.push_str(" Hiện đại (学姐/学弟, 姐/哥, 老师, 妈, 叔…): `学弟，你不要吓学姐好不好` → `Em đừng dọa chị được không`, KHÔNG `Đàn em, cậu đừng dọa đàn chị`; không dùng `đàn em`, `đàn chị`, `đàn anh`, `học tỷ`, `học đệ` trong thoại.");
    }
    if ancient {
        pronoun.push_str(" Cổ đại (师姐/师弟, 师兄/师妹, 姐姐, 师父, 娘…): `师弟，你不要吓师姐` → `Sư đệ đừng dọa sư tỷ`, KHÔNG `Sư đệ, ngươi đừng dọa ta`.");
    }
    rules.push(pronoun);
    let own = match setting {
        GenreSetting::Ancient => "mặc định `ta`",
        GenreSetting::Modern => "mặc định `mình`",
        GenreSetting::Mixed => "mặc định `mình` ở cảnh hiện đại, `ta` ở cảnh cổ đại",
    };
    rules.push(format!(
        "- **Độc thoại nội tâm.** Ý nghĩ đặt trong ngoặc kép mà không nói với ai: mỗi nhân vật chốt MỘT ngôi tự xưng trong cả chương, {own}; không đổi qua lại giữa các ngôi."
    ));
    if modern {
        rules.push("- **老师 ngoài trường học.** 老师 dùng để gọi nghệ sĩ, tiền bối trong nghề, nhân viên đoàn phim… vẫn là `thầy` / `cô` + họ theo giới người được gọi (`贺老师` là nữ → `cô Hạ`); `各位老师` → `các thầy cô`.".to_string());
    }
    rules.join("\n")
}

/// Dấu hiệu quan hệ thân mật trong raw — đủ đặc hiệu để không bật vì từ thường (không lấy 孩子, 吻 trần).
const INTIMACY_MARKERS: &[&str] = &[
    "结婚", "订婚", "求婚", "领证", "嫁给", "老公", "老婆", "男朋友", "女朋友", "男友", "女友", "未婚夫", "未婚妻", "接吻",
    "亲吻", "恋人", "情侣", "夫妻", "我爱你", "我喜欢你", "表白", "约会", "成亲", "洞房", "夫君", "娘子", "相公", "道侣",
];

/// Một đôi nhân vật mà cặp xưng hô đang là kiểu người lạ trong khi chương có dấu hiệu thân mật.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairReview {
    pub a: String,
    pub b: String,
    /// `a→b` và `b→a` đang hiệu lực (chiều nào chưa có cặp thì None).
    pub forward: Option<String>,
    pub backward: Option<String>,
    pub markers: Vec<String>,
}

/// Cặp kiểu người lạ: tự xưng `tôi`, hoặc cổ đại `ta` + gọi `ngươi`/`cô nương`/`công tử`…
fn is_distant(target: &str) -> bool {
    let Some((own, other)) = normalize_pair(target) else { return false };
    let (own, other) = (own.to_lowercase(), other.to_lowercase());
    own == "tôi" || (own == "ta" && ["ngươi", "cô nương", "công tử", "đạo hữu", "các hạ"].contains(&other.as_str()))
}

/// Harness tự tìm đôi cần xét lại thay vì trông vào model tự nhận ra: model coi cặp trong danh sách là lệnh
/// phải giữ (thử thật: đang bàn chuyện cưới vẫn `tôi–cô`). Điều kiện: chương có dấu hiệu thân mật, một chiều
/// của đôi còn kiểu người lạ, không chiều nào bị người dùng ghim. `pairs` là cặp hiệu lực đã lọc theo chương.
pub fn pairs_to_review(pairs: &StringMap, cast: &Cast, source: &str) -> Vec<PairReview> {
    let markers: Vec<String> =
        INTIMACY_MARKERS.iter().filter(|marker| source.contains(*marker)).take(4).map(|m| m.to_string()).collect();
    if markers.is_empty() {
        return vec![];
    }
    let pinned = |key: &str| cast.addressing.get(key).is_some_and(|timeline| timeline.pinned);
    let mut reviews: Vec<PairReview> = Vec::new();
    for key in pairs.keys() {
        let sides = addressing_sides(key);
        let [a, b] = sides.as_slice() else { continue };
        if reviews.iter().any(|r| (r.a == *a && r.b == *b) || (r.a == *b && r.b == *a)) {
            continue;
        }
        let reverse = format!("{b}{}{a}", crate::story::ADDRESSING_ARROW);
        let (forward, backward) = (pairs.get(key).cloned(), pairs.get(&reverse).cloned());
        let distant = [&forward, &backward].iter().any(|target| target.as_deref().is_some_and(is_distant));
        if !distant || pinned(key) || pinned(&reverse) {
            continue;
        }
        reviews.push(PairReview { a: a.to_string(), b: b.to_string(), forward, backward, markers: markers.clone() });
    }
    reviews
}

/// Luật xưng hô chung của bối cảnh — phần TĨNH (giống nhau ở mọi chương của truyện), đứng trước từ điển
/// để nằm trong đoạn đầu prompt mà nhà cung cấp cache được. Nằm ngoài base để truyện dùng base sửa
/// tay/prompt riêng cũng nhận.
pub fn addressing_rules_section(setting: GenreSetting) -> String {
    format!(
        "\n# Xưng hô\n\nCác luật này đứng trên mọi bảng đại từ và bảng kính ngữ phía trên.\n\n{}\n",
        addressing_rules(setting)
    )
}

/// Phần ĐỘNG theo chương: nhân vật có mặt đã rõ giới, cặp xưng hô đang hiệu lực, các đôi cần xét lại.
/// Rỗng khi chương không có gì để liệt kê.
pub fn chapter_addressing_section(
    present: &[PresentCharacter],
    pairs: &StringMap,
    reviews: &[PairReview],
    setting: GenreSetting,
) -> String {
    let mut section = String::new();
    if !present.is_empty() {
        let list: Vec<String> =
            present.iter().map(|c| format!("- {} ({}): {}", c.source, c.name, c.gender.label())).collect();
        section.push_str(&format!("\nNhân vật có mặt trong chương — giới tính đã chốt:\n\n{}\n", list.join("\n")));
    }
    let arrow = crate::story::ADDRESSING_ARROW;
    let under_review = |key: &str| {
        reviews.iter().any(|r| key == format!("{}{arrow}{}", r.a, r.b) || key == format!("{}{arrow}{}", r.b, r.a))
    };
    let keep: StringMap =
        pairs.iter().filter(|(key, _)| !under_review(key)).map(|(key, target)| (key.clone(), target.clone())).collect();
    if !keep.is_empty() {
        section.push_str(&format!(
            "\nCặp xưng hô đang dùng — `甲→乙: X–Y` nghĩa là trong thoại 甲 tự xưng X và gọi 乙 là Y (cặp ngược `乙→甲` có mục riêng):\n\n{}\n\nGiữ đúng các cặp này. Chỉ đổi khi chương này cho thấy quan hệ hai người đã khác hẳn lúc chốt cặp (thành người yêu, vợ chồng, kết nghĩa, trở mặt thành thù); khi đổi phải đổi cả hai chiều và khai lại cặp trong danh sách glossary đề xuất kèm `note`.\n",
            crate::prompt::json_pretty(&keep)
        ));
    }
    if !reviews.is_empty() {
        let lovers = match setting {
            GenreSetting::Ancient => "tình nhân, phu thê thường là ta–nàng và thiếp–chàng",
            GenreSetting::Modern => "người yêu, vợ chồng thường là anh–em; nữ lớn tuổi hơn hẳn thì chị–em",
            GenreSetting::Mixed => "hiện đại: người yêu, vợ chồng thường là anh–em; cổ đại: ta–nàng và thiếp–chàng",
        };
        let list: Vec<String> = reviews
            .iter()
            .map(|r| {
                let current: Vec<String> = [(&r.a, &r.b, &r.forward), (&r.b, &r.a, &r.backward)]
                    .into_iter()
                    .filter_map(|(from, to, target)| target.as_ref().map(|t| format!("`{from}{arrow}{to}: {t}`")))
                    .collect();
                format!("- {} ↔ {}: đang {}; dấu hiệu trong raw: {}", r.a, r.b, current.join(", "), r.markers.join(", "))
            })
            .collect();
        section.push_str(&format!(
            "\nCặp cần xét lại trong chương này — xưng hô dưới đây chốt từ lúc hai người còn xa lạ, mà chương có dấu hiệu quan hệ thân mật:\n\n{}\n\nĐọc chương rồi quyết định cho từng đôi. Nếu hai người đã là người yêu, đính hôn hoặc vợ chồng: KHÔNG dùng cặp cũ — dùng cặp hợp quan hệ hiện tại cho cả hai chiều trong cả chương ({lovers}) và BẮT BUỘC khai lại cả hai chiều trong danh sách glossary đề xuất kèm `note`. Nếu dấu hiệu chỉ là nói đùa, nói về người khác, hoặc hai người chưa thành đôi: giữ cặp cũ và không khai.\n",
            list.join("\n")
        ));
    }
    if section.is_empty() {
        section
    } else {
        format!("\n# Nhân vật và xưng hô trong chương\n{section}")
    }
}

/// Vấn đề xưng hô ở một đoạn (chỉ số 0-based) — check quy về `Violation`, soát dùng để chấm bản soát.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParagraphIssue {
    pub paragraph: usize,
    pub message: String,
}

/// Tìm `phrase` đứng thành từ riêng trong `text` (hai bên không dính chữ/số), trả đúng cụm trong bản dịch.
fn find_whole_phrase(text: &[char], phrase: &str) -> Option<String> {
    let needle: Vec<char> = phrase.chars().collect();
    if needle.is_empty() || text.len() < needle.len() {
        return None;
    }
    (0..=text.len() - needle.len())
        .filter(|&at| text[at..at + needle.len()] == needle[..])
        .filter(|&at| at == 0 || !text[at - 1].is_alphanumeric())
        .find(|&at| text.get(at + needle.len()).is_none_or(|c| !c.is_alphanumeric()))
        .map(|at| text[at..at + needle.len()].iter().collect())
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Từ trái giới đặt trước họ: nhân vật nữ mà bị gọi `thầy/anh/ông/chú + họ`, nam mà bị gọi `cô/chị/bà/dì + họ`.
fn wrong_gender_terms(gender: Gender) -> &'static [&'static str] {
    match gender {
        Gender::Female => &["thầy", "anh", "ông", "chú"],
        Gender::Male => &["cô", "chị", "bà", "dì"],
    }
}

/// Đoạn raw gọi nhân vật bằng `họ + chức danh trung tính` (`贺老师`) mà bản dịch đoạn đó dùng từ trái giới
/// kèm họ (`thầy Hạ` cho nữ). Chỉ xét khi họ đó ứng với đúng một nhân vật — ưu tiên người có mặt trong
/// chương — và nhân vật đã rõ giới; tên Việt phải khớp số tiếng với tên Hán để lấy được họ.
pub fn gender_violations(
    raw: &[String],
    translated: &[Option<String>],
    names: &StringMap,
    cast: &Cast,
) -> Vec<ParagraphIssue> {
    let chapter = raw.join("\n");
    let surname = |han: &str| han.chars().next();
    let is_title_form = |han: &str| {
        let rest: String = han.chars().skip(1).collect();
        NEUTRAL_TITLES.contains(&rest.as_str())
    };
    let mut issues = Vec::new();
    for (index, (raw_paragraph, text)) in raw.iter().zip(translated).enumerate() {
        let Some(text) = text else { continue };
        let chars: Vec<char> = text.chars().collect();
        for (han, viet) in names {
            let Some(gender) = cast.gender_of(han) else { continue };
            let aliases = title_aliases(han, raw_paragraph);
            let Some(alias) = aliases.first() else { continue };
            let same: Vec<&String> =
                names.keys().filter(|key| surname(key) == surname(han) && !is_title_form(key)).collect();
            let present: Vec<&String> =
                same.iter().copied().filter(|key| glossary_entry_matches_source(key, &chapter)).collect();
            let pool = if present.is_empty() { same } else { present };
            if pool.len() != 1 || pool[0] != han {
                continue;
            }
            let syllables: Vec<&str> = viet.split_whitespace().collect();
            if syllables.len() != han.chars().count() {
                continue;
            }
            let found = wrong_gender_terms(gender).iter().find_map(|term| {
                find_whole_phrase(&chars, &format!("{term} {}", syllables[0]))
                    .or_else(|| find_whole_phrase(&chars, &format!("{} {}", capitalize(term), syllables[0])))
            });
            let Some(found) = found else { continue };
            let label = format!("{alias} là {han} ({})", gender.label());
            let message = if alias.ends_with("老师") {
                let right = if gender == Gender::Female { "cô" } else { "thầy" };
                format!("Sai giới: {label} → `{right} {}`, bản dịch viết `{found}`", syllables[0])
            } else {
                format!("Sai giới: {label}, bản dịch viết `{found}`")
            };
            issues.push(ParagraphIssue { paragraph: index, message });
        }
    }
    issues
}

static QUOTED: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r#"[“"][^”"]*[”"]?"#).unwrap());
static COARSE_VOCATIVE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r#"(?i)[“",]\s*(đàn chị|đàn em|đàn anh|học tỷ|học đệ|học muội|học trưởng)\s*,"#).unwrap()
});
const SENIORITY_TERMS: &[&str] = &["学姐", "学弟", "学长", "学妹"];

/// Raw gọi nhau bằng 学姐/学弟/学长/学妹 mà thoại bản dịch dùng `đàn em`, `đàn chị`… làm lời gọi.
pub fn vocative_violations(raw: &[String], translated: &[Option<String>]) -> Vec<ParagraphIssue> {
    raw.iter()
        .zip(translated)
        .enumerate()
        .filter(|(_, (raw_paragraph, _))| SENIORITY_TERMS.iter().any(|term| raw_paragraph.contains(term)))
        .filter_map(|(index, (_, text))| {
            let text = text.as_deref()?;
            let found = QUOTED
                .find_iter(text)
                .find_map(|quote| COARSE_VOCATIVE.captures(quote.as_str()).map(|c| c[1].to_string()))?;
            Some(ParagraphIssue {
                paragraph: index,
                message: format!(
                    "Danh xưng thô trong thoại: `{found}` → dùng em/chị/anh làm đại từ, bỏ `đàn em/đàn chị`"
                ),
            })
        })
        .collect()
}

/// Mọi vấn đề xưng hô harness tự bắt được ở bản dịch một chương.
pub fn addressing_issues(
    raw: &[String],
    translated: &[Option<String>],
    names: &StringMap,
    cast: &Cast,
) -> Vec<ParagraphIssue> {
    let mut issues = gender_violations(raw, translated, names, cast);
    issues.extend(vocative_violations(raw, translated));
    issues
}

// ---- Học từ đề xuất của model khi chốt chương ----

const FEMALE_MARKERS: &[&str] = &["她", "女", "姐", "妹", "妈", "娘", "妻", "嫂", "姑", "婆", "姨", "媳", "夫人", "小姐"];
const MALE_MARKERS: &[&str] = &["他", "男", "哥", "弟", "爸", "爹", "叔", "伯", "爷", "先生", "公子", "兄", "丈夫"];

/// Raw có dấu hiệu của giới `gender` (她/女/姐… hoặc 他/男/哥…) — chốt chặn tối thiểu cho giới model khai.
pub fn gender_evidence(text: &str, gender: Gender) -> bool {
    let markers = if gender == Gender::Female { FEMALE_MARKERS } else { MALE_MARKERS };
    markers.iter().any(|marker| text.contains(marker))
}

fn pair_text(target: &str) -> String {
    match normalize_pair(target) {
        Some((own, other)) => format!("{own}–{other}"),
        None => target.trim().to_string(),
    }
}

/// Nhận giới model khai kèm entry `names`: tên phải là nhân vật đã biết (`names`), có mặt trong raw chương,
/// và raw có dấu hiệu của đúng giới đó. Đã có giới thì không đè; khai ngược ghi vào `disputed`.
pub fn learn_genders(
    entries: &Value,
    raw: &str,
    names: &std::collections::HashSet<String>,
    cast: &mut Cast,
    chapter_id: &str,
) -> Vec<String> {
    let mut notes = Vec::new();
    for item in entries.as_array().into_iter().flatten() {
        let Some(record) = item.as_object() else { continue };
        let text = |key: &str| record.get(key).and_then(Value::as_str).unwrap_or("").trim();
        let (source, category) = (text("source"), text("category"));
        let Some(gender) = Gender::parse(text("gender")) else { continue };
        if !(category.is_empty() || category == "names") || !names.contains(source) || !raw.contains(source) {
            continue;
        }
        if !gender_evidence(raw, gender) {
            continue;
        }
        let character = cast.characters.entry(source.to_string()).or_default();
        match character.gender {
            None => {
                character.gender = Some(gender);
                character.source = EntrySource::Auto;
                character.chapter = chapter_id.to_string();
            }
            Some(known) if known != gender => {
                if !character.disputed.iter().any(|chapter| chapter == chapter_id) {
                    character.disputed.push(chapter_id.to_string());
                }
                notes.push(format!("{source} đang là {}, chương này khai giới ngược ({})", known.label(), gender.label()));
            }
            Some(_) => {}
        }
    }
    notes
}

/// Siết cặp xưng hô MỚI trước khi học: hai bên phải là nhân vật có tên, không trái giới đã chốt; cặp viết
/// ngược được đảo. Entry nhóm khác đi qua nguyên vẹn.
pub fn filter_new_pairs(
    pairs: Vec<crate::glossary::ExtractedPair>,
    names: &std::collections::HashSet<String>,
    cast: &Cast,
) -> Vec<crate::glossary::ExtractedPair> {
    pairs
        .into_iter()
        .filter_map(|mut pair| {
            if pair.category != "addressing" {
                return Some(pair);
            }
            let sides = addressing_sides(&pair.source);
            if sides.len() != 2 || !sides.iter().all(|side| names.contains(*side)) {
                return None;
            }
            pair.target = pair_text(&pair.target);
            (!pair_gender_conflict(&pair.source, &pair.target, cast)).then_some(pair)
        })
        .collect()
}

/// Nhận thay đổi cho cặp ĐÃ CÓ (ở `story.json` hoặc mốc trước): phải có `note`, không ghim, khác cặp đang
/// hiệu lực, không đảo về giá trị ngay trước đó, hợp giới. Ghi mốc `from` = chương đang chốt; dịch lại cùng
/// chương thì thay mốc cũ của chương đó.
pub fn apply_address_changes(
    entries: &Value,
    raw: &str,
    base: &StringMap,
    cast: &mut Cast,
    chapter_id: &str,
) -> Vec<String> {
    let mut notes = Vec::new();
    for item in entries.as_array().into_iter().flatten() {
        let Some(record) = item.as_object() else { continue };
        let text = |key: &str| record.get(key).and_then(Value::as_str).unwrap_or("").trim();
        if text("category") != "addressing" || text("note").is_empty() {
            continue;
        }
        let Some((key, target)) = crate::glossary::parse_addressing_entry(text("source"), text("target"), raw) else {
            continue;
        };
        let target = pair_text(&target);
        if !base.contains_key(&key) && !cast.addressing.contains_key(&key) {
            continue; // cặp mới → đi đường học cặp mới
        }
        if cast.addressing.get(&key).is_some_and(|timeline| timeline.pinned) || pair_gender_conflict(&key, &target, cast) {
            continue;
        }
        let mut history: Vec<String> = base.get(&key).map(|value| pair_text(value)).into_iter().collect();
        if let Some(timeline) = cast.addressing.get(&key) {
            let mut earlier: Vec<&AddressChange> = timeline
                .changes
                .iter()
                .filter(|change| natural_chapter_compare(&change.from, chapter_id) == Ordering::Less)
                .collect();
            earlier.sort_by(|a, b| natural_chapter_compare(&a.from, &b.from));
            history.extend(earlier.into_iter().map(|change| pair_text(&change.target)));
        }
        let current = history.last().cloned().unwrap_or_default();
        let previous = history.len().checked_sub(2).map(|index| history[index].clone());
        if target == current || previous.as_deref() == Some(target.as_str()) {
            continue;
        }
        let timeline = cast.addressing.entry(key.clone()).or_default();
        timeline.changes.retain(|change| change.from != chapter_id);
        timeline.changes.push(AddressChange {
            from: chapter_id.to_string(),
            target: target.clone(),
            note: text("note").to_string(),
            source: EntrySource::Auto,
        });
        timeline.changes.sort_by(|a, b| natural_chapter_compare(&a.from, &b.from));
        notes.push(format!("đổi xưng hô {key}: {current} → {target} ({})", text("note")));
    }
    notes
}

// ---- Bề mặt cho GUI ----

/// Các cặp trong `story.json` trái với giới đã chốt (học từ trước khi có bảng nhân vật) — để người dùng dọn.
pub fn conflicting_addressing(pairs: &StringMap, cast: &Cast) -> Vec<String> {
    pairs.iter().filter(|(key, target)| pair_gender_conflict(key, target, cast)).map(|(key, _)| key.clone()).collect()
}

/// Người dùng lưu hồ sơ truyện: cặp xưng hô họ sửa hoặc tự thêm được ghim (harness không tự đổi nữa, giá trị
/// họ nhập áp cho mọi chương nên mốc cũ bị xoá); cặp họ xoá thì gỡ luôn mốc. Trả `true` khi bảng đổi.
pub fn pin_edited_pairs(old: &StringMap, new: &StringMap, cast: &mut Cast) -> bool {
    let before = cast.clone();
    for (key, target) in new {
        if old.get(key) != Some(target) {
            cast.addressing.insert(key.clone(), AddressTimeline { pinned: true, changes: vec![] });
        }
    }
    for key in old.keys().filter(|key| !new.contains_key(*key)) {
        cast.addressing.shift_remove(key);
    }
    *cast != before
}
