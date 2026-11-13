//! Trang "Tải raw": link tải raw, extension, mẫu tên file, mục lục, tải chương qua extension (logic ở
//! `qt_ai_core::raw_download`). Extension đặt ở `<app_config_dir>/extensions/<id>/pkg/`. Mỗi truyện một lượt
//! (tải mục lục hoặc tải chương) tại một thời điểm; nút Huỷ bật cờ của lượt đó. Tải xong thì quét `raw/` như nút
//! "Quét raw/" để chương mới vào hàng đợi dịch.

use crate::error::{blocking, CmdResult, CommandError};
use crate::sidecar::qt_ai_command;
use crate::vn_cmds::{lexicon, refresh_quietly};
use crate::AppState;
use qt_ai_core::commands::init::run_init;
use qt_ai_core::raw_download::registry::{candidates, list_installed, pick, Installed};
use qt_ai_core::raw_download::run::{download, effective_template, load_toc, ChapterResult, Event, Mode, RunOptions, Summary, TocView};
use qt_ai_core::raw_download::template::{guess_template, validate_template, DEFAULT_TEMPLATE};
use qt_ai_core::raw_download::{effective_raw_url, load_download, save_download};
use qt_ai_core::story_fs::{list_raw_chapter_ids, load_story_config, story_paths};
use qt_ext::TocItem;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use tauri::{AppHandle, Emitter, Manager, Runtime};

pub const RAW_EVENT: &str = "raw-download-progress";

/// Lượt đang chạy theo truyện → cờ huỷ.
static RUNNING: LazyLock<Mutex<HashMap<String, Arc<AtomicBool>>>> = LazyLock::new(Default::default);
/// Mục lục lần tải gần nhất theo truyện — lượt tải dùng lại, khỏi gọi `toc`.
type CachedToc = (String, String, Vec<TocItem>);
static TOC_CACHE: LazyLock<Mutex<HashMap<String, CachedToc>>> = LazyLock::new(Default::default);

struct Running {
    root: String,
    cancel: Arc<AtomicBool>,
}

impl Running {
    fn start(root: &str) -> CmdResult<Running> {
        let mut map = RUNNING.lock().unwrap_or_else(|e| e.into_inner());
        if map.contains_key(root) {
            return Err(CommandError::new("invalid_state", "Truyện này đang tải raw — chờ xong hoặc bấm Huỷ."));
        }
        let cancel = Arc::new(AtomicBool::new(false));
        map.insert(root.to_string(), cancel.clone());
        Ok(Running { root: root.to_string(), cancel })
    }
}

impl Drop for Running {
    fn drop(&mut self) {
        RUNNING.lock().unwrap_or_else(|e| e.into_inner()).remove(&self.root);
    }
}

pub fn extensions_dir(config_path: &Path) -> PathBuf {
    config_path.parent().map(|dir| dir.join("extensions")).unwrap_or_else(|| PathBuf::from("extensions"))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeveloperView {
    pub name: String,
    pub url: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub developer: DeveloperView,
    pub license: Option<String>,
    pub hosts: Vec<String>,
    pub dev: bool,
}

fn summary_of(ext: &Installed) -> ExtSummary {
    let m = &ext.package.manifest;
    ExtSummary {
        id: m.id.clone(),
        name: m.name.clone(),
        version: m.version.to_string(),
        description: m.description.clone(),
        developer: DeveloperView { name: m.developer.name.clone(), url: m.developer.url.clone(), email: m.developer.email.clone() },
        license: m.license.clone(),
        hosts: m.hosts.clone(),
        dev: ext.dev,
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawInfo {
    pub raw_url: String,
    /// Truyện chưa đặt link riêng — đang theo link nguồn.
    pub raw_url_is_default: bool,
    pub source_url: String,
    pub extension: Option<ExtSummary>,
    pub candidates: Vec<ExtSummary>,
    pub installed: usize,
    pub load_errors: Vec<String>,
    /// Mẫu riêng của truyện (rỗng = theo mặc định / đoán).
    pub file_template: String,
    pub default_template: String,
    pub guessed_template: Option<String>,
    /// Mẫu sẽ dùng cho file mới.
    pub effective_template: String,
    pub extensions_dir: String,
}

fn info_inner(root: &Path, extensions: &Path) -> CmdResult<RawInfo> {
    let paths = story_paths(root);
    let story = load_story_config(&paths)?;
    let config = load_download(&paths);
    let raw_url = effective_raw_url(&config, &story);
    let (installed, load_errors) = list_installed(extensions);
    let ids = list_raw_chapter_ids(&paths)?;
    Ok(RawInfo {
        raw_url_is_default: config.raw_url.trim().is_empty(),
        source_url: story.source_url.clone(),
        extension: pick(&installed, &raw_url, &config.extension).map(summary_of),
        candidates: candidates(&installed, &raw_url).into_iter().map(summary_of).collect(),
        installed: installed.len(),
        load_errors,
        file_template: config.file_template.clone(),
        default_template: DEFAULT_TEMPLATE.to_string(),
        guessed_template: guess_template(&ids),
        effective_template: effective_template(&config, &ids),
        extensions_dir: extensions.display().to_string(),
        raw_url,
    })
}

/// Lưu link tải raw (bằng link nguồn thì lưu rỗng — tiếp tục theo link nguồn), extension đã chọn, mẫu tên.
pub fn save_inner(root: &Path, raw_url: &str, extension: &str, file_template: &str) -> CmdResult<()> {
    let template = file_template.trim();
    if !template.is_empty() {
        validate_template(template).map_err(|m| CommandError::new("invalid_state", m))?;
    }
    let paths = story_paths(root);
    let story = load_story_config(&paths)?;
    let mut config = load_download(&paths);
    let raw = raw_url.trim();
    config.raw_url = if raw == story.source_url.trim() { String::new() } else { raw.to_string() };
    config.extension = extension.trim().to_string();
    config.file_template = template.to_string();
    save_download(&paths, &config)?;
    TOC_CACHE.lock().unwrap_or_else(|e| e.into_inner()).remove(&root.display().to_string());
    Ok(())
}

fn options_for(root: &Path, extensions: &Path, anchor: Option<usize>) -> CmdResult<RunOptions> {
    let paths = story_paths(root);
    let story = load_story_config(&paths)?;
    let config = load_download(&paths);
    let raw_url = effective_raw_url(&config, &story);
    if raw_url.is_empty() {
        return Err(CommandError::new("invalid_state", "Truyện chưa có link tải raw (hoặc link nguồn)."));
    }
    let (installed, _) = list_installed(extensions);
    let extension = pick(&installed, &raw_url, &config.extension).cloned().ok_or_else(|| {
        let count = candidates(&installed, &raw_url).len();
        if count > 1 {
            CommandError::new("invalid_state", "Có nhiều extension khớp link này — chọn một extension trước.")
        } else {
            CommandError::new("invalid_state", "Chưa có extension nào khớp link tải raw này.")
        }
    })?;
    let mut opts = RunOptions::new(extension);
    opts.anchor = anchor.filter(|a| *a > 0);
    Ok(opts)
}

#[tauri::command]
pub async fn raw_info<R: Runtime>(app: AppHandle<R>, root: String) -> CmdResult<RawInfo> {
    blocking(move || {
        let state = app.state::<AppState>();
        info_inner(Path::new(&root), &extensions_dir(&state.config_path))
    })
    .await
}

#[tauri::command]
pub async fn raw_save<R: Runtime>(app: AppHandle<R>, root: String, raw_url: String, extension: String, file_template: String) -> CmdResult<RawInfo> {
    blocking(move || {
        let state = app.state::<AppState>();
        save_inner(Path::new(&root), &raw_url, &extension, &file_template)?;
        info_inner(Path::new(&root), &extensions_dir(&state.config_path))
    })
    .await
}

#[tauri::command]
pub async fn raw_toc<R: Runtime>(app: AppHandle<R>, root: String, anchor: Option<usize>) -> CmdResult<TocView> {
    blocking(move || {
        let state = app.state::<AppState>();
        let path = Path::new(&root);
        let opts = options_for(path, &extensions_dir(&state.config_path), anchor)?;
        let running = Running::start(&root)?;
        let (toc, view) = load_toc(path, &opts, running.cancel.clone())?;
        let raw_url = info_inner(path, &extensions_dir(&state.config_path))?.raw_url;
        TOC_CACHE.lock().unwrap_or_else(|e| e.into_inner()).insert(root.clone(), (opts.extension.package.manifest.id.clone(), raw_url, toc));
        Ok(view)
    })
    .await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawProgress {
    pub root: String,
    /// "toc" | "chapter" | "log"
    pub kind: &'static str,
    pub done: usize,
    pub total: usize,
    pub index: usize,
    pub title: String,
    /// "saved" | "short" | "locked" | "failed" (kind = chapter)
    pub status: &'static str,
    pub message: String,
}

fn progress_of(root: &str, event: Event) -> RawProgress {
    let base = RawProgress { root: root.to_string(), kind: "log", done: 0, total: 0, index: 0, title: String::new(), status: "", message: String::new() };
    match event {
        Event::Toc { total } => RawProgress { kind: "toc", total, ..base },
        Event::Log(message) => RawProgress { message, ..base },
        Event::Chapter { done, total, index, title, result } => {
            let (status, message) = match result {
                ChapterResult::Saved { file, short, replaced } => (if short { "short" } else { "saved" }, if replaced { format!("{file} (ghi đè, bản cũ ở work/raw-backup)") } else { file }),
                ChapterResult::Locked => ("locked", String::new()),
                ChapterResult::Failed { reason } => ("failed", reason),
            };
            RawProgress { kind: "chapter", done, total, index, title, status, message, ..base }
        }
    }
}

#[tauri::command]
pub async fn raw_download<R: Runtime>(app: AppHandle<R>, root: String, mode: String, ids: Vec<String>, anchor: Option<usize>) -> CmdResult<Summary> {
    blocking(move || {
        let state = app.state::<AppState>();
        let path = Path::new(&root);
        let mode = match mode.as_str() {
            "new" => Mode::New,
            "selected" => Mode::Selected(ids),
            "single" => Mode::Single(ids.into_iter().next().ok_or_else(|| CommandError::new("invalid_state", "Thiếu chương cần tải."))?),
            other => return Err(CommandError::new("invalid_state", format!("Chế độ tải lạ: {other}"))),
        };
        let opts = options_for(path, &extensions_dir(&state.config_path), anchor)?;
        let running = Running::start(&root)?;
        let raw_url = info_inner(path, &extensions_dir(&state.config_path))?.raw_url;
        let cached = TOC_CACHE.lock().unwrap_or_else(|e| e.into_inner()).get(&root).cloned();
        let toc = cached.filter(|(ext, url, _)| *ext == opts.extension.package.manifest.id && *url == raw_url).map(|(_, _, toc)| toc);
        let emitter = app.clone();
        let event_root = root.clone();
        let result = download(path, &opts, toc, mode, running.cancel.clone(), &mut |event| {
            let _ = emitter.emit(RAW_EVENT, &progress_of(&event_root, event));
        });
        // Chương mới (kể cả khi lỗi/huỷ giữa chừng) vào hàng đợi dịch như nút "Quét raw/".
        run_init(path, &qt_ai_command())?;
        refresh_quietly(path, &lexicon());
        Ok(result?)
    })
    .await
}

/// Huỷ lượt đang chạy của truyện (tải mục lục hoặc tải chương).
#[tauri::command]
pub fn raw_download_cancel(root: String) {
    if let Some(cancel) = RUNNING.lock().unwrap_or_else(|e| e.into_inner()).get(&root) {
        cancel.store(true, Ordering::SeqCst);
    }
}
