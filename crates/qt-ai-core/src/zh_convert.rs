//! Chuyển phồn thể → giản thể (chuẩn Đại lục) và đọc file text mọi mã hoá — dùng cho raw truyện (`zh_story`) và
//! công cụ chuyển riêng của app (một cụm chữ, một file, cả folder).
//!
//! Bộ chuyển là luật MediaWiki zh-CN (zhconv): đổi theo cụm từ nên 著名 / 乾隆 / 雪梨 giữ đúng, đổi cả từ vựng Đài Loan
//! sang Đại lục (滑鼠 → 鼠标, 軟體 → 软件) và ngoặc 「」 → “”. Luật cụm từ còn sót trợ từ 著 (等著, 念著), nên sau đó
//! mọi 著 còn lại thành 着, trừ các từ đọc "trứ" (著名, 著作, 显著, 原著…). Luật cụm từ cũng đổi 乾 trong tên riêng của
//! truyện (大乾王朝, 乾帝, 乾門) thành 干, nên 乾 đọc "Càn" được che trước khi chuyển; 鍾 ra 锺 (chữ họ theo chuẩn 2013)
//! được đưa về 钟 như truyện Đại lục vẫn viết. Bảng StrConv chữ một của qt-core (QT gốc) không dùng ở đây: nó đổi sai
//! 著名 → 着名 và bỏ sót 後, 於, 乾.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use zhconv::{get_builtin_converter, Variant};

use crate::story_fs::write_atomic;

/// 著 + chữ này đọc "trứ": 著称, 著述… (著名, 著作 xét riêng vì hay dính trợ từ: 念著名字, 做著作业).
const ZHU_FIRST: &[char] = &['称', '述', '录', '者'];
/// Chữ này + 著 đọc "trứ": 显著, 原著, 土著… Không có chữ hay đi với trợ từ 着 (大着胆子, 编着辫子, 努力着, 靠近着,
/// 依旧着, 向前着); 论 / 译 xét riêng (讨论着, 翻译着).
const ZHU_SECOND: &[char] = &['显', '卓', '昭', '土', '原', '名', '专', '巨', '遗', '拙', '佳', '杰', '撰', '鸿', '宏'];

/// Từ chừng này phần chữ Hán phải đổi thì coi là phồn thể. Giản thể thật dưới 0,1% (thư viện 63 truyện cao nhất
/// 0,06%), phồn thể thật 25–43%.
pub const TRADITIONAL_RATIO: f64 = 0.05;

/// 乾 + chữ này là tên riêng đọc "Càn" (triều, nước, tông môn, cung điện): 乾帝, 乾國, 乾門…
const QIAN_NEXT: &[char] = &[
    '帝', '皇', '国', '國', '朝', '王', '宗', '殿', '宫', '宮', '城', '京', '都', '军', '軍', '门', '門', '天', '道',
];
/// Chữ này + 乾 là 乾 nghĩa "khô" dù chữ sau là gì: 擦乾, 晒乾, 喝乾…
const DRY_PREV: &[char] = &[
    '擦', '晾', '晒', '曬', '烘', '吹', '烤', '榨', '吸', '舔', '蒸', '抹', '弄', '哭', '流', '熬', '喝', '吃', '风', '風',
];
/// Ký tự dùng riêng (vùng PUA) che 乾 tên riêng trong lúc chuyển — bộ chuyển không đụng tới.
const QIAN_MASK: char = '\u{E000}';

/// Phồn → giản chuẩn Đại lục. Giản thể đưa vào thì gần như giữ nguyên (chỉ đổi từ vựng Đài Loan, ngoặc 「」 nếu có).
pub fn to_simplified(text: &str) -> String {
    let masked = mask_qian(text);
    let converted = get_builtin_converter(Variant::ZhCN).convert(&masked);
    fix_zhe(&converted).replace(QIAN_MASK, "乾").replace('锺', "钟")
}

/// 乾 đọc "Càn": đứng sau 大 (大乾) hoặc trước chữ chỉ triều / nước / tông môn, trừ khi chữ trước là động từ làm khô.
fn mask_qian(text: &str) -> String {
    if !text.contains('乾') {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    chars
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            if c != '乾' {
                return c;
            }
            let prev = i.checked_sub(1).map(|p| chars[p]);
            let name = prev == Some('大') || chars.get(i + 1).is_some_and(|next| QIAN_NEXT.contains(next));
            if name && !prev.is_some_and(|p| DRY_PREV.contains(&p)) {
                QIAN_MASK
            } else {
                c
            }
        })
        .collect()
}

fn fix_zhe(text: &str) -> String {
    if !text.contains('著') {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    chars
        .iter()
        .enumerate()
        .map(|(i, &c)| {
            if c != '著' {
                return c;
            }
            if is_zhu(&chars, i) {
                '著'
            } else {
                '着'
            }
        })
        .collect()
}

/// 著 ở vị trí `i` đọc "trứ" (từ chỉ sáng tác / rõ rệt), không phải trợ từ 着. Xét cả chữ thứ hai bên cạnh: luật
/// cụm từ của bộ chuyển giữ 著 khi khớp 论著 / 著作 / 大著… kể cả trong 讨论著, 做著作业, 大著胆子.
fn is_zhu(chars: &[char], i: usize) -> bool {
    let at = |offset: isize| i.checked_add_signed(offset).and_then(|j| chars.get(j)).copied();
    match at(1) {
        Some('名') => return at(2) != Some('字'),
        Some('作') => return !matches!(at(2), Some('业' | '品')),
        Some(next) if ZHU_FIRST.contains(&next) => return true,
        _ => {}
    }
    match at(-1) {
        Some('论') => !matches!(at(-2), Some('讨' | '议' | '谈' | '争' | '辩' | '评')),
        Some('译') => at(-2) != Some('翻'),
        Some(prev) => ZHU_SECOND.contains(&prev),
        None => false,
    }
}

fn is_han(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c) || ('\u{3400}'..='\u{4DBF}').contains(&c)
}

/// (số chữ Hán, số chỗ phải đổi để thành giản thể) — chỉ đếm chữ Hán (ngoặc 「」 cũng bị đổi nhưng giản thể vẫn có),
/// bỏ 乾 / 著 vì giản thể vẫn dùng (大乾王朝, 著名): đếm vào thì chương giản thể ngắn bị nhận nhầm là phồn.
pub fn traditional_score(text: &str) -> (usize, usize) {
    let han = text.chars().filter(|c| is_han(*c)).count();
    let scored: String = text.chars().filter(|c| is_han(*c) && !matches!(c, '乾' | '著')).collect();
    (han, get_builtin_converter(Variant::ZhHans).count_replaced(&scored))
}

pub fn is_traditional(text: &str) -> bool {
    let (han, replaced) = traditional_score(text);
    han > 0 && replaced as f64 >= han as f64 * TRADITIONAL_RATIO
}

/// File text đã giải mã. `utf8` = file vốn là UTF-8 không BOM (ghi lại không cần đổi mã hoá).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decoded {
    pub text: String,
    pub encoding: &'static str,
    pub utf8: bool,
}

/// Đọc byte file text mọi mã hoá: BOM (UTF-8 / UTF-16), UTF-8 hợp lệ, không thì đoán (GBK, Big5, Shift_JIS…) theo
/// nội dung. Có byte 0 mà không có BOM UTF-16 → file nhị phân, trả None.
pub fn decode_text(bytes: &[u8]) -> Option<Decoded> {
    if let Some((encoding, bom)) = encoding_rs::Encoding::for_bom(bytes) {
        let (text, _) = encoding.decode_without_bom_handling(&bytes[bom..]);
        return Some(Decoded { text: text.into_owned(), encoding: encoding.name(), utf8: false });
    }
    if bytes.contains(&0) {
        return None;
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return Some(Decoded { text: text.to_string(), encoding: "UTF-8", utf8: true });
    }
    let mut detector = chardetng::EncodingDetector::new(chardetng::Iso2022JpDetection::Deny);
    detector.feed(bytes, true);
    let encoding = detector.guess(None, chardetng::Utf8Detection::Deny);
    let (text, _) = encoding.decode_without_bom_handling(bytes);
    Some(Decoded { text: text.into_owned(), encoding: encoding.name(), utf8: false })
}

/// Cách ghi kết quả của công cụ chuyển.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum OutputMode {
    /// Ghi đè file gốc (chỉ file có đổi).
    Replace,
    /// Ghi vào folder khác, giữ cấu trúc thư mục con của folder nguồn.
    Folder { dir: PathBuf },
    /// Ghi cạnh file gốc, tên thêm hậu tố trước đuôi: `chuong1.txt` → `chuong1_gian.txt`.
    Suffix { suffix: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertJob {
    /// File hoặc folder. File chọn thẳng luôn được chuyển; file trong folder lọc theo `extensions`.
    pub inputs: Vec<PathBuf>,
    pub recursive: bool,
    /// Đuôi file (không dấu chấm, không phân biệt hoa thường); rỗng = mọi file text.
    pub extensions: Vec<String>,
    pub mode: OutputMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedFile {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertReport {
    /// File có chữ phồn hoặc mã hoá khác UTF-8 — đã chuyển.
    pub converted: usize,
    /// File vốn là giản thể UTF-8: chế độ ghi đè để yên, hai chế độ còn lại vẫn ghi bản sao.
    pub unchanged: usize,
    /// Đường dẫn các file đã ghi.
    pub written: Vec<String>,
    pub skipped: Vec<SkippedFile>,
}

fn has_extension(path: &Path, extensions: &[String]) -> bool {
    extensions.is_empty()
        || path
            .extension()
            .map(|ext| ext.to_string_lossy().to_lowercase())
            .is_some_and(|ext| extensions.iter().any(|wanted| wanted.trim_start_matches('.').eq_ignore_ascii_case(&ext)))
}

/// File trong folder (đệ quy nếu cần), bỏ thư mục `exclude` (folder đích nằm trong folder nguồn), xếp theo tên.
fn walk(dir: &Path, recursive: bool, extensions: &[String], exclude: Option<&Path>, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut paths: Vec<PathBuf> = entries.filter_map(|entry| entry.ok().map(|e| e.path())).collect();
    paths.sort();
    for path in paths {
        if exclude.is_some_and(|skip| path.starts_with(skip)) {
            continue;
        }
        if path.is_dir() {
            if recursive {
                walk(&path, recursive, extensions, exclude, out);
            }
        } else if has_extension(&path, extensions) {
            out.push(path);
        }
    }
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let name = match path.extension() {
        Some(ext) => format!("{stem}{suffix}.{}", ext.to_string_lossy()),
        None => format!("{stem}{suffix}"),
    };
    path.with_file_name(name)
}

fn stem_ends_with(path: &Path, suffix: &str) -> bool {
    path.file_stem().is_some_and(|stem| stem.to_string_lossy().ends_with(suffix))
}

/// Chuyển các file / folder sang giản thể UTF-8 theo `job.mode`. Lỗi từng file ghi vào `skipped`, không dừng cả lượt.
pub fn convert_files(job: &ConvertJob) -> ConvertReport {
    let mut report = ConvertReport::default();
    let destination = match &job.mode {
        OutputMode::Folder { dir } => Some(dir.as_path()),
        _ => None,
    };
    // (file nguồn, folder gốc để tính đường dẫn tương đối khi ghi ra folder khác)
    let mut files: Vec<(PathBuf, Option<PathBuf>)> = Vec::new();
    for input in &job.inputs {
        if input.is_dir() {
            let mut found = Vec::new();
            walk(input, job.recursive, &job.extensions, destination, &mut found);
            files.extend(found.into_iter().map(|file| (file, Some(input.clone()))));
        } else if input.is_file() {
            files.push((input.clone(), None));
        } else {
            report.skipped.push(SkippedFile { path: input.display().to_string(), reason: "không tìm thấy".into() });
        }
    }
    let mut targets: HashSet<PathBuf> = HashSet::new();
    for (source, base) in files {
        let shown = source.display().to_string();
        let target = match &job.mode {
            OutputMode::Replace => source.clone(),
            OutputMode::Folder { dir } => {
                let relative = base
                    .as_deref()
                    .and_then(|base| source.strip_prefix(base).ok())
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| PathBuf::from(source.file_name().unwrap_or_default()));
                dir.join(relative)
            }
            OutputMode::Suffix { suffix } => {
                // Bản đã chuyển lần trước nằm cùng folder: không chuyển chồng thành `_gian_gian`.
                if stem_ends_with(&source, suffix) {
                    continue;
                }
                with_suffix(&source, suffix)
            }
        };
        if !targets.insert(target.clone()) {
            report.skipped.push(SkippedFile { path: shown, reason: "trùng tên file đích với file khác".into() });
            continue;
        }
        let bytes = match std::fs::read(&source) {
            Ok(bytes) => bytes,
            Err(error) => {
                report.skipped.push(SkippedFile { path: shown, reason: format!("không đọc được: {error}") });
                continue;
            }
        };
        let Some(decoded) = decode_text(&bytes) else {
            report.skipped.push(SkippedFile { path: shown, reason: "không phải file text".into() });
            continue;
        };
        let simplified = to_simplified(&decoded.text);
        let changed = !decoded.utf8 || simplified != decoded.text;
        if changed {
            report.converted += 1;
        } else {
            report.unchanged += 1;
            if target == source {
                continue;
            }
        }
        if let Some(parent) = target.parent() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                report.skipped.push(SkippedFile { path: shown, reason: format!("không tạo được folder đích: {error}") });
                continue;
            }
        }
        match write_atomic(&target, &simplified) {
            Ok(()) => report.written.push(target.display().to_string()),
            Err(error) => report.skipped.push(SkippedFile { path: shown, reason: error.to_string() }),
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn doi_theo_cum_tu_giu_tu_doc_tru_va_tro_tu_thanh_着() {
        assert_eq!(to_simplified("他著急地看著後臺，乾脆把乾淨的衣服晾乾。"), "他着急地看着后台，干脆把干净的衣服晾干。");
        assert_eq!(to_simplified("著名作家的著作，乾隆也讀過。"), "著名作家的著作，乾隆也读过。");
        assert_eq!(to_simplified("皇后在後宮裡等著，嘴裡唸著咒語。"), "皇后在后宫里等着，嘴里念着咒语。");
        assert_eq!(to_simplified("她吃了一個雪梨，用滑鼠點開。「走吧。」"), "她吃了一个雪梨，用鼠标点开。“走吧。”");
        assert_eq!(to_simplified("Tiếng Việt giữ nguyên, 显著 cũng vậy."), "Tiếng Việt giữ nguyên, 显著 cũng vậy.");
        assert_eq!(to_simplified("於是於夫人說：「鍾離先生，請。」"), "于是于夫人说：“钟离先生，请。”");
    }

    #[test]
    fn tro_tu_著_sat_tu_doc_tru_van_thanh_着() {
        assert_eq!(
            to_simplified("他們討論著新奇的事，做著作業，大著膽子翻譯著文件，編著辮子，依舊著，念著名字。"),
            "他们讨论着新奇的事，做着作业，大着胆子翻译着文件，编着辫子，依旧着，念着名字。"
        );
        assert_eq!(
            to_simplified("臭名昭著的土著讀過原著和名著，這部論著、譯著在學界著稱，成績顯著。"),
            "臭名昭著的土著读过原著和名著，这部论著、译著在学界著称，成绩显著。"
        );
    }

    #[test]
    fn giu_乾_ten_rieng_doi_乾_nghia_kho() {
        assert_eq!(to_simplified("大乾王朝的乾帝坐鎮乾元殿，乾坤一擲。"), "大乾王朝的乾帝坐镇乾元殿，乾坤一掷。");
        assert_eq!(to_simplified("乾國使者來到乾天宮，乾門弟子拜見乾皇。"), "乾国使者来到乾天宫，乾门弟子拜见乾皇。");
        assert_eq!(to_simplified("他擦乾眼淚，喝乾了酒，衣服晾乾了，乾脆走了。"), "他擦干眼泪，喝干了酒，衣服晾干了，干脆走了。");
        assert_eq!(to_simplified("嘴唇乾裂，乾巴巴地說，風乾天下。"), "嘴唇干裂，干巴巴地说，风干天下。");
        assert_eq!(to_simplified("大乾王朝"), "大乾王朝", "giản thể có tên 大乾 giữ nguyên");
    }

    #[test]
    fn nhan_phon_the_theo_ti_le_chu_phai_doi() {
        assert!(is_traditional("這是一本書，他說話很慢。"));
        assert!(!is_traditional("这是一本书，他说话很慢。"));
        assert!(!is_traditional("Không có chữ Hán."));
    }

    #[test]
    fn giai_ma_utf8_bom_gbk_big5_va_bo_file_nhi_phan() {
        assert_eq!(decode_text("中文".as_bytes()).unwrap(), Decoded { text: "中文".into(), encoding: "UTF-8", utf8: true });
        let bom = [&[0xEF, 0xBB, 0xBF][..], "中文".as_bytes()].concat();
        assert_eq!(decode_text(&bom).unwrap().text, "中文");
        assert!(!decode_text(&bom).unwrap().utf8, "có BOM thì ghi lại bỏ BOM");
        let (gbk, _, _) = encoding_rs::GBK.encode("第一章 少年的修炼之路，他慢慢地走向远方的山门。");
        let decoded = decode_text(&gbk).unwrap();
        assert_eq!((decoded.text.as_str(), decoded.encoding), ("第一章 少年的修炼之路，他慢慢地走向远方的山门。", "GBK"));
        let (big5, _, _) = encoding_rs::BIG5.encode("第一章 少年的修煉之路，他慢慢地走向遠方的山門。");
        let decoded = decode_text(&big5).unwrap();
        assert_eq!((decoded.text.as_str(), decoded.encoding), ("第一章 少年的修煉之路，他慢慢地走向遠方的山門。", "Big5"));
        assert!(decode_text(&[0x89, b'P', b'N', b'G', 0, 0, 1]).is_none());
    }

    #[test]
    fn chuyen_folder_ra_folder_khac_giu_cau_truc_va_bo_file_khong_phai_text() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("nguon");
        fs::create_dir_all(source.join("phan1")).unwrap();
        fs::write(source.join("a.txt"), "這是書").unwrap();
        fs::write(source.join("phan1").join("b.txt"), "这是书").unwrap();
        fs::write(source.join("anh.png"), [0x89, b'P', 0, 0]).unwrap();
        fs::write(source.join("c.md"), "後來").unwrap();
        let out = source.join("gian");
        let job = ConvertJob {
            inputs: vec![source.clone()],
            recursive: true,
            extensions: vec!["txt".into()],
            mode: OutputMode::Folder { dir: out.clone() },
        };
        let report = convert_files(&job);
        assert_eq!((report.converted, report.unchanged, report.written.len()), (1, 1, 2));
        assert_eq!(fs::read_to_string(out.join("a.txt")).unwrap(), "这是书");
        assert_eq!(fs::read_to_string(out.join("phan1").join("b.txt")).unwrap(), "这是书", "giản thể vẫn ghi bản sao");
        assert!(!out.join("c.md").exists(), "lọc theo đuôi");
        assert_eq!(fs::read_to_string(source.join("a.txt")).unwrap(), "這是書", "file gốc giữ nguyên");
        // Chạy lại: folder đích nằm trong folder nguồn không bị quét vào.
        assert_eq!(convert_files(&job).written.len(), 2);
    }

    #[test]
    fn ghi_de_chi_file_co_doi_va_hau_to_khong_chuyen_chong() {
        let dir = tempfile::tempdir().unwrap();
        let trad = dir.path().join("a.txt");
        let simp = dir.path().join("b.txt");
        fs::write(&trad, "這是書").unwrap();
        fs::write(&simp, "这是书").unwrap();
        let suffix = ConvertJob {
            inputs: vec![dir.path().to_path_buf()],
            recursive: false,
            extensions: vec!["txt".into()],
            mode: OutputMode::Suffix { suffix: "_gian".into() },
        };
        assert_eq!(convert_files(&suffix).written.len(), 2);
        assert_eq!(fs::read_to_string(dir.path().join("a_gian.txt")).unwrap(), "这是书");
        assert_eq!(convert_files(&suffix).written.len(), 2, "bản _gian không bị chuyển thành _gian_gian");
        assert!(!dir.path().join("a_gian_gian.txt").exists());

        let (big5, _, _) = encoding_rs::BIG5.encode("這是書");
        fs::write(&trad, &big5).unwrap();
        let replace = ConvertJob { inputs: vec![trad.clone(), simp.clone()], recursive: false, extensions: vec![], mode: OutputMode::Replace };
        let report = convert_files(&replace);
        assert_eq!((report.converted, report.unchanged, report.written.len()), (1, 1, 1));
        assert_eq!(fs::read_to_string(&trad).unwrap(), "这是书", "Big5 → giản thể UTF-8");
    }
}
