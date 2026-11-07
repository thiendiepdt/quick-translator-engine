//! Đọc file Quy Tắc: lớp từ (`[lớp]`), nhóm `[số]` quét ký tự, và các nhóm quy
//! tắc chạy trên token. Cú pháp ở spec mục 5. Ngữ nghĩa theo bản thử
//! `quytac.py` (`Program`, `compile_char_rules`).

use std::cmp::Reverse;
use std::sync::LazyLock;

use regex::{Captures, Regex};
use rustc_hash::FxHashMap;

use super::QuyTacError;

/// Chữ số Hán trừ 〇, cho `zh_digit`.
const HAN_DIGITS: &str = "零一二三四五六七八九";
const NUM_NO_ZERO: &str = "零一二三四五六七八九十百千万亿两";

/// Một chữ số Hán trong khuôn `[số]`. Chuẩn hoá đầu vào của QT chèn dấu cách quanh 〇
/// (không có trong bảng Hán Việt): "二〇一九" thành "二 〇 一九", "二〇〇〇年" thành
/// "二 〇〇〇 年". Chỉ dấu cách sát 〇 được bỏ qua; dấu cách thật giữa hai chữ số khác thì không.
fn zh_digit(others: &str) -> String {
    format!(r"\s?〇\s?|[{others}]")
}

/// Một lớp từ: chữ Hán → nghĩa thay trong khuôn (có thể rỗng).
pub(crate) type Class = FxHashMap<String, String>;

/// Nhãn khoá có sẵn, ngoài các lớp từ khai báo trong `[lớp]`.
pub(crate) const BUILTIN_SLOTS: [&str; 21] = [
    "đại-từ",
    "tên",
    "người",
    "danh-từ",
    "cụm",
    "động-từ",
    "danh-động",
    "hệ-từ",
    "có-đích",
    "tính-từ",
    "thành-ngữ",
    "số",
    "số-lượng",
    "thời-gian",
    "nơi",
    "phó-từ",
    "dấu",
    "chỉ-định",
    "nơi-cụm",
    "nơi-từ",
    "động-từ-tại",
];

/// Nhãn khoá có sẵn. Thứ tự là vị trí bit trong bộ nhớ đệm của `matcher`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Builtin {
    DaiTu,
    Ten,
    Nguoi,
    DanhTu,
    Cum,
    DongTu,
    DanhDong,
    HeTu,
    CoDich,
    TinhTu,
    ThanhNgu,
    So,
    SoLuong,
    ThoiGian,
    Noi,
    PhoTu,
    Dau,
    ChiDinh,
    NoiCum,
    NoiTu,
    DongTuTai,
}

impl Builtin {
    /// Số nhãn có sẵn.
    pub(crate) const COUNT: usize = Builtin::DongTuTai as usize + 1;

    pub(crate) fn from_name(name: &str) -> Option<Builtin> {
        Some(match name {
            "đại-từ" => Builtin::DaiTu,
            "tên" => Builtin::Ten,
            "người" => Builtin::Nguoi,
            "danh-từ" => Builtin::DanhTu,
            "cụm" => Builtin::Cum,
            "động-từ" => Builtin::DongTu,
            "danh-động" => Builtin::DanhDong,
            "hệ-từ" => Builtin::HeTu,
            "có-đích" => Builtin::CoDich,
            "tính-từ" => Builtin::TinhTu,
            "thành-ngữ" => Builtin::ThanhNgu,
            "số" => Builtin::So,
            "số-lượng" => Builtin::SoLuong,
            "thời-gian" => Builtin::ThoiGian,
            "nơi" => Builtin::Noi,
            "phó-từ" => Builtin::PhoTu,
            "dấu" => Builtin::Dau,
            "chỉ-định" => Builtin::ChiDinh,
            "nơi-cụm" => Builtin::NoiCum,
            "nơi-từ" => Builtin::NoiTu,
            "động-từ-tại" => Builtin::DongTuTai,
            _ => return None,
        })
    }
}

/// Loại khoá, phân giải một lần sau khi đọc hết file. Lớp từ phải khai báo
/// trước dòng quy tắc dùng nó (`slot_name` báo lỗi), trừ tên trùng nhãn có
/// sẵn; thành viên thêm vào lớp ở nhóm `[lớp]` sau vẫn được tính, và lớp từ
/// đè nhãn có sẵn cùng tên kể cả khi khai báo sau quy tắc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SlotKind {
    /// Lớp từ thứ `k` trong các lớp dùng làm khoá (bit `k` của `Program::class_bits`).
    Class(u16),
    Builtin(Builtin),
    /// Tên lạ: không khớp gì. Không xảy ra vì lúc đọc đã kiểm tên khoá.
    Unknown,
}

/// Khoá `{tên}` trong mẫu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Slot {
    pub(crate) name: Box<str>,
    pub(crate) kind: SlotKind,
}

impl Slot {
    fn new(name: String) -> Slot {
        Slot {
            name: name.into_boxed_str(),
            kind: SlotKind::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LeftKind {
    Ve,
    VeDong,
    MenhDe,
}

impl LeftKind {
    pub(crate) fn name(self) -> &'static str {
        match self {
            LeftKind::Ve => "vế",
            LeftKind::VeDong => "vế-động",
            LeftKind::MenhDe => "mệnh-đề",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ElKind {
    Start,
    End,
    Left(LeftKind),
    Gap,
    Many(Slot),
    Slot(Slot),
    Lit(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct El {
    pub(crate) kind: ElKind,
    pub(crate) opt: bool,
    pub(crate) neg: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RefKey {
    /// `{1}`, `{2}`… theo thứ tự khoá trong mẫu (tính từ 1).
    Index(usize),
    /// `{danh-từ}`: khoá đầu tiên mang tên đó.
    Name(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Part {
    Text(String),
    Ref { key: RefKey, filter: Option<String> },
}

#[derive(Debug)]
pub(crate) struct Rule {
    pub(crate) pre: Vec<El>,
    pub(crate) pat: Vec<El>,
    pub(crate) post: Vec<El>,
    pub(crate) tmpl: Vec<Part>,
    pub(crate) result: String,
    /// Chữ cố định bắt buộc của mẫu: mỗi danh sách là một mặt nạ bit trên
    /// `Program::need_lits` (`LitTable::words` từ), đoạn phải có ít nhất một chữ
    /// của mỗi danh sách, không thì khỏi thử.
    pub(crate) needs: Vec<u64>,
    /// Bit nhãn có sẵn (`Builtin`) và lớp từ (`SlotKind::Class(k)`, `k < 64`) của
    /// các khoá mẫu bắt buộc phải khớp một token: khoá không tuỳ chọn và khoá
    /// `{…}+` (`match_rule` trả `None` khi chúng không khớp, kể cả `{…}+?`).
    /// Ngữ cảnh, `{vế}`… và khoá lớp từ thứ 64 trở đi không tính.
    pub(crate) req_builtins: u32,
    pub(crate) req_classes: u64,
}

impl Rule {
    /// Điền `req_builtins`, `req_classes` sau khi khoá đã phân giải.
    fn index_requirements(&mut self) {
        let (mut builtins, mut classes) = (0u32, 0u64);
        for el in &self.pat {
            let slot = match &el.kind {
                ElKind::Slot(slot) if !el.opt => slot,
                ElKind::Many(slot) => slot,
                _ => continue,
            };
            match slot.kind {
                SlotKind::Builtin(builtin) => builtins |= 1 << builtin as u32,
                SlotKind::Class(k) if k < 64 => classes |= 1 << k,
                SlotKind::Class(_) | SlotKind::Unknown => {}
            }
        }
        self.req_builtins = builtins;
        self.req_classes = classes;
    }
}

/// Chỉ mục quy tắc chung (`Section::generic`) theo khoá đầu, để chọn bằng bit
/// những quy tắc mà khoá đầu có thể khớp token: khoá đầu không khớp thì quy tắc
/// không khớp. Bit `n` (từ `words` từ 64 bit) là `generic[n]`.
#[derive(Debug, Default)]
pub(crate) struct HeadIndex {
    pub(crate) words: usize,
    /// `by_builtin[b * words..]`: quy tắc có khoá đầu là nhãn có sẵn thứ `b`.
    pub(crate) by_builtin: Vec<u64>,
    /// `by_class[k * words..]`: quy tắc có khoá đầu là lớp từ thứ `k` (`k < 64`).
    pub(crate) by_class: Vec<u64>,
    /// Khoá đầu tuỳ chọn hay không phải khoá: không lọc được, luôn thử.
    pub(crate) open: Vec<u64>,
    /// Bit nhãn có sẵn / lớp từ mà quy tắc `n` đòi ở khoá đầu (0 nếu không).
    pub(crate) builtin_of: Vec<u32>,
    pub(crate) class_of: Vec<u64>,
}

impl HeadIndex {
    fn build(rules: &[Rule], generic: &[usize]) -> HeadIndex {
        let words = generic.len().div_ceil(64);
        let mut index = HeadIndex {
            words,
            by_builtin: vec![0; Builtin::COUNT * words],
            by_class: Vec::new(),
            open: vec![0; words],
            builtin_of: vec![0; generic.len()],
            class_of: vec![0; generic.len()],
        };
        for (n, &rule) in generic.iter().enumerate() {
            let (word, bit) = (n / 64, 1u64 << (n % 64));
            let head = &rules[rule].pat[0];
            let slot = match &head.kind {
                ElKind::Slot(slot) if !head.opt => slot,
                ElKind::Many(slot) => slot,
                _ => {
                    index.open[word] |= bit;
                    continue;
                }
            };
            match slot.kind {
                SlotKind::Builtin(builtin) => {
                    index.by_builtin[builtin as usize * words + word] |= bit;
                    index.builtin_of[n] = 1 << builtin as u32;
                }
                SlotKind::Class(k) if k < 64 => {
                    let k = k as usize;
                    if index.by_class.len() < (k + 1) * words {
                        index.by_class.resize((k + 1) * words, 0);
                    }
                    index.by_class[k * words + word] |= bit;
                    index.class_of[n] = 1 << k;
                }
                SlotKind::Class(_) => index.open[word] |= bit,
                // Không khớp gì: không bao giờ là ứng viên (`match_rule` cũng trả `None`).
                SlotKind::Unknown => {}
            }
        }
        index
    }
}

#[derive(Debug)]
pub(crate) struct Section {
    pub(crate) rounds: usize,
    pub(crate) result: String,
    pub(crate) rules: Vec<Rule>,
    /// Quy tắc mở đầu bằng chữ cố định, theo chữ đầu.
    pub(crate) first: FxHashMap<char, Vec<usize>>,
    /// Quy tắc mở đầu bằng `{vế}`…, theo chữ đầu của chữ cố định ngay sau.
    pub(crate) left: FxHashMap<char, Vec<usize>>,
    pub(crate) generic: Vec<usize>,
    /// Quy tắc chung theo khoá đầu.
    pub(crate) heads: HeadIndex,
    /// Mọi khoá của `first` và `left`.
    pub(crate) key_chars: CharBloom,
    /// Mọi quy tắc trong `first` và `left`, tăng dần, không trùng.
    pub(crate) keyed: Vec<usize>,
    /// Có quy tắc không cần chữ cố định nào.
    pub(crate) always: bool,
    /// Mặt nạ mọi chữ cố định (trên `Program::need_lits`) mà quy tắc của nhóm cần.
    pub(crate) lits: Vec<u64>,
}

#[derive(Debug)]
pub(crate) struct CharRule {
    pub(crate) regex: Regex,
    pub(crate) slots: Vec<String>,
    pub(crate) tmpl: Vec<Part>,
    pub(crate) result: String,
    /// Chữ cố định bắt buộc (chỉ số vào `Program::char_lits`), mỗi danh sách
    /// phải có ít nhất một chữ trong đoạn, không thì regex chắc chắn không khớp.
    /// `None`: mẫu có cú pháp lạ, luôn chạy regex.
    pub(crate) needs: Option<Box<[Box<[u32]>]>>,
    /// `needs` dạng mặt nạ bit trên `char_lits`, mỗi danh sách `char_lits.words()` từ.
    pub(crate) need_masks: Option<Box<[u64]>>,
    sort_key: usize,
}

/// Lọc Bloom 512 bit trên tập ký tự: bit chưa bật thì chắc chắn ký tự không có
/// trong tập (không bao giờ báo thiếu), nên dùng để khỏi tra bảng băm.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct CharBloom([u64; 8]);

impl CharBloom {
    fn bit(c: char) -> (usize, u64) {
        let hash = (c as u32).wrapping_mul(0x9e37_79b9) >> 23;
        ((hash >> 6) as usize, 1 << (hash & 63))
    }

    pub(crate) fn insert(&mut self, c: char) {
        let (word, bit) = Self::bit(c);
        self.0[word] |= bit;
    }

    pub(crate) fn may_contain(&self, c: char) -> bool {
        let (word, bit) = Self::bit(c);
        self.0[word] & bit != 0
    }
}

/// Bảng chữ cố định cần dò trong đoạn: mỗi chữ một chỉ số, tra theo chữ đầu.
/// Chỉ mục theo chữ đầu dựng một lần ở `finish`, sau khi đọc xong.
#[derive(Debug, Default)]
pub(crate) struct LitTable {
    lits: Vec<Box<str>>,
    /// Chữ đầu → khúc `ids[a..b]`: chỉ số các chữ mở đầu bằng nó, tăng dần.
    by_head: FxHashMap<char, (u32, u32)>,
    ids: Box<[u32]>,
    /// Bit `c` của ký tự `c` trong BMP bật đúng khi nó là chữ đầu của một chữ
    /// cố định, nên phần lớn ký tự khỏi tra `by_head`; ký tự ngoài BMP thì tra thẳng.
    bmp_heads: Box<[u64]>,
}

/// Số ký tự của BMP (U+0000–U+FFFF), cỡ bảng bit `LitTable::bmp_heads`.
const BMP: usize = 0x10000;

impl LitTable {
    fn intern(&mut self, lit: &str) -> usize {
        if let Some(id) = self.lits.iter().position(|known| &**known == lit) {
            return id;
        }
        assert!(!lit.is_empty(), "chữ cố định không rỗng");
        u32::try_from(self.lits.len()).expect("quá nhiều chữ cố định");
        self.lits.push(lit.into());
        self.lits.len() - 1
    }

    /// Dựng chỉ mục theo chữ đầu, trả phần dư của các mảng.
    fn finish(&mut self) {
        self.lits.shrink_to_fit();
        let mut heads: Vec<(char, u32)> = self
            .lits
            .iter()
            .zip(0u32..)
            .map(|(lit, id)| (lit.chars().next().expect("chữ cố định không rỗng"), id))
            .collect();
        heads.sort_unstable();
        self.ids = heads.iter().map(|&(_, id)| id).collect();
        let mut bmp_heads = vec![0u64; BMP / 64];
        self.by_head.clear();
        for (&(head, _), at) in heads.iter().zip(0u32..) {
            self.by_head.entry(head).or_insert((at, at)).1 = at + 1;
            if let Some(word) = bmp_heads.get_mut(head as usize / 64) {
                *word |= 1 << (head as u32 % 64);
            }
        }
        self.by_head.shrink_to_fit();
        self.bmp_heads = bmp_heads.into_boxed_slice();
    }

    /// Số từ 64 bit của một tập chỉ số chữ (ít nhất 1).
    pub(crate) fn words(&self) -> usize {
        self.lits.len().div_ceil(64).max(1)
    }

    /// Bit `n` của `present` thành `text.contains(chữ thứ n)`, quét `text` một lượt.
    pub(crate) fn find(&self, text: &str, present: &mut Vec<u64>) {
        present.clear();
        present.resize(self.words(), 0);
        for (at, c) in text.char_indices() {
            let code = c as usize;
            let bmp_word = self.bmp_heads.get(code / 64);
            if code < BMP && bmp_word.is_none_or(|word| word & (1 << (code % 64)) == 0) {
                continue;
            }
            let Some(&(from, to)) = self.by_head.get(&c) else {
                continue;
            };
            for &id in &self.ids[from as usize..to as usize] {
                let id = id as usize;
                let (word, bit) = (id / 64, 1 << (id % 64));
                if present[word] & bit == 0 && text[at..].starts_with(&*self.lits[id]) {
                    present[word] |= bit;
                }
            }
        }
    }
}

/// Chữ thứ `id` có trong tập bit `present` của `LitTable::find`.
#[cfg(test)]
pub(crate) fn has_lit(present: &[u64], id: usize) -> bool {
    present[id / 64] & (1 << (id % 64)) != 0
}

#[derive(Debug)]
pub(crate) struct Program {
    pub(crate) classes: FxHashMap<String, Class>,
    pub(crate) char_rules: Vec<CharRule>,
    pub(crate) sections: Vec<Section>,
    /// Chữ cố định mà `Rule::needs` trỏ tới.
    pub(crate) need_lits: LitTable,
    /// Chữ cố định mà `CharRule::needs` trỏ tới.
    pub(crate) char_lits: LitTable,
    /// Chữ Hán → các lớp từ có chữ đó, bit `k` cho `SlotKind::Class(k)` (64 lớp đầu).
    pub(crate) class_bits: FxHashMap<Box<str>, u64>,
}

static CONTEXT_SPLIT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s)\|(?:\s|$)").unwrap());
static TEMPLATE_REF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{([^{}|]+)(?:\|([^{}]+))?\}").unwrap());
static CHAR_SLOT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{([^{}]+)\}").unwrap());
/// Độ ưu tiên luật `[số]` như bản thử: bỏ khoá, tiêu đề, khuôn, ký tự tuỳ chọn
/// rồi đếm ký tự còn lại.
static SORT_STRIP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{[^{}]*\}|\[số\]| = .*|.\?").unwrap());

/// Số lượt `[tên N]` lớn nhất (bộ luật mặc định dùng tới 3): mỗi lượt quét
/// lại cả đoạn, số lớn tuỳ ý thì một file luật làm treo bản dịch.
const MAX_ROUNDS: usize = 20;

/// Tách `vế trái = khuôn`. Dòng `上 = ` đã bị cắt dấu cách cuối thành `上 =`;
/// bản thử (`partition(" = ")`) vẫn nhận nó: cả dòng là vế trái (chữ `=` thành
/// chữ cố định) và khuôn rỗng. Làm y như vậy; dòng thiếu hẳn ` = ` vẫn là lỗi.
fn split_rule(line: &str, line_no: usize) -> Result<(&str, &str), QuyTacError> {
    match line.split_once(" = ") {
        Some(parts) => Ok(parts),
        None if line.ends_with(" =") => Ok((line, "")),
        None => Err(err(line_no, "thiếu ' = '")),
    }
}

fn err(line: usize, message: impl Into<String>) -> QuyTacError {
    QuyTacError {
        line,
        message: message.into(),
    }
}

impl Program {
    pub(crate) fn parse(source: &str) -> Result<Program, QuyTacError> {
        #[derive(PartialEq)]
        enum Mode {
            Outside,
            Classes,
            Chars,
            Rules,
        }
        let mut classes: FxHashMap<String, Class> = FxHashMap::default();
        let mut char_lines: Vec<(usize, String)> = Vec::new();
        let mut sections: Vec<Section> = Vec::new();
        let mut mode = Mode::Outside;
        for (index, raw) in source.lines().enumerate() {
            let line_no = index + 1;
            let raw = if index == 0 {
                raw.trim_start_matches('\u{feff}')
            } else {
                raw
            };
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') {
                let header: Vec<&str> = line
                    .trim_matches(|c| c == '[' || c == ']')
                    .split_whitespace()
                    .collect();
                let Some(&name) = header.first() else {
                    return Err(err(line_no, "tiêu đề nhóm rỗng"));
                };
                mode = match name {
                    "lớp" => Mode::Classes,
                    "số" => Mode::Chars,
                    _ => {
                        let rounds = match header.get(1) {
                            Some(value) => value.parse::<usize>().map_err(|_| {
                                err(line_no, format!("số lượt không hợp lệ: {value}"))
                            })?,
                            None => 1,
                        };
                        if rounds > MAX_ROUNDS {
                            return Err(err(
                                line_no,
                                format!("số lượt quá lớn: {rounds} (tối đa {MAX_ROUNDS})"),
                            ));
                        }
                        let result = header.get(2).copied().unwrap_or("cụm").to_string();
                        sections.push(Section::new(rounds.max(1), result));
                        Mode::Rules
                    }
                };
                continue;
            }
            match mode {
                Mode::Outside => return Err(err(line_no, "dòng nằm ngoài mọi nhóm")),
                Mode::Classes => parse_class(line, line_no, &mut classes)?,
                Mode::Chars => char_lines.push((line_no, line.to_string())),
                Mode::Rules => {
                    let section = sections.last_mut().expect("Mode::Rules luôn có nhóm");
                    let rule = parse_rule(line, line_no, &section.result, &classes)?;
                    section.add(rule);
                }
            }
        }
        let mut char_lits = LitTable::default();
        let mut char_rules = char_lines
            .iter()
            .map(|(line_no, line)| compile_char_rule(line, *line_no, &classes, &mut char_lits))
            .collect::<Result<Vec<_>, _>>()?;
        // Nhiều chữ cố định hơn đi trước; hoà thì giữ thứ tự file (sort ổn định).
        char_rules.sort_by_key(|rule| Reverse(rule.sort_key));
        let words = char_lits.words();
        for rule in &mut char_rules {
            rule.need_masks = rule.needs.as_ref().map(|needs| {
                let mut masks = vec![0u64; needs.len() * words];
                for (list, ids) in needs.iter().enumerate() {
                    for &id in ids.iter() {
                        let id = id as usize;
                        masks[list * words + id / 64] |= 1 << (id % 64);
                    }
                }
                masks.into_boxed_slice()
            });
        }
        for section in &mut sections {
            section.finish();
        }
        let mut slot_classes: Vec<String> = Vec::new();
        let mut need_lits = LitTable::default();
        let mut need_ids: Vec<Vec<Vec<usize>>> = Vec::new();
        for rule in sections.iter_mut().flat_map(|section| section.rules.iter_mut()) {
            let elements = rule.pre.iter_mut().chain(&mut rule.pat).chain(&mut rule.post);
            for el in elements {
                if let ElKind::Slot(slot) | ElKind::Many(slot) = &mut el.kind {
                    slot.kind = if classes.contains_key(&*slot.name) {
                        let known = slot_classes.iter().position(|name| **name == *slot.name);
                        let k = known.unwrap_or_else(|| {
                            slot_classes.push(slot.name.to_string());
                            slot_classes.len() - 1
                        });
                        // Từ lớp thứ 64 không có bit mà tra theo tên: chỉ số lớn mấy cũng được.
                        SlotKind::Class(u16::try_from(k).unwrap_or(u16::MAX))
                    } else {
                        Builtin::from_name(&slot.name).map_or(SlotKind::Unknown, SlotKind::Builtin)
                    };
                }
            }
            rule.index_requirements();
            need_ids.push(
                rule.pat
                    .iter()
                    .filter(|el| !el.opt && !el.neg)
                    .filter_map(|el| match &el.kind {
                        ElKind::Lit(lits) => {
                            Some(lits.iter().map(|lit| need_lits.intern(lit)).collect())
                        }
                        _ => None,
                    })
                    .collect(),
            );
        }
        // Đủ bảng chữ rồi mới biết số từ của mặt nạ.
        let words = need_lits.words();
        let rules = sections.iter_mut().flat_map(|section| section.rules.iter_mut());
        for (rule, lists) in rules.zip(need_ids) {
            for ids in lists {
                let mut mask = vec![0u64; words];
                for id in ids {
                    mask[id / 64] |= 1 << (id % 64);
                }
                rule.needs.extend(mask);
            }
            rule.needs.shrink_to_fit();
        }
        for section in &mut sections {
            section.index_needs(words);
        }
        let mut class_bits: FxHashMap<Box<str>, u64> = FxHashMap::default();
        for (k, name) in slot_classes.iter().enumerate().take(64) {
            for zh in classes[name].keys() {
                *class_bits.entry(zh.as_str().into()).or_default() |= 1 << k;
            }
        }
        class_bits.shrink_to_fit();
        need_lits.finish();
        char_lits.finish();
        Ok(Program {
            classes,
            char_rules,
            sections,
            need_lits,
            char_lits,
            class_bits,
        })
    }
}

fn parse_class(
    line: &str,
    line_no: usize,
    classes: &mut FxHashMap<String, Class>,
) -> Result<(), QuyTacError> {
    let (name, members) = line
        .split_once(':')
        .ok_or_else(|| err(line_no, "dòng lớp thiếu ':'"))?;
    let table = classes.entry(name.trim().to_string()).or_default();
    for member in members.split_whitespace() {
        let (zh, vi) = member.split_once('=').unwrap_or((member, ""));
        table.insert(zh.to_string(), vi.replace('_', " "));
    }
    Ok(())
}

fn slot_name(
    name: &str,
    line_no: usize,
    classes: &FxHashMap<String, Class>,
) -> Result<String, QuyTacError> {
    if classes.contains_key(name) || BUILTIN_SLOTS.contains(&name) {
        Ok(name.to_string())
    } else {
        Err(err(line_no, format!("khoá không có: {{{name}}}")))
    }
}

fn parse_elements(
    text: &str,
    line_no: usize,
    classes: &FxHashMap<String, Class>,
) -> Result<Vec<El>, QuyTacError> {
    let mut out = Vec::new();
    for part in text.split_whitespace() {
        let neg = part.starts_with('!');
        let mut part = if neg { &part[1..] } else { part };
        let opt = part.ends_with('?');
        if opt {
            part = &part[..part.len() - 1];
        }
        let kind = match part {
            "^" => ElKind::Start,
            "$" => ElKind::End,
            "{vế}" => ElKind::Left(LeftKind::Ve),
            "{vế-động}" => ElKind::Left(LeftKind::VeDong),
            "{mệnh-đề}" => ElKind::Left(LeftKind::MenhDe),
            "{đoạn}" => ElKind::Gap,
            _ if part.starts_with('{') && part.ends_with("}+") => {
                ElKind::Many(Slot::new(slot_name(&part[1..part.len() - 2], line_no, classes)?))
            }
            _ if part.starts_with('{') && part.ends_with('}') => {
                ElKind::Slot(Slot::new(slot_name(&part[1..part.len() - 1], line_no, classes)?))
            }
            _ => {
                let mut lits: Vec<String> = Vec::new();
                for lit in part.trim_matches(|c| c == '(' || c == ')').split('|') {
                    if lit.is_empty() {
                        return Err(err(line_no, format!("chữ cố định rỗng trong {part}")));
                    }
                    if !lits.iter().any(|known| known == lit) {
                        lits.push(lit.to_string());
                    }
                }
                ElKind::Lit(lits)
            }
        };
        out.push(El { kind, opt, neg });
    }
    Ok(out)
}

fn parse_template(
    source: &str,
    cap_names: &[String],
    line_no: usize,
) -> Result<Vec<Part>, QuyTacError> {
    let mut parts = Vec::new();
    let mut last = 0;
    for caps in TEMPLATE_REF.captures_iter(source) {
        let whole = caps.get(0).expect("nhóm 0 luôn có");
        if whole.start() > last {
            parts.push(Part::Text(source[last..whole.start()].to_string()));
        }
        let key = &caps[1];
        let key = match key.parse::<usize>() {
            Ok(index) => {
                if index == 0 || index > cap_names.len() {
                    return Err(err(
                        line_no,
                        format!(
                            "khuôn tham chiếu {{{index}}} nhưng mẫu chỉ có {} khoá",
                            cap_names.len()
                        ),
                    ));
                }
                RefKey::Index(index)
            }
            Err(_) => {
                if !cap_names.iter().any(|name| name == key) {
                    return Err(err(
                        line_no,
                        format!("khuôn tham chiếu {{{key}}} nhưng mẫu không có khoá đó"),
                    ));
                }
                RefKey::Name(key.to_string())
            }
        };
        parts.push(Part::Ref {
            key,
            filter: caps.get(2).map(|m| m.as_str().to_string()),
        });
        last = whole.end();
    }
    if last < source.len() {
        parts.push(Part::Text(source[last..].to_string()));
    }
    Ok(parts)
}

fn parse_rule(
    line: &str,
    line_no: usize,
    default_result: &str,
    classes: &FxHashMap<String, Class>,
) -> Result<Rule, QuyTacError> {
    let (lhs, rhs) = split_rule(line, line_no)?;
    let (tmpl_src, result) = match rhs.split_once(" -> ") {
        Some((tmpl, result)) => (tmpl, result.trim().to_string()),
        None => (rhs, default_result.to_string()),
    };
    let pieces: Vec<&str> = CONTEXT_SPLIT.split(lhs).collect();
    let (pre, pat, post) = match pieces.as_slice() {
        [pat] => ("", *pat, ""),
        [pat, post] => ("", *pat, *post),
        [pre, pat, post] => (*pre, *pat, *post),
        _ => return Err(err(line_no, "quá nhiều dấu '|' ngữ cảnh")),
    };
    let pre = parse_elements(pre, line_no, classes)?;
    let pat = parse_elements(pat, line_no, classes)?;
    let post = parse_elements(post, line_no, classes)?;
    if pat.is_empty() {
        return Err(err(line_no, "mẫu rỗng"));
    }
    let mut cap_names: Vec<String> = Vec::new();
    for (index, el) in pat.iter().enumerate() {
        let next_is_lit = matches!(pat.get(index + 1).map(|e| &e.kind), Some(ElKind::Lit(_)));
        match &el.kind {
            ElKind::Left(kind) => {
                if index != 0 || !next_is_lit {
                    return Err(err(
                        line_no,
                        format!(
                            "{{{}}} phải đứng đầu mẫu và theo sau là chữ cố định",
                            kind.name()
                        ),
                    ));
                }
                cap_names.push(kind.name().to_string());
            }
            ElKind::Gap => {
                if !next_is_lit {
                    return Err(err(line_no, "{đoạn} phải theo sau là chữ cố định"));
                }
                cap_names.push("đoạn".to_string());
            }
            ElKind::Slot(slot) | ElKind::Many(slot) => cap_names.push(slot.name.to_string()),
            _ => {}
        }
    }
    let tmpl = parse_template(tmpl_src.trim(), &cap_names, line_no)?;
    Ok(Rule {
        pre,
        pat,
        post,
        tmpl,
        result,
        // Điền ở `Program::parse`, khi đã có bảng chữ cố định chung và khoá đã phân giải.
        needs: Vec::new(),
        req_builtins: 0,
        req_classes: 0,
    })
}

/// Chữ cố định bắt buộc của vế trái luật `[số]`, dạng "mỗi danh sách phải có
/// ít nhất một chữ trong đoạn": khúc chữ nằm ngoài khoá và nhóm (hai chữ liền
/// nhau trong mẫu thì liền nhau trong regex, `\s*` chỉ chèn quanh khoá), nhóm
/// `(a|b)` không tuỳ chọn, khoá lớp từ không tuỳ chọn (một trong các chữ của
/// lớp). Chữ có `?` theo sau, nhóm và khoá tuỳ chọn, khoá số thì bỏ qua.
/// `None` khi gặp cú pháp regex khác (thoát, lớp ký tự, lặp…), kể cả nhóm `(?…)`
/// không phải `(?:`: cờ như `(?i)`, `(?x)` đổi cách khớp cả phần sau nó.
fn char_rule_needs(lhs: &str, classes: &FxHashMap<String, Class>) -> Option<Vec<Vec<String>>> {
    if lhs.match_indices("(?").any(|(at, _)| !lhs[at + 2..].starts_with(':')) {
        return None;
    }
    let chars: Vec<char> = lhs.chars().collect();
    let mut needs: Vec<Vec<String>> = Vec::new();
    let mut run = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let (skip, need) = match c {
            '{' => {
                let close = chars[i..].iter().position(|&d| d == '}')?;
                let spec: String = chars[i + 1..i + close].iter().collect();
                if spec.contains('{') {
                    return None;
                }
                let optional = chars.get(i + close + 1) == Some(&'?');
                let (name, _) = spec.split_once(':').unwrap_or((&spec, ""));
                // Lớp rỗng thành `()`, khớp chuỗi rỗng: không đòi chữ nào.
                let keys = classes
                    .get(name)
                    .filter(|_| !optional && !matches!(name, "số" | "năm"))
                    .map(|class| class.keys().cloned().collect::<Vec<_>>())
                    .filter(|keys| !keys.is_empty() && !keys.iter().any(String::is_empty));
                (close + 1, keys)
            }
            '(' => {
                let mut depth = 0;
                let mut close = None;
                for (offset, &d) in chars[i..].iter().enumerate() {
                    match d {
                        '(' => depth += 1,
                        ')' => depth -= 1,
                        '\\' => return None,
                        _ => {}
                    }
                    if depth == 0 {
                        close = Some(offset);
                        break;
                    }
                }
                let close = close?;
                let body: String = chars[i + 1..i + close].iter().collect();
                let optional = chars.get(i + close + 1) == Some(&'?');
                let plain = !body.contains(|d: char| "()[]\\.*+?^${}".contains(d));
                let alternatives: Vec<String> = body.split('|').map(str::to_string).collect();
                let usable = !optional && plain && !alternatives.iter().any(String::is_empty);
                (close + 1, usable.then_some(alternatives))
            }
            '?' => (1, None),
            '\\' | '[' | ']' | '.' | '*' | '+' | '|' | '^' | '$' | ')' | '}' => return None,
            _ if chars.get(i + 1) == Some(&'?') => (2, None),
            _ => {
                run.push(c);
                i += 1;
                continue;
            }
        };
        if !run.is_empty() {
            needs.push(vec![std::mem::take(&mut run)]);
        }
        if let Some(mut alternatives) = need {
            alternatives.sort();
            needs.push(alternatives);
        }
        i += skip;
    }
    if !run.is_empty() {
        needs.push(vec![run]);
    }
    Some(needs)
}

fn compile_char_rule(
    line: &str,
    line_no: usize,
    classes: &FxHashMap<String, Class>,
    char_lits: &mut LitTable,
) -> Result<CharRule, QuyTacError> {
    let (lhs, rhs) = split_rule(line, line_no)?;
    let (tmpl_src, result) = match rhs.split_once(" -> ") {
        Some((tmpl, result)) => (tmpl, result.trim().to_string()),
        None => (rhs, "số-cụm".to_string()),
    };
    // Nhóm (a|b) của người viết không chiếm chỗ đánh số của khoá.
    let mut grouped = String::with_capacity(lhs.len() + 8);
    let mut chars = lhs.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '(' && chars.peek() != Some(&'?') {
            grouped.push_str("(?:");
        } else {
            grouped.push(c);
        }
    }
    let mut slots: Vec<String> = Vec::new();
    let mut failure: Option<String> = None;
    let pattern = CHAR_SLOT.replace_all(&grouped, |caps: &Captures| {
        let spec = &caps[1];
        let (name, bounds) = spec.split_once(':').unwrap_or((spec, ""));
        let (lo, hi) = bounds.split_once('-').unwrap_or((bounds, ""));
        slots.push(name.to_string());
        match (name, bounds.is_empty()) {
            ("số", false) => {
                format!(r"\s*((?:{}|\d){{{lo},{hi}}})\s*", zh_digit(NUM_NO_ZERO))
            }
            ("số", true) => {
                format!(r"\s*((?:{})+|\d+(?:\.\d+)?)\s*", zh_digit(NUM_NO_ZERO))
            }
            // Năm lịch: chữ số Hán đọc từng chữ, hoặc số Ả Rập trông như năm.
            // 500年, 5000年 là "500 năm", không phải năm lịch.
            ("năm", false) if lo == "3" => {
                format!(r"\s*((?:{}){{3,4}}|1[5-9]\d\d|20\d\d)\s*", zh_digit(HAN_DIGITS))
            }
            ("năm", false) => {
                format!(r"\s*((?:{}|\d){{{lo},{hi}}})\s*", zh_digit(HAN_DIGITS))
            }
            ("năm", true) => {
                format!(r"\s*((?:{}|\d)+)\s*", zh_digit(HAN_DIGITS))
            }
            _ => match classes.get(name) {
                Some(class) => {
                    let mut keys: Vec<&String> = class.keys().collect();
                    keys.sort_by(|a, b| {
                        b.chars()
                            .count()
                            .cmp(&a.chars().count())
                            .then_with(|| a.cmp(b))
                    });
                    let alternatives: Vec<String> =
                        keys.iter().map(|key| regex::escape(key)).collect();
                    format!("({})", alternatives.join("|"))
                }
                None => {
                    failure.get_or_insert_with(|| format!("khoá không có: {{{name}}}"));
                    String::new()
                }
            },
        }
    });
    if let Some(message) = failure {
        return Err(err(line_no, message));
    }
    let regex = Regex::new(&pattern)
        .map_err(|error| err(line_no, format!("mẫu số không hợp lệ: {error}")))?;
    let tmpl = parse_template(tmpl_src.trim(), &slots, line_no)?;
    let needs = char_rule_needs(lhs, classes).map(|needs| {
        let mut intern = |lit: &String| u32::try_from(char_lits.intern(lit)).expect("chữ cố định");
        needs
            .iter()
            .map(|alternatives| alternatives.iter().map(&mut intern).collect())
            .collect()
    });
    let sort_key = SORT_STRIP
        .replace_all(&format!("[số] {line}"), "")
        .chars()
        .count();
    Ok(CharRule {
        regex,
        slots,
        tmpl,
        result,
        needs,
        // Điền ở `Program::parse`, khi đã đủ bảng chữ cố định.
        need_masks: None,
        sort_key,
    })
}

fn first_chars(lits: &[String]) -> Vec<char> {
    let mut out = Vec::new();
    for c in lits.iter().filter_map(|lit| lit.chars().next()) {
        if !out.contains(&c) {
            out.push(c);
        }
    }
    out
}

impl Section {
    fn new(rounds: usize, result: String) -> Section {
        Section {
            rounds,
            result,
            rules: Vec::new(),
            first: FxHashMap::default(),
            left: FxHashMap::default(),
            generic: Vec::new(),
            heads: HeadIndex::default(),
            key_chars: CharBloom::default(),
            keyed: Vec::new(),
            always: false,
            lits: Vec::new(),
        }
    }

    /// Sau khi các quy tắc đã có `needs` (mặt nạ `words` từ) và khoá đã phân giải.
    fn index_needs(&mut self, words: usize) {
        for &c in self.first.keys().chain(self.left.keys()) {
            self.key_chars.insert(c);
        }
        self.keyed = self.first.values().chain(self.left.values()).flatten().copied().collect();
        self.keyed.sort_unstable();
        self.keyed.dedup();
        self.heads = HeadIndex::build(&self.rules, &self.generic);
        self.always = self.rules.iter().any(|rule| rule.needs.is_empty());
        self.lits = vec![0; words];
        for mask in self.rules.iter().flat_map(|rule| rule.needs.chunks(words)) {
            self.lits.iter_mut().zip(mask).for_each(|(all, m)| *all |= m);
        }
    }

    fn add(&mut self, rule: Rule) {
        let index = self.rules.len();
        let head = &rule.pat[0];
        match &head.kind {
            ElKind::Left(_) => {
                if let ElKind::Lit(lits) = &rule.pat[1].kind {
                    for first in first_chars(lits) {
                        self.left.entry(first).or_default().push(index);
                    }
                }
            }
            ElKind::Lit(lits) if !head.opt => {
                for first in first_chars(lits) {
                    self.first.entry(first).or_default().push(index);
                }
            }
            _ => self.generic.push(index),
        }
        self.rules.push(rule);
    }

    /// Trong mỗi danh sách: mẫu dài hơn trước, rồi nhiều chữ cố định hơn; hoà giữ thứ tự file.
    fn finish(&mut self) {
        let rules = &self.rules;
        let key = |index: &usize| {
            let rule = &rules[*index];
            let lits = rule
                .pat
                .iter()
                .filter(|el| matches!(el.kind, ElKind::Lit(_)))
                .count();
            (Reverse(rule.pat.len()), Reverse(lits))
        };
        for list in self.first.values_mut().chain(self.left.values_mut()) {
            list.sort_by_key(key);
        }
        self.generic.sort_by_key(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "[lớp]
vị-trí: 上=trên 中央=giữa
lượng-từ: 层=tầng 个=

[số]
{số}点半 = {1} giờ rưỡi -> thời-gian
第{số}(章|回) = Chương {1|số}: -> lit

[của 3]
{danh-từ} {vị-trí} | !{danh-từ} = {vị-trí} {danh-từ} -> nơi-cụm
这个 {danh-từ}+ | !的 = {1} này
{vế-động} 的时候 = khi {vế-động}

[nghĩa]
^ | 可是 | = Nhưng -> lit
";

    #[test]
    fn parses_classes_sections_and_indexes() {
        let program = Program::parse(SAMPLE).unwrap();
        assert_eq!(program.classes["vị-trí"]["中央"], "giữa");
        assert_eq!(program.classes["lượng-từ"]["个"], "");
        assert_eq!(program.sections.len(), 2);
        let cua = &program.sections[0];
        assert_eq!(cua.rounds, 3);
        assert_eq!(cua.generic, vec![0]);
        assert_eq!(cua.first[&'这'], vec![1]);
        assert_eq!(cua.left[&'的'], vec![2]);
        assert_eq!(cua.rules[0].result, "nơi-cụm");
        assert_eq!(cua.rules[1].result, "cụm");
        assert_eq!(cua.rules[0].post.len(), 1);
        assert!(cua.rules[0].post[0].neg);
        let nghia = &program.sections[1];
        assert_eq!(nghia.rules[0].pre[0].kind, ElKind::Start);
        assert_eq!(nghia.rules[0].result, "lit");
    }

    #[test]
    fn char_rules_sort_by_fixed_characters_and_match() {
        let program = Program::parse(SAMPLE).unwrap();
        assert_eq!(program.char_rules.len(), 2);
        // " 第(章|回)" dài hơn " 点半" nên đứng trước.
        let chapter = &program.char_rules[0];
        assert_eq!(chapter.result, "lit");
        let caps = chapter.regex.captures("第 12 章").unwrap();
        assert_eq!(&caps[1], "12");
        assert!(program.char_rules[1].regex.is_match("三点半"));
    }

    /// `find` bật đúng bit của mọi chữ cố định có trong chuỗi, kể cả chữ ngoài BMP
    /// và chữ chung chữ đầu.
    #[test]
    fn lit_table_finds_exactly_the_contained_literals() {
        let alphabet: Vec<char> = "的时候一个a😀b上".chars().collect();
        let mut seed: u64 = 3;
        let mut next = || {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            (seed >> 33) as usize
        };
        for _ in 0..500 {
            let mut table = LitTable::default();
            let mut lits: Vec<String> = Vec::new();
            for _ in 0..1 + next() % 70 {
                let lit: String =
                    (0..1 + next() % 3).map(|_| alphabet[next() % alphabet.len()]).collect();
                let id = table.intern(&lit);
                match lits.iter().position(|known| *known == lit) {
                    Some(known) => assert_eq!(id, known),
                    None => {
                        assert_eq!(id, lits.len());
                        lits.push(lit);
                    }
                }
            }
            table.finish();
            let mut present = Vec::new();
            for _ in 0..20 {
                let text: String =
                    (0..next() % 9).map(|_| alphabet[next() % alphabet.len()]).collect();
                table.find(&text, &mut present);
                assert_eq!(present.len(), table.words());
                for (id, lit) in lits.iter().enumerate() {
                    let contained = text.contains(lit.as_str());
                    assert_eq!(has_lit(&present, id), contained, "{lit:?} {text:?}");
                }
            }
        }
    }

    #[test]
    fn char_rule_needs_keep_only_required_literals() {
        let mut classes: FxHashMap<String, Class> = FxHashMap::default();
        let buoi = [("晚上", "tối"), ("上午", "sáng")];
        let buoi = buoi.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        classes.insert("buổi".to_string(), buoi);
        let needs = |lhs: &str| char_rule_needs(lhs, &classes);
        let lists = |lists: &[&[&str]]| {
            let lists = lists.iter().map(|list| list.iter().map(|s| s.to_string()).collect());
            Some(lists.collect::<Vec<Vec<String>>>())
        };
        assert_eq!(needs("{số:1-2}点{số:1-2}分"), lists(&[&["点"], &["分"]]));
        assert_eq!(needs("{buổi}{số:1-2}点多钟?"), lists(&[&["上午", "晚上"], &["点多"]]));
        assert_eq!(needs("{buổi}?{số}点"), lists(&[&["点"]]));
        assert_eq!(needs("{số:1-3}个?小(时|時)"), lists(&[&["小"], &["时", "時"]]));
        assert_eq!(needs("{số:1-8}(枚|块|个)?银币"), lists(&[&["银币"]]));
        assert_eq!(needs("公元前{số:1-6}年"), lists(&[&["公元前"], &["年"]]));
        assert_eq!(needs("第二天"), lists(&[&["第二天"]]));
        assert_eq!(needs("(?:年|月){số}"), lists(&[]));
        assert_eq!(needs(r"{số}\d"), None);
        assert_eq!(needs("{số}[年月]"), None);
        // Cờ đổi cách khớp cả phần sau: không lọc trước.
        assert_eq!(needs("(?i){số}kg"), None);
        assert_eq!(needs("(?x)第 {số} 章"), None);
        assert_eq!(needs("{số}(?i:kg)"), None);
        // Lớp rỗng không đòi chữ nào.
        classes.insert("trống".to_string(), FxHashMap::default());
        let needs = |lhs: &str| char_rule_needs(lhs, &classes);
        assert_eq!(needs("{số}{trống}年"), lists(&[&["年"]]));
    }

    #[test]
    fn every_builtin_slot_has_a_kind() {
        assert!(BUILTIN_SLOTS.iter().all(|name| Builtin::from_name(name).is_some()));
        assert_eq!(BUILTIN_SLOTS.len(), Builtin::COUNT);
    }

    #[test]
    fn templates_are_split_into_parts() {
        let program = Program::parse(SAMPLE).unwrap();
        let rule = &program.sections[0].rules[1];
        assert_eq!(
            rule.tmpl,
            vec![
                Part::Ref { key: RefKey::Index(1), filter: None },
                Part::Text(" này".to_string()),
            ]
        );
        let chapter = &program.char_rules[0];
        assert!(chapter.tmpl.contains(&Part::Ref {
            key: RefKey::Index(1),
            filter: Some("số".to_string()),
        }));
    }

    #[test]
    fn rejects_bad_lines_with_line_numbers() {
        let error = Program::parse("[của]\n{danh-từ} = {2}\n").unwrap_err();
        assert_eq!(error.line, 2);
        assert!(error.message.contains("{2}"), "{}", error.message);
        let error = Program::parse("[của]\n{không-có} = x\n").unwrap_err();
        assert!(error.message.contains("khoá không có"), "{}", error.message);
        let error = Program::parse("[của]\nabc\n").unwrap_err();
        assert!(error.message.contains("thiếu ' = '"), "{}", error.message);
        let error = Program::parse("abc = x\n").unwrap_err();
        assert_eq!(error.line, 1);
        let error = Program::parse("[số]\n{lạ}点 = x\n").unwrap_err();
        assert!(error.message.contains("khoá không có"), "{}", error.message);
    }

    #[test]
    fn rounds_are_capped_with_a_line_number() {
        let error = Program::parse("[của]\n{danh-từ} = x\n\n[mệnh 21]\n{danh-từ} = x\n")
            .unwrap_err();
        assert_eq!(error.line, 4);
        assert!(error.message.contains("tối đa 20"), "{}", error.message);
        let program = Program::parse("[của 20]\n{danh-từ} = x\n").unwrap();
        assert_eq!(program.sections[0].rounds, 20);
        // Cap phải trên mức bộ luật mặc định dùng.
        let base = Program::parse(include_str!("../../data/QuyTac.txt")).unwrap();
        let most = base.sections.iter().map(|section| section.rounds).max().unwrap();
        assert!(most < MAX_ROUNDS, "{most}");
    }

    /// Như bản thử (`partition`): `上 = ` bị cắt thành `上 =`, đọc được với khuôn
    /// rỗng, vế trái giữ chữ cố định `=`. Cả nhóm `[số]`.
    #[test]
    fn empty_template_parses_like_the_prototype() {
        let program = Program::parse("[nghĩa]\n上 = \n\n[số]\n三 =\n").unwrap();
        let rule = &program.sections[0].rules[0];
        assert!(rule.tmpl.is_empty());
        let lits: Vec<&ElKind> = rule.pat.iter().map(|el| &el.kind).collect();
        assert_eq!(
            lits,
            [&ElKind::Lit(vec!["上".to_string()]), &ElKind::Lit(vec!["=".to_string()])]
        );
        let chars = &program.char_rules[0];
        assert!(chars.tmpl.is_empty());
        assert!(chars.regex.is_match("三 =") && !chars.regex.is_match("三"));
    }

    #[test]
    fn default_rule_files_parse() {
        let base = include_str!("../../data/QuyTac.txt");
        let program = Program::parse(base).unwrap();
        let rules: usize = program.sections.iter().map(|s| s.rules.len()).sum();
        assert_eq!(rules + program.char_rules.len(), 205);
        let extra = include_str!("../../data/QuyTac-giong-muot.txt");
        Program::parse(&format!("{base}\n{extra}")).unwrap();
    }
}
