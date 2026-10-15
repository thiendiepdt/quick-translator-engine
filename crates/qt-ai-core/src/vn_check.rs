//! Quét nội dung nhạy cảm về Việt Nam trong một truyện — spec docs/superpowers/specs/2026-10-15-vn-sensitive-check-design.md.
//! Luật khớp ở `scan_chapter`; lưu + quét tăng dần ở `refresh` (Task 3).

use crate::error::{CoreError, Result};
use crate::paragraphs::paragraphs_of;
use crate::story::natural_chapter_compare;
use crate::story_fs::{list_raw_chapter_ids, read_text, write_atomic, StoryPaths};
use crate::vn_lexicon::{fnv64, Group, Lexicon, Level};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Từ đi kèm cho từ yếu (ngoài một mục strong bất kỳ cùng đoạn).
const COOCCUR: &[&str] = &[
    "主权", "领土", "领海", "岛礁", "群岛", "越南", "安南", "交趾", "chủ quyền", "lãnh thổ", "lãnh hải", "quần đảo",
    "Việt Nam", "An Nam", "Giao Chỉ",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Raw,
    Out,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub paragraph: usize,
    pub entry: usize,
    pub side: Side,
    pub matched: String,
    pub count: usize,
}

/// Đoạn của bản dịch = các dòng không rỗng — cùng cách `replace.rs` đánh số đoạn.
pub fn out_paragraphs(out: &str) -> Vec<String> {
    out.lines().map(str::trim).filter(|line| !line.is_empty()).map(str::to_string).collect()
}

/// `find_zh`/`find_vi` đã bỏ chỗ bị loại trừ nên không cần kiểm tra lại ở đây.
fn cooccur(lex: &Lexicon, texts: &[&str], entry: usize) -> bool {
    texts.iter().any(|text| COOCCUR.iter().any(|word| text.contains(word)))
        || texts.iter().any(|text| {
            lex.find_zh(text).into_iter().chain(lex.find_vi(text)).any(|hit| hit.entry != entry && !lex.entries[hit.entry].weak)
        })
}

/// Các chỗ khớp của một chương, gộp theo (đoạn, mục). `out` = None → chương chưa dịch, quét raw.
pub fn scan_chapter(lex: &Lexicon, raw: &str, out: Option<&str>) -> Vec<Finding> {
    let raw_paragraphs = paragraphs_of(raw);
    let mut found: BTreeMap<(usize, usize), Finding> = BTreeMap::new();
    let mut push = |paragraph: usize, entry: usize, side: Side, matched: &str| {
        found
            .entry((paragraph, entry))
            .and_modify(|finding| finding.count += 1)
            .or_insert(Finding { paragraph, entry, side, matched: matched.to_string(), count: 1 });
    };
    match out {
        Some(out) => {
            let out_paragraphs = out_paragraphs(out);
            let aligned = out_paragraphs.len() == raw_paragraphs.len();
            for (index, paragraph) in out_paragraphs.iter().enumerate() {
                let raw = if aligned { raw_paragraphs.get(index).map(String::as_str) } else { None };
                for hit in lex.find_vi(paragraph) {
                    if raw.is_some_and(|raw| !lex.mentions_zh(hit.entry, raw)) {
                        continue;
                    }
                    if lex.entries[hit.entry].weak && !cooccur(lex, &[paragraph, raw.unwrap_or("")], hit.entry) {
                        continue;
                    }
                    push(index, hit.entry, Side::Out, &paragraph[hit.start..hit.end]);
                }
            }
        }
        None => {
            for (index, paragraph) in raw_paragraphs.iter().enumerate() {
                for hit in lex.find_zh(paragraph) {
                    if lex.entries[hit.entry].weak && !cooccur(lex, &[paragraph], hit.entry) {
                        continue;
                    }
                    push(index, hit.entry, Side::Raw, &paragraph[hit.start..hit.end]);
                }
            }
        }
    }
    found.into_values().collect()
}

pub const VN_FILE: &str = "vn-check.json";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    #[default]
    Pending,
    Ok,
    Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiVerdict {
    pub label: String,
    pub reason: String,
    pub model: String,
    pub at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VnHit {
    /// `<chương>:<đoạn>:<mục>`.
    pub id: String,
    pub chapter: String,
    pub paragraph: usize,
    pub entry: String,
    pub group: Group,
    pub level: Level,
    pub side: Side,
    pub matched: String,
    pub count: usize,
    /// Hash đoạn raw + đoạn dịch — đoạn đổi chữ thì trạng thái về pending.
    pub fingerprint: String,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub ai: Option<AiVerdict>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterMark {
    /// (kích thước, mtime ms) của raw và out.
    pub raw: (u64, u64),
    pub out: Option<(u64, u64)>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct VnFile {
    pub version: u32,
    pub lexicon_hash: String,
    pub chapters: BTreeMap<String, ChapterMark>,
    pub hits: Vec<VnHit>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VnSummary {
    pub hits: usize,
    /// Chưa xét + cần xử lý.
    pub open: usize,
    /// Số chương vừa đọc lại.
    pub scanned: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HitView {
    #[serde(flatten)]
    pub hit: VnHit,
    /// Đoạn trước, đoạn chứa, đoạn sau (có thể ít hơn ở đầu/cuối chương).
    pub raw: Vec<String>,
    pub out: Vec<String>,
    /// Vị trí đoạn chứa trong `raw` / `out`.
    pub focus: usize,
}

fn file_path(paths: &StoryPaths) -> PathBuf {
    paths.root.join(VN_FILE)
}

fn raw_path(paths: &StoryPaths, id: &str) -> PathBuf {
    paths.raw_dir.join(format!("{id}.txt"))
}

fn out_path(paths: &StoryPaths, id: &str) -> PathBuf {
    paths.out_dir.join(format!("{id}.txt"))
}

fn mark(path: &Path) -> Option<(u64, u64)> {
    let meta = std::fs::metadata(path).ok()?;
    let mtime = meta.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_millis() as u64;
    Some((meta.len(), mtime))
}

fn chapter_mark(paths: &StoryPaths, id: &str) -> ChapterMark {
    ChapterMark { raw: mark(&raw_path(paths, id)).unwrap_or_default(), out: mark(&out_path(paths, id)) }
}

pub fn load_file(paths: &StoryPaths) -> VnFile {
    read_text(&file_path(paths)).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
}

/// Tuần tự hoá mọi chu trình đọc → sửa → ghi `vn-check.json` trong cùng tiến trình (GUI chạy AI,
/// bấm nút và refresh trên nhiều luồng). Không chặn được tiến trình khác, vd. CLI `qt-ai accept`.
static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn lock() -> std::sync::MutexGuard<'static, ()> {
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// Như `load_file` nhưng dành cho chu trình ghi: file có mà không đọc/parse được thì chép sang
/// `vn-check.json.bad` trước, để lần ghi sau không xoá mất các quyết định cũ.
fn load_for_update(paths: &StoryPaths) -> Result<VnFile> {
    let path = file_path(paths);
    if !path.exists() {
        return Ok(VnFile::default());
    }
    if let Some(file) = read_text(&path).ok().and_then(|text| serde_json::from_str(&text).ok()) {
        return Ok(file);
    }
    let mut bad = path.as_os_str().to_owned();
    bad.push(".bad");
    let bad = PathBuf::from(bad);
    std::fs::copy(&path, &bad).map_err(CoreError::io(&bad))?;
    Ok(VnFile::default())
}

fn save_file(paths: &StoryPaths, file: &VnFile) -> Result<()> {
    let text = serde_json::to_string_pretty(file).expect("VnFile luôn serialize được");
    write_atomic(&file_path(paths), &format!("{text}\n"))
}

fn sort_hits(hits: &mut [VnHit]) {
    hits.sort_by(|a, b| {
        natural_chapter_compare(&a.chapter, &b.chapter).then(a.paragraph.cmp(&b.paragraph)).then(a.entry.cmp(&b.entry))
    });
}

pub fn open_count(file: &VnFile) -> usize {
    file.hits.iter().filter(|hit| hit.status != Status::Ok).count()
}

fn chapter_hits(lex: &Lexicon, id: &str, raw: &str, out: Option<&str>) -> Vec<VnHit> {
    let raw_paragraphs = paragraphs_of(raw);
    let out_list = out.map(out_paragraphs).unwrap_or_default();
    scan_chapter(lex, raw, out)
        .into_iter()
        .map(|finding| {
            let entry = &lex.entries[finding.entry];
            let raw_text = raw_paragraphs.get(finding.paragraph).map(String::as_str).unwrap_or("");
            let out_text = out_list.get(finding.paragraph).map(String::as_str).unwrap_or("");
            VnHit {
                id: format!("{id}:{}:{}", finding.paragraph, entry.id),
                chapter: id.to_string(),
                paragraph: finding.paragraph,
                entry: entry.id.clone(),
                group: entry.group,
                level: entry.level,
                side: finding.side,
                matched: finding.matched,
                count: finding.count,
                fingerprint: format!("{:016x}", fnv64(&format!("{raw_text}\u{1}{out_text}"))),
                status: Status::Pending,
                ai: None,
            }
        })
        .collect()
}

fn scan_one(paths: &StoryPaths, lex: &Lexicon, id: &str) -> Vec<VnHit> {
    let Ok(raw) = read_text(&raw_path(paths, id)) else { return vec![] };
    let out = read_text(&out_path(paths, id)).ok();
    chapter_hits(lex, id, &raw, out.as_deref())
}

/// Quét song song (≤ 8 luồng) như `replace.rs`.
fn scan_many(paths: &StoryPaths, lex: &Lexicon, ids: &[&str]) -> Result<Vec<VnHit>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(1, 8);
    let size = ids.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let workers: Vec<_> = ids
            .chunks(size)
            .map(|part| scope.spawn(move || part.iter().flat_map(|id| scan_one(paths, lex, id)).collect::<Vec<_>>()))
            .collect();
        let mut all = Vec::new();
        for worker in workers {
            all.extend(worker.join().map_err(|_| CoreError::Internal("luồng quét VN bị lỗi".into()))?);
        }
        Ok(all)
    })
}

/// Hit mới cùng id và cùng fingerprint với hit cũ thì giữ trạng thái + nhãn AI.
fn carry(mut hit: VnHit, previous: &HashMap<String, VnHit>) -> VnHit {
    if let Some(old) = previous.get(&hit.id) {
        if old.fingerprint == hit.fingerprint {
            hit.status = old.status;
            hit.ai = old.ai.clone();
        }
    }
    hit
}

/// Quét tăng dần cả truyện: chỉ đọc chương mới/đổi (theo kích thước + mtime); từ điển đổi thì quét lại toàn bộ.
pub fn refresh(paths: &StoryPaths, lex: &Lexicon) -> Result<VnSummary> {
    let _guard = lock();
    let old = load_for_update(paths)?;
    let reset = old.lexicon_hash != lex.hash;
    let ids = list_raw_chapter_ids(paths)?;
    let marks: Vec<(String, ChapterMark)> = ids.iter().map(|id| (id.clone(), chapter_mark(paths, id))).collect();
    let todo: Vec<&str> =
        marks.iter().filter(|(id, m)| reset || old.chapters.get(id) != Some(m)).map(|(id, _)| id.as_str()).collect();
    let fresh = scan_many(paths, lex, &todo)?;
    let todo_set: HashSet<&str> = todo.iter().copied().collect();
    let live: HashSet<&str> = ids.iter().map(String::as_str).collect();
    let previous: HashMap<String, VnHit> = old.hits.iter().map(|hit| (hit.id.clone(), hit.clone())).collect();
    let mut hits: Vec<VnHit> = old
        .hits
        .iter()
        .filter(|hit| !reset && live.contains(hit.chapter.as_str()) && !todo_set.contains(hit.chapter.as_str()))
        .cloned()
        .collect();
    hits.extend(fresh.into_iter().map(|hit| carry(hit, &previous)));
    sort_hits(&mut hits);
    let scanned = todo.len();
    let file = VnFile { version: 1, lexicon_hash: lex.hash.clone(), chapters: marks.into_iter().collect(), hits };
    let summary = VnSummary { hits: file.hits.len(), open: open_count(&file), scanned };
    if file != old {
        save_file(paths, &file)?;
    }
    Ok(summary)
}

/// Quét lại đúng một chương (khi chốt chương); trả số chỗ chưa xét + cần xử lý của chương đó.
/// Từ điển đổi → bỏ dấu các chương khác để lần `refresh` sau quét lại chúng.
pub fn check_chapter(paths: &StoryPaths, lex: &Lexicon, id: &str) -> Result<usize> {
    let _guard = lock();
    let mut file = load_for_update(paths)?;
    if file.lexicon_hash != lex.hash {
        file.chapters.clear();
        file.lexicon_hash = lex.hash.clone();
    }
    let previous: HashMap<String, VnHit> =
        file.hits.iter().filter(|hit| hit.chapter == id).map(|hit| (hit.id.clone(), hit.clone())).collect();
    file.hits.retain(|hit| hit.chapter != id);
    let fresh: Vec<VnHit> = scan_one(paths, lex, id).into_iter().map(|hit| carry(hit, &previous)).collect();
    let open = fresh.iter().filter(|hit| hit.status != Status::Ok).count();
    file.hits.extend(fresh);
    sort_hits(&mut file.hits);
    file.chapters.insert(id.to_string(), chapter_mark(paths, id));
    file.version = 1;
    save_file(paths, &file)?;
    Ok(open)
}

pub fn set_status(paths: &StoryPaths, ids: &[String], status: Status) -> Result<usize> {
    let _guard = lock();
    let mut file = load_for_update(paths)?;
    let mut changed = 0;
    for hit in file.hits.iter_mut().filter(|hit| ids.contains(&hit.id)) {
        if hit.status != status {
            hit.status = status;
            changed += 1;
        }
    }
    if changed > 0 {
        save_file(paths, &file)?;
    }
    Ok(changed)
}

pub fn set_ai(paths: &StoryPaths, verdicts: &[(String, AiVerdict)]) -> Result<usize> {
    let _guard = lock();
    let mut file = load_for_update(paths)?;
    let mut changed = 0;
    for (id, verdict) in verdicts {
        if let Some(hit) = file.hits.iter_mut().find(|hit| &hit.id == id) {
            hit.ai = Some(verdict.clone());
            changed += 1;
        }
    }
    if changed > 0 {
        save_file(paths, &file)?;
    }
    Ok(changed)
}

/// Số chỗ chưa xét + cần xử lý; None khi truyện chưa từng quét (không có file).
pub fn flag_count(paths: &StoryPaths) -> Option<usize> {
    file_path(paths).exists().then(|| open_count(&load_file(paths)))
}

/// Mọi chỗ khớp kèm đoạn trước/sau (raw và bản dịch) cho tab Kiểm tra VN.
pub fn report(paths: &StoryPaths) -> Vec<HitView> {
    let file = load_file(paths);
    let mut cache: HashMap<String, (Vec<String>, Vec<String>)> = HashMap::new();
    file.hits
        .into_iter()
        .map(|hit| {
            let (raw, out) = cache.entry(hit.chapter.clone()).or_insert_with(|| {
                (
                    read_text(&raw_path(paths, &hit.chapter)).map(|t| paragraphs_of(&t)).unwrap_or_default(),
                    read_text(&out_path(paths, &hit.chapter)).map(|t| out_paragraphs(&t)).unwrap_or_default(),
                )
            });
            let lo = hit.paragraph.saturating_sub(1);
            let take = hit.paragraph + 2 - lo;
            let window = |list: &Vec<String>| list.iter().skip(lo).take(take).cloned().collect::<Vec<_>>();
            let (raw, out) = (window(raw), window(out));
            HitView { focus: hit.paragraph - lo, raw, out, hit }
        })
        .collect()
}
