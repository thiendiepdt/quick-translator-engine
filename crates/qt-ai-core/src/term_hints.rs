//! Gợi ý thuật ngữ lặp nhiều chương mà glossary chưa có (thẩm định 11/2026, B2): model hay coi thuật ngữ ngành có
//! thật là từ thường nên không khai — bac-si: 规培 72 chương, 总住院 64, 三甲 46 mà không có mục, mỗi chương dịch
//! một kiểu. Mỗi chương nối vào chỉ dẫn khai glossary tối đa 10 cụm: có trong chương này, có mặt ≥ max(10, 5%) số
//! chương raw, chưa có trong glossary / bảng prompt / bảng thành ngữ, không phải từ thường (IDF jieba < 9,5, sinh
//! bằng `apps/qt-ai-cli/scripts/gen-common-words.mjs`), không chứa hư từ, và đứng độc lập như một từ (≥3 chữ khác
//! nhau đứng trước và ≥3 chữ khác nhau đứng sau — loại mảnh vắt ngang hai từ kiểu 凡看 trong 林凡看). Model lọc lần
//! cuối: chỉ dẫn nói rõ từ thường thì bỏ qua.
//!
//! Số chương của mỗi cụm tính một lần và lưu ở `work/term-stats.json`; raw đổi (thêm / bớt / sửa file) thì tính lại.

use crate::story::StoryConfig;
use crate::story_fs::StoryPaths;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

static COMMON: LazyLock<HashSet<&'static str>> = LazyLock::new(|| include_str!("../data/common-words.txt").lines().collect());

/// Cụm truyện mạng nào cũng lặp nhiều chương (看向, 回头看, 没来得及) — sinh từ thư viện mẫu bằng
/// `apps/qt-ai-cli/scripts/gen-novel-common.mjs`; thuật ngữ riêng của truyện (规培, 主刀) chỉ lặp ở ít truyện.
static NOVEL_COMMON: LazyLock<HashSet<&'static str>> =
    LazyLock::new(|| include_str!("../data/novel-common.txt").lines().collect());

/// Từ phổ thông của tiếng Trung (关系, 后来…): dịch thế nào là tuỳ câu, không coi là thuật ngữ cố định.
pub(crate) fn is_common_word(word: &str) -> bool {
    COMMON.contains(word)
}

/// Chữ hư / chữ quá phổ biến: cụm chứa chúng gần như luôn là mảnh câu, không phải thuật ngữ.
const FUNCTION_CHARS: &str = "的了着过是在我你他她它们这那就也都还又和与说道一不有个人么吗呢吧啊么之而";

/// Hư từ / chữ đệm hay dính vào từ thường thành cụm giả (准备好, 直接把, 能让, 但却).
const SPLIT_CHARS: &str = "的了着过是在我你他她它们这那就也都还又和与说道一不有个人么吗呢吧啊之而到好把却让能但被将给从向对会要想得地所为再才更最很太已连没竟便只刚";

const MAX_HINTS: usize = 10;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct TermStats {
    /// Dấu vân tay của raw/ (số file + tổng byte + thời điểm sửa mới nhất) để biết khi nào tính lại.
    fingerprint: String,
    chapters: usize,
    /// Cụm phổ biến đứng độc lập: số chương có mặt.
    terms: HashMap<String, usize>,
}

fn han(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c)
}

fn raw_files(paths: &StoryPaths) -> Vec<std::path::PathBuf> {
    let mut files: Vec<_> = std::fs::read_dir(&paths.raw_dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
        .collect();
    files.sort();
    files
}

fn fingerprint(files: &[std::path::PathBuf]) -> String {
    let mut bytes = 0u64;
    let mut newest = 0u128;
    for file in files {
        if let Ok(meta) = std::fs::metadata(file) {
            bytes += meta.len();
            let modified = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok());
            newest = newest.max(modified.map_or(0, |d| d.as_millis()));
        }
    }
    format!("{}:{bytes}:{newest}", files.len())
}

/// Cụm n chữ Hán liền nhau trong một chương (mỗi cụm tính một lần).
fn grams(chars: &[char], n: usize) -> HashSet<String> {
    chars.windows(n).filter(|window| window.iter().all(|c| han(*c))).map(|window| window.iter().collect()).collect()
}

fn build(paths: &StoryPaths, files: &[std::path::PathBuf]) -> TermStats {
    let texts: Vec<Vec<char>> = files.iter().filter_map(|file| std::fs::read_to_string(file).ok()).map(|t| t.chars().collect()).collect();
    let chapters = texts.len();
    let threshold = (chapters * 5).div_ceil(100).max(10);
    // Đếm theo bậc: cụm n+1 chữ chỉ được đếm khi hai cụm n chữ bên trong đã phổ biến.
    let mut frequent: HashMap<String, usize> = HashMap::new();
    let mut previous: HashSet<String> = HashSet::new();
    for n in 2..=4 {
        let mut df: HashMap<String, usize> = HashMap::new();
        for chars in &texts {
            for gram in grams(chars, n) {
                if n > 2 {
                    let inner: Vec<char> = gram.chars().collect();
                    let (head, tail): (String, String) = (inner[..n - 1].iter().collect(), inner[1..].iter().collect());
                    if !previous.contains(&head) || !previous.contains(&tail) {
                        continue;
                    }
                }
                *df.entry(gram).or_default() += 1;
            }
        }
        previous = df.into_iter().filter(|(_, count)| *count >= threshold).map(|(gram, count)| {
            frequent.insert(gram.clone(), count);
            gram
        }).collect();
    }
    // Đứng độc lập: đếm chữ khác nhau đứng ngay trước / sau mỗi cụm phổ biến (đầu, cuối câu tính là một kiểu).
    let mut left: HashMap<&str, HashSet<char>> = HashMap::new();
    let mut right: HashMap<&str, HashSet<char>> = HashMap::new();
    // Lần xuất hiện mà chữ đầu dính chữ trước (hoặc chữ cuối dính chữ sau) thành từ thường: 桌+子上, 抬+头看.
    let mut seen: HashMap<&str, (usize, usize)> = HashMap::new();
    let keys: HashSet<&str> = frequent.keys().map(String::as_str).collect();
    for chars in &texts {
        for start in 0..chars.len() {
            for n in 2..=4 {
                let Some(window) = chars.get(start..start + n) else { break };
                let gram: String = window.iter().collect();
                let Some(&key) = keys.get(gram.as_str()) else { continue };
                let before = start.checked_sub(1).map_or('^', |i| if han(chars[i]) { chars[i] } else { '^' });
                let after = chars.get(start + n).map_or('$', |c| if han(*c) { *c } else { '$' });
                let l = left.entry(key).or_default();
                if l.len() < 3 {
                    l.insert(before);
                }
                let r = right.entry(key).or_default();
                if r.len() < 3 {
                    r.insert(after);
                }
                let glued_left = before != '^' && COMMON.contains([before, window[0]].iter().collect::<String>().as_str());
                let glued_right = after != '$' && COMMON.contains([window[n - 1], after].iter().collect::<String>().as_str());
                let entry = seen.entry(key).or_default();
                entry.0 += 1;
                entry.1 += usize::from(glued_left || glued_right);
            }
        }
    }
    let free = |gram: &str| left.get(gram).is_some_and(|s| s.len() >= 3) && right.get(gram).is_some_and(|s| s.len() >= 3);
    let glued = |gram: &str| seen.get(gram).is_some_and(|(total, glued)| glued * 2 > *total);
    // Mảnh của một từ thường dài hơn (看起 ⊂ 看起来, 来越 ⊂ 越来越, 到时 ⊂ 到时候).
    let inside_common = |gram: &str| COMMON.iter().any(|word| word.len() > gram.len() && word.contains(gram));
    // Số, kể cả số + đơn vị đo (百丈, 数十丈, 三千里).
    let numeric = |gram: &str| {
        let digits = "一二三四五六七八九十百千万亿两几半数";
        let chars: Vec<char> = gram.chars().collect();
        let body = match chars.split_last() {
            Some((last, rest)) if !rest.is_empty() && "丈里米尺寸斤年月天日次个人名位条件块步".contains(*last) => rest,
            _ => &chars[..],
        };
        body.iter().all(|c| digits.contains(*c))
    };
    // Ghép từ hai mảnh thường (能+让, 准备+好, 直接+把, 感知+到): cả hai nửa đều là từ thường hoặc một hư từ.
    let piece = |part: &[char]| {
        let text: String = part.iter().collect();
        (part.len() == 1 && SPLIT_CHARS.contains(part[0])) || (part.len() > 1 && COMMON.contains(text.as_str()))
    };
    let compound = |gram: &str| {
        let chars: Vec<char> = gram.chars().collect();
        (1..chars.len()).any(|at| piece(&chars[..at]) && piece(&chars[at..]))
    };
    let terms = frequent
        .iter()
        .filter(|(gram, _)| free(gram) && !glued(gram) && !numeric(gram) && !inside_common(gram) && !compound(gram))
        .map(|(gram, count)| (gram.clone(), *count))
        .collect();
    let _ = paths;
    TermStats { fingerprint: fingerprint(files), chapters, terms }
}

fn load_or_build(paths: &StoryPaths) -> TermStats {
    let files = raw_files(paths);
    let cache = paths.work_dir.join("term-stats.json");
    let print = fingerprint(&files);
    if let Some(stats) = std::fs::read_to_string(&cache).ok().and_then(|text| serde_json::from_str::<TermStats>(&text).ok()) {
        if stats.fingerprint == print {
            return stats;
        }
    }
    let stats = build(paths, &files);
    if std::fs::create_dir_all(&paths.work_dir).is_ok() {
        let _ = std::fs::write(&cache, serde_json::to_string(&stats).unwrap_or_default());
    }
    stats
}

/// Cụm chồng mép với khoá đã có (到卫 / 以卫 với tên 卫渊): đuôi cụm là đầu khoá hoặc đầu cụm là đuôi khoá.
fn overlaps(gram: &str, key: &str) -> bool {
    if key.chars().count() < 2 {
        return false;
    }
    let g: Vec<char> = gram.chars().collect();
    let k: Vec<char> = key.chars().collect();
    (1..g.len().min(k.len())).any(|n| g[g.len() - n..] == k[..n] || g[..n] == k[k.len() - n..])
}

/// Dòng gợi ý cho chỉ dẫn khai glossary của chương có raw `source`; rỗng khi không có cụm nào.
pub fn hint_line(paths: &StoryPaths, story: &StoryConfig, source: &str) -> String {
    let stats = load_or_build(paths);
    if stats.terms.is_empty() {
        return String::new();
    }
    let known: Vec<&str> = story.glossary.values().flat_map(|group| group.keys()).map(String::as_str).collect();
    let table = crate::glossary::prompt_table_keys(crate::prompt::base_prompt(&story.genre));
    let idioms = crate::base::BaseStore::from_env().idioms();
    let mut hits: Vec<(&str, usize)> = stats
        .terms
        .iter()
        .filter(|(gram, _)| source.contains(gram.as_str()))
        .filter(|(gram, _)| !COMMON.contains(gram.as_str()) && !gram.chars().any(|c| FUNCTION_CHARS.contains(c)))
        .filter(|(gram, _)| !NOVEL_COMMON.contains(gram.as_str()))
        .filter(|(gram, _)| !known.iter().any(|key| key.contains(gram.as_str()) || gram.contains(key) || overlaps(gram, key)))
        .filter(|(gram, _)| !table.contains(gram.as_str()) && !idioms.keys().any(|idiom| idiom.contains(gram.as_str())))
        .map(|(gram, count)| (gram.as_str(), *count))
        .collect();
    // Cụm nằm trong cụm dài hơn gần cùng số chương thì chỉ giữ cụm dài (规培 / 规培生).
    let all = hits.clone();
    hits.retain(|(gram, count)| !all.iter().any(|(other, n)| other.len() > gram.len() && other.contains(gram) && n * 10 >= count * 9));
    hits.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    hits.truncate(MAX_HINTS);
    if hits.is_empty() {
        return String::new();
    }
    let list: Vec<String> = hits.iter().map(|(gram, count)| format!("{gram} ({count} chương)")).collect();
    format!(
        "Cụm lặp nhiều chương chưa có trong từ điển: {} — chỉ khai khi là thuật ngữ, chỉ số, chức danh hay tên gọi cố định của truyện (common hoặc đúng nhóm, target chép đúng cách bạn vừa dịch); từ tả động tác, màu sắc, cảm giác, đồ vật thường (染成, 淡金色, 蜷缩, 指关节) là từ thường — bỏ qua, không khai.",
        list.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goi_y_thuat_ngu_lap_doc_lap_bo_tu_thuong_va_manh_cau() {
        let dir = tempfile::tempdir().unwrap();
        let paths = crate::story_fs::story_paths(dir.path());
        std::fs::create_dir_all(&paths.raw_dir).unwrap();
        // 12 chương: 规培 đứng với nhiều chữ trước / sau khác nhau; 林凡 + 看 luôn đi liền (mảnh 凡看 không độc lập).
        let lefts = ["研", "和", "新", "老", "名", "位"];
        let rights = ["生", "期", "医", "制", "班", "楼"];
        for i in 0..12 {
            let text = format!(
                "{}规培{}很忙。林凡看着窗外。时候到了。{}看向{}。{}连自己{}。",
                lefts[i % 6],
                rights[i % 6],
                rights[i % 6],
                lefts[i % 6],
                lefts[(i + 1) % 6],
                rights[(i + 2) % 6]
            );
            std::fs::write(paths.raw_dir.join(format!("{i:04}.txt")), text).unwrap();
        }
        let story = StoryConfig::normalize(&serde_json::json!({ "name": "t" }));
        let line = hint_line(&paths, &story, "今天规培的人很多。林凡看着。他看向窗外。连自己都不信。");
        assert!(!line.contains("连自己"), "hư từ + từ thường: {line}");
        assert!(line.contains("规培 (12 chương)"), "{line}");
        assert!(!line.contains("凡看"), "{line}");
        assert!(!line.contains("时候"), "từ thường: {line}");
        assert!(!line.contains("看向"), "cụm truyện mạng nào cũng lặp: {line}");
        // Đã có trong glossary thì không gợi ý.
        let mut known = story.clone();
        known.glossary.entry("common".into()).or_default().insert("规培".into(), "đào tạo chuẩn hóa".into());
        assert!(!hint_line(&paths, &known, "今天规培的人很多。").contains("规培"));
    }
}
