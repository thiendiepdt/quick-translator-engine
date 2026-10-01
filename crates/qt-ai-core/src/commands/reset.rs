use crate::commands::init::run_init;
use crate::error::{CoreError, Result};
use crate::story::StoryConfig;
use crate::story_fs::{load_state, load_story_config, resolve_root, save_state, save_story_config, story_paths};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct ResetOutcome {
    /// Số chương đã đưa về hàng đợi.
    pub chapters: usize,
    /// AGENTS.md / workflow từng bị sửa tay — bản sửa tay đã lưu thành `<tên>.bak` trước khi ép về template.
    pub template_backups: Vec<String>,
}

fn backup_path(path: &Path) -> PathBuf {
    let mut bak = path.as_os_str().to_owned();
    bak.push(".bak");
    PathBuf::from(bak)
}

/// Đưa truyện về như lúc vừa tạo để dịch lại từ đầu sau khi app đổi lớn: hồ sơ chỉ còn tên + link, glossary
/// và bảng nhân vật xoá sạch, mọi chương về hàng đợi, work/ dọn. `raw/`, `out/` (kể cả .bak), `export/` và
/// cài đặt harness giữ nguyên; bản dịch cũ trong out/ chỉ bị đè khi chương đó được dịch lại. Bản trước reset
/// nằm ở `story.json.bak`, `state.json.bak`, `cast.json.bak`. AGENTS.md + workflows bị ép về template hiện tại,
/// bản sửa tay lưu `.bak` cạnh đó.
pub fn run_reset(root: &Path, qt_ai_command: &str) -> Result<ResetOutcome> {
    let paths = story_paths(&resolve_root(root));
    let old_story = load_story_config(&paths)?;
    let settings = load_state(&paths)?.settings;

    let state_bak = backup_path(&paths.state_json);
    fs::copy(&paths.state_json, &state_bak).map_err(CoreError::io(&state_bak))?;
    if paths.cast_json.exists() {
        let cast_bak = backup_path(&paths.cast_json);
        fs::rename(&paths.cast_json, &cast_bak).map_err(CoreError::io(&cast_bak))?;
    }
    let mut story = StoryConfig::empty();
    story.name = old_story.name;
    story.source_url = old_story.source_url;
    save_story_config(&paths, &story)?; // tự giữ story.json.bak

    // Dọn work/ — trừ file chấm đầu (.session.lock do phiên dịch quản).
    if let Ok(entries) = fs::read_dir(&paths.work_dir) {
        for entry in entries.filter_map(|entry| entry.ok()) {
            if !entry.file_name().to_string_lossy().starts_with('.') {
                let _ = fs::remove_file(entry.path());
            }
        }
    }

    let template_backups = crate::templates::force_templates(&paths.root, qt_ai_command)?;

    // State mới dựng lại từ raw/ như một truyện vừa init; chỉ cài đặt harness theo người dùng.
    fs::remove_file(&paths.state_json).map_err(CoreError::io(&paths.state_json))?;
    run_init(&paths.root, qt_ai_command)?;
    let mut state = load_state(&paths)?;
    state.settings = settings;
    save_state(&paths, &state)?;
    Ok(ResetOutcome { chapters: state.chapters.len(), template_backups })
}
