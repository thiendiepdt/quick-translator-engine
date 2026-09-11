use crate::error::{CoreError, Result};
use crate::story::natural_chapter_compare;
use crate::story_fs::{load_state, read_text, resolve_root, story_paths, write_text, ChapterStatus};
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

/// Gộp out/<id>.txt của chương done trong [from..to], mỗi chương cách đúng một dòng trống (format
/// "Tải chương" của web). Mặc định khoảng = done đầu..done cuối; file ra export/<from>-<to>.txt.
pub fn run_export(root: &Path, options: &ExportOptions) -> Result<ExportResult> {
    let paths = story_paths(&resolve_root(root));
    let state = load_state(&paths)?;
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
    let mut parts = Vec::new();
    for id in &ids {
        let file = paths.out_dir.join(format!("{id}.txt"));
        if !file.exists() {
            return Err(CoreError::InvalidState(format!("Chương {id} done nhưng thiếu {}.", file.display())));
        }
        let text = read_text(&file)?;
        let trimmed = text.trim_end();
        if !trimmed.is_empty() {
            parts.push(trimmed.to_string());
        }
    }
    let out_path = match &options.out {
        Some(out) => resolve_root(out),
        None => paths.root.join("export").join(format!("{from}-{to}.txt")),
    };
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).map_err(CoreError::io(parent))?;
    }
    write_text(&out_path, &format!("{}\n", parts.join("\n\n")))?;
    Ok(ExportResult { out_path, ids, gaps })
}
