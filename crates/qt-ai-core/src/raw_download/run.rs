//! Một lượt tải raw (spec 10): mục lục → khớp chương đã có → chọn chương theo chế độ → gọi extension → ghi
//! `raw/` theo mẫu tên, cập nhật sổ chương ngay sau mỗi chương (huỷ hay tắt app thì lần sau chỉ tải phần còn
//! thiếu) → thử lại lỗi tạm. Mỗi lượt một môi trường JS mới (`qt_ext::Session`).

use super::content::prepare;
use super::matching::{match_existing, Existing, MatchedBy, Matching, Mismatch, TocRef};
use super::registry::Installed;
use super::template::{guess_template, render, sorts_after, unique_stem, validate_template, NameParts, DEFAULT_TEMPLATE};
use super::{effective_raw_url, load_download, save_download, DownloadConfig};
use crate::error::{CoreError, Result};
use crate::story::StoryConfig;
use crate::story_fs::{list_raw_chapter_ids, load_story_config, read_raw_chapter, resolve_root, story_paths, write_atomic, StoryPaths};
use qt_ext::{check_setting, ChapterFormat, ExtError, LoginRecord, Scope, Session, SessionOptions, TocItem};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// Từ sau chương cuối đã có tới hết, bỏ chương khoá.
    New,
    Selected(Vec<String>),
    Single(String),
}

impl Mode {
    fn label(&self) -> &'static str {
        match self {
            Mode::New => "new",
            Mode::Selected(_) => "selected",
            Mode::Single(_) => "single",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RowStatus {
    Have,
    Missing,
    Locked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TocRow {
    pub index: usize,
    pub id: String,
    pub title: String,
    pub locked: bool,
    pub file: Option<String>,
    pub status: RowStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TocView {
    pub rows: Vec<TocRow>,
    pub by: MatchedBy,
    pub last_index: Option<usize>,
    pub mismatches: Vec<Mismatch>,
    pub unmatched_files: usize,
    /// Mẫu đang dùng cho file mới.
    pub template: String,
    /// Tên file của chương kế tiếp và nó có sắp sau file cuối đã có không.
    pub next_name: Option<String>,
    pub next_sorts_after: bool,
}

#[derive(Clone)]
pub struct RunOptions {
    pub extension: Installed,
    pub session: SessionOptions,
    /// "Chương cuối đã có là mục số K" (từ 1); `None` = theo sổ hoặc số file raw.
    pub anchor: Option<usize>,
    /// Chờ trước mỗi lượt thử lại lỗi tạm.
    pub retry_delays: Vec<Duration>,
}

impl RunOptions {
    pub fn new(extension: Installed) -> Self {
        let session = SessionOptions { allow_local: extension.dev, ..Default::default() };
        RunOptions { extension, session, anchor: None, retry_delays: vec![Duration::from_secs(2), Duration::from_secs(6)] }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ChapterResult {
    Saved { file: String, short: bool, replaced: bool },
    Locked,
    Failed { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Toc { total: usize },
    Chapter { done: usize, total: usize, index: usize, title: String, result: ChapterResult },
    Log(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Failure {
    pub index: usize,
    pub title: String,
    pub reason: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub saved: Vec<usize>,
    pub short: Vec<usize>,
    pub locked: Vec<usize>,
    pub failed: Vec<Failure>,
    pub cancelled: bool,
}

fn ext_error(error: ExtError) -> CoreError {
    CoreError::InvalidState(format!("extension: {error}"))
}

fn retryable(error: &ExtError) -> bool {
    matches!(error, ExtError::Network(_) | ExtError::Timeout(_) | ExtError::Extension { retry: true, .. })
}

struct Story {
    paths: StoryPaths,
    config: StoryConfig,
    download: DownloadConfig,
    raw_url: String,
    existing: Vec<Existing>,
}

fn first_line(text: &str) -> String {
    text.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("").to_string()
}

fn load_story(root: &Path) -> Result<Story> {
    let paths = story_paths(&resolve_root(root));
    let config = load_story_config(&paths)?;
    let download = load_download(&paths);
    let raw_url = effective_raw_url(&download, &config);
    if raw_url.is_empty() {
        return Err(CoreError::InvalidState("Truyện chưa có link tải raw (hoặc link nguồn).".into()));
    }
    let mut existing = Vec::new();
    for id in list_raw_chapter_ids(&paths)? {
        let title = read_raw_chapter(&paths, &id).map(|t| first_line(&t)).unwrap_or_default();
        existing.push(Existing { id, title });
    }
    Ok(Story { paths, config, download, raw_url, existing })
}

/// Mẫu cho file mới: mẫu của truyện → đoán từ tên cũ → mẫu chung.
pub fn effective_template(download: &DownloadConfig, existing_ids: &[String]) -> String {
    let own = download.file_template.trim();
    if !own.is_empty() && validate_template(own).is_ok() {
        return own.to_string();
    }
    guess_template(existing_ids).unwrap_or_else(|| DEFAULT_TEMPLATE.to_string())
}

/// `ctx.settings`: mỗi trường lấy giá trị đã lưu (chung trong `<config>`, theo truyện trong `download.json`), không
/// có thì `default` của manifest, không có nữa thì `null`.
fn settings_for(extension: &Installed, download: &DownloadConfig, global: &Map<String, Value>) -> Value {
    let manifest = &extension.package.manifest;
    let own = download.settings.get(&manifest.id);
    let mut settings = Map::new();
    for field in &manifest.settings {
        let saved = match field.scope {
            Scope::Story => own.and_then(|o| o.get(&field.key)).and_then(|v| check_setting(field, v).ok()),
            Scope::Global => global.get(&field.key).cloned(),
        };
        settings.insert(field.key.clone(), saved.or_else(|| field.default.clone()).unwrap_or(Value::Null));
    }
    Value::Object(settings)
}

/// Cài đặt chung đã lưu của extension (kể cả secret); không có `ExtStore` thì rỗng.
fn global_settings(opts: &RunOptions) -> Result<Map<String, Value>> {
    match &opts.session.store {
        Some(store) => store.settings(&opts.extension.package.manifest).map_err(ext_error),
        None => Ok(Map::new()),
    }
}

fn current_of(story: &Story, toc: Option<&[TocItem]>, matching: Option<&Matching>) -> Value {
    let (Some(toc), Some(matching)) = (toc, matching) else {
        return match story.existing.last() {
            None => Value::Null,
            Some(last) => {
                let id = story.download.chapters.iter().find(|(_, file)| **file == last.id).map(|(id, _)| id.clone());
                json!({ "index": story.existing.len(), "id": id, "title": last.title })
            }
        };
    };
    match matching.last_index {
        None => Value::Null,
        Some(index) => {
            let file = matching.files[index - 1].as_deref().unwrap_or("");
            let title = story.existing.iter().find(|e| e.id == file).map(|e| e.title.clone()).unwrap_or_default();
            json!({ "index": index, "id": toc[index - 1].id, "title": title })
        }
    }
}

/// `ctx` cho extension; `auth` và `session` do `qt_ext::Session` chèn.
fn build_ctx(story: &Story, extension: &Installed, global: &Map<String, Value>, current: Value, mode: &str, ids: &[String]) -> Value {
    json!({
        "sourceUrl": story.config.source_url,
        "rawUrl": story.raw_url,
        "story": { "name": story.config.name, "chapterCount": story.existing.len(), "current": current },
        "request": { "mode": mode, "ids": ids },
        "settings": settings_for(extension, &story.download, global),
    })
}

fn do_match(story: &Story, toc: &[TocItem], anchor: Option<usize>) -> Matching {
    let refs: Vec<TocRef> = toc.iter().map(|t| TocRef { id: &t.id, title: &t.title }).collect();
    match_existing(&refs, &story.download.chapters, &story.existing, anchor)
}

fn view(story: &Story, toc: &[TocItem], matching: &Matching) -> TocView {
    let rows = toc
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let file = matching.files[i].clone();
            let status = if file.is_some() {
                RowStatus::Have
            } else if item.locked {
                RowStatus::Locked
            } else {
                RowStatus::Missing
            };
            TocRow { index: i + 1, id: item.id.clone(), title: item.title.clone(), locked: item.locked, file, status }
        })
        .collect();
    let ids: Vec<String> = story.existing.iter().map(|e| e.id.clone()).collect();
    let template = effective_template(&story.download, &ids);
    let next = matching.last_index.unwrap_or(0) + 1;
    let next_name = toc.get(next - 1).map(|item| render(&template, &NameParts { index: next, id: &item.id, title: &item.title, volume: item.volume.as_deref() }));
    let next_sorts_after = match (&next_name, ids.last()) {
        (Some(name), Some(last)) => sorts_after(name, last),
        _ => true,
    };
    TocView {
        rows,
        by: matching.by,
        last_index: matching.last_index,
        mismatches: matching.mismatches.clone(),
        unmatched_files: matching.unmatched_files,
        template,
        next_name,
        next_sorts_after,
    }
}

fn start_session(opts: &RunOptions, cancel: &Arc<AtomicBool>) -> Result<Session> {
    let mut session = opts.session.clone();
    session.engine.cancel = cancel.clone();
    session.allow_local = session.allow_local || opts.extension.dev;
    Session::start(opts.extension.package.clone(), session).map_err(ext_error)
}

/// Tải mục lục và khớp với raw đã có. Trả cả mục lục để lượt tải dùng lại, khỏi gọi `toc` lần nữa.
pub fn load_toc(root: &Path, opts: &RunOptions, cancel: Arc<AtomicBool>) -> Result<(Vec<TocItem>, TocView)> {
    let story = load_story(root)?;
    let global = global_settings(opts)?;
    let session = start_session(opts, &cancel)?;
    let ctx = build_ctx(&story, &opts.extension, &global, current_of(&story, None, None), "toc", &[]);
    let toc = session.toc(&ctx).map_err(ext_error)?;
    let matching = do_match(&story, &toc, opts.anchor);
    let shown = view(&story, &toc, &matching);
    Ok((toc, shown))
}

/// Nút "Đăng nhập": gọi `login` của extension ngay. `root` = truyện đang mở (để `ctx` có link và cài đặt của
/// truyện); `None` khi đăng nhập từ trang quản lý extension. `None` trả về = extension không có hàm `login`.
pub fn login(root: Option<&Path>, opts: &RunOptions, cancel: Arc<AtomicBool>) -> Result<Option<LoginRecord>> {
    let global = global_settings(opts)?;
    let ctx = match root {
        Some(root) => {
            let story = load_story(root)?;
            build_ctx(&story, &opts.extension, &global, current_of(&story, None, None), "login", &[])
        }
        None => json!({
            "sourceUrl": "", "rawUrl": "", "story": Value::Null,
            "request": { "mode": "login", "ids": [] },
            "settings": settings_for(&opts.extension, &DownloadConfig::default(), &global),
        }),
    };
    start_session(opts, &cancel)?.login(&ctx).map_err(ext_error)
}

fn backup_name(stem: &str) -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    format!("{stem}.{secs}.txt")
}

fn write_chapter(paths: &StoryPaths, stem: &str, text: &str) -> Result<bool> {
    let target = paths.raw_dir.join(format!("{stem}.txt"));
    let replaced = target.exists();
    if replaced {
        let dir = paths.work_dir.join("raw-backup");
        std::fs::create_dir_all(&dir).map_err(CoreError::io(&dir))?;
        let backup = dir.join(backup_name(stem));
        std::fs::copy(&target, &backup).map_err(CoreError::io(&backup))?;
    }
    std::fs::create_dir_all(&paths.raw_dir).map_err(CoreError::io(&paths.raw_dir))?;
    write_atomic(&target, text)?;
    Ok(replaced)
}

/// `toc`: mục lục từ `load_toc` (cùng truyện, cùng extension); `None` thì gọi `toc` trong lượt này.
pub fn download(root: &Path, opts: &RunOptions, toc: Option<Vec<TocItem>>, mode: Mode, cancel: Arc<AtomicBool>, on_event: &mut dyn FnMut(Event)) -> Result<Summary> {
    let mut story = load_story(root)?;
    let global = global_settings(opts)?;
    let session = start_session(opts, &cancel)?;
    let toc = match toc {
        Some(toc) => toc,
        None => {
            let ctx = build_ctx(&story, &opts.extension, &global, current_of(&story, None, None), "toc", &[]);
            session.toc(&ctx).map_err(ext_error)?
        }
    };
    let matching = do_match(&story, &toc, opts.anchor);

    // Lượt đầu với truyện cũ (khớp theo thứ tự): ghi các cặp đã khớp vào sổ để lần sau khớp theo mã.
    if matching.by == MatchedBy::Order {
        for (item, file) in toc.iter().zip(&matching.files) {
            if let Some(file) = file {
                story.download.chapters.insert(item.id.clone(), file.clone());
            }
        }
        save_download(&story.paths, &story.download)?;
    }

    let last = matching.last_index.unwrap_or(0);
    let targets: Vec<(usize, TocItem)> = match &mode {
        Mode::New => toc.iter().enumerate().filter(|(i, t)| i + 1 > last && !t.locked).map(|(i, t)| (i + 1, t.clone())).collect(),
        Mode::Selected(ids) => toc.iter().enumerate().filter(|(_, t)| ids.contains(&t.id)).map(|(i, t)| (i + 1, t.clone())).collect(),
        Mode::Single(id) => toc.iter().enumerate().filter(|(_, t)| &t.id == id).map(|(i, t)| (i + 1, t.clone())).collect(),
    };
    let ids: Vec<String> = targets.iter().map(|(_, t)| t.id.clone()).collect();
    let current = current_of(&story, Some(&toc), Some(&matching));
    let ctx = build_ctx(&story, &opts.extension, &global, current, mode.label(), &ids);

    let total = targets.len();
    on_event(Event::Toc { total });
    let existing_ids: Vec<String> = story.existing.iter().map(|e| e.id.clone()).collect();
    let template = effective_template(&story.download, &existing_ids);
    let mut taken: HashSet<String> = existing_ids.into_iter().collect();
    // Sổ chương giữ theo thứ tự mục lục (chương xong trước sau không đều khi tải song song).
    let position: std::collections::HashMap<&str, usize> = toc.iter().enumerate().map(|(i, t)| (t.id.as_str(), i)).collect();
    let mut summary = Summary::default();
    let mut done = 0;
    let mut pending = targets;
    let delays: Vec<Option<Duration>> = std::iter::once(None).chain(opts.retry_delays.iter().copied().map(Some)).collect();

    for (round, delay) in delays.iter().enumerate() {
        if pending.is_empty() || cancel.load(Ordering::SeqCst) {
            break;
        }
        if let Some(delay) = delay {
            on_event(Event::Log(format!("Thử lại {} chương lỗi sau {} giây", pending.len(), delay.as_secs())));
            let until = std::time::Instant::now() + *delay;
            while std::time::Instant::now() < until && !cancel.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(50).min(until - std::time::Instant::now()));
            }
        }
        let last_round = round + 1 == delays.len();
        let batch = std::mem::take(&mut pending);
        let mut write_error: Option<CoreError> = None;
        session.chapters(&ctx, &batch, &mut |outcome| {
            let Some((index, item)) = batch.iter().find(|(i, _)| *i == outcome.index) else {
                return;
            };
            let result = match outcome.result {
                Ok(chapter) => {
                    let title = chapter.title.clone().unwrap_or_else(|| item.title.clone());
                    match prepare(&chapter.content, chapter.format == ChapterFormat::Html, &title) {
                        None => ChapterResult::Failed { reason: "nội dung trống".into() },
                        Some(prepared) => {
                            let stem = match matching.files[index - 1].clone() {
                                Some(file) => file,
                                None => {
                                    let name = render(&template, &NameParts { index: *index, id: &item.id, title: &item.title, volume: item.volume.as_deref() });
                                    unique_stem(name, &taken)
                                }
                            };
                            match write_chapter(&story.paths, &stem, &prepared.text) {
                                Ok(replaced) => {
                                    taken.insert(stem.clone());
                                    story.download.chapters.insert(item.id.clone(), stem.clone());
                                    story.download.chapters.sort_by_cached_key(|id, _| position.get(id.as_str()).copied().unwrap_or(usize::MAX));
                                    if let Err(e) = save_download(&story.paths, &story.download) {
                                        write_error.get_or_insert(e);
                                    }
                                    ChapterResult::Saved { file: stem, short: prepared.short, replaced }
                                }
                                Err(e) => {
                                    let reason = e.to_string();
                                    write_error.get_or_insert(e);
                                    ChapterResult::Failed { reason }
                                }
                            }
                        }
                    }
                }
                Err(ExtError::Locked(_)) => ChapterResult::Locked,
                Err(error) if retryable(&error) && !last_round && !cancel.load(Ordering::SeqCst) => {
                    pending.push((*index, item.clone()));
                    return;
                }
                Err(ExtError::Cancelled) => return,
                Err(error) => ChapterResult::Failed { reason: error.to_string() },
            };
            done += 1;
            match &result {
                ChapterResult::Saved { short, .. } => {
                    summary.saved.push(*index);
                    if *short {
                        summary.short.push(*index);
                    }
                }
                ChapterResult::Locked => summary.locked.push(*index),
                ChapterResult::Failed { reason } => summary.failed.push(Failure { index: *index, title: item.title.clone(), reason: reason.clone() }),
            }
            on_event(Event::Chapter { done, total, index: *index, title: item.title.clone(), result });
        });
        if let Some(error) = write_error {
            return Err(error);
        }
        pending.sort_by_key(|(i, _)| *i);
    }
    summary.cancelled = cancel.load(Ordering::SeqCst);
    for list in [&mut summary.saved, &mut summary.short, &mut summary.locked] {
        list.sort_unstable();
    }
    summary.failed.sort_by_key(|f| f.index);
    Ok(summary)
}
