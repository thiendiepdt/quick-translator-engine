use crate::error::Result;
use crate::story::StoryConfig;
use crate::story_fs::{
    ensure_story_dirs, list_raw_chapter_ids, load_state, resolve_root, save_state, save_story_config,
    story_paths, work_file, ChapterState, ChapterStatus, StoryState, WORK_KINDS,
};
use crate::templates::copy_templates;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Chương raw trùng từng byte với chương khác mà dòng đầu là `第N章` — raw tải hỏng (truong-sinh-la-meo: raw 0252
/// là bản sao của 0251, bản dịch cũ vẫn đúng nên không ai thấy). Hai tờ `请假条` giống nhau là hợp lệ nên bỏ qua.
/// Chỉ đọc nội dung những file cùng kích thước. Trả (chương sau, chương trước) theo thứ tự tên.
pub fn raw_duplicates(root: &Path) -> Vec<(String, String)> {
    let paths = story_paths(&resolve_root(root));
    // Kích thước lấy từ DirEntry (Windows trả sẵn khi liệt kê thư mục) — gọi metadata từng file chậm gấp nhiều lần.
    let Ok(entries) = fs::read_dir(&paths.raw_dir) else { return vec![] };
    let mut by_size: std::collections::BTreeMap<u64, Vec<String>> = std::collections::BTreeMap::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let (Some(stem), Some("txt")) = (path.file_stem().and_then(|s| s.to_str()), path.extension().and_then(|e| e.to_str())) else {
            continue;
        };
        if let Ok(meta) = entry.metadata() {
            by_size.entry(meta.len()).or_default().push(stem.to_string());
        }
    }
    for group in by_size.values_mut() {
        group.sort_by(|a, b| crate::story::natural_chapter_compare(a, b));
    }
    let mut found = Vec::new();
    for group in by_size.into_values().filter(|group| group.len() > 1) {
        let mut seen: Vec<(String, String)> = Vec::new();
        for id in group {
            let Ok(text) = fs::read_to_string(paths.raw_dir.join(format!("{id}.txt"))) else { continue };
            let first = text.lines().map(str::trim).find(|line| !line.is_empty()).unwrap_or("");
            if !(first.starts_with('第') && first.contains('章')) {
                continue;
            }
            match seen.iter().find(|(_, other)| *other == text) {
                Some((earlier, _)) => found.push((id.clone(), earlier.clone())),
                None => seen.push((id, text)),
            }
        }
    }
    found.sort_by(|a, b| crate::story::natural_chapter_compare(&a.0, &b.0));
    found
}

/// raw/ lệch state.json bao nhiêu: (chương raw mới chưa vào state, chương trong state đã mất raw).
/// (0, 0) là `run_init` không còn gì để làm với danh sách chương — quét cả thư viện dùng để khỏi ghi lại
/// state.json của truyện không đổi.
pub fn raw_drift(root: &Path) -> Result<(usize, usize)> {
    let paths = story_paths(&resolve_root(root));
    let state = load_state(&paths)?;
    let raw_ids: BTreeSet<String> = list_raw_chapter_ids(&paths)?.into_iter().collect();
    let added = raw_ids.iter().filter(|id| !state.chapters.contains_key(*id)).count();
    let removed = state.chapters.keys().filter(|id| !raw_ids.contains(*id)).count();
    Ok((added, removed))
}

/// Dựng khung folder truyện. Idempotent: giữ state/story sẵn có, thêm chương raw mới thành queued và
/// gỡ chương có file raw đã mất (đổi tên/xoá) — kể cả chương done, nhưng out/<id>.txt giữ nguyên trên đĩa
/// (bản dịch không bị xoá, chỉ không còn trong danh sách/export).
/// `qt_ai_command` là lệnh chạy CLI để render vào AGENTS.md (đường dẫn binary, có ngoặc kép nếu cần).
pub fn run_init(root: &Path, qt_ai_command: &str) -> Result<String> {
    let paths = story_paths(&resolve_root(root));
    ensure_story_dirs(&paths)?;
    if !paths.story_json.exists() {
        save_story_config(&paths, &StoryConfig::empty())?;
    }
    let mut state = if paths.state_json.exists() { load_state(&paths)? } else { StoryState::new() };
    let raw_ids: BTreeSet<String> = list_raw_chapter_ids(&paths)?.into_iter().collect();
    let gone: Vec<String> = state
        .chapters
        .iter()
        .filter(|(id, _)| !raw_ids.contains(*id))
        .map(|(id, _)| id.clone())
        .collect();
    for id in &gone {
        state.chapters.shift_remove(id);
        for kind in WORK_KINDS {
            let _ = fs::remove_file(work_file(&paths, id, kind));
        }
    }
    let mut added = 0;
    for id in raw_ids {
        if state.chapters.contains_key(&id) {
            continue;
        }
        state.chapters.insert(id, ChapterState::fresh(ChapterStatus::Queued));
        added += 1;
    }
    save_state(&paths, &state)?;
    let written = copy_templates(&paths.root, qt_ai_command)?;
    let removed = if gone.is_empty() {
        String::new()
    } else {
        format!(", {} gỡ vì raw đã mất — bản dịch trong out/ giữ nguyên", gone.len())
    };
    let templates = if written.is_empty() { String::new() } else { format!(" Đã ghi {}.", written.join(", ")) };
    Ok(format!(
        "Đã init {}: {} chương ({} mới thêm vào hàng đợi{}).{}",
        paths.root.display(),
        state.chapters.len(),
        added,
        removed,
        templates
    ))
}
