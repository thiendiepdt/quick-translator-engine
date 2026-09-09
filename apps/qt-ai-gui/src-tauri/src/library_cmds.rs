//! Thư viện truyện (folder cha): liệt kê, tạo truyện mới, quét chương mới, nhập chương kéo thả.
//! Spec: docs/superpowers/specs/2026-09-09-qt-ai-gui-library-design.md

use crate::error::{CmdResult, CommandError};
use crate::sidecar::qt_ai_command;
use crate::story_cmds::{snapshot, RecentSummary, StorySnapshot};
use crate::AppState;
use qt_ai_core::commands::init::run_init;
use qt_ai_core::commands::status::count_chapters;
use qt_ai_core::story_fs::{load_state, load_story_config, save_story_config, story_paths};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tauri::State;

const SLUG_MAX: usize = 80;

/// Bỏ dấu tiếng Việt (kể cả đ) — không kéo crate Unicode chỉ vì tên truyện.
fn strip_vietnamese(ch: char) -> Option<char> {
    const TABLE: &[(&str, char)] = &[
        ("aàáảãạăằắẳẵặâầấẩẫậ", 'a'),
        ("eèéẻẽẹêềếểễệ", 'e'),
        ("iìíỉĩị", 'i'),
        ("oòóỏõọôồốổỗộơờớởỡợ", 'o'),
        ("uùúủũụưừứửữự", 'u'),
        ("yỳýỷỹỵ", 'y'),
        ("đ", 'd'),
    ];
    let lower = ch.to_lowercase().next().unwrap_or(ch);
    if lower.is_ascii_alphanumeric() {
        return Some(lower);
    }
    TABLE.iter().find(|(group, _)| group.contains(lower)).map(|(_, base)| *base)
}

/// Tên folder từ tên truyện: `Ta Tuyệt Thế Chị Dâu` → `ta-tuyet-the-chi-dau`. Rỗng → `truyen`.
pub fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for ch in name.chars() {
        match strip_vietnamese(ch) {
            Some(base) => {
                if pending_dash && !out.is_empty() {
                    out.push('-');
                }
                pending_dash = false;
                out.push(base);
            }
            None => pending_dash = true,
        }
        if out.len() >= SLUG_MAX {
            break;
        }
    }
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "truyen".to_string()
    } else {
        trimmed
    }
}

fn valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= SLUG_MAX
        && slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !slug.starts_with('-')
        && !slug.ends_with('-')
}

fn summarize(root: &Path) -> RecentSummary {
    let paths = story_paths(root);
    match (load_state(&paths), load_story_config(&paths)) {
        (Ok(state), Ok(story)) => {
            let counts = count_chapters(&state);
            RecentSummary {
                root: root.display().to_string(),
                name: Some(story.name).filter(|n| !n.trim().is_empty()),
                done: Some(counts.done),
                total: Some(counts.total),
            }
        }
        _ => RecentSummary { root: root.display().to_string(), name: None, done: None, total: None },
    }
}

/// Thư mục con trực tiếp của thư viện: truyện đã init xếp theo state.json mới sửa lên đầu,
/// folder chưa init xếp cuối theo tên. Thư viện null/mất → rỗng.
pub fn list_library(library_root: Option<&Path>) -> Vec<RecentSummary> {
    let Some(library) = library_root.filter(|dir| dir.is_dir()) else { return vec![] };
    let Ok(entries) = std::fs::read_dir(library) else { return vec![] };
    let mut rows: Vec<(Option<SystemTime>, String, RecentSummary)> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .map(|entry| {
            let root = entry.path();
            let modified = std::fs::metadata(story_paths(&root).state_json).and_then(|m| m.modified()).ok();
            (modified, entry.file_name().to_string_lossy().to_lowercase(), summarize(&root))
        })
        .collect();
    rows.sort_by(|a, b| match (a.0, b.0) {
        (Some(x), Some(y)) => y.cmp(&x).then_with(|| a.1.cmp(&b.1)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.1.cmp(&b.1),
    });
    rows.into_iter().map(|(_, _, summary)| summary).collect()
}

/// Tạo `<library>/<slug>/raw/`, init, ghi tên + link. Không đụng folder đã có.
pub fn create_story_inner(library_root: &Path, name: &str, slug: &str, source_url: &str, qt_ai: &str) -> CmdResult<PathBuf> {
    let name = name.trim();
    if name.is_empty() {
        return Err(CommandError::new("invalid_state", "Tên truyện không được để trống."));
    }
    if !valid_slug(slug) {
        return Err(CommandError::new("invalid_state", "Tên folder chỉ gồm a-z, 0-9 và dấu gạch nối, không bắt đầu/kết thúc bằng gạch nối."));
    }
    if !library_root.is_dir() {
        return Err(CommandError::new("invalid_state", format!("Thư viện {} không tồn tại.", library_root.display())));
    }
    let root = library_root.join(slug);
    if root.exists() {
        return Err(CommandError::new("invalid_state", format!("Folder {} đã tồn tại — chọn tên folder khác.", root.display())));
    }
    std::fs::create_dir_all(root.join("raw")).map_err(|e| CommandError::new("io", format!("Không tạo được {}: {e}", root.display())))?;
    run_init(&root, qt_ai)?;
    let paths = story_paths(&root);
    let mut story = load_story_config(&paths)?;
    story.name = name.to_string();
    story.source_url = source_url.trim().to_string();
    save_story_config(&paths, &story)?;
    Ok(root)
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportCounts {
    pub added: Vec<String>,
    pub skipped_existing: Vec<String>,
    pub ignored: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportOutcome {
    pub added: Vec<String>,
    pub skipped_existing: Vec<String>,
    pub ignored: Vec<String>,
    pub snapshot: StorySnapshot,
}

fn is_txt(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
}

/// Copy file .txt (hoặc .txt ngay trong folder được thả, không đệ quy) vào raw/, không ghi đè; rồi init
/// để chương mới vào hàng đợi.
pub fn import_chapters_inner(root: &Path, sources: &[PathBuf], qt_ai: &str) -> CmdResult<ImportCounts> {
    let raw_dir = story_paths(root).raw_dir;
    std::fs::create_dir_all(&raw_dir).map_err(|e| CommandError::new("io", format!("Không tạo được raw/: {e}")))?;
    let mut counts = ImportCounts { added: vec![], skipped_existing: vec![], ignored: vec![] };
    let mut candidates: Vec<PathBuf> = vec![];
    for source in sources {
        if is_txt(source) {
            candidates.push(source.clone());
        } else if source.is_dir() {
            let Ok(entries) = std::fs::read_dir(source) else {
                counts.ignored.push(source.display().to_string());
                continue;
            };
            let mut inner: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| is_txt(p)).collect();
            inner.sort();
            candidates.extend(inner);
        } else {
            counts.ignored.push(source.display().to_string());
        }
    }
    for source in candidates {
        // Đuôi ghi thường `.txt` vì core chỉ nhận đúng đuôi đó làm mã chương.
        let Some(stem) = source.file_stem().map(|n| n.to_string_lossy().into_owned()) else { continue };
        let file_name = format!("{stem}.txt");
        let target = raw_dir.join(&file_name);
        if target.exists() {
            counts.skipped_existing.push(file_name);
            continue;
        }
        std::fs::copy(&source, &target)
            .map_err(|e| CommandError::new("io", format!("Không copy được {}: {e}", source.display())))?;
        counts.added.push(file_name);
    }
    run_init(root, qt_ai)?;
    Ok(counts)
}

fn session_running(state: &State<'_, AppState>, root: &str) -> bool {
    state.sessions.lock().unwrap().is_running(root)
}

#[tauri::command]
pub fn slugify_name(name: String) -> String {
    slugify(&name)
}

/// `root` = None → thư viện trong config; Some → dò folder bất kỳ (picker hỏi "đặt làm thư viện?").
#[tauri::command]
pub fn library_list(state: State<'_, AppState>, root: Option<String>) -> CmdResult<Vec<RecentSummary>> {
    let library = root.or_else(|| state.config.lock().unwrap().library_root.clone());
    Ok(list_library(library.as_deref().map(Path::new)))
}

#[tauri::command]
pub fn create_story(
    state: State<'_, AppState>,
    name: String,
    slug: String,
    source_url: String,
) -> CmdResult<StorySnapshot> {
    let library = state
        .config
        .lock()
        .unwrap()
        .library_root
        .clone()
        .ok_or_else(|| CommandError::new("invalid_state", "Chưa chọn thư viện (folder cha chứa truyện)."))?;
    let root = create_story_inner(Path::new(&library), &name, &slug, &source_url, &qt_ai_command())?;
    let root_text = root.display().to_string();
    let snap = snapshot(&root, session_running(&state, &root_text))?;
    let mut config = state.config.lock().unwrap();
    config.touch_recent(&root_text);
    config.save(&state.config_path)?;
    Ok(snap)
}

/// Quét raw/ lấy chương mới vào hàng đợi (run_init idempotent) rồi trả snapshot.
#[tauri::command]
pub fn rescan_story(state: State<'_, AppState>, root: String) -> CmdResult<StorySnapshot> {
    let path = Path::new(&root);
    run_init(path, &qt_ai_command())?;
    snapshot(path, session_running(&state, &root))
}

#[tauri::command]
pub fn import_chapters(state: State<'_, AppState>, root: String, paths: Vec<String>) -> CmdResult<ImportOutcome> {
    let path = Path::new(&root);
    let sources: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    let counts = import_chapters_inner(path, &sources, &qt_ai_command())?;
    Ok(ImportOutcome {
        added: counts.added,
        skipped_existing: counts.skipped_existing,
        ignored: counts.ignored,
        snapshot: snapshot(path, session_running(&state, &root))?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use qt_ai_core::story_fs::ChapterStatus;
    use std::fs;

    #[test]
    fn slugify_bo_dau_tieng_viet_va_gop_gach() {
        assert_eq!(slugify("Ta Tuyệt Thế Chị Dâu"), "ta-tuyet-the-chi-dau");
        assert_eq!(slugify("  Biến Thân Yêu Nữ: Ta Dựa Vào Thái Bổ Phi Thăng!  "), "bien-than-yeu-nu-ta-dua-vao-thai-bo-phi-thang");
        assert_eq!(slugify("Đại Đường 2024"), "dai-duong-2024");
        assert_eq!(slugify("---"), "truyen");
        assert_eq!(slugify(""), "truyen");
        assert_eq!(slugify("斗破苍穹"), "truyen");
        assert!(slugify(&"a".repeat(200)).len() <= SLUG_MAX);
        assert!(valid_slug("ta-tuyet-the") && !valid_slug("Ta") && !valid_slug("-a") && !valid_slug("a b") && !valid_slug(""));
    }

    #[test]
    fn create_story_tao_khung_ghi_ten_va_tu_choi_trung() {
        let lib = tempfile::tempdir().unwrap();
        let root = create_story_inner(lib.path(), " Kỳ Chiêu Nguyệt ", "ky-chieu-nguyet", "https://x/y ", "qt-ai").unwrap();
        assert_eq!(root, lib.path().join("ky-chieu-nguyet"));
        assert!(root.join("raw").is_dir() && root.join("state.json").is_file() && root.join("AGENTS.md").is_file());
        let story = load_story_config(&story_paths(&root)).unwrap();
        assert_eq!(story.name, "Kỳ Chiêu Nguyệt");
        assert_eq!(story.source_url, "https://x/y");
        assert_eq!(snapshot(&root, false).unwrap().counts.total, 0);

        let dup = create_story_inner(lib.path(), "Khác", "ky-chieu-nguyet", "", "qt-ai").unwrap_err();
        assert_eq!(dup.kind, "invalid_state");
        assert!(dup.message.contains("đã tồn tại"));
        assert_eq!(create_story_inner(lib.path(), "", "abc", "", "qt-ai").unwrap_err().kind, "invalid_state");
        assert_eq!(create_story_inner(lib.path(), "A", "Sai Slug", "", "qt-ai").unwrap_err().kind, "invalid_state");
        assert_eq!(create_story_inner(&lib.path().join("none"), "A", "abc", "", "qt-ai").unwrap_err().kind, "invalid_state");
        // Lỗi không tạo folder rác.
        assert!(!lib.path().join("abc").exists());
    }

    #[test]
    fn list_library_phan_biet_init_chua_init_va_thu_tu() {
        let lib = tempfile::tempdir().unwrap();
        fs::create_dir_all(lib.path().join("chua-init").join("raw")).unwrap();
        fs::write(lib.path().join("file-le.txt"), "x").unwrap();
        let old = create_story_inner(lib.path(), "Cũ", "cu", "", "qt-ai").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(30));
        let new = create_story_inner(lib.path(), "Mới", "moi", "", "qt-ai").unwrap();
        fs::write(new.join("raw").join("0001.txt"), "第一章").unwrap();
        run_init(&new, "qt-ai").unwrap();
        let _ = old;

        let rows = list_library(Some(lib.path()));
        let roots: Vec<&str> = rows.iter().map(|r| r.root.as_str()).collect();
        assert_eq!(rows.len(), 3, "{roots:?}");
        assert!(roots[0].ends_with("moi") && roots[1].ends_with("cu") && roots[2].ends_with("chua-init"));
        assert_eq!(rows[0].name.as_deref(), Some("Mới"));
        assert_eq!(rows[0].total, Some(1));
        assert_eq!(rows[2].name, None);
        assert_eq!(rows[2].total, None);
        assert!(list_library(None).is_empty());
        assert!(list_library(Some(&lib.path().join("none"))).is_empty());
    }

    #[test]
    fn import_chapters_copy_bo_trung_bo_duoi_khac_va_queued() {
        let lib = tempfile::tempdir().unwrap();
        let root = create_story_inner(lib.path(), "T", "t", "", "qt-ai").unwrap();
        let src = tempfile::tempdir().unwrap();
        fs::write(src.path().join("0001.txt"), "第一章").unwrap();
        fs::write(src.path().join("0002.TXT"), "第二章").unwrap();
        fs::write(src.path().join("ghi-chu.md"), "x").unwrap();
        let folder = src.path().join("batch");
        fs::create_dir_all(folder.join("sau")).unwrap();
        fs::write(folder.join("0003.txt"), "第三章").unwrap();
        fs::write(folder.join("sau").join("0009.txt"), "đệ quy không lấy").unwrap();
        fs::write(root.join("raw").join("0002.txt"), "đã có").unwrap();

        let counts = import_chapters_inner(
            &root,
            &[src.path().join("0001.txt"), src.path().join("0002.TXT"), src.path().join("ghi-chu.md"), folder.clone(), src.path().join("mat.txt")],
            "qt-ai",
        )
        .unwrap();
        assert_eq!(counts.added, vec!["0001.txt", "0003.txt"]);
        assert_eq!(counts.skipped_existing, vec!["0002.txt"]); // .TXT → so theo tên đã hạ đuôi
        assert_eq!(counts.ignored.len(), 2); // .md và file không tồn tại
        assert_eq!(fs::read_to_string(root.join("raw").join("0002.txt")).unwrap(), "đã có");
        assert!(!root.join("raw").join("0009.txt").exists());
        let state = load_state(&story_paths(&root)).unwrap();
        assert_eq!(state.chapters["0001"].status, ChapterStatus::Queued);
        assert_eq!(state.chapters["0003"].status, ChapterStatus::Queued);
    }
}
