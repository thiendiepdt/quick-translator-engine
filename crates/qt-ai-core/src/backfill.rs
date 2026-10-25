//! Rà ngược bản dịch đã chốt theo một mục glossary: đoạn raw có khoá mà bản dịch cùng đoạn không có cách dịch
//! hiện tại (mục học muộn, tên tác giả gõ lệch, người dùng vừa đổi bản dịch) → liệt kê; rồi thay một cụm sai
//! bằng cách dịch đúng, CHỈ trong các đoạn raw có khoá. Căn đoạn theo thứ tự dòng không rỗng: chương có số đoạn
//! raw khác bản dịch thì bỏ qua (không đoán).

use crate::error::Result;
use crate::paragraphs::paragraphs_of;
use crate::story_fs::{read_raw_chapter, read_text, write_atomic, StoryPaths};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageHit {
    pub chapter: String,
    /// Chỉ số 0-based của đoạn (nhãn [[n]] − 1).
    pub paragraph: usize,
    pub text: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UsageScan {
    pub hits: Vec<UsageHit>,
    /// Chương bỏ qua vì số đoạn raw ≠ bản dịch.
    pub skipped: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Backfill {
    pub chapters: usize,
    pub paragraphs: usize,
    pub occurrences: usize,
    pub skipped: usize,
}

/// Chương đã có bản dịch, theo thứ tự tên file.
pub fn translated_chapters(paths: &StoryPaths) -> Vec<(String, PathBuf)> {
    let mut files: Vec<(String, PathBuf)> = std::fs::read_dir(&paths.out_dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "txt"))
        .filter_map(|path| Some((path.file_stem()?.to_str()?.to_string(), path)))
        .collect();
    files.sort_by(|a, b| crate::story::natural_chapter_compare(&a.0, &b.0));
    files
}

/// (đoạn raw, các dòng bản dịch giữ ký tự xuống dòng, vị trí dòng không rỗng = đoạn).
type Aligned = (Vec<String>, Vec<String>, Vec<usize>);

/// Bản dịch căn theo đoạn raw. None khi số đoạn lệch raw.
fn aligned(paths: &StoryPaths, id: &str, out: &Path) -> Result<Option<Aligned>> {
    let raw = paragraphs_of(&read_raw_chapter(paths, id)?);
    let text = read_text(out)?;
    let lines: Vec<String> = text.split_inclusive('\n').map(str::to_string).collect();
    // Phần chèn cho đủ chữ (sau dòng ngăn) không phải đoạn dịch: không ghép với raw, không bị sửa theo.
    let content: Vec<usize> = crate::padding::content_lines(&lines);
    Ok((content.len() == raw.len()).then_some((raw, lines, content)))
}

/// Đoạn raw có `key` mà bản dịch cùng đoạn không chứa `target`.
pub fn scan_glossary_usage(paths: &StoryPaths, key: &str, target: &str) -> Result<UsageScan> {
    let mut scan = UsageScan::default();
    if key.trim().is_empty() || target.trim().is_empty() {
        return Ok(scan);
    }
    for (id, out) in translated_chapters(paths) {
        let Some((raw, lines, content)) = aligned(paths, &id, &out)? else {
            scan.skipped += 1;
            continue;
        };
        for (paragraph, (source, &line)) in raw.iter().zip(&content).enumerate() {
            let text = lines[line].trim();
            if source.contains(key) && !text.contains(target) {
                scan.hits.push(UsageHit { chapter: id.clone(), paragraph, text: text.chars().take(200).collect() });
            }
        }
    }
    Ok(scan)
}

/// Thay `from` bằng `to` trong các đoạn có `key` ở raw. `to` chứa `from` (`Vân Đình` → `Lục Vân Đình`) thì chỗ
/// đã đúng không bị thay chồng. `apply = false`: chỉ đếm để xem trước.
pub fn backfill_glossary(paths: &StoryPaths, key: &str, from: &str, to: &str, apply: bool) -> Result<Backfill> {
    let mut result = Backfill::default();
    if key.trim().is_empty() || from.is_empty() || from == to {
        return Ok(result);
    }
    const GUARD: &str = "\u{E000}";
    let replace = |text: &str| -> (String, usize) {
        let guarded = if to.contains(from) { text.replace(to, GUARD) } else { text.to_string() };
        let count = guarded.matches(from).count();
        (guarded.replace(from, to).replace(GUARD, to), count)
    };
    for (id, out) in translated_chapters(paths) {
        let Some((raw, mut lines, content)) = aligned(paths, &id, &out)? else {
            result.skipped += 1;
            continue;
        };
        let mut touched = false;
        for (source, &line) in raw.iter().zip(&content) {
            if !source.contains(key) {
                continue;
            }
            let (fixed, count) = replace(&lines[line]);
            if count > 0 {
                lines[line] = fixed;
                result.paragraphs += 1;
                result.occurrences += count;
                touched = true;
            }
        }
        if touched {
            result.chapters += 1;
            if apply {
                write_atomic(&out, &lines.concat())?;
            }
        }
    }
    Ok(result)
}
