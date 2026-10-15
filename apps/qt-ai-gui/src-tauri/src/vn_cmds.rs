//! Tab "Kiểm tra VN": quét, xác minh (người dùng + AI), thay thế trong bản dịch. Logic ở qt_ai_core::{vn_check, vn_ai}.

use crate::app_config::Engine;
use crate::error::{blocking, CmdResult, CommandError};
use crate::replace_cmds::refuse_if_running;
use crate::session_cmds::{resolve_agy, resolve_api, session_config};
use crate::AppState;
use qt_ai_core::api::HttpModel;
use qt_ai_core::base::BaseStore;
use qt_ai_core::replace::{find_in_story, replace_in_story, FindQuery, HitRef, ReplaceOutcome};
use qt_ai_core::session::run_once;
use qt_ai_core::story_fs::story_paths;
use qt_ai_core::vn_ai::{agy_prompt, items, parse_verdicts, payload, save_verdicts, verify_with_model, VerifyOutcome};
use qt_ai_core::vn_check::{refresh, report, set_status, HitView, Status, VnSummary};
use qt_ai_core::vn_lexicon::{builtin_entries, load_settings, save_settings, Entry, Lexicon, VnSettings};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use tauri::{AppHandle, Manager, Runtime};

fn lexicon() -> Lexicon {
    Lexicon::load(&BaseStore::from_env())
}

pub fn refresh_inner(root: &Path, lex: &Lexicon) -> CmdResult<VnSummary> {
    Ok(refresh(&story_paths(root), lex)?)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VnReplaceRequest {
    pub from: String,
    pub to: String,
    /// Chương được phép thay.
    pub chapters: Vec<String>,
    /// Chỉ các đoạn này (chương, đoạn); None = mọi chỗ khớp trong `chapters`.
    #[serde(default)]
    pub paragraphs: Option<Vec<(String, usize)>>,
}

/// Thay nguyên từ, phân biệt hoa thường, giới hạn đúng chương/đoạn bằng danh sách loại trừ của engine Tìm & thay
/// (nên có hoàn tác); xong quét lại.
pub fn replace_inner(root: &Path, request: &VnReplaceRequest, lex: &Lexicon) -> CmdResult<ReplaceOutcome> {
    let paths = story_paths(root);
    let query = FindQuery {
        find: request.from.clone(),
        replace: request.to.clone(),
        case_sensitive: true,
        whole_word: true,
        ..Default::default()
    };
    let found = find_in_story(&paths, &query, usize::MAX)?;
    let wanted: HashSet<&str> = request.chapters.iter().map(String::as_str).collect();
    let excluded_chapters: Vec<String> =
        found.chapters.iter().map(|c| c.chapter.clone()).filter(|id| !wanted.contains(id.as_str())).collect();
    let excluded_hits: Vec<HitRef> = match &request.paragraphs {
        None => vec![],
        Some(keep) => found
            .hits
            .iter()
            .filter(|hit| !keep.iter().any(|(chapter, paragraph)| chapter == &hit.chapter && *paragraph == hit.paragraph))
            .map(|hit| HitRef { chapter: hit.chapter.clone(), paragraph: hit.paragraph, occurrence: hit.occurrence })
            .collect(),
    };
    let outcome = replace_in_story(&paths, &query, &excluded_chapters, &excluded_hits)?;
    refresh(&paths, lex)?;
    Ok(outcome)
}

#[tauri::command]
pub async fn vn_refresh(root: String) -> CmdResult<VnSummary> {
    blocking(move || refresh_inner(Path::new(&root), &lexicon())).await
}

#[tauri::command]
pub async fn vn_report(root: String) -> CmdResult<Vec<HitView>> {
    blocking(move || Ok(report(&story_paths(Path::new(&root))))).await
}

#[tauri::command]
pub async fn vn_set_status(root: String, ids: Vec<String>, status: Status) -> CmdResult<usize> {
    blocking(move || Ok(set_status(&story_paths(Path::new(&root)), &ids, status)?)).await
}

/// Bản cứng của từ điển — Cài đặt liệt kê để bật/tắt từng mục.
#[tauri::command]
pub async fn vn_builtin_entries() -> CmdResult<Vec<Entry>> {
    blocking(move || Ok(builtin_entries())).await
}

#[tauri::command]
pub async fn vn_settings_get() -> CmdResult<VnSettings> {
    blocking(move || Ok(load_settings(&BaseStore::from_env()))).await
}

#[tauri::command]
pub async fn vn_settings_save(settings: VnSettings) -> CmdResult<VnSettings> {
    blocking(move || {
        let base = BaseStore::from_env();
        save_settings(&base, &settings)?;
        Ok(load_settings(&base))
    })
    .await
}

#[tauri::command]
pub async fn vn_replace<R: Runtime>(app: AppHandle<R>, root: String, request: VnReplaceRequest) -> CmdResult<ReplaceOutcome> {
    blocking(move || {
        refuse_if_running(&app, &root)?;
        replace_inner(Path::new(&root), &request, &lexicon())
    })
    .await
}

/// AI xét các chỗ `ids`: động cơ API dùng API key; agy ghi đầu vào ra work/, agent ghi kết quả ra work/.
#[tauri::command]
pub async fn vn_ai_verify<R: Runtime>(app: AppHandle<R>, root: String, ids: Vec<String>) -> CmdResult<VerifyOutcome> {
    blocking(move || {
        let state = app.state::<AppState>();
        let (engine, api, model) = {
            let config = state.config.lock().unwrap();
            (config.engine, config.api.clone(), config.model.clone())
        };
        let paths = story_paths(Path::new(&root));
        match engine {
            Engine::Api => {
                let model = HttpModel::new(resolve_api(&api)?);
                Ok(verify_with_model(&paths, &model, &ids, &AtomicBool::new(false))?)
            }
            Engine::Agy => {
                let all = items(&report(&paths), &ids);
                std::fs::create_dir_all(&paths.work_dir).map_err(|e| CommandError::new("io", e.to_string()))?;
                let input = paths.work_dir.join("vn-verify-input.json");
                let output = paths.work_dir.join("vn-verify-output.json");
                std::fs::write(&input, payload(&all)).map_err(|e| CommandError::new("io", e.to_string()))?;
                let _ = std::fs::remove_file(&output);
                let cleanup = || {
                    let _ = std::fs::remove_file(&input);
                    let _ = std::fs::remove_file(&output);
                };
                let agy = match resolve_agy(&state) {
                    Ok(agy) => agy,
                    Err(e) => {
                        cleanup();
                        return Err(e);
                    }
                };
                let config = session_config(Path::new(&root), agy, model.clone(), 1, false);
                // Lỗi chạy agy không được bỏ qua bước dọn file work/; giữ lại làm lý do cho GUI.
                let run_error = run_once(&config, &agy_prompt(&input, &output), &|_| {}).err().map(|e| e.to_string());
                let text = std::fs::read_to_string(&output).unwrap_or_default();
                let verdicts: Vec<_> = parse_verdicts(&text).into_iter().filter(|v| ids.contains(&v.id)).collect();
                let label = format!("agy {}", model.unwrap_or_else(|| "mặc định".into()));
                let saved = save_verdicts(&paths, &verdicts, &label);
                cleanup();
                let verified = saved?;
                let error = run_error.or_else(|| {
                    text.trim().is_empty().then(|| "agy không ghi kết quả (file đầu ra thiếu hoặc rỗng).".to_string())
                });
                Ok(VerifyOutcome { verified, failed: all.len().saturating_sub(verified), error })
            }
        }
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn story() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for sub in ["raw", "out"] {
            fs::create_dir_all(dir.path().join(sub)).unwrap();
        }
        fs::write(dir.path().join("raw/0001.txt"), "第1章\n\n坐船转到越南。\n\n又去越南。").unwrap();
        fs::write(dir.path().join("out/0001.txt"), "Chương 1\n\nNgồi thuyền qua Việt Nam.\n\nLại đi Việt Nam.").unwrap();
        fs::write(dir.path().join("raw/0002.txt"), "第2章\n\n越南很热。").unwrap();
        fs::write(dir.path().join("out/0002.txt"), "Chương 2\n\nViệt Nam rất nóng.").unwrap();
        dir
    }

    #[test]
    fn replace_chi_doan_da_chon_roi_quet_lai() {
        let dir = story();
        let lex = Lexicon::builtin();
        refresh_inner(dir.path(), &lex).unwrap();
        let request = VnReplaceRequest {
            from: "Việt Nam".into(),
            to: "Thái Lan".into(),
            chapters: vec!["0001".into()],
            paragraphs: Some(vec![("0001".into(), 1)]),
        };
        let outcome = replace_inner(dir.path(), &request, &lex).unwrap();
        assert_eq!((outcome.chapters, outcome.occurrences), (1, 1));
        let one = fs::read_to_string(dir.path().join("out/0001.txt")).unwrap();
        assert!(one.contains("Ngồi thuyền qua Thái Lan.") && one.contains("Lại đi Việt Nam."), "{one}");
        assert!(fs::read_to_string(dir.path().join("out/0002.txt")).unwrap().contains("Việt Nam"), "chương 2 ngoài phạm vi");
        let left: Vec<String> = qt_ai_core::vn_check::load_file(&story_paths(dir.path())).hits.iter().map(|h| h.id.clone()).collect();
        assert_eq!(left, vec!["0001:2:viet-nam", "0002:1:viet-nam"], "quét lại sau khi thay");
        // Hoàn tác bằng engine Tìm & thay sẵn có.
        qt_ai_core::replace::undo_last_replace(&story_paths(dir.path())).unwrap();
        assert!(fs::read_to_string(dir.path().join("out/0001.txt")).unwrap().contains("Ngồi thuyền qua Việt Nam."));
    }

    #[test]
    fn replace_ca_chuong_khi_khong_chon_doan() {
        let dir = story();
        let lex = Lexicon::builtin();
        let request = VnReplaceRequest { from: "Việt Nam".into(), to: "Thái Lan".into(), chapters: vec!["0001".into(), "0002".into()], paragraphs: None };
        assert_eq!(replace_inner(dir.path(), &request, &lex).unwrap().occurrences, 3);
    }

    #[test]
    fn tom_tat_truyen_co_so_co_vn_sau_khi_quet() {
        let dir = story();
        qt_ai_core::commands::init::run_init(dir.path(), "qt-ai").unwrap();
        assert_eq!(crate::summary_cache::summarize_uncached(dir.path()).vn_flags, None);
        refresh_inner(dir.path(), &Lexicon::builtin()).unwrap();
        assert_eq!(crate::summary_cache::summarize_uncached(dir.path()).vn_flags, Some(3));
    }
}
