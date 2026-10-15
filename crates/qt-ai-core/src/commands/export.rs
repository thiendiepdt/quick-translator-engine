use crate::book_meta::load_book_meta;
use crate::epub::{build_epub, chapter_from_text, identifier_for, split_story_name, utc_now, EpubBook};
use crate::error::{CoreError, Result};
use crate::han_viet::han_viet_name;
use crate::story::natural_chapter_compare;
use crate::story_fs::{
    load_state, load_story_config, read_text, resolve_root, story_paths, write_atomic_bytes, write_text, ChapterStatus,
    StoryPaths,
};
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone)]
pub struct ExportOptions {
    pub from: Option<String>,
    pub to: Option<String>,
    pub out: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ExportResult {
    pub out_path: PathBuf,
    pub ids: Vec<String>,
    /// Chương trong khoảng nhưng chưa done — file gộp bị hổng ở đó.
    pub gaps: Vec<String>,
}

/// Khoảng chương đã chọn: chương done (theo thứ tự tự nhiên) và chương chưa done nằm giữa.
#[derive(Debug)]
pub struct ExportRange {
    pub from: String,
    pub to: String,
    pub ids: Vec<String>,
    pub gaps: Vec<String>,
}

/// Chương done trong [from..to]; mặc định khoảng = done đầu..done cuối. Dùng chung cho TXT và EPUB.
pub fn select_range(paths: &StoryPaths, options: &ExportOptions) -> Result<ExportRange> {
    let state = load_state(paths)?;
    let mut all: Vec<String> = state.chapters.keys().cloned().collect();
    all.sort_by(|a, b| natural_chapter_compare(a, b));
    let done: Vec<&String> = all.iter().filter(|id| state.chapters[*id].status == ChapterStatus::Done).collect();
    let (Some(first_done), Some(last_done)) = (done.first(), done.last()) else {
        return Err(CoreError::InvalidState("Chưa có chương nào done để export.".to_string()));
    };
    let from = options.from.clone().unwrap_or_else(|| (*first_done).clone());
    let to = options.to.clone().unwrap_or_else(|| (*last_done).clone());
    for bound in [&from, &to] {
        if !state.chapters.contains_key(bound) {
            return Err(CoreError::StoryNotFound(format!("Không có chương {bound} trong state.json.")));
        }
    }
    if natural_chapter_compare(&from, &to) == Ordering::Greater {
        return Err(CoreError::InvalidState(format!("Khoảng ngược: --from {from} sau --to {to}.")));
    }
    let in_range: Vec<&String> = all
        .iter()
        .filter(|id| {
            natural_chapter_compare(id, &from) != Ordering::Less && natural_chapter_compare(id, &to) != Ordering::Greater
        })
        .collect();
    let ids: Vec<String> = in_range
        .iter()
        .filter(|id| state.chapters[**id].status == ChapterStatus::Done)
        .map(|id| (*id).clone())
        .collect();
    let gaps: Vec<String> = in_range
        .iter()
        .filter(|id| state.chapters[**id].status != ChapterStatus::Done)
        .map(|id| (*id).clone())
        .collect();
    if ids.is_empty() {
        return Err(CoreError::InvalidState(format!("Không có chương done nào trong khoảng {from}..{to}.")));
    }
    Ok(ExportRange { from, to, ids, gaps })
}

/// Bản dịch out/<id>.txt của một chương done (đã bỏ khoảng trắng cuối).
pub fn read_done_chapter(paths: &StoryPaths, id: &str) -> Result<String> {
    let file = paths.out_dir.join(format!("{id}.txt"));
    if !file.exists() {
        return Err(CoreError::InvalidState(format!("Chương {id} done nhưng thiếu {}.", file.display())));
    }
    Ok(read_text(&file)?.trim_end().to_string())
}

/// `options.out` nếu có, không thì `export/<from>-<to>.<ext>`; tạo sẵn thư mục cha.
pub fn export_path(paths: &StoryPaths, options: &ExportOptions, range: &ExportRange, ext: &str) -> Result<PathBuf> {
    let out_path = match &options.out {
        Some(out) => resolve_root(out),
        None => paths.root.join("export").join(format!("{}-{}.{ext}", range.from, range.to)),
    };
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(CoreError::io(parent))?;
    }
    Ok(out_path)
}

/// Gộp out/<id>.txt của chương done trong [from..to], mỗi chương cách đúng một dòng trống (format
/// "Tải chương" của web). Mặc định khoảng = done đầu..done cuối; file ra export/<from>-<to>.txt.
pub fn run_export(root: &Path, options: &ExportOptions) -> Result<ExportResult> {
    let paths = story_paths(&resolve_root(root));
    let range = select_range(&paths, options)?;
    let mut parts = Vec::new();
    for id in &range.ids {
        let text = read_done_chapter(&paths, id)?;
        if !text.is_empty() {
            parts.push(text);
        }
    }
    let out_path = export_path(&paths, options, &range, "txt")?;
    write_text(&out_path, &format!("{}\n", parts.join("\n\n")))?;
    Ok(ExportResult { out_path, ids: range.ids, gaps: range.gaps })
}

/// Như `run_export` nhưng ra EPUB: bìa (`cover_jpeg`, không có thì bỏ trang bìa), trang tiêu đề, trang Mục
/// lục bấm tới từng chương. Mặc định ghi `export/<thư mục truyện>_<from>-<to>.epub`.
pub fn run_export_epub(root: &Path, options: &ExportOptions, author: &str, cover_jpeg: Option<Vec<u8>>) -> Result<ExportResult> {
    let paths = story_paths(&resolve_root(root));
    let range = select_range(&paths, options)?;
    let story = load_story_config(&paths)?;
    let mut chapters = Vec::new();
    for id in &range.ids {
        let text = read_done_chapter(&paths, id)?;
        if !text.is_empty() {
            chapters.push(chapter_from_text(&text));
        }
    }
    let (title, original_title) = split_story_name(&story.name);
    let author = author.trim().to_string();
    // Bút danh gốc chỉ ghi kèm khi tác giả đang hiển thị đúng là bản phiên âm của nó (hoặc trống) — người dùng
    // đổi sang tên khác thì tên gốc không còn khớp.
    let original = load_book_meta(&paths).author_original;
    let author_original =
        if !original.is_empty() && (author.is_empty() || han_viet_name(&original) == author) { original } else { String::new() };
    let title = if title.is_empty() { paths.root.file_name().unwrap_or_default().to_string_lossy().into_owned() } else { title };
    let book = EpubBook {
        identifier: identifier_for(&format!("{}|{}", story.name, story.source_url)),
        title,
        original_title,
        author,
        author_original,
        protagonist: story.protagonist.trim().to_string(),
        summary: story.summary.trim().to_string(),
        modified: utc_now(),
        cover_jpeg,
        chapters,
    };
    let bytes = build_epub(&book)?;
    let options = match &options.out {
        Some(_) => options.clone(),
        None => {
            let slug = paths.root.file_name().unwrap_or_default().to_string_lossy().into_owned();
            let name = format!("{slug}_{}-{}.epub", range.from, range.to);
            ExportOptions { out: Some(paths.root.join("export").join(name)), ..options.clone() }
        }
    };
    let out_path = export_path(&paths, &options, &range, "epub")?;
    write_atomic_bytes(&out_path, &bytes)?;
    Ok(ExportResult { out_path, ids: range.ids, gaps: range.gaps })
}
