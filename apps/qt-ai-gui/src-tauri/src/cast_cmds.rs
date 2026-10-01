//! Bảng nhân vật (cast.json): giới tính + mốc đổi xưng hô. Tách khỏi `save_story` vì dữ liệu nằm ở file
//! riêng — bản app cũ không biết file này nên không ghi đè mất.

use crate::error::{blocking, CmdResult, CommandError};
use crate::session_cmds::resolve_api;
use crate::AppState;
use qt_ai_core::api::{HttpModel, TextModel};
use qt_ai_core::cast::{conflicting_addressing, load_cast, save_cast, Cast};
use qt_ai_core::cast_scan::{collect_snippets, scan_genders};
use qt_ai_core::story::StringMap;
use qt_ai_core::story_fs::{load_story_config, save_story_config, story_paths};
use serde::Serialize;
use serde_json::Value;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use tauri::{AppHandle, Manager, Runtime};

pub fn load_cast_inner(root: &Path) -> Cast {
    load_cast(&story_paths(root))
}

pub fn save_cast_inner(root: &Path, cast: Value) -> CmdResult<Cast> {
    let cast = Cast::normalize(&cast);
    save_cast(&story_paths(root), &cast)?;
    Ok(cast)
}

/// Xoá khỏi `story.json` các cặp xưng hô trái với giới đã chốt (học từ trước khi có bảng nhân vật).
pub fn clean_cast_addressing_inner(root: &Path) -> CmdResult<Vec<String>> {
    let paths = story_paths(root);
    let mut story = load_story_config(&paths)?;
    let cast = load_cast(&paths);
    let removed = story.glossary.get("addressing").map(|pairs| conflicting_addressing(pairs, &cast)).unwrap_or_default();
    if removed.is_empty() {
        return Ok(removed);
    }
    if let Some(pairs) = story.glossary.get_mut("addressing") {
        pairs.retain(|key, _| !removed.contains(key));
    }
    save_story_config(&paths, &story)?;
    Ok(removed)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanCastView {
    pub asked: usize,
    pub filled: usize,
    pub failed_batches: usize,
    pub cast: Cast,
}

/// Quét bù giới tính: tên trong glossary chưa rõ giới + câu trích raw → model → chỉ điền chỗ trống.
pub fn scan_cast_inner(root: &Path, model: &dyn TextModel) -> CmdResult<ScanCastView> {
    let paths = story_paths(root);
    let story = load_story_config(&paths)?;
    let mut cast = load_cast(&paths);
    let no_names = StringMap::new();
    let candidates = collect_snippets(&paths, story.glossary.get("names").unwrap_or(&no_names), &cast)?;
    let before = cast.clone();
    let outcome = scan_genders(&candidates, &mut cast, model, &AtomicBool::new(false));
    // Lỗi giữa chừng (mạng, quota) vẫn ghi các lô đã xong — quét lại chỉ hỏi phần còn thiếu.
    if cast != before {
        save_cast(&paths, &cast)?;
    }
    let outcome = outcome.map_err(|error| CommandError::new("api_failed", error.to_string()))?;
    Ok(ScanCastView { asked: outcome.asked, filled: outcome.filled, failed_batches: outcome.failed_batches, cast })
}

#[tauri::command]
pub fn cast_load(root: String) -> CmdResult<Cast> {
    Ok(load_cast_inner(Path::new(&root)))
}

#[tauri::command]
pub fn cast_save(root: String, cast: Value) -> CmdResult<Cast> {
    save_cast_inner(Path::new(&root), cast)
}

#[tauri::command]
pub fn cast_clean_addressing(root: String) -> CmdResult<Vec<String>> {
    clean_cast_addressing_inner(Path::new(&root))
}

/// Luôn đi đường API key (kể cả khi động cơ dịch đang là Antigravity): quét là một lượt JSON thuần, không
/// cần agent. Chưa có key → `api_key_missing`, giới tính vẫn nhập tay hoặc tự học dần khi dịch chương mới.
#[tauri::command]
pub async fn cast_scan<R: Runtime>(app: AppHandle<R>, root: String) -> CmdResult<ScanCastView> {
    blocking(move || {
        let api = app.state::<AppState>().config.lock().unwrap().api.clone();
        scan_cast_inner(Path::new(&root), &HttpModel::new(resolve_api(&api)?))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use qt_ai_core::api::{ApiError, ApiStep, Generated};
    use qt_ai_core::cast::Gender;
    use qt_ai_core::commands::init::run_init;
    use qt_ai_core::story_fs::{load_story_config, save_story_config};
    use serde_json::json;
    use std::fs;
    use std::sync::atomic::AtomicBool;

    fn story() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("raw")).unwrap();
        fs::write(dir.path().join("raw").join("0001.txt"), "贺静昭没想到，她是北方人。\n\n莫衡点头。").unwrap();
        run_init(dir.path(), "qt-ai").unwrap();
        let paths = story_paths(dir.path());
        let mut story = load_story_config(&paths).unwrap();
        let names = story.glossary.entry("names".to_string()).or_default();
        names.insert("贺静昭".into(), "Hạ Tĩnh Chiêu".into());
        names.insert("莫衡".into(), "Mạc Hành".into());
        let pairs = story.glossary.entry("addressing".to_string()).or_default();
        pairs.insert("莫衡→贺静昭".into(), "tôi–thầy Hạ".into());
        pairs.insert("贺静昭→莫衡".into(), "tôi–anh".into());
        save_story_config(&paths, &story).unwrap();
        dir
    }

    #[test]
    fn save_cast_chuan_hoa_roi_load_lai_dung() {
        let dir = story();
        assert!(load_cast_inner(dir.path()).characters.is_empty(), "truyện cũ chưa có cast.json");
        let saved = save_cast_inner(
            dir.path(),
            json!({ "characters": { "贺静昭": { "gender": "female", "source": "user" }, "hỏng": { "gender": "x" } } }),
        )
        .unwrap();
        assert_eq!(saved.gender_of("贺静昭"), Some(Gender::Female));
        assert_eq!(load_cast_inner(dir.path()), saved);
        assert_eq!(serde_json::to_value(&saved).unwrap()["characters"]["贺静昭"]["source"], "user");
    }

    #[test]
    fn clean_addressing_xoa_cap_trai_gioi_khoi_story_json() {
        let dir = story();
        save_cast_inner(dir.path(), json!({ "characters": { "贺静昭": { "gender": "female" } } })).unwrap();
        let removed = clean_cast_addressing_inner(dir.path()).unwrap();
        assert_eq!(removed, vec!["莫衡→贺静昭"]);
        let story = load_story_config(&story_paths(dir.path())).unwrap();
        assert!(!story.glossary["addressing"].contains_key("莫衡→贺静昭"));
        assert!(story.glossary["addressing"].contains_key("贺静昭→莫衡"));
        assert!(clean_cast_addressing_inner(dir.path()).unwrap().is_empty());
    }

    struct Scripted(&'static str);
    impl TextModel for Scripted {
        fn label(&self) -> String {
            "scripted".into()
        }
        fn generate(&self, _: ApiStep, _: &str, _: &str, _: &AtomicBool, _: &mut dyn FnMut(usize)) -> Result<Generated, ApiError> {
            unreachable!()
        }
        fn complete_json(&self, _: ApiStep, _: &str, _: &str, _: &AtomicBool) -> Result<Generated, ApiError> {
            Ok(Generated::text(self.0))
        }
    }

    #[test]
    fn scan_cast_dien_gioi_va_ghi_file() {
        let dir = story();
        let view = scan_cast_inner(dir.path(), &Scripted(r#"{"genders":{"贺静昭":"nữ","莫衡":""}}"#)).unwrap();
        assert_eq!((view.asked, view.filled, view.failed_batches), (2, 1, 0));
        assert_eq!(view.cast.gender_of("贺静昭"), Some(Gender::Female));
        assert_eq!(load_cast_inner(dir.path()).gender_of("贺静昭"), Some(Gender::Female));
        let json = serde_json::to_value(&view).unwrap();
        assert_eq!(json["failedBatches"], 0);
    }
}
