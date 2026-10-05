//! Tiêu đề chương lặp (titles.json): khảo sát các tên lặp + kiểu dịch đang có, chốt bản dịch và ghi lại dòng
//! tiêu đề của các chương đã dịch. Logic ở `qt_ai_core::titles`.

use crate::error::{blocking, CmdResult, CommandError};
use crate::AppState;
use qt_ai_core::story_fs::{load_state, story_paths};
use qt_ai_core::titles::{apply_titles, load_titles, survey_titles, TitleSurvey};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use tauri::{AppHandle, Manager, Runtime};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TitlesApplied {
    /// Số chương đã dịch có dòng tiêu đề vừa được ghi lại.
    pub changed: usize,
    /// Khảo sát sau khi áp dụng.
    pub survey: TitleSurvey,
}

pub fn survey_inner(root: &Path) -> CmdResult<TitleSurvey> {
    let paths = story_paths(root);
    let state = load_state(&paths)?;
    Ok(survey_titles(&paths, &state, &load_titles(&paths))?.0)
}

pub fn apply_inner(root: &Path, targets: &HashMap<String, String>) -> CmdResult<TitlesApplied> {
    let paths = story_paths(root);
    let state = load_state(&paths)?;
    let changed = apply_titles(&paths, &state, targets)?;
    Ok(TitlesApplied { changed, survey: survey_inner(root)? })
}

/// Các tên tiêu đề lặp ở từ hai chương trở lên — đọc dòng đầu của mọi chương raw nên chạy ngoài luồng IPC.
#[tauri::command]
pub async fn titles_survey(root: String) -> CmdResult<TitleSurvey> {
    blocking(move || survey_inner(Path::new(&root))).await
}

/// Chốt `targets` (tên gốc → bản dịch) rồi ghi lại dòng tiêu đề trong out/ cho khớp. Phiên dịch đang chạy thì
/// từ chối: phiên đang ghi out/ và titles.json của chính truyện này.
#[tauri::command]
pub async fn titles_apply<R: Runtime>(app: AppHandle<R>, root: String, targets: HashMap<String, String>) -> CmdResult<TitlesApplied> {
    blocking(move || {
        let state = app.state::<AppState>();
        if state.sessions.lock().unwrap().is_running(&root) {
            return Err(CommandError::new(
                "session_locked",
                "Truyện này đang có phiên dịch chạy — bấm Dừng trước khi sửa tiêu đề.",
            ));
        }
        apply_inner(Path::new(&root), &targets)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use qt_ai_core::commands::init::run_init;
    use qt_ai_core::story_fs::{save_state, ChapterStatus};
    use std::fs;

    #[test]
    fn survey_va_apply_tren_truyen_da_dich_lech_tieu_de() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("raw")).unwrap();
        let chapters = [
            ("0001", "第1章 夜雨（一）", "Chương 1: Mưa đêm (một)"),
            ("0002", "第2章 夜雨（二）", "Chương 2: Mưa đêm (2)"),
            ("0003", "第3章 夜雨（完）", "Chương 3: Đêm mưa (kết)"),
        ];
        for (id, raw, _) in chapters {
            fs::write(dir.path().join("raw").join(format!("{id}.txt")), format!("{raw}\n\n正文。")).unwrap();
        }
        run_init(dir.path(), "qt-ai").unwrap();
        let paths = story_paths(dir.path());
        let mut state = load_state(&paths).unwrap();
        for (id, _, translated) in chapters {
            state.chapters.get_mut(id).unwrap().status = ChapterStatus::Done;
            fs::write(paths.out_dir.join(format!("{id}.txt")), format!("{translated}\n\nThân bài.\n")).unwrap();
        }
        save_state(&paths, &state).unwrap();

        let survey = survey_inner(dir.path()).unwrap();
        assert_eq!(survey.groups.len(), 1);
        assert_eq!((survey.groups[0].target.as_str(), survey.groups[0].pending, survey.groups[0].settled), ("Mưa đêm", 2, false));

        let targets = HashMap::from([("夜雨".to_string(), "Mưa khuya".to_string())]);
        let applied = apply_inner(dir.path(), &targets).unwrap();
        assert_eq!(applied.changed, 3);
        assert!(applied.survey.groups[0].settled && applied.survey.groups[0].pending == 0);
        assert_eq!(fs::read_to_string(paths.out_dir.join("0003.txt")).unwrap(), "Chương 3: Mưa khuya (hết)\n\nThân bài.\n");
    }
}
