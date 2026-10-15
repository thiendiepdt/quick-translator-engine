//! Từ điển kiểm tra nội dung nhạy cảm về Việt Nam (spec docs/superpowers/specs/2026-10-15-vn-sensitive-check-design.md).
//! Bản cứng trong binary; phần người dùng (mục thêm, mục tắt, bảng thay thế) ở `<base>/vn-terms.json` — cùng thư mục
//! bản mặc định nên cả GUI lẫn CLI (agy gọi `qt-ai accept`) đều đọc được.

use crate::base::BaseStore;
use crate::error::{CoreError, Result};
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const SETTINGS_FILE: &str = "vn-terms.json";
/// Số ký tự mỗi bên chỗ khớp để xét mẫu loại trừ.
const WINDOW: usize = 12;
/// `越…越` (越战越勇, 越大越好, 他越战…越快): 越 là phó từ "càng", không phải nước Việt.
const CANG: &str = r"越[^，。！？、\n]{0,8}越";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Group {
    /// Tên gọi VN, người Việt, kể cả miệt thị.
    Ten,
    /// Chủ quyền biển đảo.
    Cq,
    /// Chiến tranh, lịch sử nhạy cảm.
    Ls,
    /// Địa danh, nhân vật VN hiện đại.
    Hd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Mietthi,
    Canxem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub group: Group,
    pub level: Level,
    /// Từ yếu: chỉ báo khi cùng đoạn có từ đi kèm.
    #[serde(default)]
    pub weak: bool,
    #[serde(default)]
    pub zh: Vec<String>,
    #[serde(default)]
    pub vi: Vec<String>,
    /// Regex loại trừ, xét trong cửa sổ ±12 ký tự quanh chỗ khớp (raw lẫn bản dịch).
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplaceSetting {
    Ancient,
    Modern,
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Replacement {
    pub from: String,
    pub to: String,
    pub setting: ReplaceSetting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VnSettings {
    pub extra: Vec<Entry>,
    pub disabled: Vec<String>,
    pub replacements: Vec<Replacement>,
}

impl Default for VnSettings {
    fn default() -> Self {
        VnSettings { extra: vec![], disabled: vec![], replacements: default_replacements() }
    }
}

pub fn default_replacements() -> Vec<Replacement> {
    use ReplaceSetting::*;
    let r = |from: &str, to: &str, setting| Replacement { from: from.into(), to: to.into(), setting };
    vec![
        r("Giao Chỉ", "Chiêm Thành", Ancient),
        r("An Nam", "Chiêm Thành", Ancient),
        r("Giao Châu", "Chiêm Thành", Ancient),
        r("Đại Việt", "Chiêm Thành", Ancient),
        r("Việt Nam", "Thái Lan", Modern),
        r("người Việt", "người Thái", Modern),
        r("tiếng Việt", "tiếng Thái", Modern),
        r("Hà Nội", "Băng Cốc", Modern),
        r("Sài Gòn", "Chiang Mai", Modern),
        r("TP Hồ Chí Minh", "Chiang Mai", Modern),
    ]
}

fn e(id: &str, group: Group, level: Level, weak: bool, zh: &[&str], vi: &[&str], exclude: &[&str]) -> Entry {
    let own = |list: &[&str]| list.iter().map(|s| s.to_string()).collect();
    Entry { id: id.into(), group, level, weak, zh: own(zh), vi: own(vi), exclude: own(exclude) }
}

/// Khi hai chỗ khớp chồng nhau, mục strong thắng mục weak, rồi cụm dài hơn (xem `Lexicon::resolve`) —
/// nên một cụm weak (南越, 南沙…) không còn nuốt được chỗ khớp strong nằm lẫn trong nó (云南越南).
pub fn builtin_entries() -> Vec<Entry> {
    use Group::*;
    use Level::*;
    vec![
        e("viet-nam", Ten, Canxem, false, &["越南", "越南人", "越南籍", "越南语", "越语"], &["Việt Nam", "người Việt Nam", "tiếng Việt"], &[]),
        e("khi-viet", Ten, Mietthi, false, &["越南猴子", "越南猴", "越猴", "猴国", "安南猴", "交趾蛮", "小越南"], &["khỉ Việt Nam", "khỉ Việt", "nước khỉ"], &[]),
        e("co-dau-viet", Ten, Mietthi, false, &["越南新娘", "越南媳妇", "越南老婆", "越南妹"], &["cô dâu Việt", "vợ Việt"], &[]),
        e("an-nam", Ten, Canxem, false, &["安南", "安南国", "安南人"], &["An Nam"], &[r"[平西淮延泰雅长兴永吉]安南", r"安南(?:[东西北]|方向|边|面|门|部)"]),
        e("an-nam-do-ho", Ten, Canxem, false, &["安南都护", "安南都护府"], &["An Nam đô hộ"], &[]),
        e("giao-chi", Ten, Canxem, false, &["交趾", "交阯", "交趾人", "交趾国"], &["Giao Chỉ"], &[]),
        e("dai-viet", Ten, Canxem, false, &["大越"], &["Đại Việt"], &[CANG, r"[长变增扩放]大越", r"大越(?:过|级|界|狱|野|岭|墙|位|权|轨)"]),
        e("giao-chau", Ten, Canxem, true, &["交州"], &["Giao Châu"], &[]),
        e("nam-viet", Ten, Canxem, true, &["南越", "南越国", "南越王"], &["Nam Việt"], &[r"[江湖河]南越", CANG]),
        e("nguoi-viet", Ten, Canxem, true, &["越人", "京族"], &["người Việt", "người Kinh"], &[CANG, r"(?:吴|於|百|山|闽|瓯)越人"]),
        e("hoang-sa", Cq, Canxem, false, &["西沙", "西沙群岛"], &["Hoàng Sa", "Tây Sa"], &["莫西沙", "Mạc Tây Sa"]),
        e("truong-sa", Cq, Canxem, false, &["南沙群岛"], &["quần đảo Trường Sa", "quần đảo Nam Sa"], &[]),
        e("truong-sa-yeu", Cq, Canxem, true, &["南沙"], &["Trường Sa", "Nam Sa"], &[]),
        e("trung-sa", Cq, Canxem, false, &["中沙群岛", "三沙市"], &["Trung Sa", "Tam Sa"], &[]),
        e("duong-chin-doan", Cq, Canxem, false, &["九段线", "十一段线", "南海断续线"], &["đường chín đoạn", "đường lưỡi bò", "đường mười một đoạn"], &[]),
        e("nam-hai-chu-dao", Cq, Canxem, false, &["南海诸岛", "南中国海"], &["biển Nam Trung Hoa", "quần đảo Nam Hải"], &[]),
        e("dao-da", Cq, Canxem, false, &["永兴岛", "永暑礁", "美济礁", "赤瓜礁", "黄岩岛"], &["đảo Phú Lâm", "đá Chữ Thập", "đá Vành Khăn", "Gạc Ma", "đảo Hoàng Nham"], &[]),
        e(
            "nam-hai",
            Cq,
            Canxem,
            true,
            &["南海"],
            &["Nam Hải", "Biển Đông"],
            &[
                "天南海北",
                "南海北",
                "海北天南",
                r"南海(?:派|剑|神尼|鳄神|老怪|郡|县|观音|普陀|龙王|龙宫)",
                r"Nam Hải (?:phái|kiếm|thần ni|Ngạc Thần|lão quái|quận|Long Vương|Quan Âm)",
            ],
        ),
        e("vinh-bac-bo", Cq, Canxem, true, &["北部湾", "东京湾"], &["Vịnh Bắc Bộ"], &[]),
        e("chien-tranh-1979", Ls, Canxem, false, &["对越自卫反击战", "对越反击战", "中越战争", "对越作战"], &["chiến tranh biên giới Việt", "chiến tranh Trung - Việt", "phản kích tự vệ"], &[]),
        e("lao-son", Ls, Canxem, false, &["老山战役", "两山轮战", "法卡山", "者阴山"], &["Lão Sơn", "Pháp Tạp Sơn"], &[]),
        e("hai-ba-trung", Ls, Canxem, false, &["征侧", "征贰"], &["Hai Bà Trưng", "Trưng Trắc", "Trưng Nhị"], &[r"征侧[翼面身]", r"[出远亲长]征侧"]),
        e("binh-dinh-giao-chi", Ls, Canxem, false, &["平定交趾", "郡县安南", "交趾郡", "日南郡", "九真郡"], &["bình định Giao Chỉ", "quận Giao Chỉ", "quận Nhật Nam", "quận Cửu Chân"], &[]),
        e("nhan-vat-lich-su", Ls, Canxem, false, &["李常杰", "黎利", "陈兴道", "陈国峻", "李公蕴", "阮惠"], &["Lý Thường Kiệt", "Lê Lợi", "Trần Hưng Đạo", "Trần Quốc Tuấn", "Lý Công Uẩn", "Nguyễn Huệ"], &[]),
        e("ho-chi-minh", Ls, Canxem, false, &["胡志明", "越共", "越盟", "北越"], &["Hồ Chí Minh", "Việt Cộng", "Việt Minh", "Bắc Việt"], &[r"[江河淮山湖]北越"]),
        e("chien-tranh-viet-nam", Ls, Canxem, false, &["越南战争"], &["chiến tranh Việt Nam"], &[]),
        e("viet-chien", Ls, Canxem, true, &["越战"], &[], &[CANG]),
        e("ma-vien", Ls, Canxem, true, &["马援", "伏波将军"], &["Mã Viện", "Phục Ba tướng quân"], &[]),
        e("bach-dang", Ls, Canxem, true, &["白藤江"], &["Bạch Đằng"], &[]),
        e("dia-danh", Hd, Canxem, false, &["西贡", "胡志明市", "岘港", "芽庄", "下龙湾", "湄公河", "顺化", "头顿市"], &["Sài Gòn", "TP Hồ Chí Minh", "thành phố Hồ Chí Minh", "Đà Nẵng", "Nha Trang", "Hạ Long", "Mê Kông", "Huế", "Vũng Tàu"], &[]),
        e("ha-noi", Hd, Canxem, true, &["河内"], &["Hà Nội"], &["河内郡", r"quận Hà Nội", r"Hà Nội (?:quận|vương)"]),
        e("hai-phong", Hd, Canxem, true, &["海防市"], &["Hải Phòng"], &[]),
    ]
}

/// Một chỗ khớp: chỉ số mục trong `Lexicon::entries` + vị trí byte trong văn bản.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    pub entry: usize,
    pub start: usize,
    pub end: usize,
}

pub struct Lexicon {
    pub entries: Vec<Entry>,
    /// Đổi khi từ điển đổi — `vn-check.json` khác hash thì quét lại cả truyện.
    pub hash: String,
    zh: Option<AhoCorasick>,
    zh_owner: Vec<usize>,
    vi: Option<AhoCorasick>,
    vi_owner: Vec<usize>,
    exclude: Vec<Vec<Regex>>,
}

pub(crate) fn fnv64(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn automaton(patterns: &[String], ascii_case_insensitive: bool) -> Option<AhoCorasick> {
    if patterns.is_empty() {
        return None;
    }
    AhoCorasickBuilder::new()
        .match_kind(MatchKind::Standard)
        .ascii_case_insensitive(ascii_case_insensitive)
        .build(patterns)
        .ok()
}

impl Lexicon {
    pub fn new(entries: Vec<Entry>) -> Lexicon {
        let (mut zh, mut zh_owner, mut vi, mut vi_owner) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for (index, entry) in entries.iter().enumerate() {
            for pattern in entry.zh.iter().filter(|p| !p.trim().is_empty()) {
                zh.push(pattern.clone());
                zh_owner.push(index);
            }
            for pattern in entry.vi.iter().filter(|p| !p.trim().is_empty()) {
                vi.push(pattern.clone());
                vi_owner.push(index);
            }
        }
        let exclude = entries.iter().map(|entry| entry.exclude.iter().filter_map(|p| Regex::new(p).ok()).collect()).collect();
        let hash = format!("{:016x}", fnv64(&serde_json::to_string(&entries).unwrap_or_default()));
        Lexicon { zh: automaton(&zh, false), zh_owner, vi: automaton(&vi, true), vi_owner, exclude, hash, entries }
    }

    pub fn builtin() -> Lexicon {
        Lexicon::new(builtin_entries())
    }

    pub fn with_settings(settings: &VnSettings) -> Lexicon {
        let mut entries: Vec<Entry> =
            builtin_entries().into_iter().filter(|entry| !settings.disabled.contains(&entry.id)).collect();
        entries.extend(settings.extra.iter().cloned());
        Lexicon::new(entries)
    }

    /// Từ điển có hiệu lực: bản cứng + `<base>/vn-terms.json`. File hỏng/thiếu → bản cứng.
    pub fn load(base: &BaseStore) -> Lexicon {
        Lexicon::with_settings(&load_settings(base))
    }

    /// Chỗ khớp chữ Hán, đã bỏ chỗ bị loại trừ; không chồng nhau, sắp theo vị trí.
    pub fn find_zh(&self, text: &str) -> Vec<Hit> {
        let Some(ac) = &self.zh else { return vec![] };
        let candidates = ac
            .find_overlapping_iter(text)
            .map(|m| Hit { entry: self.zh_owner[m.pattern().as_usize()], start: m.start(), end: m.end() })
            .collect();
        self.resolve(candidates, text)
    }

    /// Chỗ khớp tiếng Việt nguyên từ (ký tự liền trước/sau không phải chữ/số), đã bỏ chỗ bị loại trừ;
    /// không chồng nhau, sắp theo vị trí.
    pub fn find_vi(&self, text: &str) -> Vec<Hit> {
        let Some(ac) = &self.vi else { return vec![] };
        let candidates = ac
            .find_overlapping_iter(text)
            .filter(|m| {
                !text[..m.start()].chars().next_back().is_some_and(char::is_alphanumeric)
                    && !text[m.end()..].chars().next().is_some_and(char::is_alphanumeric)
            })
            .map(|m| Hit { entry: self.vi_owner[m.pattern().as_usize()], start: m.start(), end: m.end() })
            .collect();
        self.resolve(candidates, text)
    }

    /// Bỏ ứng viên bị loại trừ, rồi chọn tập không chồng nhau: strong trước weak, cụm dài hơn trước,
    /// bắt đầu sớm hơn trước.
    fn resolve(&self, mut candidates: Vec<Hit>, text: &str) -> Vec<Hit> {
        candidates.retain(|hit| !self.excluded(*hit, text));
        candidates.sort_by_key(|hit| (self.entries[hit.entry].weak, std::cmp::Reverse(hit.end - hit.start), hit.start));
        let mut chosen: Vec<Hit> = Vec::new();
        for hit in candidates {
            if chosen.iter().all(|kept| hit.end <= kept.start || hit.start >= kept.end) {
                chosen.push(hit);
            }
        }
        chosen.sort_by_key(|hit| hit.start);
        chosen
    }

    /// Có mẫu loại trừ của mục khớp một đoạn chồng lên chỗ khớp, trong cửa sổ ±12 ký tự.
    pub fn excluded(&self, hit: Hit, text: &str) -> bool {
        let rules = &self.exclude[hit.entry];
        if rules.is_empty() {
            return false;
        }
        let lo = text[..hit.start].char_indices().rev().nth(WINDOW - 1).map_or(0, |(i, _)| i);
        let hi = text[hit.end..].char_indices().nth(WINDOW).map_or(text.len(), |(i, _)| hit.end + i);
        let window = &text[lo..hi];
        let (start, end) = (hit.start - lo, hit.end - lo);
        rules.iter().any(|rule| rule.find_iter(window).any(|m| m.start() < end && m.end() > start))
    }

    /// Đoạn raw có một dạng chữ Hán của mục — luật đối chiếu hai bên.
    pub fn mentions_zh(&self, entry: usize, text: &str) -> bool {
        self.entries[entry].zh.iter().any(|zh| !zh.is_empty() && text.contains(zh.as_str()))
    }
}

pub fn settings_path(base: &BaseStore) -> Option<PathBuf> {
    base.dir().map(|dir| dir.join(SETTINGS_FILE))
}

pub fn load_settings(base: &BaseStore) -> VnSettings {
    settings_path(base)
        .and_then(|path| std::fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save_settings(base: &BaseStore, settings: &VnSettings) -> Result<()> {
    let path = settings_path(base)
        .ok_or_else(|| CoreError::Internal("Không có thư mục dữ liệu app để lưu cấu hình kiểm tra VN.".into()))?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(CoreError::io(dir))?;
    }
    let text = serde_json::to_string_pretty(settings).expect("VnSettings luôn serialize được");
    crate::story_fs::write_atomic(&path, &format!("{text}\n"))
}
